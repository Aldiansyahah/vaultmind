//! Knowledge graph for notes using petgraph.
//!
//! Nodes represent notes, edges represent relationships:
//! - Wikilink edges: explicit `[[target]]` links between notes
//! - Tag edges: notes sharing the same tag
//! - (Future) Similarity edges: cosine similarity above threshold

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

/// Types of edges in the knowledge graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeKind {
    /// Explicit wikilink from source to target.
    Wikilink,
    /// Shared tag between two notes.
    SharedTag(String),
    /// Semantic similarity (cosine similarity score).
    Similarity(f32),
}

/// A node in the knowledge graph representing a note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteNode {
    /// Note file path (relative to vault).
    pub path: String,
    /// Note title.
    pub title: String,
    /// Tags associated with this note.
    pub tags: Vec<String>,
}

/// A connection returned from graph queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub source_path: String,
    pub target_path: String,
    pub edge_kind: EdgeKind,
}

/// Knowledge graph engine backed by petgraph.
pub struct KnowledgeGraph {
    graph: DiGraph<NoteNode, EdgeKind>,
    path_to_node: HashMap<String, NodeIndex>,
}

impl KnowledgeGraph {
    /// Creates a new empty knowledge graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_node: HashMap::new(),
        }
    }

    /// Adds or updates a note node in the graph.
    pub fn upsert_note(&mut self, path: &str, title: &str, tags: &[String]) {
        let node = NoteNode {
            path: path.to_string(),
            title: title.to_string(),
            tags: tags.to_vec(),
        };

        if let Some(&idx) = self.path_to_node.get(path) {
            self.graph[idx] = node;
        } else {
            let idx = self.graph.add_node(node);
            self.path_to_node.insert(path.to_string(), idx);
        }
    }

    /// Removes a note and all its edges from the graph.
    pub fn remove_note(&mut self, path: &str) {
        if let Some(idx) = self.path_to_node.remove(path) {
            self.graph.remove_node(idx);
            // Rebuild path_to_node since indices may have shifted
            self.path_to_node.clear();
            for idx in self.graph.node_indices() {
                let node = &self.graph[idx];
                self.path_to_node.insert(node.path.clone(), idx);
            }
        }
    }

    /// Adds a wikilink edge from source to target.
    pub fn add_wikilink(&mut self, source_path: &str, target_path: &str) {
        if let (Some(&src), Some(&tgt)) = (
            self.path_to_node.get(source_path),
            self.path_to_node.get(target_path),
        ) {
            // Avoid duplicate edges
            let exists = self
                .graph
                .edges_connecting(src, tgt)
                .any(|e| matches!(e.weight(), EdgeKind::Wikilink));
            if !exists {
                self.graph.add_edge(src, tgt, EdgeKind::Wikilink);
            }
        }
    }

    /// Rebuilds tag-based edges for all notes sharing tags.
    pub fn rebuild_tag_edges(&mut self) {
        // Remove existing tag edges
        let tag_edges: Vec<_> = self
            .graph
            .edge_indices()
            .filter(|&e| matches!(self.graph[e], EdgeKind::SharedTag(_)))
            .collect();
        for e in tag_edges.into_iter().rev() {
            self.graph.remove_edge(e);
        }

        // Build tag → [node_indices] map
        let mut tag_map: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        for idx in self.graph.node_indices() {
            for tag in &self.graph[idx].tags {
                tag_map
                    .entry(tag.clone())
                    .or_default()
                    .push(idx);
            }
        }

        // Create edges between notes sharing tags
        for (tag, nodes) in &tag_map {
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    let exists = self
                        .graph
                        .edges_connecting(nodes[i], nodes[j])
                        .any(|e| {
                            matches!(e.weight(), EdgeKind::SharedTag(t) if t == tag)
                        });
                    if !exists {
                        self.graph
                            .add_edge(nodes[i], nodes[j], EdgeKind::SharedTag(tag.clone()));
                        self.graph
                            .add_edge(nodes[j], nodes[i], EdgeKind::SharedTag(tag.clone()));
                    }
                }
            }
        }
    }

    /// Gets all connections (edges) for a specific note.
    pub fn get_connections(&self, path: &str) -> Vec<Connection> {
        let Some(&idx) = self.path_to_node.get(path) else {
            return Vec::new();
        };

        let mut connections = Vec::new();

        // Outgoing edges
        for edge in self.graph.edges_directed(idx, Direction::Outgoing) {
            connections.push(Connection {
                source_path: path.to_string(),
                target_path: self.graph[edge.target()].path.clone(),
                edge_kind: edge.weight().clone(),
            });
        }

        // Incoming edges (backlinks)
        for edge in self.graph.edges_directed(idx, Direction::Incoming) {
            connections.push(Connection {
                source_path: self.graph[edge.source()].path.clone(),
                target_path: path.to_string(),
                edge_kind: edge.weight().clone(),
            });
        }

        connections
    }

    /// Gets notes connected to a given note within N hops.
    pub fn get_neighbors(&self, path: &str, max_depth: usize) -> Vec<String> {
        let Some(&start) = self.path_to_node.get(path) else {
            return Vec::new();
        };

        let mut visited = HashMap::new();
        visited.insert(start, 0usize);
        let mut queue = vec![(start, 0usize)];
        let mut result = Vec::new();

        while let Some((node, depth)) = queue.pop() {
            if depth > 0 {
                result.push(self.graph[node].path.clone());
            }
            if depth >= max_depth {
                continue;
            }

            for neighbor in self.graph.neighbors_undirected(node) {
                visited.entry(neighbor).or_insert_with(|| {
                    queue.push((neighbor, depth + 1));
                    depth + 1
                });
            }
        }

        result
    }

    /// Gets backlinks — notes that link TO the given note.
    pub fn get_backlinks(&self, path: &str) -> Vec<String> {
        let Some(&idx) = self.path_to_node.get(path) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(idx, Direction::Incoming)
            .filter(|e| matches!(e.weight(), EdgeKind::Wikilink))
            .map(|e| self.graph[e.source()].path.clone())
            .collect()
    }

    /// Returns total number of nodes.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns total number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns all nodes for serialization (e.g., graph visualization).
    pub fn all_nodes(&self) -> Vec<&NoteNode> {
        self.graph.node_indices().map(|i| &self.graph[i]).collect()
    }

    /// Returns all edges for serialization.
    pub fn all_edges(&self) -> Vec<Connection> {
        self.graph
            .edge_references()
            .map(|e| Connection {
                source_path: self.graph[e.source()].path.clone(),
                target_path: self.graph[e.target()].path.clone(),
                edge_kind: e.weight().clone(),
            })
            .collect()
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_graph() -> KnowledgeGraph {
        let mut g = KnowledgeGraph::new();
        g.upsert_note("a.md", "Note A", &["rust".into(), "code".into()]);
        g.upsert_note("b.md", "Note B", &["rust".into()]);
        g.upsert_note("c.md", "Note C", &["design".into()]);
        g.add_wikilink("a.md", "b.md");
        g.add_wikilink("b.md", "c.md");
        g.rebuild_tag_edges();
        g
    }

    #[test]
    fn test_node_count() {
        let g = setup_graph();
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn test_wikilink_edges() {
        let g = setup_graph();
        let conns = g.get_connections("a.md");
        let wikilinks: Vec<_> = conns
            .iter()
            .filter(|c| matches!(c.edge_kind, EdgeKind::Wikilink))
            .collect();
        assert!(!wikilinks.is_empty());
    }

    #[test]
    fn test_tag_edges() {
        let g = setup_graph();
        let conns = g.get_connections("a.md");
        let tag_edges: Vec<_> = conns
            .iter()
            .filter(|c| matches!(&c.edge_kind, EdgeKind::SharedTag(t) if t == "rust"))
            .collect();
        assert!(!tag_edges.is_empty(), "A and B share 'rust' tag");
    }

    #[test]
    fn test_backlinks() {
        let g = setup_graph();
        let backlinks = g.get_backlinks("b.md");
        assert_eq!(backlinks, vec!["a.md"]);
    }

    #[test]
    fn test_neighbors() {
        let g = setup_graph();
        let neighbors = g.get_neighbors("a.md", 1);
        assert!(neighbors.contains(&"b.md".to_string()));
    }

    #[test]
    fn test_neighbors_depth_2() {
        let g = setup_graph();
        let neighbors = g.get_neighbors("a.md", 2);
        assert!(neighbors.contains(&"b.md".to_string()));
        assert!(neighbors.contains(&"c.md".to_string()));
    }

    #[test]
    fn test_remove_note() {
        let mut g = setup_graph();
        g.remove_note("b.md");
        assert_eq!(g.node_count(), 2);
        assert!(g.get_backlinks("b.md").is_empty());
    }

    #[test]
    fn test_no_duplicate_wikilinks() {
        let mut g = KnowledgeGraph::new();
        g.upsert_note("a.md", "A", &[]);
        g.upsert_note("b.md", "B", &[]);
        g.add_wikilink("a.md", "b.md");
        g.add_wikilink("a.md", "b.md");
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn test_all_nodes_and_edges() {
        let g = setup_graph();
        assert_eq!(g.all_nodes().len(), 3);
        assert!(g.all_edges().len() >= 3); // wikilinks + tag edges
    }
}
