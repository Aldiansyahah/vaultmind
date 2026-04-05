//! Multi-modal content extractors.
//!
//! Extracts text from non-markdown file formats for indexing:
//! - PDF → plain text
//! - Images → metadata + OCR (placeholder)
//! - (Future) Audio → transcript

use std::path::Path;

use thiserror::Error;

/// Errors from content extraction.
#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported file type: {0}")]
    Unsupported(String),
    #[error("Extraction failed: {0}")]
    Failed(String),
}

type Result<T> = std::result::Result<T, ExtractError>;

/// Extracted content from a file.
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Source file path.
    pub source_path: String,
    /// Extracted plain text.
    pub text: String,
    /// File type that was processed.
    pub file_type: FileType,
    /// Optional metadata (dimensions, author, etc.)
    pub metadata: std::collections::HashMap<String, String>,
}

/// Supported file types for extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Markdown,
    Pdf,
    Csv,
    Image,
    PlainText,
    Unknown,
}

/// Detects the file type from extension.
pub fn detect_file_type(path: &Path) -> FileType {
    match path.extension().and_then(|e| e.to_str()) {
        Some("md" | "markdown") => FileType::Markdown,
        Some("pdf") => FileType::Pdf,
        Some("csv" | "tsv") => FileType::Csv,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg") => FileType::Image,
        Some("txt" | "text") => FileType::PlainText,
        _ => FileType::Unknown,
    }
}

/// Extracts text from a PDF file.
pub fn extract_pdf_text(path: &Path) -> Result<ExtractedContent> {
    let bytes = std::fs::read(path)?;

    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| ExtractError::Failed(format!("PDF extraction failed: {e}")))?;

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("page_count_estimate".into(), estimate_pdf_pages(&text).to_string());

    Ok(ExtractedContent {
        source_path: path.to_string_lossy().to_string(),
        text: text.trim().to_string(),
        file_type: FileType::Pdf,
        metadata,
    })
}

/// Extracts metadata from an image file (no OCR yet).
pub fn extract_image_metadata(path: &Path) -> Result<ExtractedContent> {
    let metadata_map = std::collections::HashMap::from([
        ("file_name".to_string(), path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()),
        ("file_size".to_string(), std::fs::metadata(path)
            .map(|m| m.len().to_string())
            .unwrap_or_default()),
    ]);

    // Placeholder text — real OCR would go here (tesseract, etc.)
    let text = format!(
        "Image: {}",
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    );

    Ok(ExtractedContent {
        source_path: path.to_string_lossy().to_string(),
        text,
        file_type: FileType::Image,
        metadata: metadata_map,
    })
}

/// Extracts text from a plain text file.
pub fn extract_plain_text(path: &Path) -> Result<ExtractedContent> {
    let text = std::fs::read_to_string(path)?;

    Ok(ExtractedContent {
        source_path: path.to_string_lossy().to_string(),
        text,
        file_type: FileType::PlainText,
        metadata: std::collections::HashMap::new(),
    })
}

/// Extracts and parses a CSV file into structured data.
pub fn extract_csv(path: &Path) -> Result<ExtractedContent> {
    let raw = std::fs::read_to_string(path)?;
    let delimiter = if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
        '\t'
    } else {
        ','
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in raw.lines() {
        let cols: Vec<String> = line.split(delimiter).map(|s| s.trim().to_string()).collect();
        rows.push(cols);
    }

    let row_count = rows.len();
    let col_count = rows.first().map(|r| r.len()).unwrap_or(0);
    let headers = rows.first().cloned().unwrap_or_default();

    // Convert to readable text for indexing
    let text = rows
        .iter()
        .map(|row| row.join(" | "))
        .collect::<Vec<_>>()
        .join("\n");

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("row_count".into(), row_count.to_string());
    metadata.insert("col_count".into(), col_count.to_string());
    metadata.insert("headers".into(), headers.join(", "));
    metadata.insert("data_json".into(), serde_json::to_string(&rows).unwrap_or_default());

    Ok(ExtractedContent {
        source_path: path.to_string_lossy().to_string(),
        text,
        file_type: FileType::Csv,
        metadata,
    })
}

/// Dispatches extraction based on file type.
pub fn extract_content(path: &Path) -> Result<ExtractedContent> {
    match detect_file_type(path) {
        FileType::Pdf => extract_pdf_text(path),
        FileType::Csv => extract_csv(path),
        FileType::Image => extract_image_metadata(path),
        FileType::PlainText => extract_plain_text(path),
        FileType::Markdown => {
            let text = std::fs::read_to_string(path)?;
            Ok(ExtractedContent {
                source_path: path.to_string_lossy().to_string(),
                text,
                file_type: FileType::Markdown,
                metadata: std::collections::HashMap::new(),
            })
        }
        FileType::Unknown => Err(ExtractError::Unsupported(
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "no extension".into()),
        )),
    }
}

fn estimate_pdf_pages(text: &str) -> usize {
    // Rough estimate: ~3000 chars per page
    (text.len() / 3000).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type(Path::new("note.md")), FileType::Markdown);
        assert_eq!(detect_file_type(Path::new("doc.pdf")), FileType::Pdf);
        assert_eq!(detect_file_type(Path::new("img.png")), FileType::Image);
        assert_eq!(detect_file_type(Path::new("file.txt")), FileType::PlainText);
        assert_eq!(detect_file_type(Path::new("file.xyz")), FileType::Unknown);
    }

    #[test]
    fn test_extract_plain_text() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "Hello world").unwrap();

        let content = extract_plain_text(&path).unwrap();
        assert_eq!(content.text, "Hello world");
        assert_eq!(content.file_type, FileType::PlainText);
    }

    #[test]
    fn test_extract_content_dispatch_markdown() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("note.md");
        std::fs::write(&path, "# Title\n\nContent").unwrap();

        let content = extract_content(&path).unwrap();
        assert_eq!(content.file_type, FileType::Markdown);
        assert!(content.text.contains("Title"));
    }

    #[test]
    fn test_extract_content_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file.xyz");
        std::fs::write(&path, "data").unwrap();

        let result = extract_content(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_csv() {
        assert_eq!(detect_file_type(Path::new("data.csv")), FileType::Csv);
        assert_eq!(detect_file_type(Path::new("data.tsv")), FileType::Csv);
    }

    #[test]
    fn test_extract_csv() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.csv");
        std::fs::write(&path, "name,age,city\nAlice,30,Jakarta\nBob,25,Bandung").unwrap();

        let content = extract_csv(&path).unwrap();
        assert_eq!(content.file_type, FileType::Csv);
        assert!(content.text.contains("Alice"));
        assert_eq!(content.metadata["row_count"], "3");
        assert_eq!(content.metadata["col_count"], "3");
        assert!(content.metadata["headers"].contains("name"));
    }

    #[test]
    fn test_image_metadata_extraction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.png");
        std::fs::write(&path, &[0u8; 100]).unwrap();

        let content = extract_image_metadata(&path).unwrap();
        assert_eq!(content.file_type, FileType::Image);
        assert!(content.metadata.contains_key("file_name"));
        assert!(content.metadata.contains_key("file_size"));
    }
}
