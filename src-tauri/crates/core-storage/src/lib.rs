//! # core-storage
//!
//! File I/O, SQLite metadata, and LanceDB vector storage for VaultMind.

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
