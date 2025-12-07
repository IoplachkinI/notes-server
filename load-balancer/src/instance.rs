use crate::config::Config;
use reqwest::Client;
use std::sync::atomic::AtomicU32;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Instance {
    base_url: String,
    rest_port: u16,
    grpc_port: u16,
    con_timeout: Duration,
    health_check_time_limit: Duration,

    pub con_count: AtomicU32,
    is_alive: bool,
    last_healthy: Option<Instant>,
}

impl Instance {
    pub fn new(instance_config: &crate::config::InstanceConfig, cfg: &Config) -> Self {
        Self {
            base_url: instance_config.base_url.clone(),
            rest_port: instance_config.rest_port,
            grpc_port: instance_config.grpc_port,
            con_timeout: cfg.connection_timeout,
            health_check_time_limit: cfg.health_check_time_limit,
            con_count: AtomicU32::default(),
            is_alive: true,
            last_healthy: None,
        }
    }

    pub fn get_rest_url(&self) -> String {
        format!("{}:{}", self.base_url, self.rest_port)
    }

    pub fn get_grpc_url(&self) -> String {
        format!("{}:{}", self.base_url, self.grpc_port)
    }

    pub async fn health_check(&mut self) {
        let client = Client::builder()
            .timeout(self.con_timeout)
            .build()
            .expect("failed to initialize a client");

        let rest_url = self.get_rest_url();
        let health_url = format!("{}/", rest_url);
        match client.get(&health_url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    tracing::warn!(
                        "Server {} responded but the status code is {}",
                        rest_url,
                        response.status().as_str()
                    );
                    return;
                }
                if !self.is_alive {
                    tracing::info!("Restored connection to server {}", rest_url);
                }
                self.is_alive = true;
                self.last_healthy = Some(Instant::now())
            }
            Err(_) => {
                if let Some(lh) = self.last_healthy
                    && Instant::now().duration_since(lh) > self.health_check_time_limit
                {
                    if self.is_alive {
                        tracing::warn!("Lost connection to server {}", rest_url);
                    }
                    self.is_alive = false;
                }
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }
}
