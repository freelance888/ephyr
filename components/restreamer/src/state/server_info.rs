//! Server's info
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

/// Server's info
#[derive(
    Clone, Debug, Deserialize, Serialize, GraphQLObject, PartialEq, Default,
)]
pub struct ServerInfo {
    /// Total CPU usage, %
    pub cpu_usage: Option<f64>,

    /// Total RAM installed on current machine
    pub ram_total: Option<f64>,

    /// Free (available) RAM
    pub ram_free: Option<f64>,

    /// Network traffic, transferred last second
    pub tx_delta: Option<f64>,

    /// Network traffic, received last second
    pub rx_delta: Option<f64>,

    /// Error message
    pub error_msg: Option<String>,
}

impl ServerInfo {
    /// Updates cpu usage
    pub fn update_cpu(&mut self, cpu: Option<f64>) {
        self.cpu_usage = cpu;
    }

    /// Sets error message
    pub fn set_error(&mut self, msg: Option<String>) {
        self.error_msg = msg;
    }

    /// Updates ram usage
    pub fn update_ram(
        &mut self,
        ram_total: Option<f64>,
        ram_free: Option<f64>,
    ) {
        self.ram_total = ram_total;
        self.ram_free = ram_free;
    }

    /// Updates traffic usage
    pub fn update_traffic_usage(
        &mut self,
        tx_delta: Option<f64>,
        rx_delta: Option<f64>,
    ) {
        self.tx_delta = tx_delta;
        self.rx_delta = rx_delta;
    }
}
