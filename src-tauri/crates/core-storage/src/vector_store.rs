//! Vector storage using LanceDB for semantic search.
//!
//! Stores chunk embeddings and supports vector similarity search
//! for RAG-quality retrieval.

use std::path::Path;
use std::sync::Arc;

use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::connect;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::table::Table;
use lancedb::Connection;
use thiserror::Error;

/// Default embedding dimension (matches all-MiniLM-L6-v2).
const EMBEDDING_DIM: i32 = 384;

/// Table name for chunk embeddings.
const TABLE_NAME: &str = "chunks";

/// Errors from vector storage operations.
#[derive(Debug, Error)]
pub enum VectorStoreError {
    #[error("LanceDB error: {0}")]
    Lance(String),
    #[error("Arrow error: {0}")]
    Arrow(String),
    #[error("Store not initialized")]
    NotInitialized,
}

type Result<T> = std::result::Result<T, VectorStoreError>;

/// A search result from vector similarity search.
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub chunk_id: String,
    pub note_path: String,
    pub heading_context: String,
    pub content: String,
    pub distance: f32,
}

/// Vector store backed by LanceDB.
pub struct VectorStore {
    db: Connection,
    table: Option<Table>,
}

impl VectorStore {
    /// Opens or creates a LanceDB database at the given path.
    pub async fn new(db_path: &Path) -> Result<Self> {
        let db = connect(db_path.to_str().unwrap_or("."))
            .execute()
            .await
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?;

        let mut store = Self { db, table: None };
        store.ensure_table().await?;
        Ok(store)
    }

    /// Ensures the chunks table exists with the correct schema.
    async fn ensure_table(&mut self) -> Result<()> {
        let table_names = self
            .db
            .table_names()
            .execute()
            .await
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?;

        if table_names.contains(&TABLE_NAME.to_string()) {
            let table = self
                .db
                .open_table(TABLE_NAME)
                .execute()
                .await
                .map_err(|e| VectorStoreError::Lance(e.to_string()))?;
            self.table = Some(table);
        }
        // Table will be created on first insert if it doesn't exist
        Ok(())
    }

    /// Returns the schema for the chunks table.
    fn schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new("note_path", DataType::Utf8, false),
            Field::new("heading_context", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    EMBEDDING_DIM,
                ),
                false,
            ),
        ]))
    }

    /// Upserts chunks for a note — deletes old chunks then inserts new ones.
    pub async fn upsert_chunks(
        &mut self,
        note_path: &str,
        chunk_ids: &[String],
        heading_contexts: &[String],
        contents: &[String],
        vectors: &[Vec<f32>],
    ) -> Result<()> {
        // First delete existing chunks for this note
        self.delete_note_chunks(note_path).await?;

        if chunk_ids.is_empty() {
            return Ok(());
        }

        let schema = Self::schema();
        let n = chunk_ids.len();

        let chunk_id_array = StringArray::from(chunk_ids.to_vec());
        let note_path_array = StringArray::from(vec![note_path.to_string(); n]);
        let heading_array = StringArray::from(heading_contexts.to_vec());
        let content_array = StringArray::from(contents.to_vec());

        // Build FixedSizeList for vectors
        let flat_values: Vec<f32> = vectors.iter().flatten().copied().collect();
        let values_array = Float32Array::from(flat_values);
        let field = Arc::new(Field::new("item", DataType::Float32, true));
        let vector_array =
            FixedSizeListArray::new(field, EMBEDDING_DIM, Arc::new(values_array), None);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(chunk_id_array),
                Arc::new(note_path_array),
                Arc::new(heading_array),
                Arc::new(content_array),
                Arc::new(vector_array),
            ],
        )
        .map_err(|e| VectorStoreError::Arrow(e.to_string()))?;

        if let Some(table) = &self.table {
            table
                .add(vec![batch])
                .execute()
                .await
                .map_err(|e| VectorStoreError::Lance(e.to_string()))?;
        } else {
            let table = self
                .db
                .create_table(TABLE_NAME, vec![batch])
                .execute()
                .await
                .map_err(|e| VectorStoreError::Lance(e.to_string()))?;
            self.table = Some(table);
        }

        Ok(())
    }

    /// Deletes all chunks for a specific note.
    pub async fn delete_note_chunks(&mut self, note_path: &str) -> Result<()> {
        if let Some(table) = &self.table {
            table
                .delete(&format!("note_path = '{}'", note_path.replace('\'', "''")))
                .await
                .map_err(|e| VectorStoreError::Lance(e.to_string()))?;
        }
        Ok(())
    }

    /// Searches for the most similar chunks to a query vector.
    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        let table = self
            .table
            .as_ref()
            .ok_or(VectorStoreError::NotInitialized)?;

        let results = table
            .vector_search(query_vector)
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?
            .limit(limit)
            .execute()
            .await
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?;

        let mut search_results = Vec::new();

        use futures::TryStreamExt;
        let batches: Vec<RecordBatch> = results
            .try_collect()
            .await
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?;

        for batch in &batches {
            let chunk_ids = batch
                .column_by_name("chunk_id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let note_paths = batch
                .column_by_name("note_path")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let headings = batch
                .column_by_name("heading_context")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let contents = batch
                .column_by_name("content")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let distances = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            if let (Some(ids), Some(paths), Some(heads), Some(conts), Some(dists)) =
                (chunk_ids, note_paths, headings, contents, distances)
            {
                for i in 0..batch.num_rows() {
                    search_results.push(VectorSearchResult {
                        chunk_id: ids.value(i).to_string(),
                        note_path: paths.value(i).to_string(),
                        heading_context: heads.value(i).to_string(),
                        content: conts.value(i).to_string(),
                        distance: dists.value(i),
                    });
                }
            }
        }

        Ok(search_results)
    }

    /// Returns the total number of chunks stored.
    pub async fn chunk_count(&self) -> Result<usize> {
        let table = match &self.table {
            Some(t) => t,
            None => return Ok(0),
        };
        let count = table
            .count_rows(None)
            .await
            .map_err(|e| VectorStoreError::Lance(e.to_string()))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_vector_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = VectorStore::new(dir.path()).await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_and_count() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(dir.path()).await.unwrap();

        let ids = vec!["chunk1".to_string()];
        let contexts = vec!["Section A".to_string()];
        let contents = vec!["Hello world content".to_string()];
        let vectors = vec![vec![0.1f32; EMBEDDING_DIM as usize]];

        store
            .upsert_chunks("test.md", &ids, &contexts, &contents, &vectors)
            .await
            .unwrap();

        assert_eq!(store.chunk_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_delete_note_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(dir.path()).await.unwrap();

        let ids = vec!["c1".to_string(), "c2".to_string()];
        let contexts = vec!["A".to_string(), "B".to_string()];
        let contents = vec!["one".to_string(), "two".to_string()];
        let vectors = vec![
            vec![0.1f32; EMBEDDING_DIM as usize],
            vec![0.2f32; EMBEDDING_DIM as usize],
        ];

        store
            .upsert_chunks("test.md", &ids, &contexts, &contents, &vectors)
            .await
            .unwrap();
        assert_eq!(store.chunk_count().await.unwrap(), 2);

        store.delete_note_chunks("test.md").await.unwrap();
        assert_eq!(store.chunk_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = VectorStore::new(dir.path()).await.unwrap();

        let mut v1 = vec![0.0f32; EMBEDDING_DIM as usize];
        v1[0] = 1.0;
        let mut v2 = vec![0.0f32; EMBEDDING_DIM as usize];
        v2[1] = 1.0;

        let ids = vec!["c1".to_string(), "c2".to_string()];
        let contexts = vec!["A".to_string(), "B".to_string()];
        let contents = vec!["apple".to_string(), "banana".to_string()];
        let vectors = vec![v1.clone(), v2];

        store
            .upsert_chunks("test.md", &ids, &contexts, &contents, &vectors)
            .await
            .unwrap();

        let results = store.search(&v1, 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "apple");
    }
}
