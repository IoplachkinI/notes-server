use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use utoipa::OpenApi;

use std::sync::Arc;

use crate::{
    dto::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
    service::NoteService,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_note,
        update_note,
        delete_note,
        get_one_note,
        get_all_notes
    ),
    components(schemas(
        NoteResponse,
        CreateNoteRequest,
        UpdateNoteRequest
    )),
    tags(
        (name = "notes", description = "Notes management API")
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    post,
    path = "/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created successfully", body = NoteResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "notes"
)]
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

#[utoipa::path(
    put,
    path = "/notes/{id}",
    params(
        ("id" = i64, Path, description = "Note ID")
    ),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, description = "Note updated successfully", body = NoteResponse),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "notes"
)]
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

#[utoipa::path(
    delete,
    path = "/notes/{id}",
    params(
        ("id" = i64, Path, description = "Note ID")
    ),
    responses(
        (status = 204, description = "Note deleted successfully"),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "notes"
)]
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

#[utoipa::path(
    get,
    path = "/notes/{id}",
    params(
        ("id" = i64, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Note found", body = NoteResponse),
        (status = 404, description = "Note not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "notes"
)]
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

#[utoipa::path(
    get,
    path = "/notes",
    responses(
        (status = 200, description = "List of all notes", body = Vec<NoteResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "notes"
)]
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
