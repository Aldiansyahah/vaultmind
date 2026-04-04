//! AI Agent with tool-calling loop.
//!
//! The agent receives a user query, uses the LLM to decide which tools to call,
//! executes the tools against the knowledge base, and returns a final response.

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::client::{LlmClient, LlmConfig, Message, ToolCall};
use crate::tools::get_tool_definitions;

/// Maximum number of tool-calling iterations before forcing a final response.
const MAX_ITERATIONS: usize = 5;

/// System prompt for the knowledge base agent.
const SYSTEM_PROMPT: &str = r#"You are VaultMind, an AI assistant for a personal knowledge management system. You help users organize, search, and connect their notes.

You have access to tools that let you interact with the user's note vault. Use them to:
- Search for relevant notes when answering questions
- Create new notes when the user asks
- Edit existing notes when requested
- Find connections between notes via backlinks

Always search before answering knowledge questions. Be concise and helpful. When creating or editing notes, use proper markdown formatting."#;

/// Result of an agent interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// The final text response from the agent.
    pub message: String,
    /// Tool calls that were executed during this interaction.
    pub tool_calls_made: Vec<String>,
    /// Number of LLM iterations used.
    pub iterations: usize,
}

/// Callback trait for executing tools against the knowledge base.
/// Implementors provide the actual tool execution logic.
pub trait ToolExecutor: Send + Sync {
    /// Executes a tool call and returns the result as a string.
    fn execute(&self, tool_name: &str, arguments: &str) -> String;
}

/// The AI Agent that orchestrates LLM + tool calls.
pub struct Agent {
    client: LlmClient,
}

impl Agent {
    /// Creates a new agent with the given LLM configuration.
    pub fn new(config: LlmConfig) -> Self {
        Self {
            client: LlmClient::new(config),
        }
    }

    /// Runs the agent loop for a user query.
    ///
    /// The agent will:
    /// 1. Send the query to the LLM with available tools
    /// 2. If the LLM requests tool calls, execute them
    /// 3. Feed tool results back to the LLM
    /// 4. Repeat until the LLM gives a final text response (max MAX_ITERATIONS)
    pub async fn run(
        &self,
        user_query: &str,
        executor: &dyn ToolExecutor,
    ) -> Result<AgentResponse, String> {
        let tools = get_tool_definitions();
        let mut messages = vec![
            Message {
                role: "system".into(),
                content: Some(SYSTEM_PROMPT.into()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "user".into(),
                content: Some(user_query.into()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let mut tool_calls_made = Vec::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                info!("Agent reached max iterations ({})", MAX_ITERATIONS);
                break;
            }

            let response = self
                .client
                .chat(&messages, Some(&tools))
                .await
                .map_err(|e| e.to_string())?;

            // Check if LLM wants to call tools
            if let Some(tool_calls) = &response.tool_calls {
                if !tool_calls.is_empty() {
                    // Add assistant message with tool calls
                    messages.push(response.clone());

                    // Execute each tool call
                    for tc in tool_calls {
                        let result = execute_tool_call(tc, executor);
                        tool_calls_made.push(format!("{}({})", tc.function.name, tc.function.arguments));

                        info!(
                            "Tool call: {}({}) → {} chars",
                            tc.function.name,
                            tc.function.arguments,
                            result.len()
                        );

                        // Add tool result message
                        messages.push(Message {
                            role: "tool".into(),
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                        });
                    }
                    continue;
                }
            }

            // No tool calls — this is the final response
            let final_message = response.content.unwrap_or_default();
            return Ok(AgentResponse {
                message: final_message,
                tool_calls_made,
                iterations,
            });
        }

        // Fallback if max iterations reached
        Ok(AgentResponse {
            message: "I've gathered some information but reached the processing limit. Please try a more specific question.".into(),
            tool_calls_made,
            iterations,
        })
    }

    /// Updates the LLM configuration.
    pub fn set_config(&mut self, config: LlmConfig) {
        self.client.set_config(config);
    }
}

fn execute_tool_call(tc: &ToolCall, executor: &dyn ToolExecutor) -> String {
    let result = executor.execute(&tc.function.name, &tc.function.arguments);
    if result.is_empty() {
        "No results found.".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExecutor;

    impl ToolExecutor for MockExecutor {
        fn execute(&self, tool_name: &str, _arguments: &str) -> String {
            match tool_name {
                "search_notes" => r#"[{"path":"test.md","title":"Test Note","snippet":"Some content"}]"#.into(),
                "read_note" => "# Test Note\n\nSome content here.".into(),
                "list_notes" => r#"["test.md","other.md"]"#.into(),
                _ => "Tool not found".into(),
            }
        }
    }

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(LlmConfig::default());
        assert_eq!(agent.client.config().model, "llama3.2");
    }

    #[test]
    fn test_mock_executor() {
        let executor = MockExecutor;
        let result = executor.execute("search_notes", r#"{"query":"test"}"#);
        assert!(result.contains("Test Note"));
    }

    #[test]
    fn test_system_prompt_exists() {
        assert!(!SYSTEM_PROMPT.is_empty());
        assert!(SYSTEM_PROMPT.contains("VaultMind"));
    }
}
