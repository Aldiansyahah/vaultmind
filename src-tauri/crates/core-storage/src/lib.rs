//! # core-storage
//!
//! File I/O, SQLite metadata, and LanceDB vector storage for VaultMind.

pub mod database;
pub mod error;
pub mod migrations;
pub mod models;

pub use database::Database;
pub use error::{Result, StorageError};
pub use models::{Note, NoteTag, Tag};

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
