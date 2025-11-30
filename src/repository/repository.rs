use tokio_postgres::{Client, NoTls};

use crate::{models::Note, repository::migrations};

pub struct Repository {
    client: Client,
}

impl Repository {
    pub async fn new(database_dsn: String) -> Result<Repository, tokio_postgres::Error> {
        let (client, con) = tokio_postgres::connect(&database_dsn, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = con.await {
                tracing::error!("connection error: {}", e);
            }
        });

        Ok(Self { client: client })
    }

    pub async fn migrate(&mut self) -> Result<(), refinery::Error> {
        let migrations_report = migrations::runner().run_async(&mut self.client).await?;

        for migration in migrations_report.applied_migrations() {
            tracing::info!(
                "Migration Applied -  Name: {}, Version: {}",
                migration.name(),
                migration.version()
            );
        }

        tracing::info!("DB migrations finished!");

        Ok(())
    }

    pub async fn create_note(&mut self, content: String) -> Result<Note, tokio_postgres::Error> {
        let row = self.client.query_one(
            "INSERT INTO notes (content) VALUES ($1) RETURNING id, content, created_at, updated_at",
            &[&content],
        ).await?;

        Ok(Note {
            id: row.get("id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_note(
        &mut self,
        id: i64,
        content: String,
    ) -> Result<Note, tokio_postgres::Error> {
        let row = self.client.query_one(
            "UPDATE notes SET content = $1 WHERE id = $2 RETURNING id, content, created_at, updated_at",
            &[&content, &id],
        ).await?;

        Ok(Note {
            id: row.get("id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn delete_note(&mut self, id: i64) -> Result<bool, tokio_postgres::Error> {
        let rows = self
            .client
            .execute("DELETE FROM notes WHERE id = $1", &[&id])
            .await?;

        Ok(rows == 1)
    }

    pub async fn get_one_note(&mut self, id: i64) -> Result<Note, tokio_postgres::Error> {
        let row = self
            .client
            .query_one(
                "SELECT id, content, created_at, updated_at FROM notes WHERE id = $1",
                &[&id],
            )
            .await?;

        Ok(Note {
            id: row.get("id"),
            content: row.get("content"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_all_notes(&mut self) -> Result<Vec<Note>, tokio_postgres::Error> {
        let rows = self
            .client
            .query("SELECT id, content, created_at, updated_at FROM notes", &[])
            .await?;

        let mut vec: Vec<Note> = Vec::new();

        for row in rows.iter() {
            vec.push(Note {
                id: row.get("id"),
                content: row.get("content"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        }

        Ok(vec)
    }
}
