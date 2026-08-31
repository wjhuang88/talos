//! Independent, read-only evaluation of completion claims.
//!
//! The evaluator deliberately receives a bounded claim snapshot rather than the executor's
//! conversation.  Its output is revalidated by the P2 state machine before it can become a
//! verdict; provider failures and malformed output never become PASS.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use talos_core::evaluation::{
    CompletionClaim, Evaluation, EvaluationError, EvaluationReport, EvidenceRef,
};
use talos_core::message::{AgentEvent, Message};
use talos_core::provider::LanguageModel;
use talos_core::tool::ToolNature;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

/// Validation status recorded in a bounded evidence snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValidationEvidenceStatus {
    /// The producer completed successfully.
    Passed,
    /// The producer completed and found a failure.
    Failed,
    /// The producer could not run or did not produce a result.
    Unavailable,
}

/// Provenance-preserving validation evidence made available to an evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ValidationEvidence {
    /// Stable evidence identity from the validation producer.
    pub evidence: EvidenceRef,
    /// Producer outcome; this is evidence, not a Goal verdict.
    pub status: ValidationEvidenceStatus,
    /// Digest of the producer record or artifact set.
    pub record_digest: String,
}

impl ValidationEvidence {
    /// Construct evidence, rejecting an absent integrity binding.
    pub fn new(
        evidence: EvidenceRef,
        status: ValidationEvidenceStatus,
        record_digest: impl Into<String>,
    ) -> Result<Self, EvaluatorError> {
        let record_digest = record_digest.into();
        if record_digest.trim().is_empty() || evidence.kind.trim().is_empty() {
            return Err(EvaluatorError::InvalidEvidence);
        }
        Ok(Self {
            evidence,
            status,
            record_digest,
        })
    }
}

/// The bounded, fresh context sent to an evaluator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvaluatorRequest {
    /// Exact claim and subject identity under inspection.
    pub claim: CompletionClaim,
    /// Validation records available as references only.
    pub validation_evidence: Vec<ValidationEvidence>,
    /// Explicitly states that evaluator tools are read-only.
    pub read_only: bool,
}

impl EvaluatorRequest {
    fn for_claim(claim: &CompletionClaim, evidence: Vec<ValidationEvidence>) -> Self {
        Self {
            claim: claim.clone(),
            validation_evidence: evidence,
            read_only: true,
        }
    }
}

/// A model assessor used by the independent evaluator. It cannot execute tools through this API.
#[async_trait]
pub trait EvaluatorAssessor: Send + Sync {
    /// Return one JSON [`EvaluationReport`] within the supplied deadline.
    async fn assess(&self, request: EvaluatorRequest, deadline: Duration)
    -> Result<String, String>;

    /// Stable evaluator identity for audit records.
    fn identity(&self) -> &str {
        "configured-evaluator"
    }
}

/// Provider-backed assessor that sends one tool-free request with a fresh context.
pub struct ProviderEvaluatorAssessor {
    provider: Arc<dyn LanguageModel>,
    identity: String,
}

impl ProviderEvaluatorAssessor {
    /// Create an assessor from an independent provider/runtime instance.
    #[must_use]
    pub fn new(provider: Arc<dyn LanguageModel>) -> Self {
        Self {
            provider,
            identity: "configured-evaluator".to_owned(),
        }
    }

    /// Set the non-secret identity exposed in audit records.
    #[must_use]
    pub fn with_identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = identity.into();
        self
    }
}

#[async_trait]
impl EvaluatorAssessor for ProviderEvaluatorAssessor {
    async fn assess(
        &self,
        request: EvaluatorRequest,
        deadline: Duration,
    ) -> Result<String, String> {
        let payload = serde_json::to_string(&request).map_err(|error| error.to_string())?;
        let messages = vec![
            Message::System {
                content: "You are an independent evaluator. Return only one JSON EvaluationReport. Use the exact claim subject and criterion IDs. Do not use tools, infer missing evidence, or certify from executor reasoning.".to_owned(),
                cache_markers: Vec::new(),
            },
            Message::User {
                content: format!("Evaluate this bounded claim snapshot; return JSON only:\n{payload}"),
            },
        ];
        let mut events = self
            .provider
            .stream(&messages)
            .await
            .map_err(|error| error.to_string())?;
        let mut output = String::new();
        let deadline = tokio::time::sleep(deadline);
        tokio::pin!(deadline);
        loop {
            tokio::select! {
                _ = &mut deadline => return Err("evaluator deadline exceeded".to_owned()),
                event = events.recv() => match event {
                    Some(AgentEvent::TextDelta { delta }) => output.push_str(&delta),
                    Some(AgentEvent::ToolCall { .. }) => return Err("evaluator tool use is forbidden".to_owned()),
                    Some(AgentEvent::Error { message }) => return Err(message),
                    Some(AgentEvent::TurnEnd { .. }) | None => break,
                    Some(_) => {}
                },
            }
        }
        if output.trim().is_empty() {
            return Err("evaluator returned no report".to_owned());
        }
        Ok(output)
    }

    fn identity(&self) -> &str {
        &self.identity
    }
}

/// Read-only admission policy for evaluator tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluatorAdmission {
    read_only: bool,
}

impl Default for EvaluatorAdmission {
    fn default() -> Self {
        Self { read_only: true }
    }
}

impl EvaluatorAdmission {
    /// Returns whether a tool nature can be admitted by the default evaluator policy.
    #[must_use]
    pub const fn allows(self, nature: ToolNature) -> bool {
        self.read_only && matches!(nature, ToolNature::Read | ToolNature::Internal)
    }
}

/// Explicit non-PASS outcome when evaluation cannot safely produce a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatorFailure {
    /// Stable evaluator identity.
    pub evaluator: String,
    /// Bounded reason suitable for audit/status output.
    pub reason: String,
}

/// Result of one independent evaluation attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluatorOutcome {
    /// A report was accepted by the P2 state machine.
    Report { evaluation: Box<Evaluation> },
    /// Evaluation ended safely without a PASS report.
    Failure(EvaluatorFailure),
}

/// Errors raised before a safe evaluator outcome can be constructed.
#[derive(Debug, Error)]
pub enum EvaluatorError {
    /// Evidence omitted its producer identity or integrity digest.
    #[error("validation evidence is missing provenance or integrity binding")]
    InvalidEvidence,
    /// The claim subject or report violated the P2 contract.
    #[error("evaluation contract rejected evaluator report: {0}")]
    Contract(#[from] EvaluationError),
}

/// Independent evaluator coordinator with bounded, fail-closed execution.
pub struct IndependentEvaluator {
    assessor: Arc<dyn EvaluatorAssessor>,
    deadline: Duration,
    admission: EvaluatorAdmission,
}

impl IndependentEvaluator {
    /// Create an evaluator. The deadline is clamped to a bounded 1ms-30s range.
    #[must_use]
    pub fn new(assessor: Arc<dyn EvaluatorAssessor>, deadline: Duration) -> Self {
        Self {
            assessor,
            deadline: deadline.clamp(Duration::from_millis(1), Duration::from_secs(30)),
            admission: EvaluatorAdmission::default(),
        }
    }

    /// Returns the read-only admission policy used by this evaluator.
    #[must_use]
    pub const fn admission(&self) -> EvaluatorAdmission {
        self.admission
    }

    /// Evaluate one exact claim, returning an explicit non-PASS failure on unsafe conditions.
    pub async fn evaluate(
        &self,
        claim: &CompletionClaim,
        validation_evidence: Vec<ValidationEvidence>,
    ) -> EvaluatorOutcome {
        self.evaluate_with_cancellation(claim, validation_evidence, CancellationToken::new())
            .await
    }

    /// Evaluate with caller-owned cancellation. Cancellation is always an explicit non-PASS.
    pub async fn evaluate_with_cancellation(
        &self,
        claim: &CompletionClaim,
        validation_evidence: Vec<ValidationEvidence>,
        cancellation: CancellationToken,
    ) -> EvaluatorOutcome {
        let request = EvaluatorRequest::for_claim(claim, validation_evidence);
        let evaluator = self.assessor.identity().to_owned();
        if request.validation_evidence.iter().any(|evidence| {
            evidence.record_digest.trim().is_empty() || evidence.evidence.kind.trim().is_empty()
        }) {
            return EvaluatorOutcome::Failure(EvaluatorFailure {
                evaluator,
                reason: "validation evidence lacks provenance or integrity binding".to_owned(),
            });
        }
        let valid_evidence: HashSet<_> = request
            .validation_evidence
            .iter()
            .filter(|evidence| {
                !evidence.record_digest.trim().is_empty()
                    && !evidence.evidence.kind.trim().is_empty()
                    && evidence.status != ValidationEvidenceStatus::Unavailable
            })
            .map(|evidence| evidence.evidence.clone())
            .collect();
        let assessment = self.assessor.assess(request, self.deadline);
        tokio::pin!(assessment);
        let raw = tokio::select! {
            _ = cancellation.cancelled() => {
                return EvaluatorOutcome::Failure(EvaluatorFailure {
                    evaluator,
                    reason: "evaluator cancelled".to_owned(),
                });
            }
            result = tokio::time::timeout(self.deadline, &mut assessment) => match result {
                Ok(Ok(raw)) => raw,
                Ok(Err(reason)) => {
                    return EvaluatorOutcome::Failure(EvaluatorFailure { evaluator, reason });
                }
                Err(_) => {
                    return EvaluatorOutcome::Failure(EvaluatorFailure {
                        evaluator,
                        reason: "evaluator deadline exceeded".to_owned(),
                    });
                }
            }
        };
        let report: EvaluationReport = match serde_json::from_str(&raw) {
            Ok(report) => report,
            Err(error) => {
                return EvaluatorOutcome::Failure(EvaluatorFailure {
                    evaluator,
                    reason: format!("malformed evaluator report: {error}"),
                });
            }
        };
        if let Err(reason) = validate_report_evidence(claim, &report, &valid_evidence) {
            return EvaluatorOutcome::Failure(EvaluatorFailure { evaluator, reason });
        }
        let mut evaluation = claim.evaluation();
        if let Err(error) = evaluation.begin() {
            return EvaluatorOutcome::Failure(EvaluatorFailure {
                evaluator,
                reason: error.to_string(),
            });
        }
        if let Err(error) = evaluation.accept_report(report) {
            return EvaluatorOutcome::Failure(EvaluatorFailure {
                evaluator,
                reason: error.to_string(),
            });
        }
        EvaluatorOutcome::Report {
            evaluation: Box::new(evaluation),
        }
    }
}

fn validate_report_evidence(
    claim: &CompletionClaim,
    report: &EvaluationReport,
    valid_evidence: &HashSet<EvidenceRef>,
) -> Result<(), String> {
    for result in &report.results {
        let Some(criterion) = claim
            .criteria
            .iter()
            .find(|criterion| criterion.id == result.criterion_id)
        else {
            return Err("evaluator report references an unknown criterion".to_owned());
        };
        if criterion.required
            && result.verdict == talos_core::evaluation::CriterionVerdict::Pass
            && (result.evidence.is_empty()
                || result
                    .evidence
                    .iter()
                    .any(|evidence| !valid_evidence.contains(evidence)))
        {
            return Err("required PASS criterion lacks valid supplied evidence".to_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use talos_core::evaluation::{
        AcceptanceCriterion, CriterionEvaluation, CriterionKind, CriterionVerdict,
        EvaluationSubject, EvaluationVerdict, WorkspaceRevision,
    };
    use talos_core::work::{WorkIdentity, WorkKind};
    use uuid::Uuid;

    fn claim() -> CompletionClaim {
        CompletionClaim::new(
            EvaluationSubject {
                mission: WorkIdentity {
                    id: Uuid::new_v4(),
                    kind: WorkKind::Mission,
                    revision: 1,
                },
                goal: WorkIdentity {
                    id: Uuid::new_v4(),
                    kind: WorkKind::Goal,
                    revision: 1,
                },
                workspace: WorkspaceRevision {
                    id: Uuid::new_v4(),
                    revision: 1,
                },
            },
            vec![AcceptanceCriterion {
                id: Uuid::new_v4(),
                kind: CriterionKind::Technical,
                statement: "works".into(),
                required: true,
            }],
            Vec::new(),
            Vec::new(),
            "executor hint",
        )
        .expect("claim")
    }

    struct Assessor(String);

    #[async_trait]
    impl EvaluatorAssessor for Assessor {
        async fn assess(
            &self,
            _request: EvaluatorRequest,
            _deadline: Duration,
        ) -> Result<String, String> {
            Ok(self.0.clone())
        }
    }

    struct HangingAssessor;

    #[async_trait]
    impl EvaluatorAssessor for HangingAssessor {
        async fn assess(
            &self,
            _request: EvaluatorRequest,
            _deadline: Duration,
        ) -> Result<String, String> {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(String::new())
        }
    }

    #[tokio::test]
    async fn malformed_output_is_explicit_failure() {
        let evaluator = IndependentEvaluator::new(
            Arc::new(Assessor("not-json".into())),
            Duration::from_secs(1),
        );
        assert!(matches!(
            evaluator.evaluate(&claim(), Vec::new()).await,
            EvaluatorOutcome::Failure(_)
        ));
    }

    #[tokio::test]
    async fn assessor_that_ignores_deadline_is_bounded() {
        let evaluator =
            IndependentEvaluator::new(Arc::new(HangingAssessor), Duration::from_millis(5));
        let outcome = evaluator.evaluate(&claim(), Vec::new()).await;
        assert!(
            matches!(outcome, EvaluatorOutcome::Failure(EvaluatorFailure { reason, .. }) if reason.contains("deadline"))
        );
    }

    #[tokio::test]
    async fn cancellation_is_explicit_failure() {
        let evaluator =
            IndependentEvaluator::new(Arc::new(HangingAssessor), Duration::from_secs(1));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let outcome = evaluator
            .evaluate_with_cancellation(&claim(), Vec::new(), cancellation)
            .await;
        assert!(
            matches!(outcome, EvaluatorOutcome::Failure(EvaluatorFailure { reason, .. }) if reason.contains("cancelled"))
        );
    }

    #[tokio::test]
    async fn valid_report_is_revalidated_and_accepted() {
        let claim = claim();
        let evidence = EvidenceRef {
            id: Uuid::new_v4(),
            kind: "validation".into(),
        };
        let result = CriterionEvaluation {
            criterion_id: claim.criteria[0].id,
            verdict: CriterionVerdict::Pass,
            evidence: vec![evidence.clone()],
            finding_ids: Vec::new(),
        };
        let report =
            EvaluationReport::new(&claim, claim.subject, vec![result], Vec::new()).expect("report");
        let raw = serde_json::to_string(&report).expect("json");
        let evaluator = IndependentEvaluator::new(Arc::new(Assessor(raw)), Duration::from_secs(1));
        let supplied =
            ValidationEvidence::new(evidence, ValidationEvidenceStatus::Passed, "digest")
                .expect("evidence");
        let outcome = evaluator.evaluate(&claim, vec![supplied]).await;
        match outcome {
            EvaluatorOutcome::Report { evaluation } => assert_eq!(
                evaluation.state,
                talos_core::evaluation::EvaluationState::Verdict(EvaluationVerdict::Pass)
            ),
            EvaluatorOutcome::Failure(error) => panic!("unexpected failure: {}", error.reason),
        }
    }

    #[tokio::test]
    async fn pass_without_supplied_evidence_is_rejected() {
        let claim = claim();
        let result = CriterionEvaluation {
            criterion_id: claim.criteria[0].id,
            verdict: CriterionVerdict::Pass,
            evidence: vec![EvidenceRef {
                id: Uuid::new_v4(),
                kind: "validation".into(),
            }],
            finding_ids: Vec::new(),
        };
        let report =
            EvaluationReport::new(&claim, claim.subject, vec![result], Vec::new()).expect("report");
        let evaluator = IndependentEvaluator::new(
            Arc::new(Assessor(serde_json::to_string(&report).expect("json"))),
            Duration::from_secs(1),
        );
        assert!(
            matches!(evaluator.evaluate(&claim, Vec::new()).await, EvaluatorOutcome::Failure(EvaluatorFailure { reason, .. }) if reason.contains("supplied evidence"))
        );
    }

    #[tokio::test]
    async fn claimed_evidence_hint_cannot_authorize_required_pass() {
        let mut claim = claim();
        let evidence = EvidenceRef {
            id: Uuid::new_v4(),
            kind: "validation".into(),
        };
        claim.claimed_evidence.push(evidence.clone());
        let result = CriterionEvaluation {
            criterion_id: claim.criteria[0].id,
            verdict: CriterionVerdict::Pass,
            evidence: vec![evidence],
            finding_ids: Vec::new(),
        };
        let report =
            EvaluationReport::new(&claim, claim.subject, vec![result], Vec::new()).expect("report");
        let evaluator = IndependentEvaluator::new(
            Arc::new(Assessor(serde_json::to_string(&report).expect("json"))),
            Duration::from_secs(1),
        );
        assert!(matches!(
            evaluator.evaluate(&claim, Vec::new()).await,
            EvaluatorOutcome::Failure(EvaluatorFailure { reason, .. })
                if reason.contains("supplied evidence")
        ));
    }

    #[tokio::test]
    async fn evidence_kind_mismatch_cannot_authorize_required_pass() {
        let claim = claim();
        let supplied = EvidenceRef {
            id: Uuid::new_v4(),
            kind: "validation".into(),
        };
        let reported = EvidenceRef {
            id: supplied.id,
            kind: "executor-claim".into(),
        };
        let result = CriterionEvaluation {
            criterion_id: claim.criteria[0].id,
            verdict: CriterionVerdict::Pass,
            evidence: vec![reported],
            finding_ids: Vec::new(),
        };
        let report =
            EvaluationReport::new(&claim, claim.subject, vec![result], Vec::new()).expect("report");
        let evaluator = IndependentEvaluator::new(
            Arc::new(Assessor(serde_json::to_string(&report).expect("json"))),
            Duration::from_secs(1),
        );
        let supplied =
            ValidationEvidence::new(supplied, ValidationEvidenceStatus::Passed, "digest")
                .expect("evidence");
        assert!(matches!(
            evaluator.evaluate(&claim, vec![supplied]).await,
            EvaluatorOutcome::Failure(EvaluatorFailure { reason, .. })
                if reason.contains("supplied evidence")
        ));
    }

    #[test]
    fn admission_rejects_side_effecting_tools() {
        let policy = EvaluatorAdmission::default();
        assert!(policy.allows(ToolNature::Read));
        assert!(policy.allows(ToolNature::Internal));
        assert!(!policy.allows(ToolNature::Write));
        assert!(!policy.allows(ToolNature::Execute));
        assert!(!policy.allows(ToolNature::Network));
    }

    #[test]
    fn evidence_requires_integrity_binding() {
        let reference = EvidenceRef {
            id: Uuid::new_v4(),
            kind: "validation".into(),
        };
        assert!(
            ValidationEvidence::new(reference.clone(), ValidationEvidenceStatus::Passed, "")
                .is_err()
        );
        assert!(
            ValidationEvidence::new(reference, ValidationEvidenceStatus::Passed, "digest").is_ok()
        );
    }
}
