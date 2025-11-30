use chrono::{DateTime, Utc};

pub struct Note {
    pub id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
