//! Agent registry and organizational hierarchy.
//!
//! Manages agent definitions with parent-child relationships
//! for multi-agent orchestration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::LlmConfig;

/// The operational role of an agent in the org hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentRole {
    /// Executes tasks directly (search, write, edit).
    Executor,
    /// Reviews and validates work from executor agents.
    Supervisor,
    /// Both executes and supervises sub-agents.
    Lead,
}

/// A skill that an agent possesses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    /// Skill identifier (e.g., "research", "summarize", "code-review").
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Proficiency level (0.0 - 1.0).
    pub proficiency: f32,
}

/// Persona configuration for an agent's behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersona {
    /// Short description of who this agent is.
    pub description: String,
    /// Communication style (e.g., "concise", "detailed", "academic").
    pub tone: String,
    /// Language preference (e.g., "en", "id").
    pub language: String,
    /// Domain expertise areas.
    pub expertise: Vec<String>,
}

impl Default for AgentPersona {
    fn default() -> Self {
        Self {
            description: String::new(),
            tone: "concise".into(),
            language: "en".into(),
            expertise: Vec::new(),
        }
    }
}

/// An agent definition with role, persona, skills, and hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub persona: AgentPersona,
    pub skills: Vec<AgentSkill>,
    pub system_prompt: String,
    pub llm_config: LlmConfig,
    pub allowed_tools: Vec<String>,
    pub parent_id: Option<String>,
    /// IDs of agents this supervisor can review.
    pub supervises: Vec<String>,
    /// Maximum concurrent tasks this agent can handle.
    pub max_concurrent_tasks: usize,
    /// Whether this agent is enabled.
    pub enabled: bool,
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

    /// Gets all supervisors for an agent.
    pub fn get_supervisors(&self, agent_id: &str) -> Vec<&AgentDef> {
        self.agents
            .values()
            .filter(|a| a.supervises.contains(&agent_id.to_string()))
            .collect()
    }

    /// Gets all executors (agents supervised by a supervisor).
    pub fn get_supervised(&self, supervisor_id: &str) -> Vec<&AgentDef> {
        if let Some(sup) = self.agents.get(supervisor_id) {
            sup.supervises
                .iter()
                .filter_map(|id| self.agents.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets all enabled agents.
    pub fn get_enabled(&self) -> Vec<&AgentDef> {
        self.agents.values().filter(|a| a.enabled).collect()
    }

    /// Gets agents by role.
    pub fn get_by_role(&self, role: &AgentRole) -> Vec<&AgentDef> {
        self.agents.values().filter(|a| &a.role == role).collect()
    }

    /// Finds the best agent for a skill.
    pub fn find_by_skill(&self, skill_id: &str) -> Option<&AgentDef> {
        self.agents
            .values()
            .filter(|a| a.enabled)
            .filter(|a| a.skills.iter().any(|s| s.id == skill_id))
            .max_by(|a, b| {
                let a_prof = a.skills.iter().find(|s| s.id == skill_id).map(|s| s.proficiency).unwrap_or(0.0);
                let b_prof = b.skills.iter().find(|s| s.id == skill_id).map(|s| s.proficiency).unwrap_or(0.0);
                a_prof.partial_cmp(&b_prof).unwrap_or(std::cmp::Ordering::Equal)
            })
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
            role: AgentRole::Executor,
            persona: AgentPersona::default(),
            skills: vec![],
            system_prompt: "You are helpful.".into(),
            llm_config: LlmConfig::default(),
            allowed_tools: vec!["search_notes".into()],
            parent_id: parent.map(String::from),
            supervises: vec![],
            max_concurrent_tasks: 1,
            enabled: true,
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

    #[test]
    fn test_supervisor_relationship() {
        let mut reg = AgentRegistry::new();
        let mut sup = make_agent("supervisor", None);
        sup.role = AgentRole::Supervisor;
        sup.supervises = vec!["worker1".into(), "worker2".into()];
        reg.register(sup);
        reg.register(make_agent("worker1", Some("supervisor")));
        reg.register(make_agent("worker2", Some("supervisor")));

        let supervised = reg.get_supervised("supervisor");
        assert_eq!(supervised.len(), 2);

        let supervisors = reg.get_supervisors("worker1");
        assert_eq!(supervisors.len(), 1);
        assert_eq!(supervisors[0].id, "supervisor");
    }

    #[test]
    fn test_get_by_role() {
        let mut reg = AgentRegistry::new();
        let mut sup = make_agent("s1", None);
        sup.role = AgentRole::Supervisor;
        reg.register(sup);
        reg.register(make_agent("e1", None));
        reg.register(make_agent("e2", None));

        assert_eq!(reg.get_by_role(&AgentRole::Executor).len(), 2);
        assert_eq!(reg.get_by_role(&AgentRole::Supervisor).len(), 1);
    }

    #[test]
    fn test_find_by_skill() {
        let mut reg = AgentRegistry::new();
        let mut a1 = make_agent("a1", None);
        a1.skills = vec![AgentSkill {
            id: "research".into(),
            description: "Web research".into(),
            proficiency: 0.9,
        }];
        let mut a2 = make_agent("a2", None);
        a2.skills = vec![AgentSkill {
            id: "research".into(),
            description: "Research".into(),
            proficiency: 0.5,
        }];
        reg.register(a1);
        reg.register(a2);

        let best = reg.find_by_skill("research").unwrap();
        assert_eq!(best.id, "a1"); // highest proficiency
    }

    #[test]
    fn test_enabled_filter() {
        let mut reg = AgentRegistry::new();
        reg.register(make_agent("a1", None));
        let mut disabled = make_agent("a2", None);
        disabled.enabled = false;
        reg.register(disabled);

        assert_eq!(reg.get_enabled().len(), 1);
    }
}
