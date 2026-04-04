//! # indexer
//!
//! Markdown AST parsing, smart chunking, and embedding generation.
//! Pipeline: Markdown -> AST -> Semantic Segments -> Chunks -> Embeddings

pub mod chunker;
pub mod embedder;
pub mod parser;
pub mod pipeline;

pub use chunker::{chunk_document, chunk_document_with_config, Chunk, ChunkerConfig};
pub use embedder::{cosine_similarity, Embedder, EmbedderError, EMBEDDING_DIM};
pub use parser::{parse_markdown, MarkdownDocument, Section};
pub use pipeline::IndexingPipeline;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
