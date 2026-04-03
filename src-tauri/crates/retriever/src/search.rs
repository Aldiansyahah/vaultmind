use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, ReloadPolicy, TantivyDocument};

use crate::error::{Result, SearchError};
use crate::models::SearchResult;

const INDEX_MEMORY_BUDGET: usize = 50_000_000;

/// Schema fields for the full-text search index.
pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("path", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT | STORED);
    schema_builder.build()
}

/// Full-text search index powered by Tantivy.
pub struct SearchIndex {
    index: Index,
    schema: Schema,
    reader: IndexReader,
}

impl SearchIndex {
    /// Creates a new search index, either from an existing directory or in-memory.
    pub fn new(index_path: Option<&Path>) -> Result<Self> {
        let schema = build_schema();

        let index = if let Some(path) = index_path {
            if path.exists() {
                Index::open_in_dir(path).map_err(SearchError::Tantivy)?
            } else {
                std::fs::create_dir_all(path).map_err(SearchError::Io)?;
                Index::create_in_dir(path, schema.clone()).map_err(SearchError::Tantivy)?
            }
        } else {
            Index::create_in_ram(schema.clone())
        };

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(SearchError::Tantivy)?;

        Ok(Self {
            index,
            schema,
            reader,
        })
    }

    /// Adds or updates a document in the index.
    pub fn upsert_document(&mut self, path: &str, title: &str, content: &str) -> Result<()> {
        let path_field = self.schema.get_field("path").expect("path field");
        let title_field = self.schema.get_field("title").expect("title field");
        let content_field = self.schema.get_field("content").expect("content field");

        let mut writer: tantivy::IndexWriter<TantivyDocument> = self
            .index
            .writer(INDEX_MEMORY_BUDGET)
            .map_err(SearchError::Tantivy)?;

        let term = Term::from_field_text(path_field, path);
        writer.delete_term(term);

        writer
            .add_document(doc!(
                path_field => path,
                title_field => title,
                content_field => content,
            ))
            .map_err(SearchError::Tantivy)?;

        writer.commit().map_err(SearchError::Tantivy)?;
        self.reader.reload().map_err(SearchError::Tantivy)?;

        Ok(())
    }

    /// Removes a document from the index by path.
    pub fn delete_document(&mut self, path: &str) -> Result<()> {
        let path_field = self.schema.get_field("path").expect("path field");
        let mut writer: tantivy::IndexWriter<TantivyDocument> = self
            .index
            .writer(INDEX_MEMORY_BUDGET)
            .map_err(SearchError::Tantivy)?;

        let term = Term::from_field_text(path_field, path);
        writer.delete_term(term);
        writer.commit().map_err(SearchError::Tantivy)?;
        self.reader.reload().map_err(SearchError::Tantivy)?;

        Ok(())
    }

    /// Searches the index and returns ranked results with snippets.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let searcher = self.reader.searcher();

        let title_field = self.schema.get_field("title").expect("title field");
        let content_field = self.schema.get_field("content").expect("content field");

        let mut query_parser =
            QueryParser::for_index(&self.index, vec![title_field, content_field]);
        query_parser.set_field_boost(title_field, 3.0);

        let parsed_query = query_parser
            .parse_query(query)
            .map_err(|e| SearchError::Tantivy(e.into()))?;

        let top_docs = TopDocs::with_limit(limit);
        let results = searcher
            .search(&parsed_query, &top_docs)
            .map_err(SearchError::Tantivy)?;

        let mut search_results = Vec::new();

        for (_score, doc_address) in results {
            let doc: TantivyDocument = searcher.doc(doc_address).map_err(SearchError::Tantivy)?;

            let path = doc
                .get_first(self.schema.get_field("path").expect("path field"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.schema.get_field("title").expect("title field"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let content = doc
                .get_first(self.schema.get_field("content").expect("content field"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let snippet = generate_snippet(&content, query);

            search_results.push(SearchResult {
                path,
                title,
                score: _score,
                snippet,
            });
        }

        Ok(search_results)
    }

    /// Returns the total number of documents in the index.
    pub fn doc_count(&self) -> u64 {
        self.reader.searcher().num_docs()
    }
}

/// Generates a highlighted snippet from content around the query terms.
fn generate_snippet(content: &str, query: &str) -> String {
    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();

    if let Some(pos) = content_lower.find(&query_lower) {
        let start = pos.saturating_sub(60);
        let end = (pos + query_lower.len() + 60).min(content.len());
        let snippet = content[start..end].trim();

        if start > 0 {
            format!("...{snippet}")
        } else {
            snippet.to_string()
        }
    } else {
        content.chars().take(150).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_index() -> SearchIndex {
        SearchIndex::new(None).expect("Failed to create test index")
    }

    #[test]
    fn test_upsert_and_search() {
        let mut index = create_test_index();

        index
            .upsert_document(
                "test.md",
                "Test Note",
                "This is a test note about Rust programming.",
            )
            .expect("Failed to upsert document");

        let results = index.search("Rust", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Note");
        assert_eq!(results[0].path, "test.md");
    }

    #[test]
    fn test_search_title_boosted() {
        let mut index = create_test_index();

        index
            .upsert_document("a.md", "Rust Guide", "Some content here.")
            .expect("Failed to upsert");
        index
            .upsert_document("b.md", "Other Note", "This mentions Rust in content.")
            .expect("Failed to upsert");

        let results = index.search("Rust", 10).expect("Failed to search");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Rust Guide");
    }

    #[test]
    fn test_delete_document() {
        let mut index = create_test_index();

        index
            .upsert_document("delete.md", "Delete Me", "Content to delete.")
            .expect("Failed to upsert");

        index
            .delete_document("delete.md")
            .expect("Failed to delete");

        let results = index.search("Content", 10).expect("Failed to search");
        assert!(results.is_empty());
    }

    #[test]
    fn test_upsert_updates_existing() {
        let mut index = create_test_index();

        index
            .upsert_document("update.md", "Old Title", "Old content.")
            .expect("Failed to upsert");

        index
            .upsert_document("update.md", "New Title", "New content about Python.")
            .expect("Failed to upsert");

        let results = index.search("Python", 10).expect("Failed to search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "New Title");

        let old_results = index.search("Old", 10).expect("Failed to search");
        assert!(old_results.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let index = create_test_index();
        let results = index.search("", 10).expect("Failed to search");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_limit() {
        let mut index = create_test_index();

        for i in 0..5 {
            index
                .upsert_document(
                    &format!("note{i}.md"),
                    &format!("Note {i}"),
                    &format!("Content about programming in Rust note {i}"),
                )
                .expect("Failed to upsert");
        }

        let results = index.search("Rust", 2).expect("Failed to search");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_generate_snippet_finds_match() {
        let content = "This is a long piece of text that contains the word programming somewhere in the middle.";
        let snippet = generate_snippet(content, "programming");
        assert!(snippet.contains("programming"));
    }

    #[test]
    fn test_generate_snippet_no_match() {
        let content = "This text does not contain the search term.";
        let snippet = generate_snippet(content, "notfound");
        assert_eq!(snippet, "This text does not contain the search term.");
    }

    #[test]
    fn test_generate_snippet_truncates() {
        let content = "Short text.";
        let snippet = generate_snippet(content, "Short");
        assert_eq!(snippet, "Short text.");
    }

    #[test]
    fn test_doc_count() {
        let mut index = create_test_index();
        assert_eq!(index.doc_count(), 0);

        index
            .upsert_document("a.md", "A", "Content A")
            .expect("Failed to upsert");
        index
            .upsert_document("b.md", "B", "Content B")
            .expect("Failed to upsert");

        assert_eq!(index.doc_count(), 2);
    }
}
