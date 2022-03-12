use std::{
    collections::HashMap,
    path::PathBuf,
};
use std::io::BufWriter;
use std::io::Write;

use tap::prelude::*;
use juniper::{
    GraphQLEnum, GraphQLObject, GraphQLScalarValue,
    GraphQLUnion, ParseScalarResult, ParseScalarValue, ScalarValue, Value,
};
use serde::{Deserialize, Serialize};
use ephyr_log::log;

use crate::{
    state::State,
};
use std::result::Result::Err;
use std::slice::Iter;
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
        let mut files = HashMap::new();
        std::fs::read_dir(root_path).unwrap()
            .for_each(|file_res|
                if let Ok(file) = file_res {
                    if let Ok(filename) = file.file_name().into_string() {
                        files.insert(filename.clone(), FileInfo {
                            file_id: filename,
                            state: FileState::Local,
                            download_state: None,
                        });
                    };
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
                    log::info!("goiing through backup endpoints");
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

    pub fn need_file(&self, filename: &str) {
        log::info!("Requesting file id: {}", filename);
        let mut all_files = self.state.files.lock_mut();
        if !all_files.iter().find(|file| file.file_id == filename).is_some() {
            let new_file = FileInfo {
                file_id: filename.to_string(),
                state: FileState::Pending,
                download_state: None,
            };
            all_files.push(new_file);
            drop(all_files);
            self.download_file(filename);
        }
    }

    fn download_file(&self, filename_ref: &str) {
        let root_dir = self.file_root_dir.to_str().unwrap().to_string();
        let state = self.state.clone();
        let filename = filename_ref.to_string();
        drop(tokio::spawn(async move {
            let client = reqwest::Client::new();
            log::info!("Creating client request");
            if let Ok(mut response) = client.get(
                format!("https://www.googleapis.com/drive/v3/files/{}?alt=media&key={}", filename,/* put the api key here*/ "").as_str()).send().await
            {
                let total = response.content_length();
                let mut current: i32 = 0;
                log::info!("Creating download state");
                //todo remove unwrap
                state.files.lock_mut().iter_mut().find(|file| file.file_id == filename)
                    .unwrap()
                    .tap_mut(|val|
                        val.download_state = Some(DownloadState {
                            max_progress: total.unwrap() as i32,
                            current_progress: current,
                        })
                    )
                    .tap_mut(|val| val.state = FileState::Downloading);

                if let Ok(file) = std::fs::OpenOptions::new().create_new(true).write(true).open(format!("{}/{}", root_dir, &filename)) {
                    let mut writer = BufWriter::new(file);
                    let mut status_changed_to_downloading = false;
                    //todo check the errors if some

                    while let Some(bytes) = response.chunk().await.unwrap_or(None) {
                        if writer.write_all(&bytes).is_err() {
                            log::error!("Could not write received bytes to a file, aborting download.");
                            state.files.lock_mut().iter_mut()
                                .find(|file| file.file_id == filename)
                                .ok_or_else(|| {log::error!("Could not send file status to Error");"Err"})?
                                .tap_mut(|val| val.state = FileState::Error);
                            break;
                        }
                        if !status_changed_to_downloading {
                            state.files.lock_mut().iter_mut()
                                .find(|file| file.file_id == filename)
                                .ok_or_else(|| {log::error!("Could not set file status to Downloading");"Err"})?
                                .tap_mut(|val| {
                                    val.state = FileState::Downloading;
                                    status_changed_to_downloading = true
                                });
                        }
                        // let mut files = state.files.lock_mut();
                        // log::info!("Bytes received, total count  {}", bytes.len());
                        current += bytes.len() as i32;
                        state.files.lock_mut().iter_mut()
                            .find(|file| file.file_id == filename)
                            .map_or(Err(format!("File {} is no longer in required files, canceling download.", filename)),
                                    |val| {
                                        val.download_state.as_mut()
                                            .map_or(Err(format!("File that is currently downloading does not have a download state")), |val| Ok(val))?
                                            // .unwrap_or_else(|| -> log::error!("File that is currently downloading does not have a download state"))
                                            .tap_mut(|val| val.current_progress = current.into())
                                            // .tap()
                                            .current_progress = current.into();
                                        Ok(())
                                    }
                            )?;
                            // .unwrap_or_else(|| {log::error!("File {} is no longer in required files, canceling download.", filename)})
                        // if let Some(file) = files.iter_mut().find(|file| file.file_id == filename) {
                        //     if let Some(ref mut download_state) = file.download_state {
                        //         download_state.current_progress = current.into();
                        //     } else {
                        //         log::error!("File that is currently downloading does not have a download state")
                        //     }
                        // } else {
                        //     log::error!("File {} is no longer in required files, canceling download.", filename);
                        //     break;
                        // }
                    }
                }

                let result = response.status().as_u16();
                Ok(result)
            } else {
                log::error!("Could not send download request for file {}", filename);
                reqwest::StatusCode::BAD_REQUEST.as_u16();
                Err("Error".to_string())
            }
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq)]
pub struct FileInfo {
    file_id: String,
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