use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct InstanceConfig {
    pub base_url: String,
    pub rest_port: u16,
    pub grpc_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub instances: Vec<InstanceConfig>,
    pub rest_port: u32,
    pub grpc_port: u32,
    pub strategy: String,
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub health_check_time_limit: Duration,
    #[serde(with = "humantime_serde")]
    pub connection_timeout: Duration,
    #[serde(default)]
    pub max_retries: Option<u32>, // None means try all alive servers
}
