//! Tool definitions for the AI agent.
//!
//! These tools allow the LLM to interact with the knowledge base:
//! search notes, create notes, edit content, and suggest connections.

use crate::client::{FunctionDef, ToolDef};

/// Returns all available tool definitions for the agent.
pub fn get_tool_definitions() -> Vec<ToolDef> {
    vec![
        search_notes_tool(),
        read_note_tool(),
        create_note_tool(),
        edit_note_tool(),
        list_notes_tool(),
        get_backlinks_tool(),
    ]
}

fn search_notes_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "search_notes".into(),
            description: "Search through all notes using full-text search. Returns matching notes with snippets.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default 5)",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        },
    }
}

fn read_note_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "read_note".into(),
            description: "Read the full content of a specific note by its file path.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The relative file path of the note (e.g., 'my-note.md')"
                    }
                },
                "required": ["path"]
            }),
        },
    }
}

fn create_note_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "create_note".into(),
            description: "Create a new markdown note with the given title and content.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "The note title (will be used as filename)"
                    },
                    "content": {
                        "type": "string",
                        "description": "The markdown content of the note"
                    }
                },
                "required": ["title", "content"]
            }),
        },
    }
}

fn edit_note_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "edit_note".into(),
            description: "Replace the content of an existing note.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The relative file path of the note"
                    },
                    "content": {
                        "type": "string",
                        "description": "The new markdown content"
                    }
                },
                "required": ["path", "content"]
            }),
        },
    }
}

fn list_notes_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "list_notes".into(),
            description: "List all notes in the vault.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    }
}

fn get_backlinks_tool() -> ToolDef {
    ToolDef {
        tool_type: "function".into(),
        function: FunctionDef {
            name: "get_backlinks".into(),
            description: "Get all notes that link to a specific note via wikilinks.".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The note path to find backlinks for"
                    }
                },
                "required": ["path"]
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions_not_empty() {
        let tools = get_tool_definitions();
        assert!(!tools.is_empty());
        assert!(tools.len() >= 5);
    }

    #[test]
    fn test_all_tools_have_names() {
        let tools = get_tool_definitions();
        for tool in &tools {
            assert!(!tool.function.name.is_empty());
            assert!(!tool.function.description.is_empty());
            assert_eq!(tool.tool_type, "function");
        }
    }

    #[test]
    fn test_tool_serialization() {
        let tools = get_tool_definitions();
        let json = serde_json::to_string(&tools).unwrap();
        assert!(json.contains("search_notes"));
        assert!(json.contains("create_note"));
    }
}
