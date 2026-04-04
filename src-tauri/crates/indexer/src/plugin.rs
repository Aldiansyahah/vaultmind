//! Plugin system for extending VaultMind with custom extractors,
//! processors, and integrations.
//!
//! Plugins implement the [`Plugin`] trait to hook into the indexing pipeline.

use serde::{Deserialize, Serialize};

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique plugin identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Description of what the plugin does.
    pub description: String,
    /// File extensions this plugin can handle (e.g., ["pdf", "docx"]).
    pub supported_extensions: Vec<String>,
}

/// Trait that all plugins must implement.
pub trait Plugin: Send + Sync {
    /// Returns plugin metadata.
    fn info(&self) -> PluginInfo;

    /// Returns true if this plugin can handle the given file extension.
    fn can_handle(&self, extension: &str) -> bool;

    /// Extracts text content from a file for indexing.
    /// Returns None if the plugin can't process this file.
    fn extract_text(&self, file_path: &std::path::Path) -> Option<String>;

    /// Optional: post-process extracted text before indexing.
    fn post_process(&self, text: &str) -> String {
        text.to_string()
    }
}

/// Registry that manages loaded plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Creates a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Registers a plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        tracing::info!("Registered plugin: {} v{}", plugin.info().name, plugin.info().version);
        self.plugins.push(plugin);
    }

    /// Finds a plugin that can handle the given file extension.
    pub fn find_handler(&self, extension: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.can_handle(extension))
            .map(|p| p.as_ref())
    }

    /// Returns info for all registered plugins.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    /// Returns the number of registered plugins.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Extracts text from a file using the appropriate plugin.
    pub fn extract(&self, file_path: &std::path::Path) -> Option<String> {
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        self.find_handler(ext)
            .and_then(|p| p.extract_text(file_path))
            .map(|text| {
                // Apply post-processing from the plugin
                if let Some(handler) = self.find_handler(ext) {
                    handler.post_process(&text)
                } else {
                    text
                }
            })
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in PDF plugin using pdf-extract.
pub struct PdfPlugin;

impl Plugin for PdfPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "builtin.pdf".into(),
            name: "PDF Extractor".into(),
            version: "1.0.0".into(),
            description: "Extracts text from PDF files".into(),
            supported_extensions: vec!["pdf".into()],
        }
    }

    fn can_handle(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case("pdf")
    }

    fn extract_text(&self, file_path: &std::path::Path) -> Option<String> {
        let bytes = std::fs::read(file_path).ok()?;
        pdf_extract::extract_text_from_mem(&bytes).ok()
    }
}

/// Built-in plain text plugin.
pub struct PlainTextPlugin;

impl Plugin for PlainTextPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "builtin.plaintext".into(),
            name: "Plain Text Reader".into(),
            version: "1.0.0".into(),
            description: "Reads plain text files".into(),
            supported_extensions: vec!["txt".into(), "text".into(), "csv".into(), "json".into(), "yaml".into(), "yml".into(), "toml".into()],
        }
    }

    fn can_handle(&self, extension: &str) -> bool {
        matches!(
            extension.to_lowercase().as_str(),
            "txt" | "text" | "csv" | "json" | "yaml" | "yml" | "toml"
        )
    }

    fn extract_text(&self, file_path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(file_path).ok()
    }
}

/// Creates a plugin registry with all built-in plugins.
pub fn default_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(PdfPlugin));
    registry.register(Box::new(PlainTextPlugin));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry() {
        let registry = default_registry();
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn test_find_pdf_handler() {
        let registry = default_registry();
        let handler = registry.find_handler("pdf");
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().info().id, "builtin.pdf");
    }

    #[test]
    fn test_find_txt_handler() {
        let registry = default_registry();
        assert!(registry.find_handler("txt").is_some());
        assert!(registry.find_handler("json").is_some());
    }

    #[test]
    fn test_no_handler_for_unknown() {
        let registry = default_registry();
        assert!(registry.find_handler("xyz").is_none());
    }

    #[test]
    fn test_list_plugins() {
        let registry = default_registry();
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 2);
        assert!(plugins.iter().any(|p| p.id == "builtin.pdf"));
    }

    #[test]
    fn test_plain_text_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "Hello from plugin").unwrap();

        let registry = default_registry();
        let text = registry.extract(&path);
        assert_eq!(text.unwrap(), "Hello from plugin");
    }
}
