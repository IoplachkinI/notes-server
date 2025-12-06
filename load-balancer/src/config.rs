use std::time::Duration;

use serde::Deserialize;

use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Config {
    pub urls: Vec<String>,
    pub port: u32,
    pub strategy: String,
    #[serde(with = "humantime_serde")]
    pub health_check_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub health_check_time_limit: Duration,
    #[serde(with = "humantime_serde")]
    pub connection_timeout: Duration,
}
