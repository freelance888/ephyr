use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Kbps {
    pub recv_30s: i64,
    pub send_30s: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hls {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Publish {
    pub active: bool,
}

#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    major: i64,
    minor: i64,
    revision: i64,
    version: String,
}
