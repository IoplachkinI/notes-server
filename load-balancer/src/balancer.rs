use crate::config::Config;
use crate::instance::Instance;
use crate::strategy;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct LoadBalancer {
    instances: Arc<RwLock<Vec<Instance>>>,
    health_check_interval: Duration,
    con_timeout: Duration,
    strategy: Arc<Mutex<Box<dyn strategy::BalancingStrategy>>>,
}

impl LoadBalancer {
    pub fn new(instances: Arc<RwLock<Vec<Instance>>>, cfg: &Config) -> Self {
        let strategy: Box<dyn strategy::BalancingStrategy> = match cfg.strategy.as_str() {
            "round_robin" => Box::new(strategy::RoundRobin::new()),
            "least_connections" => Box::new(strategy::LeastConnections::new()),
            _ => Box::new(strategy::Random::new()),
        };
        LoadBalancer {
            instances: instances.clone(),
            health_check_interval: cfg.health_check_interval,
            con_timeout: cfg.connection_timeout,
            strategy: Arc::new(Mutex::new(strategy)),
        }
    }

    pub async fn health_check_all(&self) {
        let mut interval = tokio::time::interval(self.health_check_interval);
        loop {
            interval.tick().await;
            let mut instances = self.instances.write().await;
            for instance in instances.iter_mut() {
                instance.health_check().await;
            }
        }
    }

    pub async fn forward_request(&self, request: Request) -> Result<Response, StatusCode> {
        let instances = self.instances.read().await;
        let alive_instances: Vec<Instance> = instances
            .iter()
            .filter_map(|i| match i.is_alive() {
                true => Some(i.clone()),
                false => None,
            })
            .collect();

        if alive_instances.is_empty() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        let instance = alive_instances[self
            .strategy
            .lock()
            .await
            .select_instance(alive_instances.as_slice())]
        .clone();

        tracing::info!("Redirecting request to {}", instance.get_url());

        let client = reqwest::Client::builder()
            .timeout(self.con_timeout)
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let url = format!(
            "{}{}",
            instance.get_url(),
            request
                .uri()
                .path_and_query()
                .map(|s| s.as_str())
                .unwrap_or("")
        );

        let response = tokio::time::timeout(
            self.con_timeout,
            client
                .request(request.method().clone(), &url)
                .headers(request.headers().clone())
                .body(
                    axum::body::to_bytes(request.into_body(), usize::MAX)
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                )
                .send(),
        )
        .await
        .map_err(|_| StatusCode::GATEWAY_TIMEOUT)?
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = response
            .bytes()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut axum_response = Response::builder()
            .status(status)
            .body(axum::body::Body::from(body_bytes))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        *axum_response.headers_mut() = headers;

        Ok(axum_response)
    }
}
