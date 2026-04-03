//! # VaultMind
//!
//! RAG-optimized personal knowledge management system.
//!
//! This is the Tauri application entry point that bridges the Rust backend
//! (storage, indexing, graph, retrieval, agent) with the Svelte frontend.

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_version, health_check])
        .run(tauri::generate_context!())
        .expect("error while running VaultMind");
}
