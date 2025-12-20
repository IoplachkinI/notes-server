mod config;
mod dto;
mod handler;
mod service;

use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Log setup
    tracing_subscriber::fmt().init();

    // Load config
    let cfg = config::load_config().expect("failed to locate or load config file");
    tracing::info!("Successfully loaded email service config");

    // Setup service
    let service = service::EmailService::new(cfg.clone());
    let service_ptr = Arc::new(service);

    // Setup router
    let router = Router::new()
        .route("/email", post(handler::send_email))
        .route("/", get(handler::health_check))
        .with_state(service_ptr)
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", cfg.port))
        .await
        .expect("Failed to bind to address");
    let addr = listener.local_addr().unwrap();

    tracing::info!("Email service starting, listening on {}", addr);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
