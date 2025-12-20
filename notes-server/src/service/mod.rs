use crate::{
    dto::{CreateNoteRequest, NoteResponse, UpdateNoteRequest},
    models::Note,
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
        id: i64,
        request: UpdateNoteRequest,
    ) -> Result<Option<NoteResponse>, tokio_postgres::Error> {
        self.repo
            .lock()
            .await
            .update_note(id, request.content)
            .await
            .map(|note| {
                note.map(|note| NoteResponse {
                    id: note.id,
                    content: note.content,
                })
            })
    }

    pub async fn delete_note(&self, id: i64) -> Result<bool, tokio_postgres::Error> {
        self.repo.lock().await.delete_note(id).await
    }

    pub async fn get_one_note(
        &self,
        id: i64,
    ) -> Result<Option<NoteResponse>, tokio_postgres::Error> {
        self.repo.lock().await.get_one_note(id).await.map(|note| {
            note.map(|note| NoteResponse {
                id: note.id,
                content: note.content,
            })
        })
    }

    pub async fn get_all_notes(&self) -> Result<Vec<NoteResponse>, tokio_postgres::Error> {
        self.repo.lock().await.get_all_notes().await.map(|notes| {
            notes
                .into_iter()
                .map(|note| NoteResponse {
                    id: note.id,
                    content: note.content,
                })
                .collect()
        })
    }

    pub async fn get_all_notes_with_timestamps(&self) -> Result<Vec<Note>, tokio_postgres::Error> {
        self.repo.lock().await.get_all_notes().await
    }
}
