//! # indexer
//!
//! Markdown AST parsing, smart chunking, and embedding generation.
//! Pipeline: Markdown -> AST -> Semantic Segments -> Chunks -> Embeddings

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
