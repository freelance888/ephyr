use super::common::{Kbps, Publish};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub vhost: String,
    pub app: String,
    #[serde(rename = "tcUrl")]
    pub tc_url: String,
    pub url: String,
    pub live_ms: i64,
    pub clients: i64,
    pub frames: i64,
    pub send_bytes: i64,
    pub recv_bytes: i64,
    pub kbps: Kbps,
    pub publish: Publish,
    pub video: Option<()>,
    pub audio: Option<()>,
}
