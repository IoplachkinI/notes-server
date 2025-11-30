use crate::{dto, repository::Repository};

use std::sync::Arc;

#[derive(Clone)]
pub struct NoteService {
    repo: Arc<tokio::sync::Mutex<Repository>>,
}

impl NoteService {
    pub fn new(repo: Arc<tokio::sync::Mutex<Repository>>) -> Self {
        Self { repo }
    }

    pub async fn create_note(&self, note: dto::Note) -> Result<dto::Note, tokio_postgres::Error> {
        self.repo
            .lock()
            .await
            .create_note(note.content)
            .await
            .map(|note| dto::Note {
                id: note.id,
                content: note.content,
            })
    }
}
