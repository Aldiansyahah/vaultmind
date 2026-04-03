//! # graph-engine
//!
//! Knowledge graph with explicit (wikilinks) and implicit (cosine similarity) edges.

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
