use regex::Regex;

/// Extracts wikilinks from markdown content.
///
/// Wikilinks are in the format `[[target]]`. Returns a list of unique targets
/// in the order they appear in the content.
pub fn extract_wikilinks(content: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").expect("Invalid wikilink regex");
    let mut seen = std::collections::HashSet::new();
    let mut links = Vec::new();

    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let target = m.as_str().to_string();
            if seen.insert(target.clone()) {
                links.push(target);
            }
        }
    }

    links
}

/// Extracts tags from markdown content.
///
/// Tags are in the format `#tag-name`. Returns a list of unique tags
/// in the order they appear in the content.
pub fn extract_tags(content: &str) -> Vec<String> {
    let re = Regex::new(r"(?:^|\s)#([a-zA-Z0-9_-]+)").expect("Invalid tag regex");
    let heading_re = Regex::new(r"^#{1,6}\s").expect("Invalid heading regex");
    let mut seen = std::collections::HashSet::new();
    let mut tags = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Toggle fenced code block state
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }

        // Skip markdown headings (lines starting with one or more # followed by a space)
        if heading_re.is_match(trimmed) {
            continue;
        }

        for cap in re.captures_iter(line) {
            if let Some(m) = cap.get(1) {
                let tag = m.as_str().to_string();
                if seen.insert(tag.clone()) {
                    tags.push(tag);
                }
            }
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wikilinks_single() {
        let content = "Check out [[My Note]] for more info.";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["My Note"]);
    }

    #[test]
    fn test_extract_wikilinks_multiple() {
        let content = "See [[Note A]] and [[Note B]] and [[Note C]].";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["Note A", "Note B", "Note C"]);
    }

    #[test]
    fn test_extract_wikilinks_duplicates() {
        let content = "See [[Note A]] and [[Note A]] again.";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["Note A"]);
    }

    #[test]
    fn test_extract_wikilinks_none() {
        let content = "No wikilinks here.";
        let links = extract_wikilinks(content);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_wikilinks_nested_brackets() {
        let content = "This is [[Note with [brackets]]] inside.";
        let links = extract_wikilinks(content);
        assert_eq!(links, vec!["Note with [brackets"]);
    }

    #[test]
    fn test_extract_tags_single() {
        let content = "This is a #rust note.";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_extract_tags_multiple() {
        let content = "#rust #programming #notes";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["rust", "programming", "notes"]);
    }

    #[test]
    fn test_extract_tags_duplicates() {
        let content = "#rust and #rust again";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_extract_tags_with_hyphens() {
        let content = "This is #my-tag with #another_tag.";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["my-tag", "another_tag"]);
    }

    #[test]
    fn test_extract_tags_none() {
        let content = "No tags here, just a # symbol.";
        let tags = extract_tags(content);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_extract_tags_not_at_start_of_word() {
        let content = "email#domain is not a tag but #tag is";
        let tags = extract_tags(content);
        assert_eq!(tags, vec!["tag"]);
    }
}
