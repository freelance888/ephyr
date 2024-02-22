use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tests {
    pub requests: String,
    pub errors: String,
    pub redirects: String,
    pub _vhost: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Urls {
    pub versions: String,
    pub summaries: String,
    pub rusages: String,
    pub self_proc_stats: String,
    pub system_proc_stats: String,
    pub meminfos: String,
    pub authors: String,
    pub features: String,
    pub requests: String,
    pub vhosts: String,
    pub streams: String,
    pub clients: String,
    pub raw: String,
    pub clusters: String,
    pub perf: String,
    pub tcmalloc: String,
}
