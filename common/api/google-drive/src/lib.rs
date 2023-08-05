//! Definitions of [google.drive][1] site API and a client to request it.
//!
//! [1]: https://drive.google.com

#![deny(
    broken_intra_doc_links,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

use mime::Mime;
use reqwest::{Response, StatusCode};
use serde::Deserialize;

const GDRIVE_PUBLIC_PARAMS: &str = "supportsAllDrives=True\
&supportsTeamDrives=True\
&includeItemsFromAllDrives=True\
&includeTeamDriveItems=True";
//
// /// Source file of a [`Video`].
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Source {
//     /// [URL] of this [`Source`] file, where it can be read from.
//     ///
//     /// [URL]: https://en.wikipedia.org/wiki/URL
//     pub src: Url,
//
//     /// [MIME type][1] of this [`Source`] file.
//     ///
//     /// [1]: https://en.wikipedia.org/wiki/Media_type
//     #[serde(with = "mime_serde_shim")]
//     pub r#type: Mime,
// }

/// Represents an extended file information response from Google Drive API.
#[derive(Deserialize, Debug)]
pub struct ExtendedFileInfoResponse {
    /// ID of file on the Google Drive
    pub id: String,
    /// Name of file on the Google Drive
    pub name: String,
    /// [MIME type][1] of this [`ExtendedFileInfoResponse`] file.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Media_type
    #[serde(alias = "mimeType", with = "mime_serde_shim")]
    pub mime_type: Mime,
}

impl ExtendedFileInfoResponse {
    /// Returns `true` if current object is directory otherwise `false`
    pub fn is_dir(&self) -> bool {
        self.mime_type == "application/vnd.google-apps.folder"
    }

    /// Returns `true` if current object is video file otherwise `false`
    pub fn is_video(&self) -> bool {
        self.mime_type.type_() == mime::VIDEO
    }
}

/// Represents the response from a file list request in Google Drive API.
#[derive(Deserialize, Debug)]
pub struct FileListResponse {
    /// List files or folders
    pub files: Vec<ExtendedFileInfoResponse>,
}

/// Google Drive api wrapper
#[derive(Clone, Debug)]
pub struct GoogleDriveApi {
    /// Google API Token with activated Google Drive V3 API
    pub api_key: String,
}

impl GoogleDriveApi {
    /// URL of the [google.drive][1] site API v1.
    ///
    /// [1]: https://drive.google.com
    pub const V3_URL: &'static str = "https://www.googleapis.com/drive/v3";

    /// Create new instance of [GoogleDriveApi]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// Get the list of files from a specific `GDrive` folder.
    pub async fn get_dir_content(
        &self,
        dir_id: &str,
    ) -> Result<FileListResponse, String> {
        let response = reqwest::get(
            format!(
                "{}/files?key={}&q='{dir_id}'%20in%20parents&\
                     fields=files/id,files/name,files/mimeType&\
                     {GDRIVE_PUBLIC_PARAMS}",
                GoogleDriveApi::V3_URL,
                &self.api_key
            )
            .as_str(),
        )
        .await;
        Self::get_result(response).await
    }

    /// Get the details of a single file from `GDrive`.
    pub async fn get_file_info(
        &self,
        file_id: &str,
    ) -> Result<ExtendedFileInfoResponse, String> {
        let response = reqwest::get(
            format!(
                "{}/files/{file_id}?fields=name&key={}&{GDRIVE_PUBLIC_PARAMS}",
                GoogleDriveApi::V3_URL,
                &self.api_key
            )
            .as_str(),
        )
        .await;
        Self::get_result(response).await
    }

    /// Get file binary representation
    pub async fn get_file_response(
        &self,
        file_id: &str,
    ) -> Result<Response, String> {
        let client = reqwest::ClientBuilder::new()
            .connection_verbose(false)
            .build()
            .map_err(|err| {
                format!("Could not create a reqwest Client: {err}")
            })?;

        client
            .get(
                format!(
                    "{}/files/{file_id}?alt=media&key={}\
                &{GDRIVE_PUBLIC_PARAMS}",
                    GoogleDriveApi::V3_URL,
                    self.api_key
                )
                .as_str(),
            )
            .send()
            .await
            .map_err(|err| {
                format!("Could not send download request for the file {err}")
            })
    }

    async fn get_result<T: for<'de> Deserialize<'de>>(
        response: reqwest::Result<Response>,
    ) -> Result<T, String> {
        match response {
            Err(e) => Err(format!("No valid response from the API: {e}")),
            Ok(r) => {
                let status = r.status();
                match status {
                    StatusCode::OK => r
                        .json::<T>()
                        .await
                        .map_err(|e| format!("Error parsing JSON result: {e}")),
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
                            format!("Error parsing JSON result: {e}")
                        })?),
                    _ => Err(r
                        .text()
                        .await
                        .unwrap_or_else(|e| format!("Unknown error: {e}"))),
                }
            }
        }
    }
}

/// Represents the error response from Google Drive API.
#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    /// Encapsulate Google Drive API error
    pub error: ErrorMessage,
}

/// Represents an error message in the error response from Google Drive API.
#[derive(Deserialize, Debug)]
pub struct ErrorMessage {
    /// Google Drive API error code
    pub code: u16,
    /// Google Drive API error message
    pub message: String,
}
