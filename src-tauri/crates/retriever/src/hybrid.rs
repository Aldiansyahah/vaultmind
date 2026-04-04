//! Hybrid search combining BM25 (Tantivy), vector similarity, and graph expansion.
//! Results are fused using Reciprocal Rank Fusion (RRF).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use graph_engine::KnowledgeGraph;

/// A hybrid search result after RRF fusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    /// Note path.
    pub path: String,
    /// Combined RRF score.
    pub score: f32,
    /// Best matching snippet/content.
    pub snippet: String,
    /// Note title.
    pub title: String,
    /// Which sources contributed to this result.
    pub sources: Vec<String>,
}

/// Reciprocal Rank Fusion constant (typically 60).
const RRF_K: f32 = 60.0;

/// A ranked item: (path, snippet, title).
pub type RankedItem = (String, String, String);

/// A named ranked list: (source_name, items).
pub type RankedList<'a> = (&'a str, Vec<RankedItem>);

/// Fuses multiple ranked lists using Reciprocal Rank Fusion.
///
/// Each input is a named ranked list of (path, snippet, title) tuples.
/// Returns fused results sorted by combined RRF score.
pub fn reciprocal_rank_fusion(ranked_lists: &[RankedList<'_>], limit: usize) -> Vec<HybridResult> {
    let mut scores: HashMap<String, (f32, String, String, Vec<String>)> = HashMap::new();

    for (source_name, ranked) in ranked_lists {
        for (rank, (path, snippet, title)) in ranked.iter().enumerate() {
            let rrf_score = 1.0 / (RRF_K + rank as f32 + 1.0);
            let entry = scores.entry(path.clone()).or_insert_with(|| {
                (0.0, snippet.clone(), title.clone(), Vec::new())
            });
            entry.0 += rrf_score;
            entry.3.push(source_name.to_string());

            // Keep the best snippet (from first source that provided it)
            if entry.1.is_empty() && !snippet.is_empty() {
                entry.1 = snippet.clone();
            }
        }
    }

    let mut results: Vec<HybridResult> = scores
        .into_iter()
        .map(|(path, (score, snippet, title, sources))| HybridResult {
            path,
            score,
            snippet,
            title,
            sources,
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    results
}

/// Expands a set of note paths with graph-connected neighbors.
///
/// For each result, adds notes within `depth` hops that aren't already in results.
/// Returns additional paths to consider.
pub fn graph_expand(
    initial_paths: &[String],
    graph: &KnowledgeGraph,
    depth: usize,
    max_additions: usize,
) -> Vec<String> {
    let mut seen: std::collections::HashSet<String> = initial_paths.iter().cloned().collect();
    let mut additions = Vec::new();

    for path in initial_paths {
        let neighbors = graph.get_neighbors(path, depth);
        for neighbor in neighbors {
            if !seen.contains(&neighbor) {
                seen.insert(neighbor.clone());
                additions.push(neighbor);
                if additions.len() >= max_additions {
                    return additions;
                }
            }
        }
    }

    additions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_single_list() {
        let list = vec![
            ("a.md".into(), "snippet a".into(), "Note A".into()),
            ("b.md".into(), "snippet b".into(), "Note B".into()),
        ];
        let results = reciprocal_rank_fusion(&[("bm25", list)], 10);
        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
        assert_eq!(results[0].path, "a.md");
    }

    #[test]
    fn test_rrf_two_lists_boost() {
        let bm25 = vec![
            ("a.md".into(), "s".into(), "A".into()),
            ("b.md".into(), "s".into(), "B".into()),
        ];
        let vector = vec![
            ("b.md".into(), "s".into(), "B".into()),
            ("a.md".into(), "s".into(), "A".into()),
        ];
        let results = reciprocal_rank_fusion(
            &[("bm25", bm25), ("vector", vector)],
            10,
        );
        // Both a and b appear in both lists, so both get boosted
        assert_eq!(results.len(), 2);
        // a is rank1 in bm25 + rank2 in vector, b is rank1 in vector + rank2 in bm25
        // They should have equal scores
        assert!((results[0].score - results[1].score).abs() < 1e-6);
    }

    #[test]
    fn test_rrf_limit() {
        let list = vec![
            ("a.md".into(), "s".into(), "A".into()),
            ("b.md".into(), "s".into(), "B".into()),
            ("c.md".into(), "s".into(), "C".into()),
        ];
        let results = reciprocal_rank_fusion(&[("bm25", list)], 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_rrf_sources_tracked() {
        let bm25 = vec![("a.md".into(), "s".into(), "A".into())];
        let vector = vec![("a.md".into(), "s".into(), "A".into())];
        let results = reciprocal_rank_fusion(
            &[("bm25", bm25), ("vector", vector)],
            10,
        );
        assert_eq!(results[0].sources.len(), 2);
        assert!(results[0].sources.contains(&"bm25".to_string()));
        assert!(results[0].sources.contains(&"vector".to_string()));
    }

    #[test]
    fn test_graph_expand() {
        let mut graph = KnowledgeGraph::new();
        graph.upsert_note("a.md", "A", &[]);
        graph.upsert_note("b.md", "B", &[]);
        graph.upsert_note("c.md", "C", &[]);
        graph.add_wikilink("a.md", "b.md");
        graph.add_wikilink("b.md", "c.md");

        let additions = graph_expand(&["a.md".into()], &graph, 1, 10);
        assert!(additions.contains(&"b.md".to_string()));
        assert!(!additions.contains(&"a.md".to_string())); // shouldn't include self
    }

    #[test]
    fn test_graph_expand_max() {
        let mut graph = KnowledgeGraph::new();
        for i in 0..10 {
            graph.upsert_note(&format!("{i}.md"), &format!("N{i}"), &[]);
            if i > 0 {
                graph.add_wikilink("0.md", &format!("{i}.md"));
            }
        }

        let additions = graph_expand(&["0.md".into()], &graph, 1, 3);
        assert_eq!(additions.len(), 3);
    }
}
