//! Storage-neutral completion claims and independent evaluation state.
//!
//! This module defines the contract between an executor and the evaluator.  A claim is an
//! assertion that work is ready to inspect; it is never a completion authority.  Reports are
//! bound to the exact subject revision captured by the claim and derive their aggregate verdict
//! from criterion-level results.

use crate::work::{WorkIdentity, WorkKind};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

/// The exact workspace/content revision inspected by an evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceRevision {
    /// Stable identity of the workspace/content subject.
    pub id: Uuid,
    /// Monotonic content revision (for example a commit plus dirty-tree digest revision).
    pub revision: u64,
}

/// Exact Mission, Goal and workspace subject to which a claim or report applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvaluationSubject {
    /// Mission identity and revision.
    pub mission: WorkIdentity,
    /// Goal identity and revision.
    pub goal: WorkIdentity,
    /// Workspace/content identity and revision.
    pub workspace: WorkspaceRevision,
}

impl EvaluationSubject {
    /// Validate that the subject contains the required Mission and Goal roles.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.mission.kind != WorkKind::Mission {
            return Err(EvaluationError::InvalidSubject(
                "mission identity is not a Mission",
            ));
        }
        if self.goal.kind != WorkKind::Goal {
            return Err(EvaluationError::InvalidSubject(
                "goal identity is not a Goal",
            ));
        }
        Ok(())
    }

    /// Return whether another subject is exactly the same evaluation subject.
    #[must_use]
    pub fn matches(&self, current: &Self) -> bool {
        self == current
    }
}

/// A stable reference to an artifact changed or inspected by an evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactRef {
    /// Stable artifact identity.
    pub id: Uuid,
    /// Human-readable locator, such as a path or commit object.
    pub locator: String,
}

/// A stable reference to evidence.  Evidence is referential and never a verdict authority.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRef {
    /// Stable evidence identity.
    pub id: Uuid,
    /// Producer or record kind (for example `validation`).
    pub kind: String,
}

/// The semantic kind of an acceptance criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriterionKind {
    /// A user-visible behavior requirement.
    Behavior,
    /// A technical/API contract requirement.
    Technical,
    /// A machine-verifiable validation requirement.
    Validation,
    /// A documentation or operational requirement.
    Documentation,
    /// An explicitly named project-specific criterion.
    Custom(String),
}

/// One typed acceptance criterion for a Goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    /// Stable criterion identity.
    pub id: Uuid,
    /// Criterion category.
    pub kind: CriterionKind,
    /// Short requirement text.
    pub statement: String,
    /// Required criteria determine the aggregate PASS/FAIL result.
    pub required: bool,
}

/// A completion assertion submitted by an executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompletionClaim {
    /// Stable claim identity.
    pub id: Uuid,
    /// Exact subject revision being claimed.
    pub subject: EvaluationSubject,
    /// Immutable acceptance criteria snapshot.
    pub criteria: Vec<AcceptanceCriterion>,
    /// Artifacts the executor says were changed.
    pub changed_artifacts: Vec<ArtifactRef>,
    /// Evidence references offered as hints to the evaluator.
    pub claimed_evidence: Vec<EvidenceRef>,
    /// Executor summary; never an independent verdict.
    pub executor_summary: String,
}

impl CompletionClaim {
    /// Construct a claim in the evaluation-pending state.
    pub fn new(
        subject: EvaluationSubject,
        criteria: Vec<AcceptanceCriterion>,
        changed_artifacts: Vec<ArtifactRef>,
        claimed_evidence: Vec<EvidenceRef>,
        executor_summary: impl Into<String>,
    ) -> Result<Self, EvaluationError> {
        subject.validate()?;
        validate_criteria(&criteria)?;
        Ok(Self {
            id: Uuid::new_v4(),
            subject,
            criteria,
            changed_artifacts,
            claimed_evidence,
            executor_summary: executor_summary.into(),
        })
    }

    /// Begin a storage-neutral evaluation state machine for this claim.
    #[must_use]
    pub fn evaluation(&self) -> Evaluation {
        Evaluation {
            claim: self.clone(),
            state: EvaluationState::Pending,
            report: None,
        }
    }
}

/// Criterion-level outcome produced by an evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CriterionVerdict {
    /// Criterion is satisfied.
    Pass,
    /// Criterion is not satisfied.
    Fail,
    /// Available evidence is insufficient to decide.
    Inconclusive,
}

/// Aggregate evaluation outcome derived from required criteria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationVerdict {
    /// Every required criterion passed.
    Pass,
    /// At least one required criterion failed.
    Fail,
    /// No required criterion failed, but at least one is inconclusive.
    Inconclusive,
}

/// Severity assigned to one evaluator finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    /// Informational observation.
    Info,
    /// Non-blocking concern.
    Warning,
    /// Finding that prevents a passing criterion.
    Blocking,
}

/// A bounded, criterion-linked evaluator finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvaluationFinding {
    /// Stable finding identity.
    pub id: Uuid,
    /// Criterion this finding concerns, if criterion-specific.
    pub criterion_id: Option<Uuid>,
    /// Finding severity.
    pub severity: FindingSeverity,
    /// Concise explanation for rework or audit.
    pub summary: String,
    /// Referenced evidence; references do not issue the verdict.
    pub evidence: Vec<EvidenceRef>,
}

/// One evaluator result for one acceptance criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CriterionEvaluation {
    /// Criterion identity from the claim snapshot.
    pub criterion_id: Uuid,
    /// Evaluator outcome.
    pub verdict: CriterionVerdict,
    /// Evidence references supporting the outcome.
    pub evidence: Vec<EvidenceRef>,
    /// Findings associated with this criterion.
    pub finding_ids: Vec<Uuid>,
}

/// A complete criterion-granular report bound to one exact claim subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvaluationReport {
    /// Stable report identity.
    pub id: Uuid,
    /// Claim being evaluated.
    pub claim_id: Uuid,
    /// Exact subject inspected.
    pub subject: EvaluationSubject,
    /// Criterion-level outcomes.
    pub results: Vec<CriterionEvaluation>,
    /// Findings explaining outcomes.
    pub findings: Vec<EvaluationFinding>,
    /// Deterministically derived aggregate verdict.
    pub verdict: EvaluationVerdict,
}

impl EvaluationReport {
    /// Construct a report, rejecting missing, duplicate or unknown criteria and contradictions.
    pub fn new(
        claim: &CompletionClaim,
        subject: EvaluationSubject,
        results: Vec<CriterionEvaluation>,
        findings: Vec<EvaluationFinding>,
    ) -> Result<Self, EvaluationError> {
        if !claim.subject.matches(&subject) {
            return Err(EvaluationError::SubjectMismatch);
        }
        let criterion_ids: HashSet<_> = claim
            .criteria
            .iter()
            .map(|criterion| criterion.id)
            .collect();
        let mut result_ids = HashSet::new();
        if results.iter().any(|result| {
            !criterion_ids.contains(&result.criterion_id) || !result_ids.insert(result.criterion_id)
        }) {
            return Err(EvaluationError::CriterionMismatch);
        }
        if result_ids.len() != criterion_ids.len() {
            return Err(EvaluationError::CriterionMismatch);
        }
        let finding_ids: HashSet<_> = findings.iter().map(|finding| finding.id).collect();
        if finding_ids.len() != findings.len()
            || findings.iter().any(|finding| {
                finding
                    .criterion_id
                    .is_some_and(|id| !criterion_ids.contains(&id))
                    || finding.summary.trim().is_empty()
            })
            || results
                .iter()
                .flat_map(|result| result.finding_ids.iter())
                .any(|id| !finding_ids.contains(id))
        {
            return Err(EvaluationError::FindingMismatch);
        }
        let verdict = aggregate_verdict(&claim.criteria, &results);
        Ok(Self {
            id: Uuid::new_v4(),
            claim_id: claim.id,
            subject,
            results,
            findings,
            verdict,
        })
    }
}

/// Lifecycle of an evaluation report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind", content = "verdict")]
pub enum EvaluationState {
    /// Claim is awaiting an independent evaluator.
    Pending,
    /// Evaluator is inspecting the exact subject.
    Evaluating,
    /// A report has been accepted for the current subject.
    Verdict(EvaluationVerdict),
    /// Subject changed after a report and the prior verdict no longer certifies it.
    Stale,
    /// Rework is required before a new claim can be submitted.
    Rework,
}

/// Storage-neutral state machine for one completion claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Evaluation {
    /// Immutable claim under evaluation.
    pub claim: CompletionClaim,
    /// Current lifecycle state.
    pub state: EvaluationState,
    /// Accepted report, when one exists.
    pub report: Option<EvaluationReport>,
}

impl Evaluation {
    /// Transition `Pending` to `Evaluating`.
    pub fn begin(&mut self) -> Result<(), EvaluationError> {
        if self.state != EvaluationState::Pending {
            return Err(EvaluationError::IllegalTransition);
        }
        self.state = EvaluationState::Evaluating;
        Ok(())
    }

    /// Accept a report and transition `Evaluating` to its derived verdict.
    pub fn accept_report(&mut self, report: EvaluationReport) -> Result<(), EvaluationError> {
        if self.state != EvaluationState::Evaluating || report.claim_id != self.claim.id {
            return Err(EvaluationError::IllegalTransition);
        }
        self.state = EvaluationState::Verdict(report.verdict);
        self.report = Some(report);
        Ok(())
    }

    /// Mark a verdict stale when the relevant subject revision changes.
    pub fn observe_subject(&mut self, current: EvaluationSubject) -> Result<(), EvaluationError> {
        if self.state
            != EvaluationState::Verdict(
                self.report
                    .as_ref()
                    .map_or(EvaluationVerdict::Inconclusive, |report| report.verdict),
            )
        {
            return Err(EvaluationError::IllegalTransition);
        }
        if !self.claim.subject.matches(&current) {
            self.state = EvaluationState::Stale;
        }
        Ok(())
    }

    /// Return a failed/inconclusive evaluation to the executor for rework.
    pub fn request_rework(&mut self) -> Result<(), EvaluationError> {
        match self.state {
            EvaluationState::Verdict(EvaluationVerdict::Fail | EvaluationVerdict::Inconclusive)
            | EvaluationState::Stale => {
                self.state = EvaluationState::Rework;
                Ok(())
            }
            _ => Err(EvaluationError::IllegalTransition),
        }
    }
}

/// Errors raised while constructing or advancing the evaluation contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvaluationError {
    /// Mission/Goal roles or another subject invariant is invalid.
    #[error("invalid evaluation subject: {0}")]
    InvalidSubject(&'static str),
    /// Criteria are empty, duplicated or malformed.
    #[error("invalid acceptance criteria")]
    InvalidCriteria,
    /// A report does not exactly match the claim's criterion set.
    #[error("evaluation criteria do not exactly match the claim")]
    CriterionMismatch,
    /// A report finding is duplicated, unknown or malformed.
    #[error("evaluation findings do not match the report")]
    FindingMismatch,
    /// A report was produced for a different subject revision.
    #[error("evaluation subject revision does not match the claim")]
    SubjectMismatch,
    /// An operation is not legal from the current lifecycle state.
    #[error("illegal evaluation state transition")]
    IllegalTransition,
}

fn validate_criteria(criteria: &[AcceptanceCriterion]) -> Result<(), EvaluationError> {
    let mut ids = HashSet::new();
    if criteria.is_empty()
        || criteria
            .iter()
            .any(|criterion| criterion.statement.trim().is_empty() || !ids.insert(criterion.id))
    {
        return Err(EvaluationError::InvalidCriteria);
    }
    Ok(())
}

fn aggregate_verdict(
    criteria: &[AcceptanceCriterion],
    results: &[CriterionEvaluation],
) -> EvaluationVerdict {
    let outcomes: HashMap<_, _> = results
        .iter()
        .map(|result| (result.criterion_id, result.verdict))
        .collect();
    let mut required = criteria.iter().filter(|criterion| criterion.required);
    if required
        .clone()
        .any(|criterion| outcomes.get(&criterion.id) == Some(&CriterionVerdict::Fail))
    {
        EvaluationVerdict::Fail
    } else if required
        .any(|criterion| outcomes.get(&criterion.id) == Some(&CriterionVerdict::Inconclusive))
    {
        EvaluationVerdict::Inconclusive
    } else {
        EvaluationVerdict::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> EvaluationSubject {
        EvaluationSubject {
            mission: WorkIdentity {
                id: Uuid::new_v4(),
                kind: WorkKind::Mission,
                revision: 1,
            },
            goal: WorkIdentity {
                id: Uuid::new_v4(),
                kind: WorkKind::Goal,
                revision: 2,
            },
            workspace: WorkspaceRevision {
                id: Uuid::new_v4(),
                revision: 3,
            },
        }
    }

    fn criterion(required: bool) -> AcceptanceCriterion {
        AcceptanceCriterion {
            id: Uuid::new_v4(),
            kind: CriterionKind::Behavior,
            statement: "works".into(),
            required,
        }
    }

    fn result(id: Uuid, verdict: CriterionVerdict) -> CriterionEvaluation {
        CriterionEvaluation {
            criterion_id: id,
            verdict,
            evidence: vec![],
            finding_ids: vec![],
        }
    }

    #[test]
    fn state_machine_rejects_self_certification_and_stales_on_revision() {
        let claim = CompletionClaim::new(subject(), vec![criterion(true)], vec![], vec![], "done")
            .expect("valid test fixture");
        let mut evaluation = claim.evaluation();
        assert_eq!(evaluation.state, EvaluationState::Pending);
        let report = EvaluationReport::new(
            &claim,
            claim.subject,
            vec![result(claim.criteria[0].id, CriterionVerdict::Pass)],
            vec![],
        )
        .expect("valid test fixture");
        assert_eq!(
            evaluation.accept_report(report),
            Err(EvaluationError::IllegalTransition)
        );
        evaluation.begin().expect("valid test fixture");
        let report = EvaluationReport::new(
            &claim,
            claim.subject,
            vec![result(claim.criteria[0].id, CriterionVerdict::Pass)],
            vec![],
        )
        .expect("valid test fixture");
        evaluation
            .accept_report(report)
            .expect("valid test fixture");
        let mut changed = claim.subject;
        changed.goal.revision += 1;
        evaluation
            .observe_subject(changed)
            .expect("valid test fixture");
        assert_eq!(evaluation.state, EvaluationState::Stale);
        evaluation.request_rework().expect("valid test fixture");
        assert_eq!(evaluation.state, EvaluationState::Rework);
    }

    #[test]
    fn locale_is_not_an_evaluation_subject_component() {
        let a = subject();
        let b = a;
        assert!(a.matches(&b));
    }

    #[test]
    fn aggregate_requires_required_criteria_only() {
        let required = criterion(true);
        let optional = criterion(false);
        let claim = CompletionClaim::new(
            subject(),
            vec![required.clone(), optional.clone()],
            vec![],
            vec![],
            "done",
        )
        .expect("valid test fixture");
        let report = EvaluationReport::new(
            &claim,
            claim.subject,
            vec![
                result(required.id, CriterionVerdict::Pass),
                result(optional.id, CriterionVerdict::Fail),
            ],
            vec![],
        )
        .expect("valid test fixture");
        assert_eq!(report.verdict, EvaluationVerdict::Pass);
    }

    #[test]
    fn report_rejects_duplicates_and_subject_mismatch() {
        let c = criterion(true);
        let claim = CompletionClaim::new(subject(), vec![c.clone()], vec![], vec![], "done")
            .expect("valid test fixture");
        assert_eq!(
            EvaluationReport::new(
                &claim,
                subject(),
                vec![result(c.id, CriterionVerdict::Pass)],
                vec![]
            ),
            Err(EvaluationError::SubjectMismatch)
        );
        assert_eq!(
            EvaluationReport::new(
                &claim,
                claim.subject,
                vec![
                    result(c.id, CriterionVerdict::Pass),
                    result(c.id, CriterionVerdict::Pass)
                ],
                vec![]
            ),
            Err(EvaluationError::CriterionMismatch)
        );
    }
}
