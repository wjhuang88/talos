//! Weighted associative memory graph (T43, MEM-008 Phase 1).
//!
//! Additive graph tables in the existing `talos-memory` SQLite store. Nodes
//! represent memory elements; edges represent typed associations with weights.
//! Retrieval is explicit and bounded — no automatic prompt injection (T31
//! decision: default-off).

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::MemoryStoreError;

const DECAY_HALF_LIFE_DAYS: f64 = 7.0;
const DEFAULT_MAX_HOPS: usize = 3;
const DEFAULT_MIN_EDGE_WEIGHT: f64 = 0.3;
const DEFAULT_FANOUT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub impression_strength: f64,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub access_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
    pub last_reinforced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationResult {
    pub node: GraphNode,
    pub score: f64,
    pub path: Vec<String>,
    pub hop_count: usize,
}

pub(crate) fn migrate_graph(conn: &Connection) -> Result<(), MemoryStoreError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memory_graph_nodes (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            label TEXT NOT NULL,
            impression_strength REAL NOT NULL DEFAULT 1.0,
            weight REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL,
            last_accessed_at TEXT NOT NULL,
            access_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS memory_graph_edges (
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            weight REAL NOT NULL,
            created_at TEXT NOT NULL,
            last_reinforced_at TEXT NOT NULL,
            PRIMARY KEY (source_id, target_id, relation_type),
            FOREIGN KEY (source_id) REFERENCES memory_graph_nodes(id),
            FOREIGN KEY (target_id) REFERENCES memory_graph_nodes(id)
        );

        CREATE INDEX IF NOT EXISTS idx_graph_edges_source
            ON memory_graph_edges(source_id);
        CREATE INDEX IF NOT EXISTS idx_graph_edges_target
            ON memory_graph_edges(target_id);
        "#,
    )?;
    Ok(())
}

pub(crate) fn upsert_node(conn: &Connection, node: &GraphNode) -> Result<(), MemoryStoreError> {
    conn.execute(
        "INSERT INTO memory_graph_nodes (id, kind, label, impression_strength, weight, created_at, last_accessed_at, access_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            label = excluded.label,
            impression_strength = excluded.impression_strength,
            weight = excluded.weight,
            last_accessed_at = excluded.last_accessed_at,
            access_count = excluded.access_count",
        params![
            node.id,
            node.kind,
            node.label,
            node.impression_strength,
            node.weight,
            node.created_at.to_rfc3339(),
            node.last_accessed_at.to_rfc3339(),
            node.access_count,
        ],
    )?;
    Ok(())
}

pub(crate) fn upsert_edge(conn: &Connection, edge: &GraphEdge) -> Result<(), MemoryStoreError> {
    conn.execute(
        "INSERT INTO memory_graph_edges (source_id, target_id, relation_type, weight, created_at, last_reinforced_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(source_id, target_id, relation_type) DO UPDATE SET
            weight = excluded.weight,
            last_reinforced_at = excluded.last_reinforced_at",
        params![
            edge.source_id,
            edge.target_id,
            edge.relation_type,
            edge.weight,
            edge.created_at.to_rfc3339(),
            edge.last_reinforced_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub(crate) fn get_node(conn: &Connection, id: &str) -> Result<Option<GraphNode>, MemoryStoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, label, impression_strength, weight, created_at, last_accessed_at, access_count
         FROM memory_graph_nodes WHERE id = ?1",
    )?;
    let node = stmt
        .query_row(params![id], |row| {
            Ok(GraphNode {
                id: row.get(0)?,
                kind: row.get(1)?,
                label: row.get(2)?,
                impression_strength: row.get(3)?,
                weight: row.get(4)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_accessed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                access_count: row.get(7)?,
            })
        })
        .optional()?;
    Ok(node)
}

fn load_all_nodes(conn: &Connection) -> Result<HashMap<String, GraphNode>, MemoryStoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, label, impression_strength, weight, created_at, last_accessed_at, access_count
         FROM memory_graph_nodes",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GraphNode {
            id: row.get(0)?,
            kind: row.get(1)?,
            label: row.get(2)?,
            impression_strength: row.get(3)?,
            weight: row.get(4)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_accessed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            access_count: row.get(7)?,
        })
    })?;
    let mut nodes = HashMap::new();
    for row in rows {
        let node = row?;
        nodes.insert(node.id.clone(), node);
    }
    Ok(nodes)
}

fn load_all_edges(conn: &Connection) -> Result<Vec<GraphEdge>, MemoryStoreError> {
    let mut stmt = conn.prepare(
        "SELECT source_id, target_id, relation_type, weight, created_at, last_reinforced_at
         FROM memory_graph_edges",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GraphEdge {
            source_id: row.get(0)?,
            target_id: row.get(1)?,
            relation_type: row.get(2)?,
            weight: row.get(3)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_reinforced_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    })?;
    let mut edges = Vec::new();
    for row in rows {
        edges.push(row?);
    }
    Ok(edges)
}

fn recency_decay(last_accessed: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
    let elapsed = (now - last_accessed).num_seconds() as f64 / 86400.0;
    if elapsed <= 0.0 {
        return 1.0;
    }
    0.5_f64.powf(elapsed / DECAY_HALF_LIFE_DAYS)
}

pub(crate) fn recall_associative(
    conn: &Connection,
    seed_node_ids: &[&str],
    max_hops: usize,
    min_edge_weight: f64,
    fanout: usize,
    now: DateTime<Utc>,
) -> Result<Vec<AssociationResult>, MemoryStoreError> {
    if seed_node_ids.is_empty() {
        return Ok(Vec::new());
    }

    let hops = if max_hops == 0 {
        DEFAULT_MAX_HOPS
    } else {
        max_hops
    };
    let min_w = if min_edge_weight <= 0.0 {
        DEFAULT_MIN_EDGE_WEIGHT
    } else {
        min_edge_weight
    };
    let fan = if fanout == 0 { DEFAULT_FANOUT } else { fanout };

    let nodes = load_all_nodes(conn)?;
    let edges = load_all_edges(conn)?;

    let mut adjacency: HashMap<&str, Vec<&GraphEdge>> = HashMap::new();
    for edge in &edges {
        adjacency.entry(&edge.source_id).or_default().push(edge);
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut results: Vec<AssociationResult> = Vec::new();

    for seed_id in seed_node_ids {
        if let Some(_seed_node) = nodes.get(*seed_id) {
            bfs_step(
                seed_id,
                1.0,
                &mut Vec::new(),
                &nodes,
                &adjacency,
                &mut visited,
                &mut results,
                hops,
                min_w,
                fan,
                now,
            );
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.dedup_by(|a, b| a.node.id == b.node.id);
    Ok(results)
}

#[allow(clippy::too_many_arguments)]
fn bfs_step(
    current_id: &str,
    current_activation: f64,
    path: &mut Vec<String>,
    nodes: &HashMap<String, GraphNode>,
    adjacency: &HashMap<&str, Vec<&GraphEdge>>,
    visited: &mut HashSet<String>,
    results: &mut Vec<AssociationResult>,
    remaining_hops: usize,
    min_edge_weight: f64,
    fanout: usize,
    now: DateTime<Utc>,
) {
    if remaining_hops == 0 {
        return;
    }
    if visited.contains(current_id) {
        return;
    }
    visited.insert(current_id.to_string());

    let neighbors = match adjacency.get(current_id) {
        Some(n) => n,
        None => return,
    };

    let mut scored: Vec<(&&GraphEdge, f64)> = neighbors
        .iter()
        .filter(|e| e.weight >= min_edge_weight)
        .filter_map(|edge| {
            let target = nodes.get(&edge.target_id)?;
            let decay = recency_decay(target.last_accessed_at, now);
            let score = current_activation * edge.weight * target.impression_strength * decay;
            Some((edge, score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(fanout);

    path.push(current_id.to_string());

    for (edge, score) in &scored {
        if visited.contains(&edge.target_id) {
            continue;
        }
        if let Some(target_node) = nodes.get(&edge.target_id) {
            results.push(AssociationResult {
                node: target_node.clone(),
                score: *score,
                path: path.clone(),
                hop_count: path.len(),
            });
            bfs_step(
                &edge.target_id,
                *score,
                path,
                nodes,
                adjacency,
                visited,
                results,
                remaining_hops - 1,
                min_edge_weight,
                fanout,
                now,
            );
        }
    }

    path.pop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryStore;

    fn make_node(id: &str, label: &str, strength: f64) -> GraphNode {
        let now = Utc::now();
        GraphNode {
            id: id.to_string(),
            kind: "entity".to_string(),
            label: label.to_string(),
            impression_strength: strength,
            weight: 1.0,
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
        }
    }

    fn make_edge(source: &str, target: &str, relation: &str, weight: f64) -> GraphEdge {
        let now = Utc::now();
        GraphEdge {
            source_id: source.to_string(),
            target_id: target.to_string(),
            relation_type: relation.to_string(),
            weight,
            created_at: now,
            last_reinforced_at: now,
        }
    }

    #[test]
    fn schema_migration_creates_graph_tables() {
        let store = MemoryStore::open_memory().unwrap();
        let count: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE 'memory_graph_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count >= 2);
    }

    #[test]
    fn upsert_and_get_node() {
        let store = MemoryStore::open_memory().unwrap();
        let node = make_node("n1", "WEB-005", 0.9);
        store.graph_upsert_node(&node).unwrap();
        let retrieved = store.graph_get_node("n1").unwrap().unwrap();
        assert_eq!(retrieved.label, "WEB-005");
        assert!((retrieved.impression_strength - 0.9).abs() < 0.01);
    }

    #[test]
    fn upsert_node_is_idempotent() {
        let store = MemoryStore::open_memory().unwrap();
        store.graph_upsert_node(&make_node("n1", "A", 0.5)).unwrap();
        store
            .graph_upsert_node(&make_node("n1", "A-updated", 0.8))
            .unwrap();
        let node = store.graph_get_node("n1").unwrap().unwrap();
        assert_eq!(node.label, "A-updated");
        assert!((node.impression_strength - 0.8).abs() < 0.01);
    }

    #[test]
    fn upsert_edge_is_idempotent() {
        let store = MemoryStore::open_memory().unwrap();
        store.graph_upsert_node(&make_node("n1", "A", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("n2", "B", 1.0)).unwrap();
        store
            .graph_upsert_edge(&make_edge("n1", "n2", "used_with", 0.5))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("n1", "n2", "used_with", 0.9))
            .unwrap();
        let results = store.graph_recall(&["n1"], 3, 0.3, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 0.9).abs() < 0.15);
    }

    #[test]
    fn recall_returns_direct_neighbors_sorted_by_score() {
        let store = MemoryStore::open_memory().unwrap();
        store
            .graph_upsert_node(&make_node("seed", "seed", 1.0))
            .unwrap();
        store.graph_upsert_node(&make_node("a", "A", 0.8)).unwrap();
        store.graph_upsert_node(&make_node("b", "B", 0.4)).unwrap();
        store
            .graph_upsert_edge(&make_edge("seed", "a", "used_with", 0.9))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("seed", "b", "used_with", 0.5))
            .unwrap();
        let results = store.graph_recall(&["seed"], 3, 0.3, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
        assert_eq!(results[0].node.id, "a");
    }

    #[test]
    fn recall_multi_hop_traverses_graph() {
        let store = MemoryStore::open_memory().unwrap();
        store.graph_upsert_node(&make_node("a", "A", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("b", "B", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("c", "C", 1.0)).unwrap();
        store
            .graph_upsert_edge(&make_edge("a", "b", "used_with", 0.8))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("b", "c", "used_with", 0.7))
            .unwrap();
        let results = store.graph_recall(&["a"], 3, 0.3, 10).unwrap();
        let ids: Vec<&str> = results.iter().map(|r| r.node.id.as_str()).collect();
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"c"));
    }

    #[test]
    fn recall_respects_min_edge_weight() {
        let store = MemoryStore::open_memory().unwrap();
        store
            .graph_upsert_node(&make_node("seed", "S", 1.0))
            .unwrap();
        store
            .graph_upsert_node(&make_node("strong", "Strong", 1.0))
            .unwrap();
        store
            .graph_upsert_node(&make_node("weak", "Weak", 1.0))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("seed", "strong", "used_with", 0.8))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("seed", "weak", "used_with", 0.1))
            .unwrap();
        let results = store.graph_recall(&["seed"], 3, 0.3, 10).unwrap();
        let ids: Vec<&str> = results.iter().map(|r| r.node.id.as_str()).collect();
        assert!(ids.contains(&"strong"));
        assert!(!ids.contains(&"weak"));
    }

    #[test]
    fn recall_respects_max_hops() {
        let store = MemoryStore::open_memory().unwrap();
        store.graph_upsert_node(&make_node("a", "A", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("b", "B", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("c", "C", 1.0)).unwrap();
        store.graph_upsert_node(&make_node("d", "D", 1.0)).unwrap();
        store
            .graph_upsert_edge(&make_edge("a", "b", "used_with", 0.9))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("b", "c", "used_with", 0.9))
            .unwrap();
        store
            .graph_upsert_edge(&make_edge("c", "d", "used_with", 0.9))
            .unwrap();
        let results = store.graph_recall(&["a"], 2, 0.3, 10).unwrap();
        let ids: Vec<&str> = results.iter().map(|r| r.node.id.as_str()).collect();
        assert!(ids.contains(&"b"));
        assert!(ids.contains(&"c"));
        assert!(!ids.contains(&"d"));
    }

    #[test]
    fn recall_empty_seeds_returns_empty() {
        let store = MemoryStore::open_memory().unwrap();
        let results = store.graph_recall(&[], 3, 0.3, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn recall_deterministic_for_same_graph_state() {
        let store1 = MemoryStore::open_memory().unwrap();
        let store2 = MemoryStore::open_memory().unwrap();
        for store in [&store1, &store2] {
            store.graph_upsert_node(&make_node("a", "A", 1.0)).unwrap();
            store.graph_upsert_node(&make_node("b", "B", 0.8)).unwrap();
            store.graph_upsert_node(&make_node("c", "C", 0.6)).unwrap();
            store
                .graph_upsert_edge(&make_edge("a", "b", "used_with", 0.9))
                .unwrap();
            store
                .graph_upsert_edge(&make_edge("a", "c", "used_with", 0.5))
                .unwrap();
            store
                .graph_upsert_edge(&make_edge("b", "c", "used_with", 0.7))
                .unwrap();
        }
        let r1 = store1.graph_recall(&["a"], 3, 0.3, 10).unwrap();
        let r2 = store2.graph_recall(&["a"], 3, 0.3, 10).unwrap();
        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.node.id, b.node.id);
            assert!((a.score - b.score).abs() < 0.001);
        }
    }

    #[test]
    fn recency_decay_decreases_over_time() {
        let now = Utc::now();
        let recent = now;
        let old = now - chrono::Duration::days(7);
        let ancient = now - chrono::Duration::days(30);
        let d_recent = recency_decay(recent, now);
        let d_old = recency_decay(old, now);
        let d_ancient = recency_decay(ancient, now);
        assert!(d_recent > d_old);
        assert!(d_old > d_ancient);
        assert!((d_recent - 1.0).abs() < 0.01);
        assert!((d_old - 0.5).abs() < 0.01);
    }
}
