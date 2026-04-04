//! Indexing pipeline that orchestrates: parse → chunk → embed → store.
//!
//! Consumes file watcher events and processes notes through the full
//! indexing pipeline, updating both the vector store and search index.

use std::path::Path;

use sha2::{Digest, Sha256};
use tracing::{info, warn};

use core_storage::{Database, VectorStore};
use retriever::SearchIndex;

use crate::chunker::chunk_document;
use crate::embedder::Embedder;
use crate::parser::parse_markdown;

/// The indexing pipeline that coordinates all indexing operations.
pub struct IndexingPipeline {
    embedder: Option<Embedder>,
}

impl IndexingPipeline {
    /// Creates a new pipeline. If no ONNX model path is provided,
    /// embedding generation is skipped (chunks still stored in Tantivy).
    pub fn new(model_path: Option<&Path>) -> Self {
        let embedder = model_path.and_then(|p| {
            match Embedder::new(p) {
                Ok(e) => {
                    info!("Loaded embedding model from {:?}", p);
                    Some(e)
                }
                Err(e) => {
                    warn!("Failed to load embedding model: {e}. Vector search disabled.");
                    None
                }
            }
        });

        Self { embedder }
    }

    /// Indexes a single note file: parse → chunk → embed → store.
    ///
    /// Returns the number of chunks created.
    pub async fn index_note(
        &self,
        note_path: &str,
        content: &str,
        db: &Database,
        search_index: &mut SearchIndex,
        vector_store: Option<&mut VectorStore>,
    ) -> Result<usize, String> {
        let content_hash = compute_hash(content);

        // Check if content has changed
        if let Ok(Some(existing)) = db.get_note_by_path(note_path) {
            if existing.content_hash == content_hash {
                return Ok(0); // Skip unchanged
            }
        }

        // Extract title from first heading or filename
        let title = extract_title(note_path, content);

        // Upsert note in SQLite
        match db.get_note_by_path(note_path) {
            Ok(Some(note)) => {
                if let Some(id) = note.id {
                    db.update_note(id, &title, &content_hash)
                        .map_err(|e| e.to_string())?;
                }
            }
            Ok(None) => {
                db.create_note(note_path, &title, &content_hash)
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => return Err(e.to_string()),
        }

        // Update Tantivy full-text index
        search_index
            .upsert_document(note_path, &title, content)
            .map_err(|e| e.to_string())?;

        // Parse and chunk
        let doc = parse_markdown(content);
        let chunks = chunk_document(&doc, note_path);

        // Generate embeddings and store in vector DB
        if let (Some(embedder), Some(vs)) = (&self.embedder, vector_store) {
            let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();

            let embeddings = embedder
                .embed_batch(&chunk_texts)
                .map_err(|e| e.to_string())?;

            let ids: Vec<String> = chunks.iter().map(|c| c.id.clone()).collect();
            let contexts: Vec<String> = chunks.iter().map(|c| c.heading_context.clone()).collect();
            let contents: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();

            vs.upsert_chunks(note_path, &ids, &contexts, &contents, &embeddings)
                .await
                .map_err(|e| e.to_string())?;
        }

        let chunk_count = chunks.len();
        info!(
            "Indexed note '{}': {} chunks",
            note_path, chunk_count
        );

        Ok(chunk_count)
    }

    /// Removes a note from all indexes.
    pub async fn remove_note(
        &self,
        note_path: &str,
        db: &Database,
        search_index: &mut SearchIndex,
        vector_store: Option<&mut VectorStore>,
    ) -> Result<(), String> {
        // Remove from SQLite
        if let Ok(Some(note)) = db.get_note_by_path(note_path) {
            if let Some(id) = note.id {
                db.delete_note(id).map_err(|e| e.to_string())?;
            }
        }

        // Remove from Tantivy
        search_index
            .delete_document(note_path)
            .map_err(|e| e.to_string())?;

        // Remove from vector store
        if let Some(vs) = vector_store {
            vs.delete_note_chunks(note_path)
                .await
                .map_err(|e| e.to_string())?;
        }

        info!("Removed note '{}' from all indexes", note_path);
        Ok(())
    }

    /// Indexes all markdown files in a directory (bulk indexing).
    ///
    /// Returns (indexed_count, skipped_count, error_count).
    pub async fn index_vault(
        &self,
        vault_path: &Path,
        db: &Database,
        search_index: &mut SearchIndex,
        vector_store: &mut Option<VectorStore>,
    ) -> (usize, usize, usize) {
        let mut indexed = 0;
        let mut skipped = 0;
        let mut errors = 0;

        let md_files = collect_markdown_files(vault_path);
        let total = md_files.len();
        info!("Starting bulk index of {} markdown files", total);

        for (i, path) in md_files.iter().enumerate() {
            let relative = path
                .strip_prefix(vault_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match self
                        .index_note(&relative, &content, db, search_index, vector_store.as_mut())
                        .await
                    {
                        Ok(0) => skipped += 1,
                        Ok(_) => indexed += 1,
                        Err(e) => {
                            warn!("Failed to index {}: {}", relative, e);
                            errors += 1;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read {}: {}", relative, e);
                    errors += 1;
                }
            }

            if (i + 1) % 50 == 0 || i + 1 == total {
                info!("Progress: {}/{} files processed", i + 1, total);
            }
        }

        info!(
            "Bulk index complete: {} indexed, {} skipped (unchanged), {} errors",
            indexed, skipped, errors
        );

        (indexed, skipped, errors)
    }

    /// Returns whether the pipeline has an active embedder.
    pub fn has_embedder(&self) -> bool {
        self.embedder.is_some()
    }
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_title(note_path: &str, content: &str) -> String {
    // Try to extract from first H1
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.trim().to_string();
        }
    }
    // Fallback to filename without extension
    Path::new(note_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| note_path.to_string())
}

fn collect_markdown_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                    files.extend(collect_markdown_files(&path));
                }
            } else if let Some(ext) = path.extension() {
                if ext == "md" || ext == "markdown" {
                    files.push(path);
                }
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let h1 = compute_hash("hello");
        let h2 = compute_hash("hello");
        let h3 = compute_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert!(!h1.is_empty());
    }

    #[test]
    fn test_extract_title_from_heading() {
        let title = extract_title("note.md", "# My Title\n\nContent");
        assert_eq!(title, "My Title");
    }

    #[test]
    fn test_extract_title_fallback() {
        let title = extract_title("my-note.md", "No heading here");
        assert_eq!(title, "my-note");
    }

    #[test]
    fn test_collect_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "# A").unwrap();
        std::fs::write(dir.path().join("b.txt"), "not md").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/c.md"), "# C").unwrap();

        let files = collect_markdown_files(dir.path());
        assert_eq!(files.len(), 2);
    }
}
