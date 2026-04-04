use serde::{Deserialize, Serialize};

/// A search result from the full-text index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Relative path of the note.
    pub path: String,
    /// Title of the note.
    pub title: String,
    /// BM25 score of the match.
    pub score: f32,
    /// Highlighted snippet showing the match context.
    pub snippet: String,
}
