//! Markdown AST parser using pulldown-cmark.
//!
//! Parses markdown content into a structured [`MarkdownDocument`] with
//! heading hierarchy and content blocks for downstream chunking.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

/// A parsed markdown document with hierarchical sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// Top-level content before any heading.
    pub preamble: String,
    /// Sections organized by headings.
    pub sections: Vec<Section>,
}

/// A section within a markdown document, defined by a heading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Heading level (1-6).
    pub level: u8,
    /// Heading text.
    pub title: String,
    /// Plain text content under this heading (before any sub-heading).
    pub content: String,
    /// Nested sub-sections.
    pub children: Vec<Section>,
}

impl MarkdownDocument {
    /// Returns the total number of sections (including nested) in the document.
    pub fn section_count(&self) -> usize {
        fn count(sections: &[Section]) -> usize {
            sections
                .iter()
                .map(|s| 1 + count(&s.children))
                .sum()
        }
        count(&self.sections)
    }

    /// Returns all sections flattened with their heading breadcrumb context.
    pub fn flat_sections(&self) -> Vec<(String, &Section)> {
        let mut result = Vec::new();
        fn flatten<'a>(sections: &'a [Section], prefix: &str, out: &mut Vec<(String, &'a Section)>) {
            for section in sections {
                let context = if prefix.is_empty() {
                    section.title.clone()
                } else {
                    format!("{} > {}", prefix, section.title)
                };
                out.push((context.clone(), section));
                flatten(&section.children, &context, out);
            }
        }
        flatten(&self.sections, "", &mut result);
        result
    }
}

/// Parses markdown content into a structured [`MarkdownDocument`].
pub fn parse_markdown(content: &str) -> MarkdownDocument {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_TASKLISTS;

    let parser = Parser::new_ext(content, options);
    let events: Vec<Event> = parser.collect();

    let mut doc = MarkdownDocument {
        preamble: String::new(),
        sections: Vec::new(),
    };

    let mut current_text = String::new();
    let mut in_heading = false;
    let mut heading_level: u8 = 0;
    let mut heading_text = String::new();

    // Stack of (level, Section) for building hierarchy
    let mut section_stack: Vec<(u8, Section)> = Vec::new();

    for event in events {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Flush current text to the right place
                flush_text(&mut doc, &mut section_stack, &current_text);
                current_text.clear();

                in_heading = true;
                heading_level = heading_level_to_u8(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;

                let new_section = Section {
                    level: heading_level,
                    title: heading_text.trim().to_string(),
                    content: String::new(),
                    children: Vec::new(),
                };

                // Pop sections from stack that are at same or deeper level
                while let Some((stack_level, _)) = section_stack.last() {
                    if *stack_level >= heading_level {
                        let (_, finished) = section_stack.pop().unwrap();
                        if let Some((_, parent)) = section_stack.last_mut() {
                            parent.children.push(finished);
                        } else {
                            doc.sections.push(finished);
                        }
                    } else {
                        break;
                    }
                }

                section_stack.push((heading_level, new_section));
            }
            Event::Text(text) | Event::Code(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_heading {
                    heading_text.push(' ');
                } else {
                    current_text.push('\n');
                }
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                current_text.push_str("\n\n");
            }
            Event::Start(Tag::CodeBlock(_)) => {
                current_text.push_str("```\n");
            }
            Event::End(TagEnd::CodeBlock) => {
                current_text.push_str("\n```\n\n");
            }
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => {
                current_text.push('\n');
            }
            Event::Start(Tag::Item) => {
                current_text.push_str("- ");
            }
            Event::End(TagEnd::Item) => {
                current_text.push('\n');
            }
            Event::Start(Tag::BlockQuote(_)) => {
                current_text.push_str("> ");
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    // Flush remaining text
    flush_text(&mut doc, &mut section_stack, &current_text);

    // Unwind the stack
    while let Some((_, finished)) = section_stack.pop() {
        if let Some((_, parent)) = section_stack.last_mut() {
            parent.children.push(finished);
        } else {
            doc.sections.push(finished);
        }
    }

    doc
}

fn flush_text(doc: &mut MarkdownDocument, stack: &mut [(u8, Section)], text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some((_, section)) = stack.last_mut() {
        if !section.content.is_empty() {
            section.content.push_str("\n\n");
        }
        section.content.push_str(trimmed);
    } else {
        if !doc.preamble.is_empty() {
            doc.preamble.push_str("\n\n");
        }
        doc.preamble.push_str(trimmed);
    }
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "# Hello\n\nWorld\n";
        let doc = parse_markdown(content);
        assert!(doc.preamble.is_empty());
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "Hello");
        assert!(doc.sections[0].content.contains("World"));
    }

    #[test]
    fn test_parse_preamble() {
        let content = "Some intro text\n\n# Heading\n\nBody\n";
        let doc = parse_markdown(content);
        assert!(doc.preamble.contains("Some intro text"));
        assert_eq!(doc.sections.len(), 1);
    }

    #[test]
    fn test_parse_nested_headings() {
        let content = "# H1\n\nTop\n\n## H2a\n\nNested A\n\n## H2b\n\nNested B\n\n### H3\n\nDeep\n";
        let doc = parse_markdown(content);
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "H1");
        assert_eq!(doc.sections[0].children.len(), 2);
        assert_eq!(doc.sections[0].children[0].title, "H2a");
        assert_eq!(doc.sections[0].children[1].title, "H2b");
        assert_eq!(doc.sections[0].children[1].children.len(), 1);
        assert_eq!(doc.sections[0].children[1].children[0].title, "H3");
    }

    #[test]
    fn test_parse_multiple_h1() {
        let content = "# First\n\nA\n\n# Second\n\nB\n";
        let doc = parse_markdown(content);
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.sections[0].title, "First");
        assert_eq!(doc.sections[1].title, "Second");
    }

    #[test]
    fn test_flat_sections() {
        let content = "# H1\n\nTop\n\n## H2\n\nNested\n\n### H3\n\nDeep\n";
        let doc = parse_markdown(content);
        let flat = doc.flat_sections();
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].0, "H1");
        assert_eq!(flat[1].0, "H1 > H2");
        assert_eq!(flat[2].0, "H1 > H2 > H3");
    }

    #[test]
    fn test_section_count() {
        let content = "# A\n\n## B\n\n### C\n\n# D\n\n";
        let doc = parse_markdown(content);
        assert_eq!(doc.section_count(), 4);
    }

    #[test]
    fn test_parse_code_block() {
        let content = "# Code\n\n```rust\nfn main() {}\n```\n";
        let doc = parse_markdown(content);
        assert!(doc.sections[0].content.contains("fn main()"));
    }

    #[test]
    fn test_parse_empty() {
        let doc = parse_markdown("");
        assert!(doc.preamble.is_empty());
        assert!(doc.sections.is_empty());
    }
}
