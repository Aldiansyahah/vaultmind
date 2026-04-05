//! # agent-runtime
//!
//! LLM integration with tool-calling for knowledge base manipulation.
//! Multi-agent orchestration with heartbeats, tasks, and org hierarchy.

pub mod agent;
pub mod audit;
pub mod budget;
pub mod client;
pub mod heartbeat;
pub mod orchestrator;
pub mod registry;
pub mod tasks;
pub mod tools;

pub use agent::{Agent, AgentResponse, ToolExecutor};
pub use audit::{AuditAction, AuditEntry, AuditLog};
pub use budget::{BudgetCheckResult, BudgetLimit, BudgetManager, BudgetSummary, BudgetUsage};
pub use client::{LlmClient, LlmConfig, Message};
pub use heartbeat::{AgentState, AgentStatusInfo, HeartbeatMonitor};
pub use orchestrator::{AgentOrchestrator, DashboardData, TaskInfo};
pub use registry::{AgentDef, AgentPersona, AgentRegistry, AgentRole, AgentSkill, OrgNode};
pub use tasks::{Task, TaskQueue, TaskSchedule, TaskStatus};
pub use tools::get_tool_definitions;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
