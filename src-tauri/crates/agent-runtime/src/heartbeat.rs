//! Agent heartbeat monitoring and health tracking.
//!
//! Tracks agent state, last activity, and task statistics.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// The operational state of an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Busy,
    Offline,
    Error,
}

/// Health and activity status of an agent.
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub agent_id: String,
    pub state: AgentState,
    pub last_heartbeat: Instant,
    pub current_task: Option<String>,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
}

/// Serializable version of AgentStatus for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusInfo {
    pub agent_id: String,
    pub state: AgentState,
    pub seconds_since_heartbeat: u64,
    pub current_task: Option<String>,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
}

impl AgentStatus {
    fn to_info(&self) -> AgentStatusInfo {
        AgentStatusInfo {
            agent_id: self.agent_id.clone(),
            state: self.state.clone(),
            seconds_since_heartbeat: self.last_heartbeat.elapsed().as_secs(),
            current_task: self.current_task.clone(),
            tasks_completed: self.tasks_completed,
            tasks_failed: self.tasks_failed,
        }
    }
}

/// Monitors agent health via heartbeats.
pub struct HeartbeatMonitor {
    statuses: HashMap<String, AgentStatus>,
    timeout: Duration,
}

impl HeartbeatMonitor {
    /// Creates a new monitor with the given offline timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            statuses: HashMap::new(),
            timeout,
        }
    }

    /// Records a heartbeat from an agent.
    pub fn heartbeat(&mut self, agent_id: &str, state: AgentState) {
        let status = self
            .statuses
            .entry(agent_id.into())
            .or_insert_with(|| AgentStatus {
                agent_id: agent_id.into(),
                state: AgentState::Idle,
                last_heartbeat: Instant::now(),
                current_task: None,
                tasks_completed: 0,
                tasks_failed: 0,
            });
        status.state = state;
        status.last_heartbeat = Instant::now();
    }

    /// Gets status of a specific agent.
    pub fn get_status(&self, agent_id: &str) -> Option<AgentStatusInfo> {
        self.statuses.get(agent_id).map(|s| s.to_info())
    }

    /// Gets status of all agents.
    pub fn get_all_statuses(&self) -> Vec<AgentStatusInfo> {
        self.statuses.values().map(|s| s.to_info()).collect()
    }

    /// Gets agents that haven't sent a heartbeat within the timeout.
    pub fn get_offline_agents(&self) -> Vec<AgentStatusInfo> {
        self.statuses
            .values()
            .filter(|s| s.last_heartbeat.elapsed() > self.timeout)
            .map(|s| {
                let mut info = s.to_info();
                info.state = AgentState::Offline;
                info
            })
            .collect()
    }

    /// Marks an agent as starting a task.
    pub fn mark_task_start(&mut self, agent_id: &str, task_id: &str) {
        if let Some(status) = self.statuses.get_mut(agent_id) {
            status.state = AgentState::Busy;
            status.current_task = Some(task_id.into());
            status.last_heartbeat = Instant::now();
        }
    }

    /// Marks an agent as finishing a task.
    pub fn mark_task_end(&mut self, agent_id: &str, success: bool) {
        if let Some(status) = self.statuses.get_mut(agent_id) {
            status.state = AgentState::Idle;
            status.current_task = None;
            status.last_heartbeat = Instant::now();
            if success {
                status.tasks_completed += 1;
            } else {
                status.tasks_failed += 1;
            }
        }
    }

    /// Removes an agent from monitoring.
    pub fn remove(&mut self, agent_id: &str) {
        self.statuses.remove(agent_id);
    }

    /// Returns number of monitored agents.
    pub fn count(&self) -> usize {
        self.statuses.len()
    }
}

impl Default for HeartbeatMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat() {
        let mut monitor = HeartbeatMonitor::default();
        monitor.heartbeat("a1", AgentState::Idle);

        let status = monitor.get_status("a1").unwrap();
        assert_eq!(status.state, AgentState::Idle);
        assert_eq!(status.agent_id, "a1");
    }

    #[test]
    fn test_task_tracking() {
        let mut monitor = HeartbeatMonitor::default();
        monitor.heartbeat("a1", AgentState::Idle);

        monitor.mark_task_start("a1", "task-1");
        let status = monitor.get_status("a1").unwrap();
        assert_eq!(status.state, AgentState::Busy);
        assert_eq!(status.current_task.as_deref(), Some("task-1"));

        monitor.mark_task_end("a1", true);
        let status = monitor.get_status("a1").unwrap();
        assert_eq!(status.state, AgentState::Idle);
        assert!(status.current_task.is_none());
        assert_eq!(status.tasks_completed, 1);
    }

    #[test]
    fn test_failed_task() {
        let mut monitor = HeartbeatMonitor::default();
        monitor.heartbeat("a1", AgentState::Idle);
        monitor.mark_task_start("a1", "task-1");
        monitor.mark_task_end("a1", false);

        let status = monitor.get_status("a1").unwrap();
        assert_eq!(status.tasks_failed, 1);
        assert_eq!(status.tasks_completed, 0);
    }

    #[test]
    fn test_offline_detection() {
        let mut monitor = HeartbeatMonitor::new(Duration::from_millis(1));
        monitor.heartbeat("a1", AgentState::Idle);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(5));

        let offline = monitor.get_offline_agents();
        assert_eq!(offline.len(), 1);
        assert_eq!(offline[0].state, AgentState::Offline);
    }

    #[test]
    fn test_all_statuses() {
        let mut monitor = HeartbeatMonitor::default();
        monitor.heartbeat("a1", AgentState::Idle);
        monitor.heartbeat("a2", AgentState::Busy);

        assert_eq!(monitor.get_all_statuses().len(), 2);
    }

    #[test]
    fn test_remove() {
        let mut monitor = HeartbeatMonitor::default();
        monitor.heartbeat("a1", AgentState::Idle);
        monitor.remove("a1");
        assert_eq!(monitor.count(), 0);
    }
}
