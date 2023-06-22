pub const GDRIVE_PUBLIC_PARAMS: &str = "supportsAllDrives=True\
&supportsTeamDrives=True\
&includeItemsFromAllDrives=True\
&includeTeamDriveItems=True";

use crate::file_manager::FileId;
use reqwest::{Response, StatusCode};
use serde::Deserialize;

/// Used to deserialize Google API call for the file details
#[derive(Deserialize)]
pub struct FileNameResponse {
    /// Name of the file
    pub name: String,
}

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

#[derive(Deserialize)]
pub struct FileListResponse {
    // TODO fix this
    // pub(crate) kind: String,
    // pub(crate) incomplete_search: bool,
    pub files: Vec<ExtendedFileInfoResponse>,
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

pub struct GoogleDriveApi;

impl GoogleDriveApi {
    /// Get list of files from specific GDrive folder
    pub async fn get_dir_content(
        api_key: &str,
        dir_id: &str,
    ) -> Result<FileListResponse, String> {
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
        Self::get_result(response).await
    }

    /// Get single file from GDrive
    pub async fn get_file_info(
        file_id: FileId,
        api_key: &str,
    ) -> Result<ExtendedFileInfoResponse, String> {
        let response = reqwest::get(
            format!(
                "https://www.googleapis.com/drive/v3/files/{file_id}?
                fields=name&key={api_key}&{GDRIVE_PUBLIC_PARAMS}"
            )
            .as_str(),
        )
        .await;
        Self::get_result(response).await
    }

    async fn get_result<T: for<'de> Deserialize<'de>>(
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

            Err(r.text().await.map_err(|e| format!("Unknown error: {e}"))?)
        }
    }
}
