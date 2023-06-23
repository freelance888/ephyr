use crate::file_manager::FileId;
use reqwest::{Response, StatusCode};
use serde::Deserialize;

pub const GDRIVE_PUBLIC_PARAMS: &str = "supportsAllDrives=True\
&supportsTeamDrives=True\
&includeItemsFromAllDrives=True\
&includeTeamDriveItems=True";

/// Used to deserialize Google API call for the file details
#[derive(Deserialize)]
pub struct FileNameResponse {
    /// Name of the file
    pub name: String,
}

/// Represents an extended file information response from Google Drive API.
#[derive(Deserialize, Debug)]
pub struct ExtendedFileInfoResponse {
    pub id: String,
    pub name: String,
    #[serde(alias = "mimeType")]
    pub mime_type: String,
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

/// Represents the response from a file list request in Google Drive API.
#[derive(Deserialize)]
pub struct FileListResponse {
    // TODO fix this
    // pub(crate) kind: String,
    // pub(crate) incomplete_search: bool,
    pub files: Vec<ExtendedFileInfoResponse>,
}

/// Google Drive api wrapper
pub struct GoogleDriveApi {
    pub(crate) api_key: String,
}

impl GoogleDriveApi {
    ///
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// Get the list of files from a specific GDrive folder.
    pub async fn get_dir_content(
        &self,
        dir_id: &str,
    ) -> Result<FileListResponse, String> {
        let response = reqwest::get(
            format!(
                "https://www.googleapis.com/drive/v3/files?\
                     key={}&q='{dir_id}'%20in%20parents&\
                     fields=files/id,files/name,files/mimeType&\
                     {GDRIVE_PUBLIC_PARAMS}",
                &self.api_key
            )
            .as_str(),
        )
        .await;
        Self::get_result(response).await
    }

    /// Get the details of a single file from GDrive.
    pub async fn get_file_info(
        &self,
        file_id: &FileId,
    ) -> Result<ExtendedFileInfoResponse, String> {
        let response = reqwest::get(
            format!(
                "https://www.googleapis.com/drive/v3/files/{file_id}?
                fields=name&key={}&{GDRIVE_PUBLIC_PARAMS}",
                &self.api_key
            )
            .as_str(),
        )
        .await;
        Self::get_result(response).await
    }

    pub async fn get_file_response(
        &self,
        file_id: &FileId,
    ) -> Result<Response, String> {
        let client = reqwest::ClientBuilder::new()
            .connection_verbose(false)
            .build()
            .map_err(|err| {
                format!("Could not create a reqwest Client: {err}")
            })?;

        Ok(client
            .get(
                format!(
                    "https://www.googleapis.com/drive/v3/files/\
                            {file_id}?alt=media&key={}\
                            &{GDRIVE_PUBLIC_PARAMS}",
                    self.api_key
                )
                .as_str(),
            )
            .send()
            .await
            .map_err(|err| {
                format!("Could not send download request for the file")
            })?)
    }

    async fn get_result<T: for<'de> Deserialize<'de>>(
        response: reqwest::Result<Response>,
    ) -> Result<T, String> {
        match response {
            Err(e) => Err(format!("No valid response from the API: {}", e)),
            Ok(r) => {
                let status = r.status();
                match status {
                    StatusCode::OK => r.json::<T>().await.map_err(|e| {
                        format!("Error parsing JSON result: {}", e)
                    }),
                    StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => Err(r
                        .json::<ErrorResponse>()
                        .await
                        .map(|x| {
                            format!(
                                "Http response: {} {}",
                                x.error.code, x.error.message
                            )
                        })
                        .map_err(|e| {
                            format!("Error parsing JSON result: {}", e)
                        })?),
                    _ => Err(r
                        .text()
                        .await
                        .unwrap_or_else(|e| format!("Unknown error: {}", e))),
                }
            }
        }
    }
}

/// Represents the error response from Google Drive API.
#[derive(Deserialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ErrorMessage,
}

/// Represents an error message in the error response from Google Drive API.
#[derive(Deserialize)]
pub(crate) struct ErrorMessage {
    pub(crate) code: u16,
    pub(crate) message: String,
}
