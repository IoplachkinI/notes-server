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

use std::{cell::RefCell, env, sync::Arc};

use handlers::rest;
use repository::Repository;

use tower_http::trace::TraceLayer;

use service::NoteService;

#[tokio::main]
async fn main() {
    // Log setup
    tracing_subscriber::fmt::init();

    // Fetch env variables
    let database_dsn =
        env::var("PG_DSN").expect("database dsn must be provided as an ENV variable");

    // Repository creation and migration
    let repo = Repository::new(database_dsn)
        .await
        .expect("failed to establish database connection");
    let repo_ptr = Arc::new(tokio::sync::Mutex::new(repo));

    repo_ptr
        .lock()
        .await
        .migrate()
        .await
        .expect("failed to migrate database");

    // Service creation
    let service = NoteService::new(repo_ptr.clone());

    // Router config
    let app = Router::new()
        .route("/", get(root))
        .route("/notes", post(rest::create_note))
        .with_state(Arc::new(service))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    // Starting router
    tracing::info!("Started listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}

async fn root() -> Response {
    (StatusCode::OK, "Hello world!").into_response()
}
