use std::path::PathBuf;
use std::io::BufWriter;
use std::io::Write;
use std::panic::AssertUnwindSafe;

use tap::prelude::*;
use juniper::{GraphQLEnum, GraphQLObject, GraphQLScalarValue,
              graphql_scalar,
              ParseScalarResult, ParseScalarValue, ScalarValue};
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
    // todo - when exporting/importing JSON from GUI it does not accept FILE as InputEndpoint
    pub fn new(options: &Opts, state: State) -> Self {
        let root_path = options.file_root.as_path();
        std::fs::create_dir_all(root_path);
        let file_id_list = state.files.lock_mut()
            .pipe_borrow_mut(|files| {
                let mut list = Vec::new();
                std::fs::read_dir(root_path)
                    .expect("Cannot read the provided file root directory")
                    .for_each(|file_res|
                        if let Ok(file) = file_res {
                            if let Ok(filename) = file.file_name().into_string() {
                                files.push(FileInfo {
                                    file_id: filename.clone(),
                                    name: None,
                                    state: FileState::Local,
                                    download_state: None,
                                });
                                list.push(filename.clone());
                            };
                        }
                    );
                return list;
            });

        let api_key_opt = state.settings.lock_mut().google_api_key.clone();
        if let Some(api_key) = api_key_opt {
            let state_cpy = state.clone();
            drop(tokio::spawn(async move {
                for file_id in file_id_list {
                    if let Ok(filename) = Self::request_file_info(&file_id, &api_key).await {
                        state_cpy.files.lock_mut().iter_mut().find(|file| file.file_id == file_id)
                            .map_or_else(|| log::error!("Could not find file with the provided id: {}", file_id),
                                    |file_info| file_info.name = Some(filename));
                    } else {
                        log::error!("Could not get info for the file: {}", file_id);
                    }
                }
            }));
        }

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

    async fn request_file_info(file_id: &str, api_key: &str) -> Result<String, reqwest::Error> {
        Ok(reqwest::get(format!("https://www.googleapis.com/drive/v3/files/{}?fields=name&key={}", file_id, api_key).as_str())
            .await?.json::<FileInfoResponse>().await?.name)
    }

    pub fn need_file(&self, file_id: &str) {
        let mut all_files = self.state.files.lock_mut();
        if !all_files.iter().find(|file| file.file_id == file_id).is_some() {
            let new_file = FileInfo {
                file_id: file_id.to_string(),
                name: None,
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
                let api_key = state.settings.lock_mut().google_api_key.clone()
                    .ok_or("No API key provided")?;

                let client = reqwest::ClientBuilder::new()
                    .connection_verbose(false)
                    .build().map_err(|err| "Could not create a reqwest Client".to_string())?;

                // Get file name from the API
                let _ = Self::request_file_info(&filename, &api_key).await
                    .map_err(|err| "Could not get file info for the file".to_string())?
                    .pipe(|file_name|
                        state.files.lock_mut().iter_mut().find(|file| file.file_id == filename)
                            .map_or(Err("Could not find file with the provided id".to_string()),
                                    |file_info| Ok(file_info.name = Some(file_name)))
                    )?;

                // Download the file contents
                if let Ok(mut response) = client.get(
                    format!("https://www.googleapis.com/drive/v3/files/{}?alt=media&key={}", filename, api_key).as_str()).send().await
                {
                    let total = response.content_length();
                    let mut current: NetworkByteSize = NetworkByteSize(0);
                    // Create FileInfo Download state and set the state to Downloading
                    state.files.lock_mut().iter_mut().find(|file| file.file_id == filename)
                        .ok_or("Could not find file with provided file ID".to_string())?
                        .tap_mut(|val|
                            val.download_state = Some(DownloadState {
                                max_progress: NetworkByteSize(total.unwrap()),
                                current_progress: current,
                            })
                        )
                        .tap_mut(|val| val.state = FileState::Downloading);

                    // Try opening the target file where the downloaded bytes will be written
                    if let Ok(file) = std::fs::OpenOptions::new().create_new(true).write(true).open(format!("{}/{}", root_dir, &filename)) {
                        let mut writer = BufWriter::new(file);

                        // Download loop for updating the progress
                        while let Some(bytes) = response.chunk().await.unwrap_or(None) {
                            // If there is a problem with writing the downloaded bytes to a file stop the download and print error
                            if writer.write_all(&bytes).is_err() {
                                state.files.lock_mut().iter_mut()
                                    .find(|file| file.file_id == filename)
                                    .ok_or_else(|| "Could not send file status to Error".to_string())?
                                    .tap_mut(|val| val.state = FileState::Error);
                                return Err("Could not write received bytes to a file, aborting download.".to_string());
                            }

                            current.0 += bytes.len() as u64;
                            // Update download progress in the FileInfo
                            state.files.lock_mut().iter_mut()
                                .find(|file| file.file_id == filename)
                                .ok_or("File is no longer in the required files, canceling download.".to_string())?
                                .download_state.as_mut()
                                .ok_or("The file does not have a download state.".to_string())?
                                .current_progress = current.into();
                        }
                    } else {
                        return Err("Could not create a file with writing privileges.".to_string());
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
        }));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq)]
pub struct FileInfo {
    file_id: String,
    name: Option<String>,
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
    max_progress: NetworkByteSize,
    current_progress: NetworkByteSize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct NetworkByteSize(u64);
#[graphql_scalar()]
impl<S> juniper::GraphQLScalar for NetworkByteSize where S: juniper::ScalarValue {
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(self.0.to_owned() as f64)
    }

    fn from_input_value(value: &juniper::InputValue) -> Option<NetworkByteSize> {
        value.as_scalar_value().map(|s| NetworkByteSize(s.as_float().unwrap() as u64))
    }

    fn from_str<'a>(value: juniper::ScalarToken<'a>) -> juniper::ParseScalarResult<'a, S> {
        <NetworkByteSize as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

#[derive(Deserialize)]
struct FileInfoResponse {
    name: String,
}