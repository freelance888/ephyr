use std::path::PathBuf;
use std::io::BufWriter;
use std::io::Write;
use std::panic::AssertUnwindSafe;

use tap::prelude::*;
use juniper::{
    GraphQLEnum, GraphQLObject, GraphQLScalarValue,
    GraphQLUnion, ParseScalarResult, ParseScalarValue, ScalarValue, Value,
};
use serde::{Deserialize, Serialize};
use ephyr_log::log;

use crate::state::State;
use std::result::Result::Err;
use std::slice::Iter;
use futures::{FutureExt, StreamExt};
use crate::cli::Opts;
use crate::state::{InputSrc, Restream};


#[derive(Debug, Default)]
pub struct FileManager {
    file_root_dir: PathBuf,
    state: State,
}

impl FileManager {
    pub fn new(options: &Opts, state: State) -> Self {
        let root_path = options.file_root.as_path();
        std::fs::create_dir_all(root_path);
        state.files.lock_mut().tap_mut(|files| {
            std::fs::read_dir(root_path)
                .expect("Cannot read the provided file root directory")
                .for_each(|file_res|
                    if let Ok(file) = file_res {
                        if let Ok(filename) = file.file_name().into_string() {
                            files.push(FileInfo {
                                file_id: filename.clone(),
                                name: filename,
                                state: FileState::Local,
                                download_state: None,
                            });
                        };
                    });
        });
        Self {
            file_root_dir: options.file_root.clone(),
            state,
        }
    }

    pub fn check_files(&self, restreams: Iter<'_, Restream>) {
        restreams.for_each(|restream| {
            if let Some(InputSrc::Failover(fo)) = &restream.input.src {
                fo.inputs.iter().filter_map(|input| {
                    let endpoint = input.endpoints.first().unwrap();
                    if endpoint.is_file() {
                        endpoint.file_id.as_ref()
                    } else {
                        None
                    }
                }).for_each(|file_id| {
                    log::info!("Found file endpoint, requesting file");
                    self.need_file(file_id)
                });
            }
        });
    }

    async fn request_file_info(file_id: &str) -> Result<String, reqwest::Error> {
        Ok(reqwest::get(format!("https://www.googleapis.com/drive/v3/files/{}?fields=name&key={}", file_id,/* put the api key here*/ "").as_str())
            .await?.json::<FileInfoResponse>().await?.name)
    }

    pub fn need_file(&self, file_id: &str) {
        let mut all_files = self.state.files.lock_mut();
        if !all_files.iter().find(|file| file.file_id == file_id).is_some() {
            let new_file = FileInfo {
                file_id: file_id.to_string(),
                name: file_id.to_string(),
                state: FileState::Pending,
                download_state: None,
            };
            all_files.push(new_file);
            drop(all_files);
            self.download_file(file_id);
        }
    }

    fn download_file(&self, filename_ref: &str) {
        let root_dir = self.file_root_dir.to_str().unwrap().to_string();
        let state = self.state.clone();
        let filename = filename_ref.to_string();
        drop(tokio::spawn(async move {
            async {
                let client = reqwest::ClientBuilder::new()
                    .connection_verbose(false)
                    .build().map_err(|err| "Could not create a reqwest Client".to_string())?;

                // Get file name from the API
                let _ = Self::request_file_info(&filename).await
                    .map_err(|err| "Could not get file info for the file".to_string())?
                    .pipe(|file_name|
                        state.files.lock_mut().iter_mut().find(|file| file.file_id == filename)
                            .map_or(Err("Could not find file with the provided id".to_string()),
                                    |file_info| Ok(file_info.name = file_name))
                    )?;

                // Download the file contents
                if let Ok(mut response) = client.get(
                    format!("https://www.googleapis.com/drive/v3/files/{}?alt=media&key={}", filename,/* put the api key here*/ "").as_str()).send().await
                {
                    let total = response.content_length();
                    let mut current: i32 = 0;
                    state.files.lock_mut().iter_mut().find(|file| file.file_id == filename)
                        .ok_or("Could not find file with provided file ID".to_string())?
                        .tap_mut(|val|
                            val.download_state = Some(DownloadState {
                                max_progress: total.unwrap() as i32,
                                current_progress: current,
                            })
                        )
                        .tap_mut(|val| val.state = FileState::Downloading);

                    if let Ok(file) = std::fs::OpenOptions::new().create_new(true).write(true).open(format!("{}/{}", root_dir, &filename)) {
                        let mut writer = BufWriter::new(file);

                        while let Some(bytes) = response.chunk().await.unwrap_or(None) {
                            if writer.write_all(&bytes).is_err() {
                                state.files.lock_mut().iter_mut()
                                    .find(|file| file.file_id == filename)
                                    .ok_or_else(|| "Could not send file status to Error".to_string())?
                                    .tap_mut(|val| val.state = FileState::Error);
                                return Err("Could not write received bytes to a file, aborting download.".to_string());
                            }

                            current += bytes.len() as i32;
                            state.files.lock_mut().iter_mut()
                                .find(|file| file.file_id == filename)
                                .ok_or("File is no longer in required files, canceling download.".to_string())?
                                .download_state.as_mut()
                                .ok_or("The file does not have a download state.".to_string())?
                                .current_progress = current.into();
                        }
                    } else {
                        return Err("Could not create a file for writing.".to_string());
                    }

                    Ok(response.status().as_u16())
                } else {
                    Err("Could not send download request for the file".to_string())
                }
            }
                .await
                .map_err(|err| {
                    log::error!("Could not download file {}: {}", &filename, err);
                });
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq)]
pub struct FileInfo {
    file_id: String,
    name: String,
    state: FileState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    download_state: Option<DownloadState>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq)]
pub enum FileState {
    Pending,
    Downloading,
    Local,
    Error
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq)]
pub struct DownloadState {
    max_progress: i32,
    current_progress: i32,
}

#[derive(Deserialize)]
struct FileInfoResponse {
    name: String,
}