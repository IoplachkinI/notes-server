use crate::{
    dto::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
    repository::Repository,
};

use std::sync::Arc;

#[derive(Clone)]
pub struct NoteService {
    repo: Arc<tokio::sync::Mutex<Repository>>,
}

impl NoteService {
    pub const fn new(repo: Arc<tokio::sync::Mutex<Repository>>) -> Self {
        Self { repo }
    }

    pub async fn create_note(
        &self,
        request: CreateNoteRequest,
    ) -> Result<NoteResponse, tokio_postgres::Error> {
        self.repo
            .lock()
            .await
            .create_note(request.content)
            .await
            .map(|note| NoteResponse {
                id: note.id,
                content: note.content,
            })
    }

    pub async fn update_note(
        &self,
        request: UpdateNoteRequest,
    ) -> Result<NoteResponse, tokio_postgres::Error> {
        self.repo
            .lock()
            .await
            .update_note(request.id, request.content)
            .await
            .map(|note| NoteResponse {
                id: note.id,
                content: note.content,
            })
    }

    pub async fn delete_note(&self, id: i64) -> Result<bool, tokio_postgres::Error> {
        self.repo.lock().await.delete_note(id).await
    }
}
