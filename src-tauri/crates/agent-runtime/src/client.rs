//! LLM API client supporting OpenAI-compatible endpoints.
//!
//! Works with OpenAI, Anthropic (via proxy), Ollama, and any
//! OpenAI-compatible API.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from LLM API operations.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("No API key configured")]
    NoApiKey,
    #[error("Parse error: {0}")]
    Parse(String),
}

type Result<T> = std::result::Result<T, LlmError>;

/// Configuration for the LLM client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// API base URL (e.g., "https://api.openai.com/v1" or "http://localhost:11434/v1")
    pub base_url: String,
    /// API key (optional for local models like Ollama)
    pub api_key: Option<String>,
    /// Model name (e.g., "gpt-4o-mini", "llama3.2")
    pub model: String,
    /// Maximum tokens in response
    pub max_tokens: u32,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: None,
            model: "llama3.2".to_string(),
            max_tokens: 2048,
            temperature: 0.3,
        }
    }
}

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// A tool call from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// Function call details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool definition for the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDef,
}

/// Function definition within a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Chat completion request.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDef>>,
    max_tokens: u32,
    temperature: f32,
}

/// Chat completion response.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// LLM API client.
pub struct LlmClient {
    config: LlmConfig,
    http: Client,
}

impl LlmClient {
    /// Creates a new LLM client with the given configuration.
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    /// Sends a chat completion request and returns the assistant's message.
    pub async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolDef]>,
    ) -> Result<Message> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
        };

        let mut req_builder = self.http.post(&url).json(&request);

        if let Some(key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {key}"));
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("Status {status}: {body}")));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| LlmError::Api("No choices in response".into()))
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Updates the configuration.
    pub fn set_config(&mut self, config: LlmConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert!(config.base_url.contains("localhost"));
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_message_serialize() {
        let msg = Message {
            role: "user".into(),
            content: Some("Hello".into()),
            tool_calls: None,
            tool_call_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
        assert!(!json.contains("tool_calls"));
    }

    #[test]
    fn test_tool_def_serialize() {
        let tool = ToolDef {
            tool_type: "function".into(),
            function: FunctionDef {
                name: "search".into(),
                description: "Search notes".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    }
                }),
            },
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("search"));
    }
}
