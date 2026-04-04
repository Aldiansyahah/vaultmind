//! # core-storage
//!
//! File I/O, SQLite metadata, and LanceDB vector storage for VaultMind.

pub mod database;
pub mod error;
pub mod file_ops;
pub mod migrations;
pub mod models;
pub mod parser;
pub mod vector_store;
pub mod watcher;

pub use database::Database;
pub use error::{Result, StorageError};
pub use file_ops::{
    create_note, delete_note, list_vault_files, move_note, read_note_content, rename_note,
    write_note_content, VaultEntry,
};
pub use models::{Note, NoteTag, Tag};
pub use parser::{extract_tags, extract_wikilinks};
pub use vector_store::{VectorSearchResult, VectorStore, VectorStoreError};
pub use watcher::VaultWatcher;
pub use watcher::WatchEvent;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
