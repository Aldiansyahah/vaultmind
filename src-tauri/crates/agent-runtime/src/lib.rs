//! # agent-runtime
//!
//! LLM integration with tool-calling for knowledge base manipulation.
//! Supports: search, create, edit, link, suggest, split notes.

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
