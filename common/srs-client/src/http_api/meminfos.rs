use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MemInfos {
    pub ok: bool,
    pub sample_time: i64,
    pub percent_ram: f64,
    pub percent_swap: f64,
    #[serde(rename = "MemActive")]
    pub mem_active: i64,
    #[serde(rename = "RealInUse")]
    pub real_in_use: i64,
    #[serde(rename = "NotInUse")]
    pub not_in_use: i64,
    #[serde(rename = "MemTotal")]
    pub mem_total: i64,
    #[serde(rename = "MemFree")]
    pub mem_free: i64,
    #[serde(rename = "Buffers")]
    pub buffers: i64,
    #[serde(rename = "Cached")]
    pub cached: i64,
    #[serde(rename = "SwapTotal")]
    pub swap_total: i64,
    #[serde(rename = "SwapFree")]
    pub swap_free: i64,
}
