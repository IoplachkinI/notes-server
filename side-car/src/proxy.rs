use crate::config::Upstream;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use std::time::Duration;

#[derive(Clone)]
pub struct Proxy {
    upstream: Upstream,
    client: reqwest::Client,
    grpc_client: reqwest::Client,
}

impl Proxy {
    pub fn new(upstream: Upstream) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let grpc_client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create gRPC client");

        Proxy {
            upstream,
            client,
            grpc_client,
        }
    }

    fn get_rest_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.upstream.base_url, self.upstream.rest_port
        )
    }

    fn get_grpc_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.upstream.base_url, self.upstream.grpc_port
        )
    }

    pub async fn forward_request(&self, request: Request) -> Result<Response, StatusCode> {
        let (parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let method = parts.method;
        let path_and_query = parts.uri.path_and_query().map(|s| s.as_str()).unwrap_or("");
        let headers = parts.headers;

        let upstream_url = format!("{}{}", self.get_rest_url(), path_and_query);

        tracing::debug!("Proxying {} request to {}", method, upstream_url);

        let mut upstream_request = self.client.request(method, &upstream_url);

        // Copy headers (excluding Host header which should be for upstream)
        for (name, value) in headers.iter() {
            if name != "host" {
                upstream_request = upstream_request.header(name, value);
            }
        }

        let response = upstream_request
            .body(body_bytes.to_vec())
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to forward request to {}: {}", upstream_url, e);
                StatusCode::BAD_GATEWAY
            })?;

        let status = response.status();
        tracing::debug!("Upstream response status: {}", status);
        let response_headers = response.headers().clone();
        let response_body = response.bytes().await.map_err(|e| {
            tracing::error!("Failed to read response body from {}: {}", upstream_url, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        tracing::debug!(
            "Successfully read response body, size: {} bytes",
            response_body.len()
        );

        let mut axum_response = Response::builder()
            .status(status)
            .body(axum::body::Body::from(response_body))
            .map_err(|e| {
                tracing::error!("Failed to build response: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Copy headers, but skip ones that should be set by the response builder
        let headers_to_skip = [
            "content-length",
            "transfer-encoding",
            "connection",
            "keep-alive",
        ];
        for (name, value) in response_headers.iter() {
            let name_lower = name.as_str().to_lowercase();
            if !headers_to_skip.contains(&name_lower.as_str()) {
                axum_response.headers_mut().insert(name, value.clone());
            }
        }
        Ok(axum_response)
    }

    pub async fn forward_grpc_request(&self, request: Request) -> Result<Response, StatusCode> {
        let (parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let method = parts.method;
        let path_and_query = parts.uri.path_and_query().map(|s| s.as_str()).unwrap_or("");
        let headers = parts.headers;

        let upstream_url = format!("{}{}", self.get_grpc_url(), path_and_query);

        tracing::debug!("Proxying gRPC {} request to {}", method, upstream_url);

        let mut upstream_request = self.grpc_client.request(method, &upstream_url);

        // Copy headers (excluding Host header which should be for upstream)
        for (name, value) in headers.iter() {
            if name != "host" {
                upstream_request = upstream_request.header(name, value);
            }
        }

        let response = upstream_request
            .body(body_bytes.to_vec())
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to forward gRPC request: {}", e);
                StatusCode::BAD_GATEWAY
            })?;

        let status = response.status();
        let response_headers = response.headers().clone();
        let response_body = response.bytes().await.map_err(|e| {
            tracing::error!("Failed to read gRPC response body: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let mut axum_response = Response::builder()
            .status(status)
            .body(axum::body::Body::from(response_body))
            .map_err(|e| {
                tracing::error!("Failed to build gRPC response: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // Copy headers, but skip ones that should be set by the response builder
        let headers_to_skip = [
            "content-length",
            "transfer-encoding",
            "connection",
            "keep-alive",
        ];
        for (name, value) in response_headers.iter() {
            let name_lower = name.as_str().to_lowercase();
            if !headers_to_skip.contains(&name_lower.as_str()) {
                axum_response.headers_mut().insert(name, value.clone());
            }
        }
        Ok(axum_response)
    }
}
