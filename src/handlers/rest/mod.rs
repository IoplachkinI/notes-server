use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;

use std::sync::Arc;

use crate::{dto::CreateNoteRequest, dto::UpdateNoteRequest, service::NoteService};

#[debug_handler]
pub async fn create_note(
    State(service): State<Arc<NoteService>>,
    Json(payload): Json<CreateNoteRequest>,
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
    Path(id): Path<i64>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Response {
    match service.update_note(id, payload).await {
        Ok(Some(note)) => (StatusCode::OK, Json(note)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Note not found").into_response(),
        Err(e) => {
            tracing::error!("failed to update note entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update note").into_response()
        }
    }
}

#[debug_handler]
pub async fn delete_note(State(service): State<Arc<NoteService>>, Path(id): Path<i64>) -> Response {
    match service.delete_note(id).await {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Note not found").into_response(),
        Err(e) => {
            tracing::error!("failed to delete note entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete note").into_response()
        }
    }
}

#[debug_handler]
pub async fn get_one_note(
    State(service): State<Arc<NoteService>>,
    Path(id): Path<i64>,
) -> Response {
    match service.get_one_note(id).await {
        Ok(Some(note)) => (StatusCode::OK, Json(note)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Note not found").into_response(),
        Err(e) => {
            tracing::error!("failed to get note entry: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get note").into_response()
        }
    }
}

#[debug_handler]
pub async fn get_all_notes(State(service): State<Arc<NoteService>>) -> Response {
    match service.get_all_notes().await {
        Ok(note) => (StatusCode::OK, Json(note)).into_response(),
        Err(e) => {
            tracing::error!("failed to get note entries: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get all notes").into_response()
        }
    }
}
