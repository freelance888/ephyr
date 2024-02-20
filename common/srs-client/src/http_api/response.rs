use crate::http_api::{
    client::Client,
    common::Version,
    feature::FeaturesData,
    meminfos::MemInfos,
    rusages::Rusages,
    self_proc_stats::SelfProcStats,
    stream::Stream,
    summary::{Tests, Urls},
    system_proc_stats::SystemProcStats,
    vhost::Vhost,
};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SrsClientResp {
    pub code: i64,
    pub server: String,
    pub service: String,
    pub pid: String,
    #[serde(flatten)]
    pub data: SrsClientRespData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SrsClientRespData {
    Streams { streams: Vec<Stream> },
    Clients { clients: Vec<Client> },
    Vhosts { vhosts: Vec<Vhost> },
    Summary { urls: Box<Urls>, tests: Tests },
    Version { data: Version },
    Feature { data: FeaturesData },
    Rusages { data: Rusages },
    SelfProcStats { data: Box<SelfProcStats> },
    SystemProcStats { data: SystemProcStats },
    MemInfos { data: MemInfos },
}
