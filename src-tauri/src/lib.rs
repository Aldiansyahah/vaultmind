//! # VaultMind
//!
//! RAG-optimized personal knowledge management system.
//!
//! This is the Tauri application entry point that bridges the Rust backend
//! (storage, indexing, graph, retrieval, agent) with the Svelte frontend.

use std::path::PathBuf;
use std::sync::Mutex;

use core_storage::{Database, VaultEntry};
use retriever::{SearchIndex, SearchResult};
use rusqlite::Connection;

struct AppState {
    vault_path: Mutex<Option<PathBuf>>,
    db: Mutex<Option<Database>>,
    search_index: Mutex<Option<SearchIndex>>,
}

/// Tauri IPC command: Get application version
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Tauri IPC command: Health check
#[tauri::command]
fn health_check() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "modules": {
            "core_storage": core_storage::version(),
            "indexer": indexer::version(),
            "graph_engine": graph_engine::version(),
            "retriever": retriever::version(),
            "agent_runtime": agent_runtime::version(),
        }
    })
}

/// Tauri IPC command: Set the vault path for file operations
#[tauri::command]
fn set_vault_path(path: String, state: tauri::State<AppState>) -> Result<(), String> {
    let vault_path = PathBuf::from(&path);
    if !vault_path.exists() {
        return Err(format!("Path does not exist: {path}"));
    }
    if !vault_path.is_dir() {
        return Err(format!("Path is not a directory: {path}"));
    }

    // Initialize SQLite database
    let db_path = vault_path.join(".vaultmind").join("metadata.db");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create data dir: {e}"))?;
    }
    let conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open database: {e}"))?;
    let db =
        Database::new(conn).map_err(|e| format!("Failed to initialize database: {e}"))?;
    *state.db.lock().map_err(|e| e.to_string())? = Some(db);

    // Initialize Tantivy search index
    let index_path = vault_path.join(".vaultmind").join("search_index");
    let search_index = SearchIndex::new(Some(&index_path))
        .map_err(|e| format!("Failed to initialize search index: {e}"))?;
    *state
        .search_index
        .lock()
        .map_err(|e| e.to_string())? = Some(search_index);

    *state.vault_path.lock().map_err(|e| e.to_string())? = Some(vault_path);
    Ok(())
}

/// Tauri IPC command: Get the current vault path
#[tauri::command]
fn get_vault_path(state: tauri::State<AppState>) -> Option<String> {
    state
        .vault_path
        .lock()
        .ok()
        .and_then(|p| p.clone())
        .and_then(|p| p.to_str().map(String::from))
}

/// Tauri IPC command: List all files and directories in the vault
#[tauri::command]
fn list_vault_files(state: tauri::State<AppState>) -> Result<Vec<VaultEntry>, String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::list_vault_files(&vault_path).map_err(|e| e.to_string())
}

/// Tauri IPC command: Create a new note file
#[tauri::command]
fn create_note(
    relative_path: String,
    content: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    let full_path = core_storage::create_note(&vault_path, &relative_path, &content)
        .map_err(|e| e.to_string())?;

    Ok(full_path.to_string_lossy().to_string())
}

/// Tauri IPC command: Rename a note or directory
#[tauri::command]
fn rename_note(
    old_path: String,
    new_path: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::rename_note(&vault_path, &old_path, &new_path).map_err(|e| e.to_string())
}

/// Tauri IPC command: Delete a note or directory
#[tauri::command]
fn delete_note(relative_path: String, state: tauri::State<AppState>) -> Result<(), String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::delete_note(&vault_path, &relative_path).map_err(|e| e.to_string())
}

/// Tauri IPC command: Move a note or directory
#[tauri::command]
fn move_note(
    old_path: String,
    new_path: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::move_note(&vault_path, &old_path, &new_path).map_err(|e| e.to_string())
}

/// Tauri IPC command: Read note content
#[tauri::command]
fn read_note_content(
    relative_path: String,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::read_note_content(&vault_path, &relative_path).map_err(|e| e.to_string())
}

/// Tauri IPC command: Write note content
#[tauri::command]
fn write_note_content(
    relative_path: String,
    content: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    core_storage::write_note_content(&vault_path, &relative_path, &content)
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: Extract wikilinks from note content
#[tauri::command]
fn extract_wikilinks_from_content(content: String) -> Vec<String> {
    core_storage::extract_wikilinks(&content)
}

/// Tauri IPC command: Extract tags from note content
#[tauri::command]
fn extract_tags_from_content(content: String) -> Vec<String> {
    core_storage::extract_tags(&content)
}

/// Tauri IPC command: Sync tags from content to database for a note
#[tauri::command]
fn sync_note_tags(
    note_path: String,
    tags: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| e.to_string())?;
    let db = db_guard
        .as_ref()
        .ok_or("Database not initialized")?;

    let note = db
        .get_note_by_path(&note_path)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note not found: {note_path}"))?;

    db.sync_tags_for_note(note.id.unwrap(), &tags)
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: Get all tags with their usage counts
#[tauri::command]
fn get_all_tags(state: tauri::State<AppState>) -> Result<Vec<(core_storage::Tag, i64)>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| e.to_string())?;
    let db = db_guard
        .as_ref()
        .ok_or("Database not initialized")?;

    db.list_all_tags_with_counts().map_err(|e| e.to_string())
}

/// Tauri IPC command: Get notes for a specific tag
#[tauri::command]
fn get_notes_for_tag(
    tag_id: i64,
    state: tauri::State<AppState>,
) -> Result<Vec<core_storage::Note>, String> {
    let db_guard = state
        .db
        .lock()
        .map_err(|e| e.to_string())?;
    let db = db_guard
        .as_ref()
        .ok_or("Database not initialized")?;

    db.get_notes_for_tag(tag_id).map_err(|e| e.to_string())
}

/// Tauri IPC command: Full-text search across all notes
#[tauri::command]
fn search_notes(
    query: String,
    limit: usize,
    state: tauri::State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    let search_guard = state
        .search_index
        .lock()
        .map_err(|e| e.to_string())?;
    let search_index = search_guard
        .as_ref()
        .ok_or("Search index not initialized")?;

    search_index
        .search(&query, limit)
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: Index a note for full-text search
#[tauri::command]
fn index_note(
    path: String,
    title: String,
    content: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let mut search_index = state.search_index.lock().map_err(|e| e.to_string())?;

    let index = search_index
        .as_mut()
        .ok_or("Search index not initialized")?;

    index
        .upsert_document(&path, &title, &content)
        .map_err(|e| e.to_string())
}

/// Tauri IPC command: Remove a note from the search index
#[tauri::command]
fn unindex_note(path: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut search_index = state.search_index.lock().map_err(|e| e.to_string())?;

    let index = search_index
        .as_mut()
        .ok_or("Search index not initialized")?;

    index.delete_document(&path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            vault_path: Mutex::new(None),
            db: Mutex::new(None),
            search_index: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_version,
            health_check,
            set_vault_path,
            get_vault_path,
            list_vault_files,
            create_note,
            rename_note,
            delete_note,
            move_note,
            read_note_content,
            write_note_content,
            extract_wikilinks_from_content,
            extract_tags_from_content,
            sync_note_tags,
            get_all_tags,
            get_notes_for_tag,
            search_notes,
            index_note,
            unindex_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VaultMind");
}
