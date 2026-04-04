//! # agent-runtime
//!
//! LLM integration with tool-calling for knowledge base manipulation.
//! Supports: search, create, edit, link, suggest, split notes.

pub mod agent;
pub mod client;
pub mod tools;

pub use agent::{Agent, AgentResponse, ToolExecutor};
pub use client::{LlmClient, LlmConfig, Message};
pub use tools::get_tool_definitions;

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
