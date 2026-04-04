//! Agent orchestrator that ties together registry, tasks, and heartbeat.
//!
//! Provides a unified interface for managing multiple agents,
//! assigning tasks, and monitoring health.

use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::heartbeat::{AgentState, AgentStatusInfo, HeartbeatMonitor};
use crate::registry::{AgentDef, AgentRegistry, OrgNode};
use crate::tasks::{TaskQueue, TaskSchedule, TaskStatus};
use crate::ToolExecutor;

/// Dashboard data combining all orchestration state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_agents: usize,
    pub agents_online: usize,
    pub agents_busy: usize,
    pub total_tasks: usize,
    pub tasks_pending: usize,
    pub tasks_running: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub agent_statuses: Vec<AgentStatusInfo>,
}

/// Serializable task info for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub agent_id: String,
    pub prompt: String,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub run_count: u32,
}

/// The main orchestrator coordinating agents, tasks, and health.
pub struct AgentOrchestrator {
    pub registry: AgentRegistry,
    pub tasks: TaskQueue,
    pub heartbeat: HeartbeatMonitor,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            tasks: TaskQueue::new(),
            heartbeat: HeartbeatMonitor::default(),
        }
    }

    /// Registers a new agent and starts monitoring it.
    pub fn register_agent(&mut self, def: AgentDef) {
        let id = def.id.clone();
        self.registry.register(def);
        self.heartbeat.heartbeat(&id, AgentState::Idle);
    }

    /// Unregisters an agent and stops monitoring.
    pub fn unregister_agent(&mut self, id: &str) -> Option<AgentDef> {
        self.heartbeat.remove(id);
        self.registry.unregister(id)
    }

    /// Submits a task to an agent's queue.
    pub fn submit_task(
        &mut self,
        agent_id: &str,
        prompt: &str,
        schedule: TaskSchedule,
    ) -> Result<String, String> {
        if self.registry.get(agent_id).is_none() {
            return Err(format!("Agent not found: {agent_id}"));
        }
        Ok(self.tasks.enqueue(agent_id, prompt, schedule))
    }

    /// Executes a pending task using the agent's LLM config.
    pub async fn execute_task(
        &mut self,
        task_id: &str,
        executor: &dyn ToolExecutor,
    ) -> Result<String, String> {
        let task = self.tasks.get(task_id).ok_or("Task not found")?;
        let agent_def = self
            .registry
            .get(&task.agent_id)
            .ok_or("Agent not found")?
            .clone();
        let prompt = task.prompt.clone();
        let agent_id = task.agent_id.clone();
        let task_id = task_id.to_string();

        // Mark task and agent as running
        self.tasks.mark_running(&task_id);
        self.heartbeat.mark_task_start(&agent_id, &task_id);

        // Create agent with the definition's config and prompt
        let config = agent_def.llm_config.clone();
        if config.base_url.is_empty() {
            return Err("Agent has no LLM configured".into());
        }

        let agent = Agent::new(config);
        let full_prompt = format!(
            "Your role: {}\n\n{}",
            agent_def.role, prompt
        );

        match agent.run(&full_prompt, executor).await {
            Ok(response) => {
                self.tasks
                    .mark_completed(&task_id, response.message.clone());
                self.heartbeat.mark_task_end(&agent_id, true);
                Ok(response.message)
            }
            Err(e) => {
                self.tasks.mark_failed(&task_id, e.clone());
                self.heartbeat.mark_task_end(&agent_id, false);
                Err(e)
            }
        }
    }

    /// Gets the org chart tree.
    pub fn get_org_chart(&self) -> Vec<OrgNode> {
        self.registry.get_hierarchy()
    }

    /// Gets dashboard data with aggregated stats.
    pub fn get_dashboard(&self) -> DashboardData {
        let statuses = self.heartbeat.get_all_statuses();
        let all_tasks = self.tasks.list_all();

        DashboardData {
            total_agents: self.registry.count(),
            agents_online: statuses
                .iter()
                .filter(|s| s.state != AgentState::Offline)
                .count(),
            agents_busy: statuses
                .iter()
                .filter(|s| s.state == AgentState::Busy)
                .count(),
            total_tasks: all_tasks.len(),
            tasks_pending: all_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .count(),
            tasks_running: all_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Running)
                .count(),
            tasks_completed: all_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .count(),
            tasks_failed: all_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Failed)
                .count(),
            agent_statuses: statuses,
        }
    }

    /// Lists all tasks with info.
    pub fn list_tasks(&self, agent_id: Option<&str>) -> Vec<TaskInfo> {
        let tasks = if let Some(id) = agent_id {
            self.tasks.list_for_agent(id)
        } else {
            self.tasks.list_all()
        };

        tasks
            .into_iter()
            .map(|t| TaskInfo {
                id: t.id.clone(),
                agent_id: t.agent_id.clone(),
                prompt: t.prompt.clone(),
                status: t.status.clone(),
                result: t.result.clone(),
                error: t.error.clone(),
                run_count: t.run_count,
            })
            .collect()
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::LlmConfig;

    fn make_def(id: &str, role: &str, parent: Option<&str>) -> AgentDef {
        AgentDef {
            id: id.into(),
            name: format!("Agent {id}"),
            role: role.into(),
            system_prompt: format!("You are a {role} agent."),
            llm_config: LlmConfig::default(),
            allowed_tools: vec!["search_notes".into()],
            parent_id: parent.map(String::from),
        }
    }

    #[test]
    fn test_register_and_dashboard() {
        let mut orch = AgentOrchestrator::new();
        orch.register_agent(make_def("researcher", "researcher", None));
        orch.register_agent(make_def("writer", "writer", Some("researcher")));

        let dash = orch.get_dashboard();
        assert_eq!(dash.total_agents, 2);
        assert_eq!(dash.agents_online, 2);
    }

    #[test]
    fn test_submit_task() {
        let mut orch = AgentOrchestrator::new();
        orch.register_agent(make_def("a1", "worker", None));

        let task_id = orch
            .submit_task("a1", "Do something", TaskSchedule::Once)
            .unwrap();
        assert!(!task_id.is_empty());

        let dash = orch.get_dashboard();
        assert_eq!(dash.tasks_pending, 1);
    }

    #[test]
    fn test_submit_task_unknown_agent() {
        let mut orch = AgentOrchestrator::new();
        let result = orch.submit_task("unknown", "task", TaskSchedule::Once);
        assert!(result.is_err());
    }

    #[test]
    fn test_org_chart() {
        let mut orch = AgentOrchestrator::new();
        orch.register_agent(make_def("boss", "manager", None));
        orch.register_agent(make_def("w1", "worker", Some("boss")));
        orch.register_agent(make_def("w2", "worker", Some("boss")));

        let chart = orch.get_org_chart();
        assert_eq!(chart.len(), 1);
        assert_eq!(chart[0].children.len(), 2);
    }

    #[test]
    fn test_list_tasks() {
        let mut orch = AgentOrchestrator::new();
        orch.register_agent(make_def("a1", "worker", None));
        orch.submit_task("a1", "task 1", TaskSchedule::Once).unwrap();
        orch.submit_task("a1", "task 2", TaskSchedule::Once).unwrap();

        let tasks = orch.list_tasks(Some("a1"));
        assert_eq!(tasks.len(), 2);

        let all = orch.list_tasks(None);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_unregister() {
        let mut orch = AgentOrchestrator::new();
        orch.register_agent(make_def("a1", "worker", None));
        orch.unregister_agent("a1");

        assert_eq!(orch.get_dashboard().total_agents, 0);
    }
}
