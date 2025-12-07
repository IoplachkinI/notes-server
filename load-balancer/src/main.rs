mod balancer;
mod config;
mod instance;
mod strategy;

use axum::{
    Router,
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::any,
};
use axum_macros::debug_handler;
use balancer::LoadBalancer;
use config::Config;
use instance::Instance;
use std::fs;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::trace::TraceLayer;

#[debug_handler]
async fn proxy_handler(State(balancer): State<LoadBalancer>, request: Request) -> Response {
    match balancer.forward_request(request).await {
        Ok(response) => response,
        Err(status) => (status, "Service unavailable (no alive servers)").into_response(),
    }
}

#[debug_handler]
async fn grpc_proxy_handler(State(balancer): State<LoadBalancer>, request: Request) -> Response {
    match balancer.forward_grpc_request(request).await {
        Ok(response) => response,
        Err(status) => (status, "Service unavailable (no alive servers)").into_response(),
    }
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = load_config("config.yaml").expect("failed to locate or load config file");
    tracing::info!("Successfully loaded balancer config");

    let mut instances_vec: Vec<Instance> = Vec::new();

    tracing::info!("Configured upstreams: {:?}", cfg.instances);

    for instance_config in cfg.instances.iter() {
        instances_vec.push(Instance::new(instance_config, &cfg));
    }

    let balancer = LoadBalancer::new(Arc::new(RwLock::new(instances_vec)), &cfg);

    {
        let balancer = balancer.clone();
        tokio::spawn(async move {
            balancer.health_check_all().await;
        });
    }

    let router = Router::new()
        .route("/", any(root))
        .route("/{*path}", any(proxy_handler))
        .with_state(balancer.clone())
        .layer(TraceLayer::new_for_http());

    let grpc_router = Router::new()
        .route("/{*path}", any(grpc_proxy_handler))
        .with_state(balancer)
        .layer(TraceLayer::new_for_http());

    let url = format!("0.0.0.0:{}", cfg.rest_port);
    let listener = TcpListener::bind(url.clone())
        .await
        .expect("Failed to bind to address");

    let grpc_url = format!("0.0.0.0:{}", cfg.grpc_port);
    let grpc_listener = TcpListener::bind(grpc_url.clone())
        .await
        .expect("Failed to bind to gRPC address");

    tracing::info!("HTTP Load balancer listening on {}", url);
    tracing::info!("gRPC Load balancer listening on {}", grpc_url);

    // Run both servers concurrently
    tokio::select! {
        result = axum::serve(listener, router) => {
            if let Err(e) = result {
                tracing::error!("HTTP server error: {e}");
                panic!("failed to start HTTP server: {e}");
            }
        }
        result = axum::serve(grpc_listener, grpc_router) => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {e}");
                panic!("failed to start gRPC server: {e}");
            }
        }
    }
}

#[debug_handler]
async fn root(State(balancer): State<LoadBalancer>) -> Response {
    let (alive_count, total_count) = balancer.get_health_status().await;

    let status = if alive_count > 0 {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    let body = format!(
        r#"{{"status":"{}","alive_instances":{},"total_instances":{}}}"#,
        if alive_count > 0 {
            "healthy"
        } else {
            "unhealthy"
        },
        alive_count,
        total_count
    );

    (status, body).into_response()
}
