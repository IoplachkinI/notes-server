mod config;
mod handlers;
mod proxy;

use axum::Router;
use axum::routing::any;
use axum_server::tls_rustls::RustlsConfig;
use proxy::Proxy;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = config::load_config().expect("failed to locate or load config file");
    tracing::info!("Successfully loaded side-car config");

    tracing::info!("Configured upstream: {:?}", cfg.upstream);

    let proxy = Arc::new(Proxy::new(cfg.upstream));

    let router = Router::new()
        .route("/{*path}", any(handlers::proxy_handler))
        .with_state(proxy.clone())
        .layer(TraceLayer::new_for_http());

    let grpc_router = Router::new()
        .route("/{*path}", any(handlers::grpc_proxy_handler))
        .with_state(proxy)
        .layer(TraceLayer::new_for_http());

    // Check for TLS certificate files
    let cert_path =
        std::env::var("TLS_CERT_PATH").unwrap_or_else(|_| "certs/servercert.pem".to_string());
    let key_path =
        std::env::var("TLS_KEY_PATH").unwrap_or_else(|_| "certs/serverkey.pem".to_string());

    if !(fs::metadata(&cert_path).is_ok() && fs::metadata(&key_path).is_ok()) {
        tracing::error!("No TLS certificates found! Aborting");
        panic!("No tls certificates found")
    };

    let rest_addr: SocketAddr = format!("0.0.0.0:{}", cfg.rest_port)
        .parse()
        .expect("Failed to parse REST address");
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port)
        .parse()
        .expect("Failed to parse gRPC address");

    tracing::info!(
        "Loading TLS certificates from {} and {}",
        cert_path,
        key_path
    );
    let tls_config = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .expect("Failed to load TLS certificates");

    tracing::info!("HTTPS side-car listening on {}", rest_addr);
    tracing::info!("HTTPS gRPC side-car listening on {}", grpc_addr);

    // Run both HTTPS side-cars concurrently
    tokio::select! {
        result = axum_server::bind_rustls(rest_addr, tls_config.clone())
            .serve(router.into_make_service()) => {
            if let Err(e) = result {
                tracing::error!("HTTPS server error: {e}");
                panic!("failed to start HTTPS server: {e}");
            }
        }
        result = axum_server::bind_rustls(grpc_addr, tls_config)
            .serve(grpc_router.into_make_service()) => {
            if let Err(e) = result {
                tracing::error!("HTTPS gRPC server error: {e}");
                panic!("failed to start HTTPS gRPC server: {e}");
            }
        }
    }
}
