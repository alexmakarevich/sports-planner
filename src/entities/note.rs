use serde::{Deserialize, Serialize};

// For sqlx
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)] // TODO: why though? it's all snaky anyway
pub struct NoteModel {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_published: i8, // BOOLEAN in MySQL is TINYINT(1) so we can use i8 to retrieve the record and later we can parse to Boolean
    pub created_at: Option<chrono::NaiveDateTime>,
    // pub updated_at: Option<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>>,
}
