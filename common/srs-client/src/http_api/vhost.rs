use super::common::{Hls, Kbps};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Vhost {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub clients: i64,
    pub streams: i64,
    pub send_bytes: i64,
    pub recv_bytes: i64,
    pub kbps: Kbps,
    pub hls: Hls,
}
