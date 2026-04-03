use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<i64>,
    pub path: String,
    pub title: String,
    pub content_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTag {
    pub note_id: i64,
    pub tag_id: i64,
}
