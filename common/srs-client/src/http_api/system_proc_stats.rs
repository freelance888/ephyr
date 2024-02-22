use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemProcStats {
    pub ok: bool,
    pub sample_time: i64,
    pub percent: f64,
    pub user: i64,
    pub nice: i64,
    pub sys: i64,
    pub idle: i64,
    pub iowait: i64,
    pub irq: i64,
    pub softirq: i64,
    pub steal: i64,
    pub guest: i64,
}
