//! Canonical, storage-neutral work-domain values.
//!
//! The domain is deliberately independent of persistence and presentation. During the Todo
//! compatibility window, `talos-session` remains the durable authority and projects records into
//! these values rather than maintaining a second repository.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

// Re-export completion/evaluation contracts from the canonical work-domain path.  The
// implementation lives in its own module to keep the graph model readable while allowing
// consumers to import all work-domain values from `talos_core::work`.
pub use crate::evaluation::{
    AcceptanceCriterion, ArtifactRef, CompletionClaim, CriterionEvaluation, CriterionKind,
    CriterionVerdict, Evaluation, EvaluationError, EvaluationFinding, EvaluationReport,
    EvaluationState, EvaluationSubject, EvaluationVerdict, EvidenceRef, FindingSeverity,
    WorkspaceRevision,
};

/// A durable node role in the canonical work graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkKind {
    /// A top-level mission containing goals and work units.
    Mission,
    /// A goal within a mission.
    Goal,
    /// An executable unit of work.
    WorkUnit,
}

/// Stable graph identity and revision for one work node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkIdentity {
    /// Stable UUID identity.
    pub id: Uuid,
    /// Node role.
    pub kind: WorkKind,
    /// Monotonically increasing persisted revision.
    pub revision: u64,
}

/// A storage-neutral canonical work node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkNode {
    /// Stable identity and revision.
    pub identity: WorkIdentity,
    /// Optional containing node identity.
    pub parent_id: Option<Uuid>,
    /// User-visible title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// Existing Todo-compatible status value.
    pub status: WorkStatus,
    /// Existing Todo-compatible priority value.
    pub priority: WorkPriority,
    /// Filterable tags.
    pub tags: Vec<String>,
}

/// Stable status values shared by all work-domain projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkStatus {
    /// Not started.
    Todo,
    /// Currently executing.
    InProgress,
    /// Completed.
    Completed,
    /// Blocked by a prerequisite.
    Blocked,
}

/// Stable priority values shared by all work-domain projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    Medium,
    /// High priority.
    High,
    /// Critical priority.
    Critical,
}

/// A directed dependency edge in the work graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct WorkEdge {
    /// Stable identity and revision of the durable edge subject.
    pub identity: WorkEdgeIdentity,
    /// Predecessor node that must be handled first.
    pub parent_id: Uuid,
    /// Dependent node.
    pub child_id: Uuid,
}

/// Stable identity and revision of a dependency edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct WorkEdgeIdentity {
    /// Stable UUID identity.
    pub id: Uuid,
    /// Monotonically increasing persisted revision.
    pub revision: u64,
}

/// Error returned when a proposed graph edge is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WorkGraphError {
    /// An edge points to itself.
    #[error("work node cannot depend on itself: {0}")]
    SelfDependency(Uuid),
    /// An edge would introduce a cycle.
    #[error("work dependency would create a cycle: {parent_id} -> {child_id}")]
    Cycle { parent_id: Uuid, child_id: Uuid },
    /// A graph contains an edge whose endpoint is absent.
    #[error("work edge references an unknown node")]
    UnknownNode,
    /// A node identity occurs more than once.
    #[error("work graph contains a duplicate node identity: {0}")]
    DuplicateNode(Uuid),
    /// An edge identity or endpoint pair occurs more than once.
    #[error("work graph contains a duplicate edge")]
    DuplicateEdge,
    /// A containment relationship does not match Mission/Goal/WorkUnit roles.
    #[error("invalid work containment")]
    InvalidContainment,
}

/// An immutable, validated canonical work graph snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkGraph {
    /// Nodes in deterministic insertion order.
    pub nodes: Vec<WorkNode>,
    /// Validated dependency edges.
    pub edges: Vec<WorkEdge>,
}

impl WorkGraph {
    /// Construct a graph after validating identities, containment and acyclicity.
    pub fn new(nodes: Vec<WorkNode>, edges: Vec<WorkEdge>) -> Result<Self, WorkGraphError> {
        let kinds: HashMap<_, _> = nodes
            .iter()
            .map(|node| (node.identity.id, node.identity.kind))
            .collect();
        if kinds.len() != nodes.len() {
            let mut seen = HashSet::new();
            let duplicate = nodes
                .iter()
                .find_map(|node| (!seen.insert(node.identity.id)).then_some(node.identity.id))
                .unwrap_or(Uuid::nil());
            return Err(WorkGraphError::DuplicateNode(duplicate));
        }
        if nodes.iter().any(|node| match node.identity.kind {
            WorkKind::Mission => node.parent_id.is_some(),
            WorkKind::Goal => node
                .parent_id
                .is_none_or(|parent| kinds.get(&parent) != Some(&WorkKind::Mission)),
            // Legacy Todo WorkUnits are permitted to remain session-rooted during the declared
            // compatibility window; new contained WorkUnits must have a Goal parent.
            WorkKind::WorkUnit => node
                .parent_id
                .is_some_and(|parent| kinds.get(&parent) != Some(&WorkKind::Goal)),
        }) {
            return Err(WorkGraphError::InvalidContainment);
        }
        if edges
            .iter()
            .any(|edge| !kinds.contains_key(&edge.parent_id) || !kinds.contains_key(&edge.child_id))
        {
            return Err(WorkGraphError::UnknownNode);
        }
        let mut edge_ids = HashSet::new();
        let mut endpoint_pairs = HashSet::new();
        if edges.iter().any(|edge| {
            !edge_ids.insert(edge.identity.id)
                || !endpoint_pairs.insert((edge.parent_id, edge.child_id))
        }) {
            return Err(WorkGraphError::DuplicateEdge);
        }
        let mut accepted = Vec::with_capacity(edges.len());
        for edge in edges {
            validate_edge(accepted.iter().copied(), edge)?;
            accepted.push(edge);
        }
        Ok(Self {
            nodes,
            edges: accepted,
        })
    }

    /// Return a graph with one new node, rejecting duplicate identities and invalid containment.
    pub fn with_node(&self, node: WorkNode) -> Result<Self, WorkGraphError> {
        let mut nodes = self.nodes.clone();
        nodes.push(node);
        Self::new(nodes, self.edges.clone())
    }

    /// Return a graph with one validated dependency edge.
    pub fn with_edge(&self, edge: WorkEdge) -> Result<Self, WorkGraphError> {
        let mut edges = self.edges.clone();
        edges.push(edge);
        Self::new(self.nodes.clone(), edges)
    }

    /// Return a graph without one edge identity.
    #[must_use]
    pub fn without_edge(&self, edge_id: Uuid) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self
                .edges
                .iter()
                .copied()
                .filter(|edge| edge.identity.id != edge_id)
                .collect(),
        }
    }

    /// Find one node by its stable UUID.
    #[must_use]
    pub fn node(&self, id: Uuid) -> Option<&WorkNode> {
        self.nodes.iter().find(|node| node.identity.id == id)
    }
}

/// Validate adding one edge to an existing acyclic edge set.
pub fn validate_edge(
    edges: impl IntoIterator<Item = WorkEdge>,
    edge: WorkEdge,
) -> Result<(), WorkGraphError> {
    if edge.parent_id == edge.child_id {
        return Err(WorkGraphError::SelfDependency(edge.parent_id));
    }
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for existing in edges {
        adjacency
            .entry(existing.parent_id)
            .or_default()
            .push(existing.child_id);
    }
    let mut stack = vec![edge.child_id];
    let mut visited = HashSet::new();
    while let Some(node) = stack.pop() {
        if node == edge.parent_id {
            return Err(WorkGraphError::Cycle {
                parent_id: edge.parent_id,
                child_id: edge.child_id,
            });
        }
        if visited.insert(node)
            && let Some(children) = adjacency.get(&node)
        {
            stack.extend(children.iter().copied());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_self_dependency_and_cycles() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_eq!(
            validate_edge(
                [],
                WorkEdge {
                    identity: WorkEdgeIdentity {
                        id: Uuid::new_v4(),
                        revision: 1
                    },
                    parent_id: a,
                    child_id: a
                }
            ),
            Err(WorkGraphError::SelfDependency(a))
        );
        assert_eq!(
            validate_edge(
                [WorkEdge {
                    identity: WorkEdgeIdentity {
                        id: Uuid::new_v4(),
                        revision: 1
                    },
                    parent_id: a,
                    child_id: b
                }],
                WorkEdge {
                    identity: WorkEdgeIdentity {
                        id: Uuid::new_v4(),
                        revision: 1
                    },
                    parent_id: b,
                    child_id: a
                }
            ),
            Err(WorkGraphError::Cycle {
                parent_id: b,
                child_id: a
            })
        );
    }

    #[test]
    fn accepts_disconnected_edge() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert!(
            validate_edge(
                [],
                WorkEdge {
                    identity: WorkEdgeIdentity {
                        id: Uuid::new_v4(),
                        revision: 1
                    },
                    parent_id: a,
                    child_id: b
                }
            )
            .is_ok()
        );
    }
}
