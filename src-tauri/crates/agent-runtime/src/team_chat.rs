//! Internal team chat system for user ↔ multi-agent communication.
//!
//! Supports:
//! - Channels (general, per-project, per-agent DM)
//! - Messages with sender identity (user or agent)
//! - @mentions to direct questions to specific agents
//! - Message threading
//! - Agent auto-response based on mentions and channel membership

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_MESSAGES_PER_CHANNEL: usize = 1000;

/// Who sent a message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Sender {
    User,
    Agent(String), // agent_id
    System,
}

/// A chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub channel_id: String,
    pub sender: Sender,
    pub content: String,
    pub mentions: Vec<String>, // agent IDs mentioned with @
    pub reply_to: Option<u64>, // thread: parent message ID
    pub timestamp: u64,
}

/// A chat channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Agent IDs that are members of this channel.
    pub agent_members: Vec<String>,
    /// Whether user is a member (always true for now).
    pub user_member: bool,
    pub created_at: u64,
}

/// Info about pending agent responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingResponse {
    pub agent_id: String,
    pub channel_id: String,
    pub trigger_message_id: u64,
}

/// The team chat system.
pub struct TeamChat {
    channels: HashMap<String, Channel>,
    messages: HashMap<String, VecDeque<ChatMessage>>, // channel_id -> messages
    next_msg_id: u64,
    pending_responses: Vec<PendingResponse>,
}

impl TeamChat {
    pub fn new() -> Self {
        let mut chat = Self {
            channels: HashMap::new(),
            messages: HashMap::new(),
            next_msg_id: 1,
            pending_responses: Vec::new(),
        };

        // Create default #general channel
        chat.create_channel("general", "General", "Main discussion channel", vec![]);
        chat
    }

    /// Creates a new channel.
    pub fn create_channel(
        &mut self,
        id: &str,
        name: &str,
        description: &str,
        agent_members: Vec<String>,
    ) -> &Channel {
        let channel = Channel {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            agent_members,
            user_member: true,
            created_at: now_millis(),
        };
        self.channels.insert(id.into(), channel);
        self.messages.entry(id.into()).or_default();
        self.channels.get(id).unwrap()
    }

    /// Adds an agent to a channel.
    pub fn add_agent_to_channel(&mut self, channel_id: &str, agent_id: &str) -> bool {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            if !channel.agent_members.contains(&agent_id.to_string()) {
                channel.agent_members.push(agent_id.into());
            }
            true
        } else {
            false
        }
    }

    /// Removes an agent from a channel.
    pub fn remove_agent_from_channel(&mut self, channel_id: &str, agent_id: &str) {
        if let Some(channel) = self.channels.get_mut(channel_id) {
            channel.agent_members.retain(|id| id != agent_id);
        }
    }

    /// Sends a message from the user.
    pub fn send_user_message(
        &mut self,
        channel_id: &str,
        content: &str,
    ) -> Option<ChatMessage> {
        self.send_message(channel_id, Sender::User, content, None)
    }

    /// Sends a message from an agent.
    pub fn send_agent_message(
        &mut self,
        channel_id: &str,
        agent_id: &str,
        content: &str,
        reply_to: Option<u64>,
    ) -> Option<ChatMessage> {
        self.send_message(channel_id, Sender::Agent(agent_id.into()), content, reply_to)
    }

    /// Sends a system message.
    pub fn send_system_message(
        &mut self,
        channel_id: &str,
        content: &str,
    ) -> Option<ChatMessage> {
        self.send_message(channel_id, Sender::System, content, None)
    }

    fn send_message(
        &mut self,
        channel_id: &str,
        sender: Sender,
        content: &str,
        reply_to: Option<u64>,
    ) -> Option<ChatMessage> {
        if !self.channels.contains_key(channel_id) {
            return None;
        }

        let mentions = extract_mentions(content);
        let id = self.next_msg_id;
        self.next_msg_id += 1;

        let msg = ChatMessage {
            id,
            channel_id: channel_id.into(),
            sender: sender.clone(),
            content: content.into(),
            mentions: mentions.clone(),
            reply_to,
            timestamp: now_millis(),
        };

        let messages = self.messages.entry(channel_id.into()).or_default();
        messages.push_back(msg.clone());

        // Trim old messages
        while messages.len() > MAX_MESSAGES_PER_CHANNEL {
            messages.pop_front();
        }

        // If user sent message, queue agent responses for mentioned/channel agents
        if sender == Sender::User {
            self.queue_agent_responses(channel_id, &mentions, id);
        }

        Some(msg)
    }

    /// Queues agent responses after a user message.
    fn queue_agent_responses(
        &mut self,
        channel_id: &str,
        mentions: &[String],
        trigger_id: u64,
    ) {
        let channel = match self.channels.get(channel_id) {
            Some(c) => c,
            None => return,
        };

        if mentions.is_empty() {
            // No mentions — all channel agents should respond
            for agent_id in &channel.agent_members {
                self.pending_responses.push(PendingResponse {
                    agent_id: agent_id.clone(),
                    channel_id: channel_id.into(),
                    trigger_message_id: trigger_id,
                });
            }
        } else {
            // Only mentioned agents respond
            for agent_id in mentions {
                if channel.agent_members.contains(agent_id) {
                    self.pending_responses.push(PendingResponse {
                        agent_id: agent_id.clone(),
                        channel_id: channel_id.into(),
                        trigger_message_id: trigger_id,
                    });
                }
            }
        }
    }

    /// Gets pending responses that need to be processed.
    pub fn take_pending_responses(&mut self) -> Vec<PendingResponse> {
        std::mem::take(&mut self.pending_responses)
    }

    /// Gets messages from a channel.
    pub fn get_messages(&self, channel_id: &str, limit: usize) -> Vec<&ChatMessage> {
        self.messages
            .get(channel_id)
            .map(|msgs| msgs.iter().rev().take(limit).collect::<Vec<_>>().into_iter().rev().collect())
            .unwrap_or_default()
    }

    /// Gets thread replies for a message.
    pub fn get_thread(&self, channel_id: &str, parent_id: u64) -> Vec<&ChatMessage> {
        self.messages
            .get(channel_id)
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| m.reply_to == Some(parent_id) || m.id == parent_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Lists all channels.
    pub fn list_channels(&self) -> Vec<&Channel> {
        self.channels.values().collect()
    }

    /// Gets a channel by ID.
    pub fn get_channel(&self, id: &str) -> Option<&Channel> {
        self.channels.get(id)
    }

    /// Deletes a channel and its messages.
    pub fn delete_channel(&mut self, id: &str) -> bool {
        if id == "general" {
            return false; // Can't delete default channel
        }
        self.channels.remove(id);
        self.messages.remove(id);
        true
    }

    /// Gets unread count (messages since last agent response in channel).
    pub fn channel_message_count(&self, channel_id: &str) -> usize {
        self.messages
            .get(channel_id)
            .map(|m| m.len())
            .unwrap_or(0)
    }
}

impl Default for TeamChat {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts @mentions from message content.
fn extract_mentions(content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .filter(|w| w.starts_with('@') && w.len() > 1)
        .map(|w| w.trim_start_matches('@').trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
    fn test_default_channel() {
        let chat = TeamChat::new();
        assert!(chat.get_channel("general").is_some());
        assert_eq!(chat.list_channels().len(), 1);
    }

    #[test]
    fn test_create_channel() {
        let mut chat = TeamChat::new();
        chat.create_channel("project-x", "Project X", "Work channel", vec!["agent-1".into()]);
        assert_eq!(chat.list_channels().len(), 2);
        let ch = chat.get_channel("project-x").unwrap();
        assert_eq!(ch.agent_members, vec!["agent-1"]);
    }

    #[test]
    fn test_send_and_get_messages() {
        let mut chat = TeamChat::new();
        chat.send_user_message("general", "Hello agents!");
        chat.send_agent_message("general", "agent-1", "Hi there!", None);

        let msgs = chat.get_messages("general", 50);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].sender, Sender::User);
        assert_eq!(msgs[1].sender, Sender::Agent("agent-1".into()));
    }

    #[test]
    fn test_mentions() {
        let mentions = extract_mentions("Hey @researcher can you look at @writer work?");
        assert_eq!(mentions.len(), 2);
        assert!(mentions.contains(&"researcher".to_string()));
        assert!(mentions.contains(&"writer".to_string()));
    }

    #[test]
    fn test_no_mentions() {
        let mentions = extract_mentions("Hello everyone");
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_pending_responses_with_mention() {
        let mut chat = TeamChat::new();
        chat.add_agent_to_channel("general", "researcher");
        chat.add_agent_to_channel("general", "writer");

        chat.send_user_message("general", "Hey @researcher find me papers on AI");

        let pending = chat.take_pending_responses();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].agent_id, "researcher");
    }

    #[test]
    fn test_pending_responses_no_mention_all_agents() {
        let mut chat = TeamChat::new();
        chat.add_agent_to_channel("general", "a1");
        chat.add_agent_to_channel("general", "a2");

        chat.send_user_message("general", "What's the status?");

        let pending = chat.take_pending_responses();
        assert_eq!(pending.len(), 2); // Both agents respond
    }

    #[test]
    fn test_thread_replies() {
        let mut chat = TeamChat::new();
        let msg = chat.send_user_message("general", "Question?").unwrap();
        let msg_id = msg.id;

        chat.send_agent_message("general", "a1", "Answer!", Some(msg_id));
        chat.send_agent_message("general", "a2", "My answer!", Some(msg_id));

        let thread = chat.get_thread("general", msg_id);
        assert_eq!(thread.len(), 3); // original + 2 replies
    }

    #[test]
    fn test_add_remove_agent() {
        let mut chat = TeamChat::new();
        chat.add_agent_to_channel("general", "a1");
        assert_eq!(chat.get_channel("general").unwrap().agent_members.len(), 1);

        chat.remove_agent_from_channel("general", "a1");
        assert_eq!(chat.get_channel("general").unwrap().agent_members.len(), 0);
    }

    #[test]
    fn test_delete_channel() {
        let mut chat = TeamChat::new();
        chat.create_channel("temp", "Temp", "Temporary", vec![]);

        assert!(chat.delete_channel("temp"));
        assert!(chat.get_channel("temp").is_none());

        // Can't delete general
        assert!(!chat.delete_channel("general"));
    }

    #[test]
    fn test_system_message() {
        let mut chat = TeamChat::new();
        chat.send_system_message("general", "Agent researcher joined the channel");

        let msgs = chat.get_messages("general", 50);
        assert_eq!(msgs[0].sender, Sender::System);
    }

    #[test]
    fn test_message_count() {
        let mut chat = TeamChat::new();
        chat.send_user_message("general", "One");
        chat.send_user_message("general", "Two");
        assert_eq!(chat.channel_message_count("general"), 2);
    }
}
