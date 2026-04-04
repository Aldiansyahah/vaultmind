use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

use notify::event::EventKind;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Result, StorageError};
use crate::watcher::events::WatchEvent;
use crate::watcher::filter::{is_markdown_file, should_ignore_path};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

struct DebounceState {
    last_event: Instant,
    pending_event: Option<WatchEvent>,
}

/// A file system watcher for a VaultMind vault directory.
///
/// Watches for markdown file changes and emits structured [`WatchEvent`]s
/// that the indexing pipeline can consume.
pub struct VaultWatcher {
    watcher: Option<RecommendedWatcher>,
    event_rx: Receiver<WatchEvent>,
    _event_tx: Sender<WatchEvent>,
}

impl VaultWatcher {
    /// Creates a new `VaultWatcher`.
    ///
    /// The watcher does not start watching immediately. Call [`VaultWatcher::start`]
    /// to begin watching the vault directory.
    pub fn new() -> Self {
        let (event_tx, event_rx) = channel();
        Self {
            watcher: None,
            event_rx,
            _event_tx: event_tx,
        }
    }

    /// Starts watching the vault directory for markdown file changes.
    ///
    /// Returns an error if the path does not exist or is not a directory.
    pub fn start(&mut self, vault_path: &Path) -> Result<()> {
        if !vault_path.exists() {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Vault path not found: {vault_path:?}"),
            )));
        }

        if !vault_path.is_dir() {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Vault path is not a directory: {vault_path:?}"),
            )));
        }

        let (raw_tx, raw_rx) = channel::<Event>();
        let vault_root = vault_path.to_path_buf();

        let raw_watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    if let Err(e) = raw_tx.send(event) {
                        tracing::error!("Failed to send raw event: {e}");
                    }
                }
            },
            notify::Config::default(),
        )?;

        // Use the existing _event_tx so that event_rx receives the debounced events
        let debounce_tx = self._event_tx.clone();
        std::thread::spawn(move || {
            debounce_loop(raw_rx, debounce_tx, &vault_root);
        });

        let mut watcher = raw_watcher;
        watcher.watch(vault_path, RecursiveMode::Recursive)?;

        self.watcher = Some(watcher);

        Ok(())
    }

    /// Stops watching and cleans up resources.
    pub fn stop(&mut self) {
        self.watcher.take();
    }

    /// Receives the next event, blocking until one is available.
    pub fn recv(&self) -> std::result::Result<WatchEvent, std::sync::mpsc::RecvError> {
        self.event_rx.recv()
    }

    /// Tries to receive an event without blocking.
    pub fn try_recv(&self) -> std::result::Result<WatchEvent, std::sync::mpsc::TryRecvError> {
        self.event_rx.try_recv()
    }
}

impl Default for VaultWatcher {
    fn default() -> Self {
        Self::new()
    }
}

fn debounce_loop(rx: Receiver<Event>, tx: Sender<WatchEvent>, _vault_root: &Path) {
    let mut debounce_map: std::collections::HashMap<PathBuf, DebounceState> =
        std::collections::HashMap::new();

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                let paths: Vec<&PathBuf> = event
                    .paths
                    .iter()
                    .filter(|p| !should_ignore_path(p) && is_markdown_file(p))
                    .collect();

                if paths.is_empty() {
                    continue;
                }

                let watch_event = convert_to_watch_event(&event);

                for path in paths {
                    let state = debounce_map
                        .entry(path.clone())
                        .or_insert_with(|| DebounceState {
                            last_event: Instant::now(),
                            pending_event: None,
                        });

                    if let Some(ref we) = watch_event {
                        state.pending_event = Some(we.clone());
                    }
                    state.last_event = Instant::now();
                }

                let now = Instant::now();
                let mut ready_events: Vec<WatchEvent> = Vec::new();
                let mut expired_paths: Vec<PathBuf> = Vec::new();

                for (path, state) in &mut debounce_map {
                    if now.duration_since(state.last_event) >= DEBOUNCE_DURATION {
                        if let Some(event) = state.pending_event.take() {
                            ready_events.push(event);
                        }
                        expired_paths.push(path.clone());
                    }
                }

                for path in expired_paths {
                    debounce_map.remove(&path);
                }

                for event in ready_events {
                    if let Err(e) = tx.send(event) {
                        tracing::error!("Failed to send debounced event: {e}");
                        return;
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let now = Instant::now();
                let mut ready_events: Vec<WatchEvent> = Vec::new();
                let mut expired_paths: Vec<PathBuf> = Vec::new();

                for (path, state) in &mut debounce_map {
                    if now.duration_since(state.last_event) >= DEBOUNCE_DURATION {
                        if let Some(event) = state.pending_event.take() {
                            ready_events.push(event);
                        }
                        expired_paths.push(path.clone());
                    }
                }

                for path in expired_paths {
                    debounce_map.remove(&path);
                }

                for event in ready_events {
                    if let Err(e) = tx.send(event) {
                        tracing::error!("Failed to send debounced event: {e}");
                        return;
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return;
            }
        }
    }
}

fn convert_to_watch_event(event: &Event) -> Option<WatchEvent> {
    let paths: Vec<&PathBuf> = event
        .paths
        .iter()
        .filter(|p| !should_ignore_path(p) && is_markdown_file(p))
        .collect();

    if paths.is_empty() {
        return None;
    }

    match event.kind {
        EventKind::Create(_) => Some(WatchEvent::FileCreated(paths[0].clone())),
        EventKind::Modify(_) => Some(WatchEvent::FileModified(paths[0].clone())),
        EventKind::Remove(_) => Some(WatchEvent::FileDeleted(paths[0].clone())),
        EventKind::Access(_) | EventKind::Other | EventKind::Any => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    fn create_temp_vault() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_vault_watcher_detects_create() {
        let vault = create_temp_vault();
        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        let md_path = vault.path().join("test.md");
        fs::write(&md_path, "# Hello").expect("Failed to write file");

        thread::sleep(Duration::from_millis(500));

        while let Ok(event) = watcher.try_recv() {
            if let WatchEvent::FileCreated(path) = event {
                assert_eq!(path, md_path);
                return;
            }
        }
    }

    #[test]
    fn test_vault_watcher_detects_modify() {
        let vault = create_temp_vault();
        let md_path = vault.path().join("existing.md");
        fs::write(&md_path, "# Original").expect("Failed to write file");

        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        thread::sleep(Duration::from_millis(100));

        fs::write(&md_path, "# Modified").expect("Failed to modify file");

        thread::sleep(Duration::from_millis(500));

        while let Ok(event) = watcher.try_recv() {
            if let WatchEvent::FileModified(path) = event {
                assert_eq!(path, md_path);
                return;
            }
        }
    }

    #[test]
    fn test_vault_watcher_detects_delete() {
        let vault = create_temp_vault();
        let md_path = vault.path().join("to_delete.md");
        fs::write(&md_path, "# Delete me").expect("Failed to write file");

        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        thread::sleep(Duration::from_millis(100));

        fs::remove_file(&md_path).expect("Failed to delete file");

        thread::sleep(Duration::from_millis(500));

        while let Ok(event) = watcher.try_recv() {
            if let WatchEvent::FileDeleted(path) = event {
                assert_eq!(path, md_path);
                return;
            }
        }
    }

    #[test]
    fn test_vault_watcher_ignores_non_md() {
        let vault = create_temp_vault();
        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        let txt_path = vault.path().join("ignore.txt");
        fs::write(&txt_path, "not markdown").expect("Failed to write file");

        thread::sleep(Duration::from_millis(500));

        while let Ok(event) = watcher.try_recv() {
            match event {
                WatchEvent::FileCreated(p)
                | WatchEvent::FileModified(p)
                | WatchEvent::FileDeleted(p) => {
                    assert!(
                        is_markdown_file(&p),
                        "Should not get events for non-markdown files, got: {p:?}"
                    );
                }
                WatchEvent::FileRenamed(_, p) => {
                    assert!(
                        is_markdown_file(&p),
                        "Should not get events for non-markdown files, got: {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_vault_watcher_ignores_hidden_files() {
        let vault = create_temp_vault();
        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        let hidden_path = vault.path().join(".hidden.md");
        fs::write(&hidden_path, "# Hidden").expect("Failed to write file");

        thread::sleep(Duration::from_millis(500));

        while let Ok(event) = watcher.try_recv() {
            match event {
                WatchEvent::FileCreated(p)
                | WatchEvent::FileModified(p)
                | WatchEvent::FileDeleted(p) => {
                    assert!(
                        !should_ignore_path(&p),
                        "Should not get events for hidden files, got: {p:?}"
                    );
                }
                WatchEvent::FileRenamed(_, p) => {
                    assert!(
                        !should_ignore_path(&p),
                        "Should not get events for hidden files, got: {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_vault_watcher_fails_for_nonexistent_path() {
        let mut watcher = VaultWatcher::new();
        let result = watcher.start(Path::new("/nonexistent/vault/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_watcher_fails_for_file_path() {
        let vault = create_temp_vault();
        let file_path = vault.path().join("file.md");
        fs::write(&file_path, "# Test").expect("Failed to write file");

        let mut watcher = VaultWatcher::new();
        let result = watcher.start(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_debounce_prevents_duplicate_events() {
        let vault = create_temp_vault();
        let mut watcher = VaultWatcher::new();
        watcher
            .start(vault.path())
            .expect("Failed to start watcher");

        let md_path = vault.path().join("debounce_test.md");

        for i in 0..5 {
            fs::write(&md_path, format!("# Content {i}")).expect("Failed to write file");
            thread::sleep(Duration::from_millis(10));
        }

        thread::sleep(Duration::from_millis(600));

        let mut event_count = 0;
        while let Ok(event) = watcher.try_recv() {
            if matches!(event, WatchEvent::FileModified(_)) {
                event_count += 1;
            }
        }

        assert!(
            event_count <= 2,
            "Expected at most 2 events after debouncing, got {event_count}"
        );
    }
}
