//! File manager for requesting and downloading files

use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use ephyr_log::log;
use juniper::{graphql_scalar, GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use crate::{
    cli::Opts,
    state::{InputEndpointKind, InputSrc, Restream, State, Status},
};
use chrono::Utc;
use reqwest::Response;
use std::{borrow::BorrowMut, result::Result::Err, slice::Iter};

/// Manages file downloads and files in the provided [`State`]
#[derive(Debug, Default)]
pub struct FileManager {
    file_root_dir: PathBuf,
    state: State,
}

impl FileManager {
    /// Creates new [`FileManager`] with the provided [`State`]
    #[must_use]
    pub fn new(options: &Opts, state: State) -> Self {
        let root_path = options.file_root.as_path();
        drop(std::fs::create_dir_all(root_path));
        let file_id_list = state.files.lock_mut().pipe_borrow_mut(|files| {
            let mut list = Vec::new();
            std::fs::read_dir(root_path)
                .expect("Cannot read the provided file root directory")
                .for_each(|file_res| {
                    if let Ok(file) = file_res {
                        if let Ok(filename) = file.file_name().into_string() {
                            files.push(FileInfo {
                                file_id: filename.clone(),
                                name: None,
                                state: FileState::Local,
                                download_state: None,
                            });
                            list.push(filename);
                        };
                    }
                });
            list
        });

        let api_key_opt = state.settings.lock_mut().google_api_key.clone();
        if let Some(api_key) = api_key_opt {
            let state_cpy = state.clone();
            drop(tokio::spawn(async move {
                for file_id in file_id_list {
                    let _ =
                        Self::update_file_info(&file_id, &api_key, &state_cpy)
                            .await;
                }
            }));
        }

        Self {
            file_root_dir: options.file_root.clone(),
            state,
        }
    }

    /// Checks all the [`crate::state::Input`]s and if some has
    /// [`crate::state::InputEndpoint`] of type
    /// [`crate::state::InputEndpointKind::File`] tries to download it,
    /// if the given ID does not exist in the file list.
    pub fn check_files(&self, restreams: Iter<'_, Restream>) {
        restreams.for_each(|restream| {
            if let Some(InputSrc::Failover(fo)) = &restream.input.src {
                fo.inputs
                    .iter()
                    .filter_map(|input| {
                        input.endpoints.first().and_then(|endpoint| {
                            if endpoint.is_file() {
                                endpoint.file_id.as_ref()
                            } else {
                                None
                            }
                        })
                    })
                    .for_each(|file_id| self.need_file(file_id));
            }
        });
    }

    /// Checks if the provided file ID already exists in the file list,
    /// if not the download of the given file is queued.
    pub fn need_file(&self, file_id: &str) {
        let mut all_files = self.state.files.lock_mut();
        if !all_files.iter().any(|file| file.file_id == file_id) {
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

    /// Retrieves file info (currently only the file name) from the Google API
    async fn update_file_info<'a>(
        file_id: &'a str,
        api_key: &'a str,
        state: &'a State,
    ) -> Result<(), &'static str> {
        let filename = reqwest::get(
            format!(
                "https://www.googleapis.com/drive/v3/files/{}?fields=name&\
                 key={}",
                file_id, api_key
            )
            .as_str(),
        )
        .await
        .map_err(|_err| "No valid response from the API")?
        .json::<FileInfoResponse>()
        .await
        .map_err(|_err| "Could not parse the JSON received from the API")?
        .name;

        state
            .files
            .lock_mut()
            .iter_mut()
            .find(|file| file.file_id == file_id)
            .map_or_else(
                || {
                    log::error!(
                        "Could not find file \
                             with the provided id: {}",
                        file_id
                    );
                    Err("Could not find the provided file ID")
                },
                |file_info| {
                    file_info.name = Some(filename);
                    Ok(())
                },
            )
    }

    /// Spawns a separate process that tries to download given file ID
    fn download_file(&self, file_id_ref: &str) {
        let root_dir = self.file_root_dir.to_str().unwrap().to_string();
        let state = self.state.clone();
        let file_id = file_id_ref.to_string();
        drop(tokio::spawn(async move {
            let _ = async {
                let api_key = state
                    .settings
                    .lock_mut()
                    .google_api_key
                    .clone()
                    .ok_or("No API key provided")?;

                let client = reqwest::ClientBuilder::new()
                    .connection_verbose(false)
                    .build()
                    .map_err(|_err| {
                        "Could not create a reqwest Client".to_string()
                    })?;

                // Get file name from the API
                Self::update_file_info(&file_id, &api_key, &state)
                    .await
                    .map_err(|_err| {
                        "Could not get file info for the file".to_string()
                    })?;

                // Download the file contents
                if let Ok(mut response) = client
                    .get(
                        format!(
                            "https://www.googleapis.com/drive/v3/files/{}?\
                             alt=media&key={}",
                            file_id, api_key
                        )
                        .as_str(),
                    )
                    .send()
                    .await
                {
                    let total = response.content_length();
                    // Create FileInfo Download state and set the state
                    // to Downloading
                    state
                        .files
                        .lock_mut()
                        .iter_mut()
                        .find(|file| file.file_id == file_id)
                        .ok_or_else(|| {
                            "Could not find file with the \
                             provided file ID"
                                .to_string()
                        })?
                        .pipe_borrow_mut(|val| {
                            val.download_state = Some(DownloadState {
                                max_progress: NetworkByteSize(total.unwrap()),
                                current_progress: NetworkByteSize(0),
                            });
                            val.state = FileState::Downloading;
                        });

                    Self::download_and_write_bytes(
                        &file_id,
                        &root_dir,
                        response.borrow_mut(),
                        &state,
                    )
                    .await?;

                    Ok(response.status().as_u16())
                } else {
                    Err("Could not send download request for the file"
                        .to_string())
                }
            }
            .await
            .map_err(|err| {
                log::error!("Could not download file {}: {}", &file_id, err);
                state
                    .files
                    .lock_mut()
                    .iter_mut()
                    .find(|file| file.file_id == file_id)
                    .map_or_else(
                        || log::error!("Could not set the file state to error"),
                        |val| val.state = FileState::Error,
                    );
            });
        }));
    }

    /// Runs the while loop receiving bytes in packets, writes them to file
    /// and tracks progress
    async fn download_and_write_bytes(
        file_id: &str,
        root_dir: &str,
        response: &mut Response,
        state: &State,
    ) -> Result<(), String> {
        // Try opening the target file where the downloaded
        // bytes will be written
        if let Ok(file) = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(format!("{}/{}", root_dir, &file_id))
        {
            let mut writer = BufWriter::new(file);
            let mut last_update = Utc::now();

            let mut current: NetworkByteSize = NetworkByteSize(0);
            // Download loop for updating the progress
            while let Some(bytes) = response.chunk().await.unwrap_or(None) {
                // If there is a problem with writing the downloaded
                // bytes to a file stop the download and print error
                if writer.write_all(&bytes).is_err() {
                    return Err("Could not write received bytes to \
                                     a file, aborting download."
                        .to_string());
                }

                current.0 += bytes.len() as u64;
                // Update download progress in the FileInfo,
                // but only each 400ms
                if Utc::now()
                    .signed_duration_since(last_update)
                    .num_milliseconds()
                    > 400
                {
                    state
                        .files
                        .lock_mut()
                        .iter_mut()
                        .find(|file| file.file_id == file_id)
                        .ok_or_else(|| {
                            "File is no longer in the required \
                                        files, canceling download."
                                .to_string()
                        })?
                        .download_state
                        .as_mut()
                        .ok_or_else(|| {
                            "The file does not have a \
                                        download state."
                                .to_string()
                        })?
                        .current_progress = current;
                    last_update = Utc::now();
                }
            }
            writer.flush().map_err(|_err| {
                "Could not write all downloaded bytes to the file.".to_string()
            })?;

            state
                .files
                .lock_mut()
                .iter_mut()
                .find(|file| file.file_id == file_id)
                .ok_or_else(|| {
                    "File is no longer in the required \
                                files, canceling download."
                        .to_string()
                })?
                .pipe_borrow_mut(|file| {
                    // The download state should be definitely
                    // present at this point
                    file.download_state.as_mut().unwrap().current_progress =
                        current;
                    file.state = FileState::Local;
                });

            // set the endpoints with this file ID to Online, this
            // also sends the update to Restrams to restart the
            // ffmpeg processes without this the ffmpeg won't get
            // notified that the file has become available
            state.restreams.lock_mut().iter_mut().for_each(|restream| {
                if let Some(InputSrc::Failover(input_src)) =
                    restream.input.src.as_mut()
                {
                    input_src.inputs.iter_mut().for_each(|failover| {
                        failover
                            .endpoints
                            .iter_mut()
                            .filter(|endpoint| {
                                endpoint.kind == InputEndpointKind::File
                                    && endpoint.file_id.is_some()
                                    && endpoint
                                        .file_id
                                        .as_ref()
                                        .unwrap()
                                        .eq(&file_id)
                            })
                            .for_each(|endpoint| {
                                endpoint.status = Status::Online;
                            });
                    });
                }
            });
            Ok(())
        } else {
            Err("Could not create a file with \
                                    writing privileges."
                .to_string())
        }
    }
}

/// Represents a File with given ID and hold additional information
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct FileInfo {
    /// ID of the file
    pub file_id: String,

    /// Name of the file if API call for the name was successful
    name: Option<String>,

    /// State of the file
    pub state: FileState,

    /// If the file is downloading the state of the download
    #[serde(default, skip_serializing_if = "Option::is_none")]
    download_state: Option<DownloadState>,
}

/// State in which the file represented by [`FileInfo`] can be in
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq,
)]
pub enum FileState {
    /// The file download is pending first server response
    Pending,
    /// The file is downloading
    Downloading,
    /// File is downloaded and saved in the directory provided
    /// as parameter at startup
    Local,
    /// Error was encountered during the download
    Error,
}

/// Download progress indication
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct DownloadState {
    /// Expected size in bytes of the whole file
    max_progress: NetworkByteSize,
    /// Number of currently downloaded bytes
    current_progress: NetworkByteSize,
}

/// Custom GraphQL type for u64
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct NetworkByteSize(u64);
#[graphql_scalar()]
impl<S> juniper::GraphQLScalar for NetworkByteSize
where
    S: juniper::ScalarValue,
{
    fn resolve(&self) -> juniper::Value {
        juniper::Value::scalar(self.0.to_owned().to_string())
    }

    fn from_input_value(
        value: &juniper::InputValue,
    ) -> Option<NetworkByteSize> {
        value.as_scalar_value().map(|s| {
            NetworkByteSize(s.as_string().unwrap().parse::<u64>().unwrap())
        })
    }

    fn from_str(
        value: juniper::ScalarToken<'_>,
    ) -> juniper::ParseScalarResult<'_, S> {
        <NetworkByteSize as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

/// Used to deserialize Google API call for the file details
#[derive(Deserialize)]
struct FileInfoResponse {
    /// Name of the file
    name: String,
}
