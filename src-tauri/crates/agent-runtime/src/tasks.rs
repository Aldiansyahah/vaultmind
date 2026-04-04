//! Mission / task system for multi-agent orchestration.
//!
//! Tasks are units of work assigned to agents. They can be one-shot
//! or scheduled on intervals.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Status of a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// How a task should be scheduled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSchedule {
    /// Run once immediately.
    Once,
    /// Run on a fixed interval.
    Interval(Duration),
    /// Cron-like schedule string (e.g., "0 */6 * * *").
    Cron(String),
}

/// A task assigned to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub prompt: String,
    pub status: TaskStatus,
    pub schedule: TaskSchedule,
    #[serde(skip)]
    pub created_at: Option<Instant>,
    #[serde(skip)]
    pub started_at: Option<Instant>,
    #[serde(skip)]
    pub completed_at: Option<Instant>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub run_count: u32,
}

/// Queue that manages all tasks.
pub struct TaskQueue {
    tasks: HashMap<String, Task>,
    next_id: u64,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }

    /// Enqueues a new task and returns its ID.
    pub fn enqueue(&mut self, agent_id: &str, prompt: &str, schedule: TaskSchedule) -> String {
        let id = format!("task-{}", self.next_id);
        self.next_id += 1;

        let task = Task {
            id: id.clone(),
            agent_id: agent_id.into(),
            prompt: prompt.into(),
            status: TaskStatus::Pending,
            schedule,
            created_at: Some(Instant::now()),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            run_count: 0,
        };

        self.tasks.insert(id.clone(), task);
        id
    }

    /// Cancels a task.
    pub fn cancel(&mut self, task_id: &str) -> bool {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Running {
                task.status = TaskStatus::Cancelled;
                return true;
            }
        }
        false
    }

    /// Gets a task by ID.
    pub fn get(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Lists pending tasks for a specific agent.
    pub fn list_pending(&self, agent_id: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.agent_id == agent_id && t.status == TaskStatus::Pending)
            .collect()
    }

    /// Lists all tasks.
    pub fn list_all(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// Lists tasks for a specific agent.
    pub fn list_for_agent(&self, agent_id: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| t.agent_id == agent_id)
            .collect()
    }

    /// Marks a task as running.
    pub fn mark_running(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Running;
            task.started_at = Some(Instant::now());
        }
    }

    /// Marks a task as completed with a result.
    pub fn mark_completed(&mut self, task_id: &str, result: String) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Instant::now());
            task.result = Some(result);
            task.run_count += 1;
        }
    }

    /// Marks a task as failed with an error.
    pub fn mark_failed(&mut self, task_id: &str, error: String) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Failed;
            task.completed_at = Some(Instant::now());
            task.error = Some(error);
            task.run_count += 1;
        }
    }

    /// Gets tasks with recurring schedules.
    pub fn get_scheduled(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|t| !matches!(t.schedule, TaskSchedule::Once) && t.status != TaskStatus::Cancelled)
            .collect()
    }

    /// Returns total number of tasks.
    pub fn count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue() {
        let mut q = TaskQueue::new();
        let id = q.enqueue("agent-1", "Search for Rust notes", TaskSchedule::Once);
        assert!(!id.is_empty());
        assert_eq!(q.count(), 1);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut q = TaskQueue::new();
        let id = q.enqueue("agent-1", "Do work", TaskSchedule::Once);

        assert_eq!(q.get(&id).unwrap().status, TaskStatus::Pending);

        q.mark_running(&id);
        assert_eq!(q.get(&id).unwrap().status, TaskStatus::Running);

        q.mark_completed(&id, "Done!".into());
        let task = q.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result.as_deref(), Some("Done!"));
        assert_eq!(task.run_count, 1);
    }

    #[test]
    fn test_task_failure() {
        let mut q = TaskQueue::new();
        let id = q.enqueue("agent-1", "Do work", TaskSchedule::Once);
        q.mark_running(&id);
        q.mark_failed(&id, "Something broke".into());

        let task = q.get(&id).unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error.as_deref(), Some("Something broke"));
    }

    #[test]
    fn test_cancel() {
        let mut q = TaskQueue::new();
        let id = q.enqueue("agent-1", "Do work", TaskSchedule::Once);
        assert!(q.cancel(&id));
        assert_eq!(q.get(&id).unwrap().status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_list_pending() {
        let mut q = TaskQueue::new();
        q.enqueue("a1", "task 1", TaskSchedule::Once);
        q.enqueue("a1", "task 2", TaskSchedule::Once);
        q.enqueue("a2", "task 3", TaskSchedule::Once);

        assert_eq!(q.list_pending("a1").len(), 2);
        assert_eq!(q.list_pending("a2").len(), 1);
    }

    #[test]
    fn test_scheduled_tasks() {
        let mut q = TaskQueue::new();
        q.enqueue("a1", "once", TaskSchedule::Once);
        q.enqueue("a1", "interval", TaskSchedule::Interval(Duration::from_secs(3600)));
        q.enqueue("a1", "cron", TaskSchedule::Cron("0 * * * *".into()));

        assert_eq!(q.get_scheduled().len(), 2);
    }

    #[test]
    fn test_list_for_agent() {
        let mut q = TaskQueue::new();
        q.enqueue("a1", "t1", TaskSchedule::Once);
        q.enqueue("a2", "t2", TaskSchedule::Once);

        assert_eq!(q.list_for_agent("a1").len(), 1);
    }
}
