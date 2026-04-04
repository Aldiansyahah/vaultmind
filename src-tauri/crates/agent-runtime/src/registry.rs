//! Agent registry and organizational hierarchy.
//!
//! Manages agent definitions with parent-child relationships
//! for multi-agent orchestration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::LlmConfig;

/// An agent definition with role, config, and hierarchy position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub role: String,
    pub system_prompt: String,
    pub llm_config: LlmConfig,
    pub allowed_tools: Vec<String>,
    pub parent_id: Option<String>,
}

/// A node in the org chart tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgNode {
    pub agent: AgentDef,
    pub children: Vec<OrgNode>,
}

/// Registry that manages all agent definitions.
pub struct AgentRegistry {
    agents: HashMap<String, AgentDef>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Registers a new agent or updates an existing one.
    pub fn register(&mut self, def: AgentDef) {
        self.agents.insert(def.id.clone(), def);
    }

    /// Removes an agent and orphans its children (sets parent_id to None).
    pub fn unregister(&mut self, id: &str) -> Option<AgentDef> {
        let removed = self.agents.remove(id);
        if removed.is_some() {
            // Orphan children
            let children_ids: Vec<String> = self
                .agents
                .values()
                .filter(|a| a.parent_id.as_deref() == Some(id))
                .map(|a| a.id.clone())
                .collect();
            for child_id in children_ids {
                if let Some(child) = self.agents.get_mut(&child_id) {
                    child.parent_id = None;
                }
            }
        }
        removed
    }

    /// Gets an agent by ID.
    pub fn get(&self, id: &str) -> Option<&AgentDef> {
        self.agents.get(id)
    }

    /// Lists all registered agents.
    pub fn list(&self) -> Vec<&AgentDef> {
        self.agents.values().collect()
    }

    /// Gets direct children of an agent.
    pub fn get_children(&self, parent_id: &str) -> Vec<&AgentDef> {
        self.agents
            .values()
            .filter(|a| a.parent_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Gets root agents (no parent).
    pub fn get_roots(&self) -> Vec<&AgentDef> {
        self.agents
            .values()
            .filter(|a| a.parent_id.is_none())
            .collect()
    }

    /// Builds the full org chart as a tree.
    pub fn get_hierarchy(&self) -> Vec<OrgNode> {
        let roots = self.get_roots();
        roots.iter().map(|r| self.build_tree(r)).collect()
    }

    fn build_tree(&self, agent: &AgentDef) -> OrgNode {
        let children = self.get_children(&agent.id);
        OrgNode {
            agent: agent.clone(),
            children: children.iter().map(|c| self.build_tree(c)).collect(),
        }
    }

    /// Returns total number of agents.
    pub fn count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: &str, parent: Option<&str>) -> AgentDef {
        AgentDef {
            id: id.into(),
            name: format!("Agent {id}"),
            role: "general".into(),
            system_prompt: "You are helpful.".into(),
            llm_config: LlmConfig::default(),
            allowed_tools: vec!["search_notes".into()],
            parent_id: parent.map(String::from),
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("a1", None));
        assert!(reg.get("a1").is_some());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("a1", None));
        reg.unregister("a1");
        assert!(reg.get("a1").is_none());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_hierarchy() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("boss", None));
        reg.register(make_agent("worker1", Some("boss")));
        reg.register(make_agent("worker2", Some("boss")));

        let roots = reg.get_roots();
        assert_eq!(roots.len(), 1);

        let children = reg.get_children("boss");
        assert_eq!(children.len(), 2);

        let tree = reg.get_hierarchy();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn test_unregister_orphans_children() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("parent", None));
        reg.register(make_agent("child", Some("parent")));

        reg.unregister("parent");
        let child = reg.get("child").unwrap();
        assert!(child.parent_id.is_none());
    }

    #[test]
    fn test_list() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("a", None));
        reg.register(make_agent("b", None));
        assert_eq!(reg.list().len(), 2);
    }
}
