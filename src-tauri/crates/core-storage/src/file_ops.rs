use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, StorageError};

/// Validates that a resolved path is contained within the vault directory.
/// Prevents path traversal attacks (e.g., `../../etc/passwd`).
fn validate_within_vault(vault_path: &Path, full_path: &Path) -> Result<()> {
    let canonical_vault = vault_path
        .canonicalize()
        .map_err(StorageError::from)?;
    // For paths that don't exist yet, canonicalize the parent
    let canonical_full = if full_path.exists() {
        full_path.canonicalize().map_err(StorageError::from)?
    } else if let Some(parent) = full_path.parent() {
        if parent.exists() {
            let canonical_parent = parent.canonicalize().map_err(StorageError::from)?;
            canonical_parent.join(full_path.file_name().unwrap_or_default())
        } else {
            // Parent doesn't exist yet — resolve as much as possible
            full_path.to_path_buf()
        }
    } else {
        full_path.to_path_buf()
    };

    if !canonical_full.starts_with(&canonical_vault) {
        return Err(StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("Path escapes vault directory: {}", full_path.display()),
        )));
    }
    Ok(())
}

/// Represents a file or directory entry in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<VaultEntry>>,
}

/// Lists all files and directories in the vault, returning a tree structure.
pub fn list_vault_files(vault_path: &Path) -> Result<Vec<VaultEntry>> {
    if !vault_path.exists() {
        return Err(StorageError::NotFound(format!(
            "Vault path not found: {vault_path:?}"
        )));
    }

    if !vault_path.is_dir() {
        return Err(StorageError::NotFound(format!(
            "Vault path is not a directory: {vault_path:?}"
        )));
    }

    let mut entries = Vec::new();
    read_dir_recursive(vault_path, vault_path, &mut entries)?;
    entries.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

fn read_dir_recursive(root: &Path, current: &Path, entries: &mut Vec<VaultEntry>) -> Result<()> {
    let read_dir = fs::read_dir(current).map_err(StorageError::from)?;

    for entry_result in read_dir {
        let entry = entry_result.map_err(StorageError::from)?;
        let path = entry.path();
        let file_name = entry.file_name().to_str().unwrap_or("").to_string();

        if should_skip_entry(&file_name) {
            continue;
        }

        let is_directory = path.is_dir();
        let relative_path = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| file_name.clone());

        let children = if is_directory {
            let mut child_entries = Vec::new();
            read_dir_recursive(root, &path, &mut child_entries)?;
            child_entries.sort_by(|a, b| {
                b.is_directory
                    .cmp(&a.is_directory)
                    .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
            Some(child_entries)
        } else {
            None
        };

        entries.push(VaultEntry {
            name: file_name,
            path: relative_path,
            is_directory,
            children,
        });
    }

    Ok(())
}

fn should_skip_entry(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }

    let skip_names = ["node_modules", "target", ".git", ".svn"];
    if skip_names.contains(&name) {
        return true;
    }

    false
}

/// Creates a new markdown file in the vault.
///
/// If the path contains directories that don't exist, they will be created.
pub fn create_note(vault_path: &Path, relative_path: &str, content: &str) -> Result<PathBuf> {
    let full_path = vault_path.join(relative_path);
    validate_within_vault(vault_path, &full_path)?;

    if full_path.exists() {
        return Err(StorageError::Duplicate(format!(
            "File already exists: {relative_path}"
        )));
    }

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(StorageError::from)?;
    }

    fs::write(&full_path, content).map_err(StorageError::from)?;

    Ok(full_path)
}

/// Renames a file or directory in the vault.
pub fn rename_note(
    vault_path: &Path,
    old_relative_path: &str,
    new_relative_path: &str,
) -> Result<()> {
    let old_full_path = vault_path.join(old_relative_path);
    let new_full_path = vault_path.join(new_relative_path);
    validate_within_vault(vault_path, &old_full_path)?;
    validate_within_vault(vault_path, &new_full_path)?;

    if !old_full_path.exists() {
        return Err(StorageError::NotFound(format!(
            "File not found: {old_relative_path}"
        )));
    }

    if new_full_path.exists() {
        return Err(StorageError::Duplicate(format!(
            "Target already exists: {new_relative_path}"
        )));
    }

    if let Some(parent) = new_full_path.parent() {
        fs::create_dir_all(parent).map_err(StorageError::from)?;
    }

    fs::rename(&old_full_path, &new_full_path).map_err(StorageError::from)?;

    Ok(())
}

/// Deletes a file or directory from the vault.
pub fn delete_note(vault_path: &Path, relative_path: &str) -> Result<()> {
    let full_path = vault_path.join(relative_path);
    validate_within_vault(vault_path, &full_path)?;

    if !full_path.exists() {
        return Err(StorageError::NotFound(format!(
            "File not found: {relative_path}"
        )));
    }

    if full_path.is_dir() {
        fs::remove_dir_all(&full_path).map_err(StorageError::from)?;
    } else {
        fs::remove_file(&full_path).map_err(StorageError::from)?;
    }

    Ok(())
}

/// Moves a file or directory to a new location within the vault.
pub fn move_note(
    vault_path: &Path,
    old_relative_path: &str,
    new_relative_path: &str,
) -> Result<()> {
    let old_full_path = vault_path.join(old_relative_path);
    let new_full_path = vault_path.join(new_relative_path);
    validate_within_vault(vault_path, &old_full_path)?;
    validate_within_vault(vault_path, &new_full_path)?;

    if !old_full_path.exists() {
        return Err(StorageError::NotFound(format!(
            "File not found: {old_relative_path}"
        )));
    }

    if new_full_path.exists() {
        return Err(StorageError::Duplicate(format!(
            "Target already exists: {new_relative_path}"
        )));
    }

    if let Some(parent) = new_full_path.parent() {
        fs::create_dir_all(parent).map_err(StorageError::from)?;
    }

    fs::rename(&old_full_path, &new_full_path).map_err(StorageError::from)?;

    Ok(())
}

/// Reads the content of a note file.
pub fn read_note_content(vault_path: &Path, relative_path: &str) -> Result<String> {
    let full_path = vault_path.join(relative_path);
    validate_within_vault(vault_path, &full_path)?;

    if !full_path.exists() {
        return Err(StorageError::NotFound(format!(
            "File not found: {relative_path}"
        )));
    }

    if !full_path.is_file() {
        return Err(StorageError::NotFound(format!(
            "Path is not a file: {relative_path}"
        )));
    }

    fs::read_to_string(&full_path).map_err(StorageError::from)
}

/// Writes content to a note file.
pub fn write_note_content(vault_path: &Path, relative_path: &str, content: &str) -> Result<()> {
    let full_path = vault_path.join(relative_path);
    validate_within_vault(vault_path, &full_path)?;

    if !full_path.exists() {
        return Err(StorageError::NotFound(format!(
            "File not found: {relative_path}"
        )));
    }

    if !full_path.is_file() {
        return Err(StorageError::NotFound(format!(
            "Path is not a file: {relative_path}"
        )));
    }

    fs::write(&full_path, content).map_err(StorageError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_vault() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_list_vault_files_empty() {
        let vault = create_temp_vault();
        let entries = list_vault_files(vault.path()).expect("Failed to list files");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_list_vault_files_with_files() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("a.md"), "# A").expect("Failed to create file");
        fs::write(vault.path().join("b.md"), "# B").expect("Failed to create file");

        let entries = list_vault_files(vault.path()).expect("Failed to list files");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.md");
        assert_eq!(entries[1].name, "b.md");
    }

    #[test]
    fn test_list_vault_files_with_directories() {
        let vault = create_temp_vault();
        fs::create_dir(vault.path().join("notes")).expect("Failed to create dir");
        fs::write(vault.path().join("notes/test.md"), "# Test").expect("Failed to create file");
        fs::write(vault.path().join("readme.md"), "# Readme").expect("Failed to create file");

        let entries = list_vault_files(vault.path()).expect("Failed to list files");
        assert_eq!(entries.len(), 2);

        let dir_entry = entries
            .iter()
            .find(|e| e.is_directory)
            .expect("Directory not found");
        assert_eq!(dir_entry.name, "notes");
        assert!(dir_entry.children.is_some());
        assert_eq!(dir_entry.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_list_vault_files_skips_hidden() {
        let vault = create_temp_vault();
        fs::write(vault.path().join(".hidden.md"), "# Hidden").expect("Failed to create file");
        fs::write(vault.path().join("visible.md"), "# Visible").expect("Failed to create file");

        let entries = list_vault_files(vault.path()).expect("Failed to list files");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible.md");
    }

    #[test]
    fn test_create_note() {
        let vault = create_temp_vault();
        let path =
            create_note(vault.path(), "new.md", "# New Note").expect("Failed to create note");
        assert!(path.exists());
        let content = fs::read_to_string(&path).expect("Failed to read file");
        assert_eq!(content, "# New Note");
    }

    #[test]
    fn test_create_note_with_nested_dirs() {
        let vault = create_temp_vault();
        let path = create_note(vault.path(), "deep/nested/note.md", "# Nested")
            .expect("Failed to create note");
        assert!(path.exists());
    }

    #[test]
    fn test_create_note_duplicate_rejected() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("existing.md"), "# Existing").expect("Failed to create file");

        let result = create_note(vault.path(), "existing.md", "# Duplicate");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_note() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("old.md"), "# Old").expect("Failed to create file");

        rename_note(vault.path(), "old.md", "new.md").expect("Failed to rename");
        assert!(!vault.path().join("old.md").exists());
        assert!(vault.path().join("new.md").exists());
    }

    #[test]
    fn test_rename_note_not_found() {
        let vault = create_temp_vault();
        let result = rename_note(vault.path(), "nonexistent.md", "new.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_note_file() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("delete.md"), "# Delete").expect("Failed to create file");

        delete_note(vault.path(), "delete.md").expect("Failed to delete");
        assert!(!vault.path().join("delete.md").exists());
    }

    #[test]
    fn test_delete_note_directory() {
        let vault = create_temp_vault();
        fs::create_dir(vault.path().join("dir")).expect("Failed to create dir");
        fs::write(vault.path().join("dir/file.md"), "# File").expect("Failed to create file");

        delete_note(vault.path(), "dir").expect("Failed to delete");
        assert!(!vault.path().join("dir").exists());
    }

    #[test]
    fn test_delete_note_not_found() {
        let vault = create_temp_vault();
        let result = delete_note(vault.path(), "nonexistent.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_move_note() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("source.md"), "# Source").expect("Failed to create file");

        move_note(vault.path(), "source.md", "dest.md").expect("Failed to move");
        assert!(!vault.path().join("source.md").exists());
        assert!(vault.path().join("dest.md").exists());
    }

    #[test]
    fn test_read_note_content() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("read.md"), "# Content").expect("Failed to create file");

        let content = read_note_content(vault.path(), "read.md").expect("Failed to read");
        assert_eq!(content, "# Content");
    }

    #[test]
    fn test_write_note_content() {
        let vault = create_temp_vault();
        fs::write(vault.path().join("write.md"), "# Old").expect("Failed to create file");

        write_note_content(vault.path(), "write.md", "# New").expect("Failed to write");
        let content = fs::read_to_string(vault.path().join("write.md")).expect("Failed to read");
        assert_eq!(content, "# New");
    }
}
