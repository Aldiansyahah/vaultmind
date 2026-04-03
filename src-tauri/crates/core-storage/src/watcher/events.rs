use std::path::PathBuf;

/// Represents a file system event that the indexing pipeline can consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    /// A new markdown file was created
    FileCreated(PathBuf),
    /// An existing markdown file was modified
    FileModified(PathBuf),
    /// A markdown file was deleted
    FileDeleted(PathBuf),
    /// A markdown file was renamed (old_path, new_path)
    FileRenamed(PathBuf, PathBuf),
}

impl WatchEvent {
    pub fn path(&self) -> &PathBuf {
        match self {
            WatchEvent::FileCreated(path)
            | WatchEvent::FileModified(path)
            | WatchEvent::FileDeleted(path) => path,
            WatchEvent::FileRenamed(_, new_path) => new_path,
        }
    }
}
