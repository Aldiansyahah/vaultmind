//! # graph-engine
//!
//! Knowledge graph with explicit (wikilinks) and implicit (shared tags, cosine similarity) edges.

pub mod graph;

pub use graph::{Connection, EdgeKind, KnowledgeGraph, NoteNode};

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
