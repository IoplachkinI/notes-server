use crate::proxy::Proxy;
use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use std::sync::Arc;

#[debug_handler]
pub async fn proxy_handler(State(side_car): State<Arc<Proxy>>, request: Request) -> Response {
    tracing::info!("Forwarding request to inner service");
    match side_car.forward_request(request).await {
        Ok(response) => response,
        Err(status) => (status, "Service unavailable").into_response(),
    }
}

#[debug_handler]
pub async fn grpc_proxy_handler(State(side_car): State<Arc<Proxy>>, request: Request) -> Response {
    tracing::info!("Forwarding request to inner service");
    match side_car.forward_grpc_request(request).await {
        Ok(response) => response,
        Err(status) => (status, "Service unavailable").into_response(),
    }
}
