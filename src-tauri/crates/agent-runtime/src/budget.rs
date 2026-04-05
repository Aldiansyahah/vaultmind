//! Agent budget controls for tracking and limiting API usage.
//!
//! Tracks token usage, API call counts, and costs per agent.
//! Enforces limits to prevent runaway spending.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Budget limits for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimit {
    /// Maximum tokens per day (0 = unlimited).
    pub max_tokens_per_day: u64,
    /// Maximum API calls per day (0 = unlimited).
    pub max_calls_per_day: u64,
    /// Maximum cost in USD per day (0.0 = unlimited).
    pub max_cost_per_day: f64,
    /// Maximum tokens per single request.
    pub max_tokens_per_request: u64,
}

impl Default for BudgetLimit {
    fn default() -> Self {
        Self {
            max_tokens_per_day: 100_000,
            max_calls_per_day: 100,
            max_cost_per_day: 1.0,
            max_tokens_per_request: 4096,
        }
    }
}

/// Current usage counters for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub tokens_used_today: u64,
    pub calls_today: u64,
    pub cost_today: f64,
    pub tokens_total: u64,
    pub calls_total: u64,
    pub cost_total: f64,
    /// Unix timestamp of last reset (start of day).
    pub last_reset_timestamp: u64,
}

/// Result of a budget check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BudgetCheckResult {
    /// Within budget, proceed.
    Allowed,
    /// Over token limit.
    TokenLimitExceeded { used: u64, limit: u64 },
    /// Over API call limit.
    CallLimitExceeded { used: u64, limit: u64 },
    /// Over cost limit.
    CostLimitExceeded { used: f64, limit: f64 },
}

/// Budget manager tracking usage per agent.
pub struct BudgetManager {
    limits: HashMap<String, BudgetLimit>,
    usage: HashMap<String, BudgetUsage>,
}

impl BudgetManager {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
            usage: HashMap::new(),
        }
    }

    /// Sets budget limits for an agent.
    pub fn set_limits(&mut self, agent_id: &str, limits: BudgetLimit) {
        self.limits.insert(agent_id.into(), limits);
    }

    /// Gets budget limits for an agent.
    pub fn get_limits(&self, agent_id: &str) -> Option<&BudgetLimit> {
        self.limits.get(agent_id)
    }

    /// Gets current usage for an agent.
    pub fn get_usage(&self, agent_id: &str) -> BudgetUsage {
        self.usage.get(agent_id).cloned().unwrap_or_default()
    }

    /// Checks if an agent is within budget for a request.
    pub fn check_budget(&self, agent_id: &str) -> BudgetCheckResult {
        let limits = match self.limits.get(agent_id) {
            Some(l) => l,
            None => return BudgetCheckResult::Allowed, // No limits set
        };

        let usage = self.usage.get(agent_id).cloned().unwrap_or_default();

        if limits.max_tokens_per_day > 0 && usage.tokens_used_today >= limits.max_tokens_per_day {
            return BudgetCheckResult::TokenLimitExceeded {
                used: usage.tokens_used_today,
                limit: limits.max_tokens_per_day,
            };
        }

        if limits.max_calls_per_day > 0 && usage.calls_today >= limits.max_calls_per_day {
            return BudgetCheckResult::CallLimitExceeded {
                used: usage.calls_today,
                limit: limits.max_calls_per_day,
            };
        }

        if limits.max_cost_per_day > 0.0 && usage.cost_today >= limits.max_cost_per_day {
            return BudgetCheckResult::CostLimitExceeded {
                used: usage.cost_today,
                limit: limits.max_cost_per_day,
            };
        }

        BudgetCheckResult::Allowed
    }

    /// Records token usage for an agent.
    pub fn record_usage(
        &mut self,
        agent_id: &str,
        tokens: u64,
        cost: f64,
    ) {
        let usage = self.usage.entry(agent_id.into()).or_default();
        usage.tokens_used_today += tokens;
        usage.calls_today += 1;
        usage.cost_today += cost;
        usage.tokens_total += tokens;
        usage.calls_total += 1;
        usage.cost_total += cost;
    }

    /// Resets daily counters for all agents.
    pub fn reset_daily(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for usage in self.usage.values_mut() {
            usage.tokens_used_today = 0;
            usage.calls_today = 0;
            usage.cost_today = 0.0;
            usage.last_reset_timestamp = now;
        }
    }

    /// Gets a summary of all agents' budget status.
    pub fn get_summary(&self) -> Vec<BudgetSummary> {
        let mut summaries = Vec::new();
        for (agent_id, limits) in &self.limits {
            let usage = self.get_usage(agent_id);
            let status = self.check_budget(agent_id);
            summaries.push(BudgetSummary {
                agent_id: agent_id.clone(),
                limits: limits.clone(),
                usage,
                status,
            });
        }
        summaries
    }

    /// Removes an agent's budget tracking.
    pub fn remove(&mut self, agent_id: &str) {
        self.limits.remove(agent_id);
        self.usage.remove(agent_id);
    }
}

/// Combined budget summary for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub agent_id: String,
    pub limits: BudgetLimit,
    pub usage: BudgetUsage,
    pub status: BudgetCheckResult,
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = BudgetLimit::default();
        assert_eq!(limits.max_tokens_per_day, 100_000);
        assert_eq!(limits.max_calls_per_day, 100);
    }

    #[test]
    fn test_within_budget() {
        let mut mgr = BudgetManager::new();
        mgr.set_limits("a1", BudgetLimit::default());
        assert_eq!(mgr.check_budget("a1"), BudgetCheckResult::Allowed);
    }

    #[test]
    fn test_token_limit_exceeded() {
        let mut mgr = BudgetManager::new();
        mgr.set_limits("a1", BudgetLimit {
            max_tokens_per_day: 100,
            ..Default::default()
        });
        mgr.record_usage("a1", 101, 0.0);

        match mgr.check_budget("a1") {
            BudgetCheckResult::TokenLimitExceeded { used, limit } => {
                assert_eq!(used, 101);
                assert_eq!(limit, 100);
            }
            other => panic!("Expected TokenLimitExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_call_limit_exceeded() {
        let mut mgr = BudgetManager::new();
        mgr.set_limits("a1", BudgetLimit {
            max_calls_per_day: 2,
            ..Default::default()
        });
        mgr.record_usage("a1", 10, 0.0);
        mgr.record_usage("a1", 10, 0.0);

        match mgr.check_budget("a1") {
            BudgetCheckResult::CallLimitExceeded { used, limit } => {
                assert_eq!(used, 2);
                assert_eq!(limit, 2);
            }
            other => panic!("Expected CallLimitExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_cost_limit() {
        let mut mgr = BudgetManager::new();
        mgr.set_limits("a1", BudgetLimit {
            max_cost_per_day: 0.50,
            ..Default::default()
        });
        mgr.record_usage("a1", 1000, 0.51);

        match mgr.check_budget("a1") {
            BudgetCheckResult::CostLimitExceeded { .. } => {}
            other => panic!("Expected CostLimitExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_no_limits_always_allowed() {
        let mgr = BudgetManager::new();
        assert_eq!(mgr.check_budget("unknown"), BudgetCheckResult::Allowed);
    }

    #[test]
    fn test_record_and_get_usage() {
        let mut mgr = BudgetManager::new();
        mgr.record_usage("a1", 500, 0.05);
        mgr.record_usage("a1", 300, 0.03);

        let usage = mgr.get_usage("a1");
        assert_eq!(usage.tokens_used_today, 800);
        assert_eq!(usage.calls_today, 2);
        assert!((usage.cost_today - 0.08).abs() < 0.001);
    }

    #[test]
    fn test_reset_daily() {
        let mut mgr = BudgetManager::new();
        mgr.record_usage("a1", 500, 0.05);
        mgr.reset_daily();

        let usage = mgr.get_usage("a1");
        assert_eq!(usage.tokens_used_today, 0);
        assert_eq!(usage.tokens_total, 500); // Total preserved
    }

    #[test]
    fn test_get_summary() {
        let mut mgr = BudgetManager::new();
        mgr.set_limits("a1", BudgetLimit::default());
        mgr.record_usage("a1", 100, 0.01);

        let summary = mgr.get_summary();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary[0].usage.tokens_used_today, 100);
    }
}
