//! Audit log for tracking all agent and system actions.
//!
//! Every significant action is recorded with timestamp, actor,
//! action type, and details for traceability and debugging.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Maximum number of log entries kept in memory.
const MAX_LOG_SIZE: usize = 10_000;

/// Categories of auditable actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditAction {
    // Agent lifecycle
    AgentRegistered,
    AgentUnregistered,
    AgentConfigUpdated,

    // Task lifecycle
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,

    // Note operations
    NoteCreated,
    NoteEdited,
    NoteDeleted,
    NoteRenamed,

    // System
    VaultPathSet,
    ReindexStarted,
    ReindexCompleted,
    SearchPerformed,

    // LLM
    LlmApiCalled,
    LlmTokensUsed,

    // Budget
    BudgetWarning,
    BudgetExceeded,

    // Git
    GitCommitCreated,
    GitRepoLinked,
    GitRepoUnlinked,

    // Generic
    Custom(String),
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID.
    pub id: u64,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Who performed the action (agent ID, "system", or "user").
    pub actor: String,
    /// What action was performed.
    pub action: AuditAction,
    /// Target of the action (note path, agent ID, etc.).
    pub target: Option<String>,
    /// Additional details as JSON.
    pub details: Option<String>,
    /// Whether the action succeeded.
    pub success: bool,
}

/// In-memory audit log with bounded size.
pub struct AuditLog {
    entries: VecDeque<AuditEntry>,
    next_id: u64,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            next_id: 1,
        }
    }

    /// Records an action in the audit log.
    pub fn record(
        &mut self,
        actor: &str,
        action: AuditAction,
        target: Option<&str>,
        details: Option<&str>,
        success: bool,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let entry = AuditEntry {
            id,
            timestamp: now_millis(),
            actor: actor.into(),
            action,
            target: target.map(String::from),
            details: details.map(String::from),
            success,
        };

        self.entries.push_back(entry);

        // Trim if over max size
        while self.entries.len() > MAX_LOG_SIZE {
            self.entries.pop_front();
        }

        id
    }

    /// Gets all entries (newest first).
    pub fn get_all(&self) -> Vec<&AuditEntry> {
        self.entries.iter().rev().collect()
    }

    /// Gets entries filtered by actor.
    pub fn get_by_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.actor == actor)
            .collect()
    }

    /// Gets entries filtered by action type.
    pub fn get_by_action(&self, action: &AuditAction) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| &e.action == action)
            .collect()
    }

    /// Gets the N most recent entries.
    pub fn get_recent(&self, limit: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    /// Gets entries for a specific target.
    pub fn get_by_target(&self, target: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.target.as_deref() == Some(target))
            .collect()
    }

    /// Gets failed actions.
    pub fn get_failures(&self) -> Vec<&AuditEntry> {
        self.entries.iter().rev().filter(|e| !e.success).collect()
    }

    /// Returns total number of entries.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get() {
        let mut log = AuditLog::new();
        log.record("user", AuditAction::NoteCreated, Some("test.md"), None, true);
        assert_eq!(log.count(), 1);
        assert_eq!(log.get_all()[0].actor, "user");
    }

    #[test]
    fn test_sequential_ids() {
        let mut log = AuditLog::new();
        let id1 = log.record("user", AuditAction::NoteCreated, None, None, true);
        let id2 = log.record("user", AuditAction::NoteEdited, None, None, true);
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_filter_by_actor() {
        let mut log = AuditLog::new();
        log.record("agent-1", AuditAction::TaskStarted, None, None, true);
        log.record("user", AuditAction::NoteCreated, None, None, true);
        log.record("agent-1", AuditAction::TaskCompleted, None, None, true);

        assert_eq!(log.get_by_actor("agent-1").len(), 2);
        assert_eq!(log.get_by_actor("user").len(), 1);
    }

    #[test]
    fn test_filter_by_action() {
        let mut log = AuditLog::new();
        log.record("user", AuditAction::NoteCreated, None, None, true);
        log.record("user", AuditAction::NoteCreated, None, None, true);
        log.record("user", AuditAction::NoteDeleted, None, None, true);

        assert_eq!(log.get_by_action(&AuditAction::NoteCreated).len(), 2);
    }

    #[test]
    fn test_get_recent() {
        let mut log = AuditLog::new();
        for i in 0..10 {
            log.record("user", AuditAction::NoteEdited, Some(&format!("n{i}.md")), None, true);
        }
        assert_eq!(log.get_recent(3).len(), 3);
    }

    #[test]
    fn test_get_failures() {
        let mut log = AuditLog::new();
        log.record("agent", AuditAction::TaskCompleted, None, None, true);
        log.record("agent", AuditAction::TaskFailed, None, Some("timeout"), false);

        assert_eq!(log.get_failures().len(), 1);
        assert_eq!(log.get_failures()[0].action, AuditAction::TaskFailed);
    }

    #[test]
    fn test_max_size_trim() {
        let mut log = AuditLog::new();
        for i in 0..MAX_LOG_SIZE + 100 {
            log.record("user", AuditAction::NoteEdited, None, None, true);
        }
        assert_eq!(log.count(), MAX_LOG_SIZE);
    }

    #[test]
    fn test_clear() {
        let mut log = AuditLog::new();
        log.record("user", AuditAction::NoteCreated, None, None, true);
        log.clear();
        assert_eq!(log.count(), 0);
    }

    #[test]
    fn test_get_by_target() {
        let mut log = AuditLog::new();
        log.record("user", AuditAction::NoteEdited, Some("a.md"), None, true);
        log.record("user", AuditAction::NoteEdited, Some("b.md"), None, true);
        log.record("user", AuditAction::NoteEdited, Some("a.md"), None, true);

        assert_eq!(log.get_by_target("a.md").len(), 2);
    }
}
