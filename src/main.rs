mod dto;
mod handlers;
mod models;
mod repository;
mod service;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};

use std::{env, sync::Arc};

use handlers::rest;
use repository::Repository;

use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use service::NoteService;

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
    let service = NoteService::new(repo_ptr.clone());

    // Router config
    let app = Router::new()
        .route("/", get(root))
        .route("/notes", post(rest::create_note))
        .route("/notes/{id}", put(rest::update_note))
        .route("/notes/{id}", delete(rest::delete_note))
        .route("/notes/{id}", get(rest::get_one_note))
        .route("/notes", get(rest::get_all_notes))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", rest::ApiDoc::openapi()))
        .with_state(Arc::new(service))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    // Starting router
    let addr = listener.local_addr().unwrap();
    tracing::info!("Server starting, listening on {}", addr);
    tracing::info!("Server is ready to accept connections");

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        tracing::error!("Server error: {e}");
        panic!("failed to start server: {e}");
    });
}

async fn root() -> Response {
    (StatusCode::OK, "Hello world!").into_response()
}
