//! File manager for requesting and downloading files

use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use derive_more::{Deref, Display, Into};
use ephyr_log::{tracing, Instrument};
use juniper::{GraphQLEnum, GraphQLObject, GraphQLScalar, ScalarValue};
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use crate::{
    cli::Opts,
    display_panic,
    file_manager::api_response::{get_gdrive_result, ErrorResponse},
    state::{InputEndpointKind, InputSrc, State, Status},
    stream_probe::stream_probe,
    stream_statistics::StreamStatistics,
};
use chrono::{Local, Utc};
use ephyr_log::tracing::instrument;
use futures::{FutureExt, TryFutureExt};
use reqwest::{Response, StatusCode};
use std::{
    borrow::BorrowMut, ffi::OsString, fs::DirEntry, panic::AssertUnwindSafe,
    result::Result::Err,
};

const GDRIVE_PUBLIC_PARAMS: &str = "supportsAllDrives=True\
&supportsTeamDrives=True\
&includeItemsFromAllDrives=True\
&includeTeamDriveItems=True";

/// Commands for handling operations on files
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileCommand {
    /// Notifies that file backup was added/removed to/from restream or
    /// [`PlaylistFileInfo`] was loaded for specific `Restream`
    ListOfFilesChanged,

    /// Request for redo download file from Google Drive with
    /// specific [`FileId`].
    /// File will be waiting until the queue has capacity
    /// to download file
    NeedDownloadFiles(Vec<FileId>),

    /// Start download process for specific [`FileId`]
    StartDownloadFile(Vec<FileId>),
}

/// Identity of file on `Google Drive`.
#[derive(
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    Hash,
    Into,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    GraphQLScalar,
)]
#[graphql(transparent)]
pub struct FileId(String);

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
        let root_path = options.file_root.clone();
        drop(std::fs::create_dir_all(root_path.clone()));

        Self {
            file_root_dir: root_path,
            state,
        }
    }

    /// Command processing
    #[instrument(skip_all, name = "file_manager::handle_commands")]
    pub fn handle_commands(&self) {
        let commands: Vec<FileCommand> =
            self.state.file_commands.lock_mut().drain(..).collect();

        commands.iter().for_each(|c| match c {
            FileCommand::ListOfFilesChanged => self.check_files(),

            FileCommand::NeedDownloadFiles(file_ids) => {
                let mut files = self.state.files.lock_mut();
                files.retain(|file| !file_ids.contains(&file.file_id));
                drop(files);
                self.sync_with_state();
                for file_id in file_ids {
                    self.need_file(file_id, None);
                }
            }

            FileCommand::StartDownloadFile(file_ids) => self
                .state
                .files
                .lock_mut()
                .iter()
                .filter(|f| file_ids.iter().any(|id| f.file_id == *id))
                .for_each(|f| {
                    self.download_file(&f.file_id, f.clone().name);
                }),
        });
    }

    /// Checks all the [`crate::state::Input`]s and if some has
    /// [`crate::state::InputEndpoint`] of type
    /// [`crate::state::InputEndpointKind::File`] tries to download it,
    /// if the given ID does not exist in the file list.
    pub fn check_files(&self) {
        self.state.file_commands.lock_mut().clear();

        let mut files_data = vec![];
        let restreams = self.state.restreams.lock_mut();
        restreams.iter().for_each(|restream| {
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
                    .for_each(|file_id| {
                        files_data.push((file_id, None));
                    });
            }
            restream.playlist.queue.iter().for_each(|file| {
                files_data.push((&file.file_id, Some(file.name.clone())));
            });
        });

        // Removes not used files from state
        let mut files = self.state.files.lock_mut();
        files.retain(|f| {
            files_data
                .clone()
                .into_iter()
                .any(|(file_id, _)| &f.file_id == file_id)
        });
        drop(files);

        self.sync_with_state();

        // Check if file need to be downloaded
        for (file_id, file_name) in files_data {
            self.need_file(file_id, file_name);
        }
    }

    /// Sync files on disks with files in state
    fn sync_with_state(&self) {
        let are_files_the_same = |f: &LocalFileInfo, de: &DirEntry| {
            OsString::from(&f.file_id.0) == de.file_name()
        };

        let mut files = self.state.files.lock_mut();
        let disk_files: Vec<_> =
            std::fs::read_dir(self.file_root_dir.as_path())
                .expect("Cannot read the provided file root directory")
                .filter_map(Result::ok)
                .filter(|entry| match entry.file_type() {
                    // Returns only files, skips directories
                    Ok(file_type) => file_type.is_file(),
                    _ => false,
                })
                .collect();

        /// Find files on disk that do not have corresponding files
        /// in state and delete them
        disk_files.iter().for_each(|df| {
            if !files.iter().any(|f| are_files_the_same(f, df)) {
                let file_path = self.file_root_dir.join(df.file_name());
                let _ = std::fs::remove_file(file_path).map_err(|err| {
                    tracing::error!("Can not delete file. {}", err);
                });
            }
        });

        /// Find files in state that do not have corresponding file on disk
        /// and set their state to [`FileState::DownloadError`]
        files
            .iter_mut()
            .filter(|f| f.state != FileState::Waiting)
            .for_each(|f| {
                if !disk_files.iter().any(|df| are_files_the_same(f, df)) {
                    f.state = FileState::DownloadError;
                    f.download_state = None;
                    f.stream_stat = None;
                    f.error = Some("There is no file on disk.".to_string());
                }
            })
    }

    /// Checks if the provided file ID already exists in the file list,
    /// if not add it to the queue
    pub fn need_file(&self, file_id: &FileId, file_name: Option<String>) {
        let mut all_files = self.state.files.lock_mut();
        if !all_files.iter().any(|file| &file.file_id == file_id) {
            let new_file = LocalFileInfo {
                file_id: file_id.clone(),
                name: file_name,
                state: FileState::Waiting,
                download_state: None,
                error: None,
                stream_stat: None,
            };
            all_files.push(new_file);
        }
    }

    /// Retrieves file info (currently only the file name) from the Google API
    async fn update_file_info<'a>(
        file_id: &FileId,
        api_key: &'a str,
        state: &'a State,
    ) -> Result<(), String> {
        let response = reqwest::get(
            format!(
                "https://www.googleapis.com/drive/v3/files/{file_id}?
                fields=name&key={api_key}&{GDRIVE_PUBLIC_PARAMS}"
            )
            .as_str(),
        )
        .await;

        let filename =
            get_gdrive_result::<api_response::FileNameResponse>(response)
                .await?
                .name;

        state
            .files
            .lock_mut()
            .iter_mut()
            .find(|file| &file.file_id == file_id)
            .map_or_else(
                || {
                    tracing::error!(
                        "Could not find file \
                             with the provided id: {}",
                        file_id
                    );
                    Err("Could not find the provided file ID".to_string())
                },
                |file_info| {
                    file_info.name = Some(filename);
                    Ok(())
                },
            )
    }

    /// Spawns a separate process that tries to download given file ID
    #[allow(clippy::too_many_lines)]
    fn download_file(&self, id: &FileId, file_name: Option<String>) {
        let root_dir = self.file_root_dir.to_str().unwrap().to_string();
        let state = self.state.clone();
        let file_id = id.clone();
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
                    .map_err(|err| {
                        format!("Could not create a reqwest Client: {err}")
                    })?;

                // Get file name from the API
                if file_name.is_none() {
                    Self::update_file_info(&file_id, &api_key, &state)
                        .await
                        .map_err(|err| {
                            format!(
                                "Could not get file info for the file: {err}"
                            )
                        })?;
                } else {
                    state
                        .files
                        .lock_mut()
                        .iter_mut()
                        .find(|file| file.file_id == file_id)
                        .map(|file_info| file_info.name = file_name)
                        .ok_or_else(|| {
                            format!(
                                "Could not find file with the \
                            provided file ID: {file_id}"
                            )
                        })?;
                }

                // Download the file contents
                if let Ok(mut response) = client
                    .get(
                        format!(
                            "https://www.googleapis.com/drive/v3/files/\
                            {file_id}?alt=media&key={api_key}\
                            &{GDRIVE_PUBLIC_PARAMS}"
                        )
                        .as_str(),
                    )
                    .send()
                    .await
                {
                    let status = response.status();
                    if status != StatusCode::OK {
                        if status == 403 {
                            return Err(response
                                .json::<ErrorResponse>()
                                .await
                                .map(|r| {
                                    format!(
                                        "Http response: {} {}",
                                        r.error.code, r.error.message
                                    )
                                })
                                .map_err(|e| {
                                    format!(
                                        "Status: {status}. Unknown error {e}"
                                    )
                                })?);
                        }

                        let error_response = response.text().await;
                        return Err(format!(
                            "Can't download file. Http response: {} {}",
                            status,
                            error_response.unwrap_or_default(),
                        ));
                    }

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
                tracing::error!(
                    "Could not download file {}: {}",
                    &file_id,
                    err
                );
                state
                    .files
                    .lock_mut()
                    .iter_mut()
                    .find(|file| file.file_id == file_id)
                    .map_or_else(
                        || {
                            tracing::error!(
                                "Could not set the file state to error"
                            );
                        },
                        |val| {
                            val.state = FileState::DownloadError;
                            val.error = Some(err);
                        },
                    );
            });
        }));
    }

    /// Runs the while loop receiving bytes in packets, writes them to file
    /// and tracks progress
    async fn download_and_write_bytes(
        file_id: &FileId,
        root_dir: &str,
        response: &mut Response,
        state: &State,
    ) -> Result<(), String> {
        // Try opening the target file where the downloaded
        // bytes will be written
        let file_path = format!("{root_dir}/{}", &file_id);
        let file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(file_path.clone())
            .map_err(|err| format!("Can't create file: {}", err))?;

        let mut writer = BufWriter::new(file);
        let mut last_update = Utc::now();

        let mut current: NetworkByteSize = NetworkByteSize(0);
        // Download loop for updating the progress
        while let Some(bytes) = response.chunk().await.unwrap_or(None) {
            // If there is a problem with writing the downloaded
            // bytes to a file stop the download and print error
            if writer.write_all(&bytes).is_err() {
                return Err("Could not write received bytes to a file,\
                    aborting download."
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
                    .find(|file| {
                        &file.file_id == file_id
                            && file.state != FileState::DownloadError
                    })
                    .ok_or_else(|| {
                        "File is no longer in the required
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
            .find(|file| &file.file_id == file_id)
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
                update_stream_info(
                    file_id.clone(),
                    file_path.clone(),
                    state.clone(),
                );
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
                                    .eq(file_id)
                        })
                        .for_each(|endpoint| {
                            endpoint.status = Status::Online;
                        });
                });
            }
        });
        Ok(())
    }
}

/// Update stream info for downloaded file
fn update_stream_info(file_id: FileId, url: String, state: State) {
    drop(tokio::spawn(
        AssertUnwindSafe(
            async move {
                let result = stream_probe(url).await;
                state
                    .set_file_stream_info(&file_id, result)
                    .unwrap_or_else(|e| tracing::error!("{}", e));
            }
            .in_current_span(),
        )
        .catch_unwind()
        .map_err(move |p| {
            tracing::warn!("Can not fetch stream info: {}", display_panic(&p),);
        }),
    ));
}

/// Represents a File with given ID and hold additional information
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct LocalFileInfo {
    /// ID of the file
    pub file_id: FileId,

    /// Name of the file if API call for the name was successful
    pub name: Option<String>,

    /// State of the file
    pub state: FileState,

    /// Download error message
    pub error: Option<String>,

    /// Corresponding stream info
    pub stream_stat: Option<StreamStatistics>,

    /// If the file is downloading the state of the download
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_state: Option<DownloadState>,
}

impl From<api_response::ExtendedFileInfoResponse> for LocalFileInfo {
    fn from(file_response: api_response::ExtendedFileInfoResponse) -> Self {
        LocalFileInfo {
            file_id: FileId(file_response.id),
            name: Some(file_response.name),
            state: FileState::Pending,
            download_state: None,
            error: None,
            stream_stat: None,
        }
    }
}

/// Information necessary for every video in playlist
#[derive(
    Debug, Clone, Serialize, Deserialize, GraphQLObject, PartialEq, Eq,
)]
pub struct PlaylistFileInfo {
    /// Google ID of this file
    pub file_id: FileId,

    /// Name of this file
    pub name: String,

    /// Whether the file was already played
    pub was_played: bool,
}

impl From<api_response::ExtendedFileInfoResponse> for PlaylistFileInfo {
    fn from(file_response: api_response::ExtendedFileInfoResponse) -> Self {
        PlaylistFileInfo {
            file_id: FileId(file_response.id),
            name: file_response.name,
            was_played: false,
        }
    }
}

/// State in which the file represented by [`LocalFileInfo`]
/// and [`PlaylistFileInfo`] can be in
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, GraphQLEnum, PartialEq, Eq,
)]
pub enum FileState {
    /// The file is waiting for starting download process
    Waiting,

    /// The file download is pending first server response
    Pending,

    /// The file is downloading
    Downloading,

    /// File is downloaded and saved in the directory provided
    /// as parameter at startup
    Local,

    /// Error was encountered during the download
    DownloadError,
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
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, GraphQLScalar,
)]
#[graphql(with = Self)]
struct NetworkByteSize(u64);

impl NetworkByteSize {
    #[allow(clippy::wrong_self_convention, clippy::trivially_copy_pass_by_ref)]
    fn to_output<S: ScalarValue>(&self) -> juniper::Value<S> {
        juniper::Value::scalar(self.0.to_owned().to_string())
    }

    fn from_input<S>(value: &juniper::InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        value
            .as_scalar_value()
            .map(|s| {
                NetworkByteSize(s.as_string().unwrap().parse::<u64>().unwrap())
            })
            .ok_or_else(|| {
                "Cannot parse NetworkByteSize(u64) from provided input"
                    .to_string()
            })
    }

    fn parse_token<S>(
        value: juniper::ScalarToken<'_>,
    ) -> juniper::ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <NetworkByteSize as juniper::ParseScalarValue<S>>::from_str(value)
    }
}

/// Retrieves list of video files from a Google drive folder
///
/// # Errors
///
/// Any error from Google Drive API
pub async fn get_video_list_from_gdrive_folder(
    api_key: &str,
    folder_id: &str,
) -> Result<Vec<PlaylistFileInfo>, String> {
    let mut response =
        api_response::FileListResponse::retrieve_dir_content_from_api(
            api_key, folder_id,
        )
        .await?;
    response.filter_only_video_files();
    Ok(response
        .files
        .drain(..)
        .map(PlaylistFileInfo::from)
        .collect())
}

pub(crate) mod api_response {
    use crate::file_manager::GDRIVE_PUBLIC_PARAMS;
    use reqwest::{Response, StatusCode};
    use serde::Deserialize;

    /// Used to deserialize Google API call for the file details
    #[derive(Deserialize)]
    pub(crate) struct FileNameResponse {
        /// Name of the file
        pub(crate) name: String,
    }

    #[derive(Deserialize, Debug)]
    pub(crate) struct ExtendedFileInfoResponse {
        pub(crate) id: String,
        pub(crate) name: String,
        #[serde(alias = "mimeType")]
        pub(crate) mime_type: String,
    }

    impl ExtendedFileInfoResponse {
        #[allow(dead_code)]
        pub(crate) fn is_dir(&self) -> bool {
            self.mime_type == "application/vnd.google-apps.folder"
        }

        pub(crate) fn is_video(&self) -> bool {
            self.mime_type.starts_with("video")
        }
    }

    #[derive(Deserialize)]
    pub(crate) struct FileListResponse {
        // TODO fix this
        // pub(crate) kind: String,
        // pub(crate) incomplete_search: bool,
        pub(crate) files: Vec<ExtendedFileInfoResponse>,
    }

    impl FileListResponse {
        pub(crate) async fn retrieve_dir_content_from_api(
            api_key: &str,
            dir_id: &str,
        ) -> Result<Self, String> {
            let response = reqwest::get(
                format!(
                    "https://www.googleapis.com/drive/v3/files?\
                     key={api_key}&q='{dir_id}'%20in%20parents&\
                     fields=files/id,files/name,files/mimeType&\
                     {GDRIVE_PUBLIC_PARAMS}",
                )
                .as_str(),
            )
            .await;
            get_gdrive_result::<Self>(response).await
        }

        pub(crate) fn filter_only_video_files(&mut self) {
            self.files.retain(ExtendedFileInfoResponse::is_video);
        }
    }

    ///
    #[derive(Deserialize)]
    pub(crate) struct ErrorResponse {
        pub(crate) error: ErrorMessage,
    }

    ///
    #[derive(Deserialize)]
    pub(crate) struct ErrorMessage {
        pub(crate) code: u16,
        pub(crate) message: String,
    }

    pub(crate) async fn get_gdrive_result<T: for<'de> Deserialize<'de>>(
        response: reqwest::Result<Response>,
    ) -> Result<T, String> {
        let error_parsing = |e| format!("Error parsing JSON result: {e}");

        match response {
            Err(e) => Err(format!("No valid response from the API: {e}")),
            Ok(r) => {
                let status = r.status();
                if status == StatusCode::OK {
                    return r.json::<T>().await.map_err(error_parsing);
                } else if status == 403 {
                    return Err(r
                        .json::<ErrorResponse>()
                        .await
                        .map(|x| {
                            format!(
                                "Http response: {} {}",
                                x.error.code, x.error.message
                            )
                        })
                        .map_err(error_parsing)?);
                }

                Err(r
                    .text()
                    .await
                    .map_err(|e| format!("Unknown error: {e}"))?)
            }
        }
    }
}
