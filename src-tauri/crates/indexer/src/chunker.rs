//! Semantic chunker that splits a parsed markdown document into chunks
//! suitable for embedding and vector search.
//!
//! Chunks are created by heading boundaries, with a target size of
//! 256-512 characters. Large sections are split further, and small
//! sections are kept as-is to preserve semantic coherence.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::parser::{MarkdownDocument, Section};

/// Target chunk size in characters (not tokens, as a simple approximation).
const TARGET_CHUNK_SIZE: usize = 400;
/// Maximum chunk size before forced split.
const MAX_CHUNK_SIZE: usize = 800;
/// Overlap in characters between split chunks for context preservation.
const CHUNK_OVERLAP: usize = 80;

/// A semantic chunk of a note, ready for embedding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier (hash of note_path + chunk_index).
    pub id: String,
    /// Path of the source note.
    pub note_path: String,
    /// Heading breadcrumb context (e.g., "Section > Subsection").
    pub heading_context: String,
    /// The chunk text content.
    pub content: String,
    /// Index of this chunk within the note.
    pub chunk_index: usize,
}

/// Configuration for the chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    pub target_size: usize,
    pub max_size: usize,
    pub overlap: usize,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            target_size: TARGET_CHUNK_SIZE,
            max_size: MAX_CHUNK_SIZE,
            overlap: CHUNK_OVERLAP,
        }
    }
}

/// Chunks a parsed markdown document into semantic segments.
pub fn chunk_document(doc: &MarkdownDocument, note_path: &str) -> Vec<Chunk> {
    chunk_document_with_config(doc, note_path, &ChunkerConfig::default())
}

/// Chunks a document with custom configuration.
pub fn chunk_document_with_config(
    doc: &MarkdownDocument,
    note_path: &str,
    config: &ChunkerConfig,
) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut index = 0;

    // Chunk the preamble (content before any heading)
    if !doc.preamble.trim().is_empty() {
        for text in split_text(&doc.preamble, config) {
            chunks.push(make_chunk(note_path, "", &text, index));
            index += 1;
        }
    }

    // Chunk each section recursively
    chunk_sections(&doc.sections, note_path, "", config, &mut chunks, &mut index);

    chunks
}

fn chunk_sections(
    sections: &[Section],
    note_path: &str,
    parent_context: &str,
    config: &ChunkerConfig,
    chunks: &mut Vec<Chunk>,
    index: &mut usize,
) {
    for section in sections {
        let context = if parent_context.is_empty() {
            section.title.clone()
        } else {
            format!("{} > {}", parent_context, section.title)
        };

        // Include heading title as part of chunk content for better retrieval
        let section_text = if section.content.trim().is_empty() {
            section.title.clone()
        } else {
            format!("{}\n\n{}", section.title, section.content)
        };

        for text in split_text(&section_text, config) {
            chunks.push(make_chunk(note_path, &context, &text, *index));
            *index += 1;
        }

        // Recurse into children
        chunk_sections(&section.children, note_path, &context, config, chunks, index);
    }
}

/// Splits text into chunks respecting size limits.
/// Tries to split on paragraph boundaries, then sentence boundaries.
fn split_text(text: &str, config: &ChunkerConfig) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if trimmed.len() <= config.max_size {
        return vec![trimmed.to_string()];
    }

    let mut result = Vec::new();
    let paragraphs: Vec<&str> = trimmed.split("\n\n").collect();
    let mut current = String::new();

    for para in paragraphs {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }

        if current.len() + para.len() + 2 > config.target_size && !current.is_empty() {
            result.push(current.trim().to_string());
            // Add overlap from end of previous chunk
            let overlap_start = current.len().saturating_sub(config.overlap);
            let overlap = &current[overlap_start..];
            current = format!("{}\n\n{}", overlap.trim(), para);
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    // If we still have chunks that are too large, force-split them
    let mut final_result = Vec::new();
    for chunk in result {
        if chunk.len() > config.max_size {
            final_result.extend(force_split(&chunk, config));
        } else {
            final_result.push(chunk);
        }
    }

    final_result
}

/// Force-splits a chunk that exceeds max size by splitting on sentence boundaries.
fn force_split(text: &str, config: &ChunkerConfig) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for sentence in text.split(". ") {
        let piece = if current.is_empty() {
            sentence.to_string()
        } else {
            format!("{}. {}", current, sentence)
        };

        if piece.len() > config.target_size && !current.is_empty() {
            result.push(format!("{}.", current.trim()));
            current = sentence.to_string();
        } else {
            current = piece;
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

fn make_chunk(note_path: &str, context: &str, content: &str, index: usize) -> Chunk {
    let mut hasher = DefaultHasher::new();
    note_path.hash(&mut hasher);
    index.hash(&mut hasher);
    let id = format!("{:x}", hasher.finish());

    Chunk {
        id,
        note_path: note_path.to_string(),
        heading_context: context.to_string(),
        content: content.to_string(),
        chunk_index: index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_markdown;

    #[test]
    fn test_chunk_simple_note() {
        let md = "# My Note\n\nSome content here.\n";
        let doc = parse_markdown(md);
        let chunks = chunk_document(&doc, "test.md");
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].note_path, "test.md");
        assert!(chunks[0].content.contains("content"));
    }

    #[test]
    fn test_chunk_preserves_context() {
        let md = "# Top\n\n## Sub\n\nContent under sub\n";
        let doc = parse_markdown(md);
        let chunks = chunk_document(&doc, "test.md");
        let sub_chunk = chunks.iter().find(|c| c.content.contains("Content under sub"));
        assert!(sub_chunk.is_some());
        assert_eq!(sub_chunk.unwrap().heading_context, "Top > Sub");
    }

    #[test]
    fn test_chunk_unique_ids() {
        let md = "# A\n\nFirst\n\n# B\n\nSecond\n";
        let doc = parse_markdown(md);
        let chunks = chunk_document(&doc, "test.md");
        assert!(chunks.len() >= 2);
        assert_ne!(chunks[0].id, chunks[1].id);
    }

    #[test]
    fn test_chunk_preamble() {
        let md = "Intro text\n\n# Heading\n\nBody\n";
        let doc = parse_markdown(md);
        let chunks = chunk_document(&doc, "test.md");
        let preamble = chunks.iter().find(|c| c.heading_context.is_empty());
        assert!(preamble.is_some());
        assert!(preamble.unwrap().content.contains("Intro text"));
    }

    #[test]
    fn test_chunk_indices_sequential() {
        let md = "# A\n\nOne\n\n# B\n\nTwo\n\n# C\n\nThree\n";
        let doc = parse_markdown(md);
        let chunks = chunk_document(&doc, "test.md");
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i);
        }
    }

    #[test]
    fn test_chunk_large_content_splits() {
        let long_para = "word ".repeat(200);
        let md = format!("# Big\n\n{}\n", long_para);
        let doc = parse_markdown(&md);
        let config = ChunkerConfig {
            target_size: 100,
            max_size: 200,
            overlap: 20,
        };
        let chunks = chunk_document_with_config(&doc, "test.md", &config);
        assert!(chunks.len() > 1, "Should split large content into multiple chunks");
    }

    #[test]
    fn test_chunk_empty_doc() {
        let doc = parse_markdown("");
        let chunks = chunk_document(&doc, "test.md");
        assert!(chunks.is_empty());
    }
}
