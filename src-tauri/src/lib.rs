//! # VaultMind
//!
//! RAG-optimized personal knowledge management system.
//!
//! This is the Tauri application entry point that bridges the Rust backend
//! (storage, indexing, graph, retrieval, agent) with the Svelte frontend.

use std::path::PathBuf;
use std::sync::Mutex;

use core_storage::{Database, VaultEntry};
use graph_engine::KnowledgeGraph;
use indexer::IndexingPipeline;
use retriever::{SearchIndex, SearchResult};
use rusqlite::Connection;

struct AppState {
    vault_path: Mutex<Option<PathBuf>>,
    db: Mutex<Option<Database>>,
    search_index: Mutex<Option<SearchIndex>>,
    pipeline: Mutex<Option<IndexingPipeline>>,
    graph: Mutex<Option<KnowledgeGraph>>,
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

    // Initialize indexing pipeline (without embedder for now — model download TBD)
    let pipeline = IndexingPipeline::new(None);
    *state.pipeline.lock().map_err(|e| e.to_string())? = Some(pipeline);

    // Initialize knowledge graph
    *state.graph.lock().map_err(|e| e.to_string())? = Some(KnowledgeGraph::new());

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

/// Tauri IPC command: Trigger full vault reindex
#[tauri::command]
fn reindex_vault(
    state: tauri::State<AppState>,
) -> Result<serde_json::Value, String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    let pipeline_guard = state.pipeline.lock().map_err(|e| e.to_string())?;
    let pipeline = pipeline_guard.as_ref().ok_or("Pipeline not initialized")?;

    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    let mut search_guard = state.search_index.lock().map_err(|e| e.to_string())?;
    let search_index = search_guard.as_mut().ok_or("Search index not initialized")?;

    // Run indexing synchronously (vector store disabled for now)
    let rt = tokio::runtime::Handle::current();
    let (indexed, skipped, errors) = rt.block_on(
        pipeline.index_vault(&vault_path, db, search_index, &mut None),
    );

    Ok(serde_json::json!({
        "indexed": indexed,
        "skipped": skipped,
        "errors": errors,
    }))
}

/// Tauri IPC command: Get indexing status
#[tauri::command]
fn get_indexing_status(state: tauri::State<AppState>) -> serde_json::Value {
    let has_pipeline = state.pipeline.lock().map(|p| p.is_some()).unwrap_or(false);
    let has_embedder = state
        .pipeline
        .lock()
        .map(|p| p.as_ref().map(|pl| pl.has_embedder()).unwrap_or(false))
        .unwrap_or(false);

    serde_json::json!({
        "pipeline_ready": has_pipeline,
        "embedder_active": has_embedder,
    })
}

/// Tauri IPC command: Get backlinks for a note
#[tauri::command]
fn get_backlinks(path: String, state: tauri::State<AppState>) -> Result<Vec<String>, String> {
    let graph_guard = state.graph.lock().map_err(|e| e.to_string())?;
    let graph = graph_guard.as_ref().ok_or("Graph not initialized")?;
    Ok(graph.get_backlinks(&path))
}

/// Tauri IPC command: Get graph neighbors within N hops
#[tauri::command]
fn get_graph_neighbors(
    path: String,
    depth: usize,
    state: tauri::State<AppState>,
) -> Result<Vec<String>, String> {
    let graph_guard = state.graph.lock().map_err(|e| e.to_string())?;
    let graph = graph_guard.as_ref().ok_or("Graph not initialized")?;
    Ok(graph.get_neighbors(&path, depth))
}

/// Tauri IPC command: Get full graph data for visualization
#[tauri::command]
fn get_graph_data(state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let graph_guard = state.graph.lock().map_err(|e| e.to_string())?;
    let graph = graph_guard.as_ref().ok_or("Graph not initialized")?;

    let nodes: Vec<serde_json::Value> = graph
        .all_nodes()
        .iter()
        .map(|n| {
            serde_json::json!({
                "path": n.path,
                "title": n.title,
                "tags": n.tags,
            })
        })
        .collect();

    let edges: Vec<serde_json::Value> = graph
        .all_edges()
        .iter()
        .map(|e| {
            serde_json::json!({
                "source": e.source_path,
                "target": e.target_path,
                "kind": format!("{:?}", e.edge_kind),
            })
        })
        .collect();

    Ok(serde_json::json!({
        "nodes": nodes,
        "edges": edges,
    }))
}

/// Tool executor that uses AppState to interact with the knowledge base.
struct VaultToolExecutor {
    vault_path: PathBuf,
    search_results: Vec<SearchResult>,
}

impl agent_runtime::ToolExecutor for VaultToolExecutor {
    fn execute(&self, tool_name: &str, arguments: &str) -> String {
        match tool_name {
            "search_notes" => {
                serde_json::to_string(&self.search_results).unwrap_or_default()
            }
            "read_note" => {
                let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
                let path = args["path"].as_str().unwrap_or("");
                core_storage::read_note_content(&self.vault_path, path)
                    .unwrap_or_else(|e| format!("Error: {e}"))
            }
            "list_notes" => {
                core_storage::list_vault_files(&self.vault_path)
                    .map(|entries| {
                        let names: Vec<String> = entries.iter().map(|e| e.path.clone()).collect();
                        serde_json::to_string(&names).unwrap_or_default()
                    })
                    .unwrap_or_else(|e| format!("Error: {e}"))
            }
            "create_note" => {
                let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
                let title = args["title"].as_str().unwrap_or("untitled");
                let content = args["content"].as_str().unwrap_or("");
                let filename = format!("{}.md", title.replace(' ', "-").to_lowercase());
                match core_storage::create_note(&self.vault_path, &filename, content) {
                    Ok(_) => format!("Created note: {filename}"),
                    Err(e) => format!("Error: {e}"),
                }
            }
            "edit_note" => {
                let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                match core_storage::write_note_content(&self.vault_path, path, content) {
                    Ok(()) => format!("Updated note: {path}"),
                    Err(e) => format!("Error: {e}"),
                }
            }
            "get_backlinks" => {
                let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
                let path = args["path"].as_str().unwrap_or("");
                format!("Backlinks for {path}: (graph not populated in this context)")
            }
            _ => format!("Unknown tool: {tool_name}"),
        }
    }
}

/// Tauri IPC command: Chat with AI agent
/// Uses real LLM when base_url + model configured, falls back to search
#[tauri::command]
fn chat_with_agent(
    message: String,
    base_url: Option<String>,
    api_key: Option<String>,
    model: Option<String>,
    state: tauri::State<AppState>,
) -> Result<serde_json::Value, String> {
    let vault_path = state
        .vault_path
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("Vault path not set")?;

    // Search for context first
    let search_guard = state.search_index.lock().map_err(|e| e.to_string())?;
    let search_index = search_guard.as_ref().ok_or("Search index not initialized")?;
    let results = search_index.search(&message, 5).map_err(|e| e.to_string())?;
    drop(search_guard);

    // Check if LLM is configured
    let has_llm = base_url.as_ref().map(|u| !u.is_empty()).unwrap_or(false)
        && model.as_ref().map(|m| !m.is_empty()).unwrap_or(false);

    if has_llm {
        // Use real LLM agent
        let config = agent_runtime::LlmConfig {
            base_url: base_url.unwrap_or_default(),
            api_key: api_key.filter(|k| !k.is_empty()),
            model: model.unwrap_or_default(),
            max_tokens: 2048,
            temperature: 0.3,
        };

        let agent = agent_runtime::Agent::new(config);
        let executor = VaultToolExecutor {
            vault_path,
            search_results: results.clone(),
        };

        let rt = tokio::runtime::Handle::current();
        match rt.block_on(agent.run(&message, &executor)) {
            Ok(response) => {
                let sources: Vec<serde_json::Value> = results
                    .iter()
                    .take(3)
                    .map(|r| serde_json::json!({"path": r.path, "title": r.title}))
                    .collect();

                Ok(serde_json::json!({
                    "response": response.message,
                    "sources": sources,
                    "tool_calls": response.tool_calls_made,
                    "iterations": response.iterations,
                    "mode": "llm"
                }))
            }
            Err(e) => {
                // Fall back to search on LLM error
                Ok(search_fallback_response(&message, &results, &format!("LLM error: {e}")))
            }
        }
    } else {
        // Search-only fallback
        Ok(search_fallback_response(&message, &results, ""))
    }
}

fn search_fallback_response(
    query: &str,
    results: &[SearchResult],
    error_note: &str,
) -> serde_json::Value {
    if results.is_empty() {
        return serde_json::json!({
            "response": format!("I couldn't find any notes related to '{}'. Try creating some notes first, then reindex your vault.", query),
            "sources": [],
            "tool_calls": [],
            "mode": "search"
        });
    }

    let context: Vec<String> = results
        .iter()
        .take(3)
        .map(|r| format!("**{}** ({})\n{}", r.title, r.path, r.snippet))
        .collect();

    let prefix = if error_note.is_empty() {
        String::new()
    } else {
        format!("*Note: {}. Showing search results instead.*\n\n", error_note)
    };

    let response = format!(
        "{}Here's what I found about '{}':\n\n{}",
        prefix,
        query,
        context.join("\n\n---\n\n")
    );

    let sources: Vec<serde_json::Value> = results
        .iter()
        .take(3)
        .map(|r| serde_json::json!({"path": r.path, "title": r.title}))
        .collect();

    serde_json::json!({
        "response": response,
        "sources": sources,
        "tool_calls": ["search_notes"],
        "mode": "search"
    })
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
            pipeline: Mutex::new(None),
            graph: Mutex::new(None),
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
            reindex_vault,
            get_indexing_status,
            get_backlinks,
            get_graph_neighbors,
            get_graph_data,
            chat_with_agent,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VaultMind");
}
