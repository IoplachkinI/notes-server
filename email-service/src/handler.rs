use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;

use std::sync::Arc;

use crate::service::{EmailService, EmailServiceError};

use crate::dto::SendEmailRequest;

#[debug_handler]
pub async fn send_email(
    State(service): State<Arc<EmailService>>,
    Json(payload): Json<SendEmailRequest>,
) -> Response {
    match service.send_email(payload).await {
        Ok(r) => (StatusCode::OK, Json(r)).into_response(),
        Err(e) => {
            tracing::error!("Failed to send email: {e}");
            match e {
                EmailServiceError::AddressFormat(_) => {
                    (StatusCode::BAD_REQUEST, Json("Invalid address format")).into_response()
                }
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed to send email"),
                )
                    .into_response(),
            }
        }
    }
}

#[debug_handler]
pub async fn health_check() -> Response {
    (StatusCode::OK, "Hello from email service!").into_response()
}
