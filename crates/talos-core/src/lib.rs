//! Talos core — foundation types, core traits, and error definitions.

pub mod approval;
pub mod background_job;
pub mod evaluation;
pub mod message;
pub mod model;
pub mod provider;
pub mod session;
pub mod submission;
pub mod tool;
pub mod tool_filter;
pub mod work;

pub use approval::{ApprovalChoice, TuiApprovalRequest};
pub use evaluation::{
    AcceptanceCriterion, ArtifactRef, CompletionClaim, CriterionEvaluation, CriterionKind,
    CriterionVerdict, Evaluation, EvaluationError, EvaluationFinding, EvaluationReport,
    EvaluationState, EvaluationSubject, EvaluationVerdict, EvidenceRef, FindingSeverity,
    WorkspaceRevision,
};
pub use work::{
    DeliveryBlockReason, DeliveryEligibility, MissionEvaluation, MissionGate, MissionGateResult,
    WorkProjectionEvent,
};
