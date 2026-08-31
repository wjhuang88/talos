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

/// A Mission-level evaluation result used by the final Delivery gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MissionEvaluation {
    /// Exact Mission identity and revision that was evaluated.
    pub mission: WorkIdentity,
    /// Independent evaluator verdict for the integrated Mission outcome.
    pub verdict: EvaluationVerdict,
}

/// Why a Mission is not eligible for Delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryBlockReason {
    /// A required Goal has no evaluation result.
    MissingGoalEvaluation,
    /// A Goal evaluation does not match the required Mission revision.
    StaleGoalEvaluation,
    /// A required Goal evaluation is not a passing verdict.
    GoalNotPassed,
    /// More than one result was supplied for the same required Goal.
    ConflictingGoalEvaluations,
    /// The independent Mission evaluation is absent.
    MissingMissionEvaluation,
    /// The Mission evaluation targets another identity or revision.
    StaleMissionEvaluation,
    /// The independent Mission evaluation did not pass.
    MissionNotPassed,
}

/// Delivery eligibility produced by the Mission final gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum DeliveryEligibility {
    /// All required Goal and Mission evaluations passed at the current revisions.
    Eligible,
    /// Delivery is denied until the reported blocker is resolved.
    Blocked { reason: DeliveryBlockReason },
}

/// Presentation-neutral event emitted by a Mission gate evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum WorkProjectionEvent {
    /// A required Goal was inspected by the gate.
    GoalObserved { goal_id: Uuid, revision: u64 },
    /// The Mission-level evaluator was inspected by the gate.
    MissionObserved { mission_id: Uuid, revision: u64 },
    /// The final Delivery eligibility was determined.
    DeliveryEligibilityChanged { eligible: bool },
}

/// Result of evaluating one Mission against its required Goal evaluations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MissionGateResult {
    /// Mission identity evaluated by the gate.
    pub mission: WorkIdentity,
    /// Delivery eligibility; blocked results never mutate work state.
    pub delivery: DeliveryEligibility,
    /// Deterministically ordered events for UI-neutral consumers.
    pub events: Vec<WorkProjectionEvent>,
}

/// Storage-neutral final gate for Mission Delivery.
#[derive(Debug, Clone, Copy)]
pub struct MissionGate<'a> {
    /// Mission identity and revision to evaluate.
    pub mission: WorkIdentity,
    /// Required Goal identities in deterministic order.
    pub required_goals: &'a [WorkIdentity],
    /// Existing revision-bound Goal evaluations.
    pub goal_evaluations: &'a [Evaluation],
    /// Independent Mission-level evaluation, if available.
    pub mission_evaluation: Option<MissionEvaluation>,
}

impl MissionGate<'_> {
    /// Evaluate required Goal and Mission results without mutating any input state.
    #[must_use]
    pub fn evaluate(&self) -> MissionGateResult {
        let mut events = Vec::with_capacity(self.required_goals.len() + 2);
        let mut blocked = None;
        for goal in self.required_goals {
            let evaluations: Vec<&Evaluation> = self
                .goal_evaluations
                .iter()
                .filter(|evaluation| {
                    evaluation.claim.subject.goal.id == goal.id
                        && evaluation.claim.subject.goal.kind == goal.kind
                })
                .collect();
            match evaluations.as_slice() {
                [] => {
                    blocked.get_or_insert(DeliveryBlockReason::MissingGoalEvaluation);
                }
                [evaluation] => {
                    events.push(WorkProjectionEvent::GoalObserved {
                        goal_id: goal.id,
                        revision: goal.revision,
                    });
                    if evaluation.claim.subject.goal != *goal
                        || evaluation.claim.subject.mission != self.mission
                    {
                        blocked.get_or_insert(DeliveryBlockReason::StaleGoalEvaluation);
                    } else if !evaluation.has_valid_pass() {
                        blocked.get_or_insert(DeliveryBlockReason::GoalNotPassed);
                    }
                }
                _ => {
                    blocked.get_or_insert(DeliveryBlockReason::ConflictingGoalEvaluations);
                }
            };
        }

        match self.mission_evaluation {
            None => {
                blocked.get_or_insert(DeliveryBlockReason::MissingMissionEvaluation);
            }
            Some(evaluation) => {
                events.push(WorkProjectionEvent::MissionObserved {
                    mission_id: evaluation.mission.id,
                    revision: evaluation.mission.revision,
                });
                if evaluation.mission != self.mission {
                    blocked.get_or_insert(DeliveryBlockReason::StaleMissionEvaluation);
                } else if evaluation.verdict != EvaluationVerdict::Pass {
                    blocked.get_or_insert(DeliveryBlockReason::MissionNotPassed);
                }
            }
        }

        let delivery = blocked.map_or(DeliveryEligibility::Eligible, |reason| {
            DeliveryEligibility::Blocked { reason }
        });
        events.push(WorkProjectionEvent::DeliveryEligibilityChanged {
            eligible: matches!(delivery, DeliveryEligibility::Eligible),
        });
        MissionGateResult {
            mission: self.mission,
            delivery,
            events,
        }
    }
}

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

    fn passing_evaluation(mission: WorkIdentity, goal: WorkIdentity) -> Evaluation {
        let subject = EvaluationSubject {
            mission,
            goal,
            workspace: WorkspaceRevision {
                id: Uuid::new_v4(),
                revision: 1,
            },
        };
        let criterion = AcceptanceCriterion {
            id: Uuid::new_v4(),
            kind: CriterionKind::Technical,
            statement: "works".into(),
            required: true,
        };
        let claim = CompletionClaim::new(subject, vec![criterion.clone()], vec![], vec![], "done")
            .expect("claim");
        let mut evaluation = claim.evaluation();
        evaluation.begin().expect("begin");
        let report = EvaluationReport::new(
            &claim,
            subject,
            vec![CriterionEvaluation {
                criterion_id: criterion.id,
                verdict: CriterionVerdict::Pass,
                evidence: vec![],
                finding_ids: vec![],
            }],
            vec![],
        )
        .expect("report");
        evaluation.accept_report(report).expect("accept");
        evaluation
    }

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

    #[test]
    fn mission_gate_requires_independent_mission_pass() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 4,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 2,
        };
        let result = MissionGate {
            mission,
            required_goals: &[goal],
            goal_evaluations: &[passing_evaluation(mission, goal)],
            mission_evaluation: None,
        }
        .evaluate();
        assert_eq!(
            result.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::MissingMissionEvaluation
            }
        );
        assert!(matches!(
            result.events.last(),
            Some(WorkProjectionEvent::DeliveryEligibilityChanged { eligible: false })
        ));
    }

    #[test]
    fn mission_gate_emits_deterministic_eligible_projection() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };
        let result = MissionGate {
            mission,
            required_goals: &[goal],
            goal_evaluations: &[passing_evaluation(mission, goal)],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(result.delivery, DeliveryEligibility::Eligible);
        assert_eq!(result.events.len(), 3);
        assert!(matches!(
            result.events.last(),
            Some(WorkProjectionEvent::DeliveryEligibilityChanged { eligible: true })
        ));
    }

    #[test]
    fn mission_gate_rejects_stale_goal_revision() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let required_goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 2,
        };
        let old_goal = WorkIdentity {
            revision: 1,
            ..required_goal
        };
        let result = MissionGate {
            mission,
            required_goals: &[required_goal],
            goal_evaluations: &[passing_evaluation(mission, old_goal)],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(
            result.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::StaleGoalEvaluation
            }
        );
    }

    #[test]
    fn p4_non_persistent_fixture_covers_claim_staleness_gate_and_delivery_projection() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };

        let work_unit = WorkNode {
            identity: WorkIdentity {
                id: Uuid::new_v4(),
                kind: WorkKind::WorkUnit,
                revision: 1,
            },
            parent_id: Some(goal.id),
            title: "fixture work unit".into(),
            description: None,
            status: WorkStatus::Completed,
            priority: WorkPriority::Medium,
            tags: vec![],
        };
        assert_eq!(work_unit.status, WorkStatus::Completed);

        // CompletionClaim::new plus an accepted report models the completed WorkUnit's
        // independent evaluation without introducing a persistence dependency in P4.
        let mut goal_evaluation = passing_evaluation(mission, goal);
        let mut changed_subject = goal_evaluation.claim.subject;
        changed_subject.goal.revision = 2;
        goal_evaluation
            .observe_subject(changed_subject)
            .expect("revision change marks the prior evaluation stale");
        goal_evaluation
            .request_rework()
            .expect("stale evaluation requires rework");
        assert_eq!(goal_evaluation.state, EvaluationState::Rework);

        let stale = MissionGate {
            mission,
            required_goals: &[changed_subject.goal],
            goal_evaluations: &[goal_evaluation.clone()],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(
            stale.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::StaleGoalEvaluation
            }
        );

        let refreshed = passing_evaluation(mission, changed_subject.goal);
        let eligible = MissionGate {
            mission,
            required_goals: &[changed_subject.goal],
            goal_evaluations: &[refreshed],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(eligible.delivery, DeliveryEligibility::Eligible);
        assert_eq!(eligible.events.len(), 3);
        assert!(serde_json::to_value(&eligible).is_ok());
    }

    #[test]
    fn mission_gate_rejects_duplicate_goal_evaluations() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };
        let first = passing_evaluation(mission, goal);
        let second = passing_evaluation(mission, goal);
        let result = MissionGate {
            mission,
            required_goals: &[goal],
            goal_evaluations: &[first, second],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(
            result.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::ConflictingGoalEvaluations
            }
        );
    }

    #[test]
    fn mission_gate_rejects_forged_passing_evaluation_without_report() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };
        let mut forged = passing_evaluation(mission, goal);
        forged.report = None;
        let result = MissionGate {
            mission,
            required_goals: &[goal],
            goal_evaluations: &[forged],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(
            result.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::GoalNotPassed
            }
        );
    }

    #[test]
    fn mission_gate_rejects_passing_state_with_valid_fail_report() {
        let mission = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Mission,
            revision: 1,
        };
        let goal = WorkIdentity {
            id: Uuid::new_v4(),
            kind: WorkKind::Goal,
            revision: 1,
        };
        let mut forged = passing_evaluation(mission, goal);
        let claim = forged.claim.clone();
        forged.report = Some(
            EvaluationReport::new(
                &claim,
                claim.subject,
                vec![CriterionEvaluation {
                    criterion_id: claim.criteria[0].id,
                    verdict: CriterionVerdict::Fail,
                    evidence: vec![],
                    finding_ids: vec![],
                }],
                vec![],
            )
            .expect("valid fail report"),
        );
        let result = MissionGate {
            mission,
            required_goals: &[goal],
            goal_evaluations: &[forged],
            mission_evaluation: Some(MissionEvaluation {
                mission,
                verdict: EvaluationVerdict::Pass,
            }),
        }
        .evaluate();
        assert_eq!(
            result.delivery,
            DeliveryEligibility::Blocked {
                reason: DeliveryBlockReason::GoalNotPassed
            }
        );
    }
}
