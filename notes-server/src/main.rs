mod dto;
mod handlers;
mod models;
mod repository;
mod service;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{any, delete, get, post, put},
};

use std::{env, sync::Arc};

use handlers::rest;
use repository::Repository;

use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use service::NoteService;

use crate::handlers::{grpc, soap};

#[tokio::main]
async fn main() {
    // Log setup
    tracing_subscriber::fmt::init();

    // Fetch env variables
    let database_dsn =
        env::var("PG_DSN").expect("database dsn must be provided as an ENV variable");

    // Repository creation and migration
    let repo = Repository::new(database_dsn).await.unwrap_or_else(|e| {
        tracing::error!("Failed to establish database connection: {e}");
        panic!("failed to establish database connection: {e}");
    });
    let repo_ptr = Arc::new(tokio::sync::Mutex::new(repo));

    repo_ptr.lock().await.migrate().await.unwrap_or_else(|e| {
        tracing::error!("Failed to migrate database: {e}");
        panic!("failed to migrate database: {e}");
    });

    // Service creation
    let service = Arc::new(NoteService::new(repo_ptr.clone()));

    // REST router config
    let rest_router = Router::new()
        .route("/", get(root))
        .route("/notes", post(rest::create_note))
        .route("/notes/{id}", put(rest::update_note))
        .route("/notes/{id}", delete(rest::delete_note))
        .route("/notes/{id}", get(rest::get_one_note))
        .route("/notes", get(rest::get_all_notes))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .config(utoipa_swagger_ui::Config::new([
                    "/rest/api-doc/openapi.json",
                ]))
                .url("/api-doc/openapi.json", rest::ApiDoc::openapi()),
        )
        .with_state(service.clone())
        .layer(TraceLayer::new_for_http());

    // SOAP router config
    let soap_router = Router::new()
        .route("/", post(soap::handle_request))
        .with_state(service.clone())
        .layer(TraceLayer::new_for_http());

    let router = Router::new()
        .route("/", any(root))
        .nest("/rest", rest_router)
        .nest("/soap", soap_router);

    let http_listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let http_addr = http_listener.local_addr().unwrap();

    // gRPC server setup
    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_service = grpc::create_grpc_server(service.clone());

    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc_service)
        .serve(grpc_addr);

    tracing::info!("REST/SOAP server starting, listening on {}", http_addr);
    tracing::info!("gRPC server starting, listening on {}", grpc_addr);
    tracing::info!("Servers are ready to accept connections");

    // Run both servers concurrently
    tokio::select! {
        result = axum::serve(http_listener, router) => {
            if let Err(e) = result {
                tracing::error!("HTTP server error: {e}");
                panic!("failed to start HTTP server: {e}");
            }
        }
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {e}");
                panic!("failed to start gRPC server: {e}");
            }
        }
    }
}

async fn root() -> Response {
    (StatusCode::OK, "Hello world!").into_response()
}
