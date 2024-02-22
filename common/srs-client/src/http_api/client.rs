use super::common::Kbps;
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Client {
    pub id: String,
    pub vhost: String,
    pub stream: String,
    pub ip: String,
    #[serde(rename = "pageUrl")]
    pub page_url: String,
    #[serde(rename = "swfUrl")]
    pub swf_url: String,
    #[serde(rename = "tcUrl")]
    pub tc_url: String,
    pub url: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub publish: bool,
    pub alive: f64,
    pub send_bytes: i64,
    pub recv_bytes: i64,
    pub kbps: Kbps,
}
