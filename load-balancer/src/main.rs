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

    tracing::info!("Configured upstreams: {:?}", cfg.urls);

    for url in cfg.urls.iter() {
        instances_vec.push(Instance::new(url.to_string(), &cfg));
    }

    let balancer = LoadBalancer::new(Arc::new(RwLock::new(instances_vec)), &cfg);

    {
        let balancer = balancer.clone();
        tokio::spawn(async move {
            balancer.health_check_all().await;
        });
    }

    let router = Router::new()
        .route("/{*path}", any(proxy_handler))
        .with_state(balancer)
        .layer(TraceLayer::new_for_http());

    let url = format!("0.0.0.0:{}", cfg.port);
    let listener = TcpListener::bind(url.clone())
        .await
        .expect("Failed to bind to address");

    tracing::info!("Load balancer listening on {}", url);

    axum::serve(listener, router).await.expect("Server failed");
}
