//! Definitions of [google.drive][1] site [API V3][2] and a client to request it.
//!
//! [1]: https://drive.google.com
//! [2]: https://developers.google.com/drive/api/reference/rest/v3

#![deny(
    rustdoc::broken_intra_doc_links,
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

use derive_more::{Display, Error};
use serde::de::DeserializeOwned;
use std::env;
use url::Url;

const GDRIVE_PUBLIC_PARAMS: &str = "supportsAllDrives=True\
&supportsTeamDrives=True\
&includeItemsFromAllDrives=True\
&includeTeamDriveItems=True";

/// Possible errors of performing [`GoogleDriveApi`] requests on [V3 API].
///
/// [V3 API]: https://developers.google.com/drive/api/reference/rest/v3
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {_0}")]
    RequestFailed(reqwest::Error),

    /// [`GoogleDriveApi`] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    #[display(fmt = "API responded with bad status: {_0}")]
    BadStatus(#[error(not(source))] reqwest::StatusCode),

    /// [`GoogleDriveApi`] responded with a bad body, which cannot be deserialized.
    #[display(fmt = "Failed to decode API response: {_0}")]
    BadBody(reqwest::Error),

    /// [`GoogleDriveApi`] responded with an error from Google API.
    #[display(fmt = "Google Drive API responded with an error: {} {}", _0.code, _0.message)]
    ApiError(responses::ApiErrorMessage),
}

/// API Responses structs mappers
pub mod responses {
    use derive_more::{Display, Error};
    use mime::Mime;
    use serde::Deserialize;
    use std::fmt::{Display, Formatter};

    /// Represents file info fetched from Google Drive API.
    #[derive(Deserialize, Debug)]
    pub struct FileInfo {
        /// ID of this [`FileInfo`] file.
        pub id: String,
        /// Name of this [`FileInfo`] file.
        pub name: String,
        /// [MIME type][1] of this [`FileInfo`] file.
        ///
        /// [1]: https://en.wikipedia.org/wiki/Media_type
        #[serde(alias = "mimeType", with = "mime_serde_shim")]
        pub mime_type: Mime,
    }

    impl FileInfo {
        /// Returns `true` if current object is directory otherwise `false`
        pub fn is_dir(&self) -> bool {
            self.mime_type == "application/vnd.google-apps.folder"
        }

        /// Returns `true` if current object is video file otherwise `false`
        pub fn is_video(&self) -> bool {
            self.mime_type.type_() == mime::VIDEO
        }
    }

    /// Represents the file info list fetched from Google Drive API.
    #[derive(Deserialize, Debug)]
    pub struct FileInfoList {
        /// List files or folders
        pub files: Vec<FileInfo>,
    }

    /// Represents the error response from Google Drive API.
    #[derive(Deserialize, Debug, Error, Display)]
    pub struct ApiError {
        /// Encapsulate Google Drive API error
        pub error: ApiErrorMessage,
    }

    /// Represents an error message in the error response from Google Drive API.
    #[derive(Deserialize, Debug, Error)]
    pub struct ApiErrorMessage {
        /// Google Drive API error code
        pub code: u16,
        /// Google Drive API error message
        pub message: String,
    }

    impl Display for ApiErrorMessage {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "ErrorMessage({}, {})", self.code, self.message)
        }
    }
}

async fn req(url: String) -> Result<reqwest::Response, Error> {
    dbg!(&url.to_string());
    let resp = reqwest::get(url).await.map_err(Error::RequestFailed)?;
    let status = resp.status();
    if !status.is_success() {
        if status == reqwest::StatusCode::BAD_REQUEST
            || status == reqwest::StatusCode::UNAUTHORIZED
        {
            // Try to deserialize the error response
            if let Ok(error_resp) = resp.json::<responses::ApiError>().await {
                return Err(Error::ApiError(error_resp.error));
            }
        }
        return Err(Error::BadStatus(status));
    }

    Ok(resp)
}

async fn req_json<T: DeserializeOwned>(url: String) -> Result<T, Error> {
    let resp = req(url).await?;
    resp.json::<T>().await.map_err(Error::BadBody)
}

/// The [V3 API files][1] resource.
///
/// [1]: https://developers.google.com/drive/api/reference/rest/v3/files
#[derive(Debug)]
pub struct Files {
    api_url: Url,
}

impl Files {
    /// Get the details of a single file.
    pub async fn get_file_info(
        &self,
        file_id: &str,
    ) -> Result<responses::FileInfo, Error> {
        let mut url = self.api_url.clone();
        _ = url.path_segments_mut().unwrap().push(file_id);
        let url = format!("{url}&fields=id,name,mimeType");
        req_json::<responses::FileInfo>(url).await
    }

    /// Get the list of files from a specific folder.
    pub async fn get_dir_content(
        &self,
        dir_id: &str,
    ) -> Result<responses::FileInfoList, Error> {
        let url = format!("{}&q='{dir_id}'%20in%20parents&fields=files/id,files/name,files/mimeType",
                          &self.api_url);
        req_json::<responses::FileInfoList>(url).await
    }

    /// Get file binary representation
    pub async fn get_file_response(
        &self,
        file_id: &str,
    ) -> Result<reqwest::Response, Error> {
        let mut url = self.api_url.clone();
        _ = url.path_segments_mut().unwrap().push(file_id);
        let url = format!("{url}&alt=media");
        req(url).await
    }
}

/// [Google Drive API V3][1] wrapper
///
/// [1]: https://developers.google.com/drive/api/reference/rest/v3
#[derive(Clone, Debug)]
pub struct GoogleDriveApi {
    /// Google API Token with activated Google Drive V3 API
    pub api_key: String,
}

impl GoogleDriveApi {
    /// URL of the Google Drive [API V3][1].
    ///
    /// [1]: https://developers.google.com/drive/api/reference/rest/v3
    pub const V3_URL: &'static str = "https://www.googleapis.com/drive/v3";

    /// Create new instance of [GoogleDriveApi]
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    /// Create new instance of [GoogleDriveApi] based on `GOOGLE_DRIVE_API_KEY` env value.
    pub fn new_from_env() -> Self {
        let api_key = env::var("GOOGLE_DRIVE_API_KEY").unwrap();
        Self { api_key }
    }

    /// Returns [Files] that encapsulate [V3 API files][1].
    ///
    /// [1]: https://developers.google.com/drive/api/reference/rest/v3/files
    pub fn files(&self) -> Files {
        let mut api_url =
            Url::parse(&format!("{}/files", GoogleDriveApi::V3_URL)).unwrap();
        api_url.set_query(Some(&format!(
            "key={}&{GDRIVE_PUBLIC_PARAMS}",
            self.api_key
        )));
        Files { api_url }
    }
}

#[cfg(test)]
mod spec {
    //! Test API with [Google Drive Dir][1]
    //!
    //! [1]: https://drive.google.com/drive/folders/17tRXMWEO3ZxBlhqxMLPnM4Dd-eRIaSAl
    use super::*;

    const DRIVE_FOLDER_ID: &'static str = "17tRXMWEO3ZxBlhqxMLPnM4Dd-eRIaSAl";
    const DRIVE_FILE_ID: &'static str = "1uN6QrrS05Hm_6L7XJxINbKEam_RRFr9X";
    const DRIVE_FILE_NAME: &'static str = "out 2023-06-13 KF.mp4";

    #[tokio::test]
    async fn retrieves_dir_content() {
        let res = GoogleDriveApi::new_from_env()
            .files()
            .get_dir_content(DRIVE_FOLDER_ID)
            .await
            .unwrap();
        assert_eq!(res.files.len(), 2);
    }

    #[tokio::test]
    async fn retrieves_file_info() {
        let res = GoogleDriveApi::new_from_env()
            .files()
            .get_file_info(DRIVE_FILE_ID)
            .await
            .unwrap();
        assert_eq!(res.id, DRIVE_FILE_ID);
        assert_eq!(res.name, DRIVE_FILE_NAME);
        assert_eq!(res.mime_type.type_(), mime::VIDEO);
    }

    #[tokio::test]
    async fn retrieves_file_response() {
        let res = GoogleDriveApi::new_from_env()
            .files()
            .get_file_response(DRIVE_FILE_ID)
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
}
