use serde::{Deserialize, Serialize};

#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Features {
    pub ssl: bool,
    pub hls: bool,
    pub hds: bool,
    pub callback: bool,
    pub api: bool,
    pub httpd: bool,
    pub dvr: bool,
    pub transcode: bool,
    pub ingest: bool,
    pub stat: bool,
    pub caster: bool,
    pub complex_send: bool,
    pub tcp_nodelay: bool,
    pub so_sendbuf: bool,
    pub mr: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeaturesData {
    pub options: String,
    pub options2: String,
    pub build: String,
    pub build2: String,
    pub features: Features,
}
