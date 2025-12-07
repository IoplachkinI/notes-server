use crate::config::Config;
use reqwest::Client;
use std::sync::atomic::AtomicU32;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Instance {
    url: String,
    con_timeout: Duration,
    health_check_time_limit: Duration,

    pub con_count: AtomicU32,
    is_alive: bool,
    last_healthy: Option<Instant>,
}

impl Instance {
    pub fn new(url: String, cfg: &Config) -> Self {
        Self {
            url,
            con_timeout: cfg.connection_timeout,
            health_check_time_limit: cfg.health_check_time_limit,
            con_count: AtomicU32::default(),
            is_alive: true,
            last_healthy: None,
        }
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub async fn health_check(&mut self) {
        let client = Client::builder()
            .timeout(self.con_timeout)
            .build()
            .expect("failed to initialize a client");

        let health_url = format!("{}/", self.url);
        match client.get(&health_url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    tracing::warn!(
                        "Server {} responded but the status code is {}",
                        self.url,
                        response.status().as_str()
                    )
                }
                self.is_alive = true;
                self.last_healthy = Some(Instant::now())
            }
            Err(_) => {
                if let Some(lh) = self.last_healthy
                    && Instant::now().duration_since(lh) > self.health_check_time_limit
                {
                    tracing::warn!("Server {} is not responding", self.url);
                    self.is_alive = false;
                }
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }
}
