//! Git operations for version history and linked repositories.
//!
//! Provides git integration for:
//! - Auto-committing note changes (version history)
//! - Viewing file history (log)
//! - Linked external repositories
//!
//! Uses `git2` is not available, so we shell out to `git` CLI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{Result, StorageError};

/// A git commit entry for version history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub files_changed: Vec<String>,
}

/// A linked git repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedRepo {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub remote_url: Option<String>,
    pub description: String,
}

/// Manages git operations for the vault.
pub struct GitManager {
    vault_path: PathBuf,
    linked_repos: HashMap<String, LinkedRepo>,
}

impl GitManager {
    /// Creates a new GitManager for the given vault path.
    pub fn new(vault_path: &Path) -> Self {
        Self {
            vault_path: vault_path.to_path_buf(),
            linked_repos: HashMap::new(),
        }
    }

    /// Initializes a git repo in the vault if not already initialized.
    pub fn init_repo(&self) -> Result<bool> {
        if self.vault_path.join(".git").exists() {
            return Ok(false); // Already initialized
        }

        run_git(&self.vault_path, &["init"])?;
        // Create .gitignore
        let gitignore = self.vault_path.join(".gitignore");
        if !gitignore.exists() {
            std::fs::write(&gitignore, ".vaultmind/\n.DS_Store\n")
                .map_err(StorageError::from)?;
        }
        Ok(true)
    }

    /// Auto-commits changes with a descriptive message.
    pub fn auto_commit(&self, message: &str) -> Result<Option<String>> {
        if !self.vault_path.join(".git").exists() {
            return Ok(None);
        }

        // Stage all changes
        run_git(&self.vault_path, &["add", "-A"])?;

        // Check if there are staged changes
        let status = run_git(&self.vault_path, &["status", "--porcelain"])?;
        if status.trim().is_empty() {
            return Ok(None); // Nothing to commit
        }

        run_git(&self.vault_path, &["commit", "-m", message])?;

        // Get the commit hash
        let hash = run_git(&self.vault_path, &["rev-parse", "--short", "HEAD"])?;
        Ok(Some(hash.trim().to_string()))
    }

    /// Gets the version history for a specific file.
    pub fn file_history(&self, relative_path: &str, limit: usize) -> Result<Vec<GitCommit>> {
        if !self.vault_path.join(".git").exists() {
            return Ok(Vec::new());
        }

        let output = run_git(
            &self.vault_path,
            &[
                "log",
                &format!("-{limit}"),
                "--format=%H|%s|%an|%ai",
                "--follow",
                "--",
                relative_path,
            ],
        )?;

        let commits = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                GitCommit {
                    hash: parts.first().unwrap_or(&"").to_string(),
                    message: parts.get(1).unwrap_or(&"").to_string(),
                    author: parts.get(2).unwrap_or(&"").to_string(),
                    timestamp: parts.get(3).unwrap_or(&"").to_string(),
                    files_changed: vec![relative_path.to_string()],
                }
            })
            .collect();

        Ok(commits)
    }

    /// Gets recent commits across the whole vault.
    pub fn recent_commits(&self, limit: usize) -> Result<Vec<GitCommit>> {
        if !self.vault_path.join(".git").exists() {
            return Ok(Vec::new());
        }

        let output = run_git(
            &self.vault_path,
            &[
                "log",
                &format!("-{limit}"),
                "--format=%H|%s|%an|%ai",
                "--name-only",
            ],
        )?;

        let mut commits = Vec::new();
        let mut current: Option<GitCommit> = None;

        for line in output.lines() {
            if line.contains('|') && line.len() > 20 {
                // This is a commit line
                if let Some(commit) = current.take() {
                    commits.push(commit);
                }
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                current = Some(GitCommit {
                    hash: parts.first().unwrap_or(&"").to_string(),
                    message: parts.get(1).unwrap_or(&"").to_string(),
                    author: parts.get(2).unwrap_or(&"").to_string(),
                    timestamp: parts.get(3).unwrap_or(&"").to_string(),
                    files_changed: Vec::new(),
                });
            } else if !line.is_empty() {
                // This is a file name
                if let Some(ref mut commit) = current {
                    commit.files_changed.push(line.to_string());
                }
            }
        }
        if let Some(commit) = current {
            commits.push(commit);
        }

        Ok(commits)
    }

    /// Gets the content of a file at a specific commit.
    pub fn file_at_commit(&self, relative_path: &str, commit_hash: &str) -> Result<String> {
        let spec = format!("{commit_hash}:{relative_path}");
        run_git(&self.vault_path, &["show", &spec])
    }

    /// Gets a diff between two commits for a file.
    pub fn file_diff(&self, relative_path: &str, old_hash: &str, new_hash: &str) -> Result<String> {
        run_git(
            &self.vault_path,
            &["diff", old_hash, new_hash, "--", relative_path],
        )
    }

    // --- Linked Repos ---

    /// Links an external git repository.
    pub fn link_repo(&mut self, repo: LinkedRepo) -> Result<()> {
        if !repo.path.exists() {
            return Err(StorageError::NotFound(format!(
                "Repo path not found: {:?}",
                repo.path
            )));
        }
        self.linked_repos.insert(repo.id.clone(), repo);
        Ok(())
    }

    /// Unlinks a repository.
    pub fn unlink_repo(&mut self, id: &str) -> Option<LinkedRepo> {
        self.linked_repos.remove(id)
    }

    /// Lists all linked repositories.
    pub fn list_repos(&self) -> Vec<&LinkedRepo> {
        self.linked_repos.values().collect()
    }

    /// Gets a linked repo by ID.
    pub fn get_repo(&self, id: &str) -> Option<&LinkedRepo> {
        self.linked_repos.get(id)
    }

    /// Checks if git is available on the system.
    pub fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Runs a git command and returns stdout.
fn run_git(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(StorageError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(StorageError::Io(std::io::Error::other(
            format!("git {} failed: {}", args.join(" "), stderr),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_available() {
        // Git should be available in CI and dev environments
        let available = GitManager::git_available();
        // Don't assert — might not be installed in all envs
        println!("Git available: {available}");
    }

    #[test]
    fn test_init_repo() {
        if !GitManager::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let mgr = GitManager::new(dir.path());

        let created = mgr.init_repo().unwrap();
        assert!(created);
        assert!(dir.path().join(".git").exists());

        // Second init should return false
        let created2 = mgr.init_repo().unwrap();
        assert!(!created2);
    }

    #[test]
    fn test_auto_commit() {
        if !GitManager::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let mgr = GitManager::new(dir.path());
        mgr.init_repo().unwrap();

        // Configure git user for commits
        run_git(dir.path(), &["config", "user.email", "test@test.com"]).unwrap();
        run_git(dir.path(), &["config", "user.name", "Test"]).unwrap();

        // Create a file and commit
        std::fs::write(dir.path().join("test.md"), "# Hello").unwrap();
        let hash = mgr.auto_commit("Initial commit").unwrap();
        assert!(hash.is_some());

        // No changes = no commit
        let hash2 = mgr.auto_commit("No changes").unwrap();
        assert!(hash2.is_none());
    }

    #[test]
    fn test_file_history() {
        if !GitManager::git_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let mgr = GitManager::new(dir.path());
        mgr.init_repo().unwrap();
        run_git(dir.path(), &["config", "user.email", "test@test.com"]).unwrap();
        run_git(dir.path(), &["config", "user.name", "Test"]).unwrap();

        std::fs::write(dir.path().join("note.md"), "v1").unwrap();
        mgr.auto_commit("Version 1").unwrap();

        std::fs::write(dir.path().join("note.md"), "v2").unwrap();
        mgr.auto_commit("Version 2").unwrap();

        let history = mgr.file_history("note.md", 10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].message, "Version 2");
    }

    #[test]
    fn test_linked_repos() {
        let dir = tempfile::tempdir().unwrap();
        let mut mgr = GitManager::new(dir.path());

        let repo = LinkedRepo {
            id: "repo1".into(),
            name: "My Repo".into(),
            path: dir.path().to_path_buf(),
            remote_url: Some("https://github.com/test/repo".into()),
            description: "Test repo".into(),
        };

        mgr.link_repo(repo).unwrap();
        assert_eq!(mgr.list_repos().len(), 1);
        assert!(mgr.get_repo("repo1").is_some());

        mgr.unlink_repo("repo1");
        assert_eq!(mgr.list_repos().len(), 0);
    }
}
