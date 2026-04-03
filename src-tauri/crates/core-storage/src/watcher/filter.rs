use std::path::Path;

/// Returns true if the path should be watched (i.e. is a markdown file).
pub fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

/// Returns true if the path should be ignored (temp files, hidden files, etc.).
pub fn should_ignore_path(path: &Path) -> bool {
    // Check all path components for hidden directories
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    return true;
                }
            }
        }
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if file_name.is_empty() {
        return true;
    }

    // Ignore common temp file patterns
    if file_name.starts_with('~') || file_name.starts_with('$') {
        return true;
    }

    // Ignore editor swap/backup files
    if file_name.ends_with('~') || file_name.ends_with(".swp") || file_name.ends_with(".swo") {
        return true;
    }

    // Ignore common OS metadata files
    let ignore_names = [".DS_Store", "Thumbs.db", "desktop.ini"];
    if ignore_names.contains(&file_name) {
        return true;
    }

    // Ignore common temp/lock file patterns
    let ignore_extensions = [".tmp", ".bak", ".lock", ".crdownload", ".part"];
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ignore_extensions.contains(&format!(".{ext}").as_str()) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_markdown_file_md() {
        assert!(is_markdown_file(Path::new("note.md")));
        assert!(is_markdown_file(Path::new("notes/deep/note.md")));
    }

    #[test]
    fn test_is_markdown_file_markdown() {
        assert!(is_markdown_file(Path::new("note.markdown")));
    }

    #[test]
    fn test_is_markdown_file_case_insensitive() {
        assert!(is_markdown_file(Path::new("note.MD")));
        assert!(is_markdown_file(Path::new("note.Md")));
        assert!(is_markdown_file(Path::new("note.MARKDOWN")));
    }

    #[test]
    fn test_is_markdown_file_rejects_non_md() {
        assert!(!is_markdown_file(Path::new("note.txt")));
        assert!(!is_markdown_file(Path::new("note.pdf")));
        assert!(!is_markdown_file(Path::new("note")));
    }

    #[test]
    fn test_should_ignore_path_hidden_files() {
        assert!(should_ignore_path(Path::new(".DS_Store")));
        assert!(should_ignore_path(Path::new(".gitignore")));
        assert!(should_ignore_path(Path::new(".hidden.md")));
        assert!(should_ignore_path(Path::new(".config/note.md")));
    }

    #[test]
    fn test_should_ignore_path_temp_files() {
        assert!(should_ignore_path(Path::new("~temp.md")));
        assert!(should_ignore_path(Path::new("$RECYCLE.BIN")));
    }

    #[test]
    fn test_should_ignore_path_swap_files() {
        assert!(should_ignore_path(Path::new("note.md~")));
        assert!(should_ignore_path(Path::new(".note.swp")));
        assert!(should_ignore_path(Path::new(".note.swo")));
    }

    #[test]
    fn test_should_ignore_path_os_files() {
        assert!(should_ignore_path(Path::new("Thumbs.db")));
        assert!(should_ignore_path(Path::new("desktop.ini")));
    }

    #[test]
    fn test_should_ignore_path_temp_extensions() {
        assert!(should_ignore_path(Path::new("note.tmp")));
        assert!(should_ignore_path(Path::new("note.bak")));
        assert!(should_ignore_path(Path::new("note.lock")));
    }

    #[test]
    fn test_should_ignore_path_valid_md() {
        assert!(!should_ignore_path(Path::new("note.md")));
        assert!(!should_ignore_path(Path::new("notes/deep/file.md")));
        assert!(!should_ignore_path(Path::new("My Note.md")));
    }
}
