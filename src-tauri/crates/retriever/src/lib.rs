//! # retriever
//!
//! Hybrid search: Vector (LanceDB) + BM25 (Tantivy) + Graph expansion + Re-ranking.

pub mod error;
pub mod hybrid;
pub mod models;
pub mod search;

pub use error::{Result, SearchError};
pub use hybrid::{graph_expand, reciprocal_rank_fusion, HybridResult};
pub use models::SearchResult;
pub use search::SearchIndex;

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
