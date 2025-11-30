use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;

use std::{collections::HashMap, sync::Arc};

use crate::{dto::Note, service::NoteService};

#[debug_handler]
pub async fn create_note(
    State(service): State<Arc<NoteService>>,
    Json(payload): Json<Note>,
) -> Response {
    match service.create_note(payload).await {
        Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
        Err(e) => {
            tracing::error!("failed to create note entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create note").into_response()
        }
    }
}

#[debug_handler]
pub async fn update_note(
    State(service): State<Arc<NoteService>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<Note>,
) -> Response {
    match service.create_note(payload).await {
        Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
        Err(e) => {
            tracing::error!("failed to create note entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create note").into_response()
        }
    }
}
