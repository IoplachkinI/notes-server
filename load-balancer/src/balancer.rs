use crate::config::Config;
use crate::instance::Instance;
use crate::strategy::{self, InstanceSnapshot};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct LoadBalancer {
    instances: Arc<RwLock<Vec<Instance>>>,
    health_check_interval: Duration,
    con_timeout: Duration,
    max_retries: Option<u32>,
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
            max_retries: cfg.max_retries,
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

    pub async fn get_health_status(&self) -> (usize, usize) {
        let instances = self.instances.read().await;
        let alive_count = instances.iter().filter(|i| i.is_alive()).count();
        let total_count = instances.len();
        (alive_count, total_count)
    }

    async fn try_forward_to_instance(
        &self,
        instance_idx: usize,
        instance_url: &str,
        method: &axum::http::Method,
        path_and_query: &str,
        headers: &axum::http::HeaderMap,
        body_bytes: &[u8],
    ) -> Result<Response, StatusCode> {
        let instances = self.instances.read().await;
        instances[instance_idx]
            .con_count
            .fetch_add(1, Ordering::Relaxed);
        drop(instances);

        let client = reqwest::Client::builder()
            .timeout(self.con_timeout)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let url = format!("{}{}", instance_url, path_and_query);

        let result = tokio::time::timeout(
            self.con_timeout,
            client
                .request(method.clone(), &url)
                .headers(headers.clone())
                .body(body_bytes.to_vec())
                .send(),
        )
        .await;

        let instances = self.instances.read().await;
        instances[instance_idx]
            .con_count
            .fetch_sub(1, Ordering::Relaxed);
        drop(instances);

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_server_error() {
                    return Err(
                        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
                    );
                }

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
            Ok(Err(_)) => Err(StatusCode::BAD_GATEWAY),
            Err(_) => Err(StatusCode::GATEWAY_TIMEOUT),
        }
    }

    pub async fn forward_request(&self, request: Request) -> Result<Response, StatusCode> {
        let (parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let method = parts.method.clone();
        let path_and_query = parts.uri.path_and_query().map(|s| s.as_str()).unwrap_or("");
        let headers = parts.headers;

        let instances = self.instances.read().await;
        let mut alive_snapshots: Vec<(usize, InstanceSnapshot)> = instances
            .iter()
            .enumerate()
            .filter_map(|(idx, i)| {
                if i.is_alive() {
                    Some((
                        idx,
                        InstanceSnapshot {
                            con_count: i.con_count.load(Ordering::Relaxed),
                            is_alive: i.is_alive(),
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        drop(instances);

        if alive_snapshots.is_empty() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        let max_retries = self
            .max_retries
            .unwrap_or(alive_snapshots.len() as u32)
            .min(alive_snapshots.len() as u32);
        let mut tried_indices = std::collections::HashSet::new();

        for attempt in 0..=max_retries {
            if alive_snapshots.is_empty() {
                break;
            }

            let snapshots: Vec<InstanceSnapshot> =
                alive_snapshots.iter().map(|(_, s)| *s).collect();
            let selected_idx_in_snapshot = self.strategy.lock().await.select_instance(&snapshots);

            if selected_idx_in_snapshot >= alive_snapshots.len() {
                tracing::error!("Strategy returned invalid index");
                break;
            }

            let actual_idx = alive_snapshots[selected_idx_in_snapshot].0;

            if tried_indices.contains(&actual_idx) {
                alive_snapshots.remove(selected_idx_in_snapshot);
                continue;
            }

            tried_indices.insert(actual_idx);

            let instances = self.instances.read().await;
            let instance_url = instances[actual_idx].get_rest_url();
            drop(instances);

            tracing::debug!(
                "Attempt {}: Redirecting request to {}",
                attempt + 1,
                instance_url
            );

            match self
                .try_forward_to_instance(
                    actual_idx,
                    &instance_url,
                    &method,
                    path_and_query,
                    &headers,
                    &body_bytes,
                )
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) if e.is_server_error() => {
                    if attempt < max_retries {
                        tracing::warn!(
                            "Request to {} failed: {:?}, trying next server",
                            instance_url,
                            e
                        );
                        alive_snapshots.remove(selected_idx_in_snapshot);
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(StatusCode::SERVICE_UNAVAILABLE)
    }

    async fn try_forward_grpc_to_instance(
        &self,
        instance_idx: usize,
        instance_url: &str,
        method: &axum::http::Method,
        path_and_query: &str,
        headers: &axum::http::HeaderMap,
        body_bytes: &[u8],
    ) -> Result<Response, StatusCode> {
        let instances = self.instances.read().await;
        instances[instance_idx]
            .con_count
            .fetch_add(1, Ordering::Relaxed);
        drop(instances);

        let client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .timeout(self.con_timeout)
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let url = format!("{}{}", instance_url, path_and_query);

        let result = tokio::time::timeout(
            self.con_timeout,
            client
                .request(method.clone(), &url)
                .headers(headers.clone())
                .body(body_bytes.to_vec())
                .send(),
        )
        .await;

        let instances = self.instances.read().await;
        instances[instance_idx]
            .con_count
            .fetch_sub(1, Ordering::Relaxed);
        drop(instances);

        match result {
            Ok(Ok(response)) => {
                let status = response.status();
                if status.is_server_error() {
                    return Err(
                        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
                    );
                }

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
            Ok(Err(_)) => Err(StatusCode::BAD_GATEWAY),
            Err(_) => Err(StatusCode::GATEWAY_TIMEOUT),
        }
    }

    pub async fn forward_grpc_request(
        &self,
        request: axum::extract::Request,
    ) -> Result<axum::response::Response, StatusCode> {
        let (parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let method = parts.method.clone();
        let path_and_query = parts.uri.path_and_query().map(|s| s.as_str()).unwrap_or("");
        let headers = parts.headers;

        let instances = self.instances.read().await;
        let mut alive_snapshots: Vec<(usize, InstanceSnapshot)> = instances
            .iter()
            .enumerate()
            .filter_map(|(idx, i)| {
                if i.is_alive() {
                    Some((
                        idx,
                        InstanceSnapshot {
                            con_count: i.con_count.load(Ordering::Relaxed),
                            is_alive: i.is_alive(),
                        },
                    ))
                } else {
                    None
                }
            })
            .collect();
        drop(instances);

        if alive_snapshots.is_empty() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        let max_retries = self
            .max_retries
            .unwrap_or(alive_snapshots.len() as u32)
            .min(alive_snapshots.len() as u32);
        let mut tried_indices = std::collections::HashSet::new();

        for attempt in 0..=max_retries {
            if alive_snapshots.is_empty() {
                break;
            }

            let snapshots: Vec<InstanceSnapshot> =
                alive_snapshots.iter().map(|(_, s)| *s).collect();
            let selected_idx_in_snapshot = self.strategy.lock().await.select_instance(&snapshots);

            if selected_idx_in_snapshot >= alive_snapshots.len() {
                tracing::error!("Strategy returned invalid index");
                break;
            }

            let actual_idx = alive_snapshots[selected_idx_in_snapshot].0;

            if tried_indices.contains(&actual_idx) {
                alive_snapshots.remove(selected_idx_in_snapshot);
                continue;
            }

            tried_indices.insert(actual_idx);

            let instances = self.instances.read().await;
            let grpc_url = instances[actual_idx].get_grpc_url();
            drop(instances);

            tracing::debug!(
                "Attempt {}: Redirecting gRPC request to {}",
                attempt + 1,
                grpc_url
            );

            match self
                .try_forward_grpc_to_instance(
                    actual_idx,
                    &grpc_url,
                    &method,
                    path_and_query,
                    &headers,
                    &body_bytes,
                )
                .await
            {
                Ok(response) => return Ok(response),
                Err(e) if e.is_server_error() => {
                    if attempt < max_retries {
                        tracing::warn!(
                            "gRPC request to {} failed: {:?}, trying next server",
                            grpc_url,
                            e
                        );
                        alive_snapshots.remove(selected_idx_in_snapshot);
                    } else {
                        return Err(e);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
