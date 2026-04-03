use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Watcher(notify::Error),
    Migration(String),
    NotFound(String),
    Duplicate(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Database(e) => write!(f, "database error: {e}"),
            StorageError::Io(e) => write!(f, "io error: {e}"),
            StorageError::Watcher(e) => write!(f, "watcher error: {e}"),
            StorageError::Migration(e) => write!(f, "migration error: {e}"),
            StorageError::NotFound(e) => write!(f, "not found: {e}"),
            StorageError::Duplicate(e) => write!(f, "duplicate: {e}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::Database(err)
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl From<notify::Error> for StorageError {
    fn from(err: notify::Error) -> Self {
        StorageError::Watcher(err)
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;
