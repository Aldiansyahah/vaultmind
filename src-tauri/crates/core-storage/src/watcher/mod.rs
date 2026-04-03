/// Structured file system events for the indexing pipeline.
pub mod events;

/// File filtering utilities for markdown files.
pub mod filter;

mod watcher_impl;

pub use events::WatchEvent;
pub use watcher_impl::VaultWatcher;
