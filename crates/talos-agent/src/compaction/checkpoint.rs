//! Bounded semantic checkpoint payloads. Raw range ownership stays with the runtime.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::token::TokenEstimator;

/// Maximum encoded checkpoint payload accepted from a provider.
pub const MAX_CHECKPOINT_BYTES: usize = 32 * 1024;

/// A derived statement with references to runtime-selected source segments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CheckpointItem {
    /// Model-generated interpretation, not authoritative source text.
    pub text: String,
    /// Source identifiers issued by the runtime, never arbitrary filesystem paths.
    pub sources: Vec<String>,
}

/// Version-one semantic payload shared by automatic and manual compaction.
///
/// All fields are required, even when a collection is empty. Generation, raw
/// ranges and parent lineage are runtime metadata and cannot be supplied here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    /// Supported payload format, currently one.
    pub schema_version: u32,
    /// Overview of the retained context.
    pub summary: CheckpointItem,
    /// Current user objective.
    pub goal: CheckpointItem,
    /// User constraints that still apply.
    pub constraints: Vec<CheckpointItem>,
    /// Completed work.
    pub completed: Vec<CheckpointItem>,
    /// Work currently in progress.
    pub in_progress: Vec<CheckpointItem>,
    /// Work still pending.
    pub pending: Vec<CheckpointItem>,
    /// Accepted decisions.
    pub decisions: Vec<CheckpointItem>,
    /// Facts attributed to source evidence; references do not prove truth.
    pub facts: Vec<CheckpointItem>,
    /// Unverified assumptions, kept distinct from facts.
    pub assumptions: Vec<CheckpointItem>,
    /// Referenced resources.
    pub resources: Vec<CheckpointItem>,
    /// Unresolved questions and failed attempts.
    pub open_loops: Vec<CheckpointItem>,
    /// Next concrete step.
    pub next_step: CheckpointItem,
}

/// Content-free checkpoint validation failures suitable for transient outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CheckpointError {
    /// Encoded payload exceeds the bounded parser input.
    #[error("checkpoint payload exceeds byte limit")]
    PayloadLimit,
    /// JSON is malformed, incomplete or contains unrecognized fields.
    #[error("invalid checkpoint schema")]
    Schema,
    /// The payload uses an unsupported version.
    #[error("unsupported checkpoint version")]
    Version,
    /// A required statement is empty or exceeds its byte bound.
    #[error("invalid checkpoint text")]
    Text,
    /// A collection or reference count exceeds its bound.
    #[error("checkpoint collection limit exceeded")]
    CollectionLimit,
    /// A reference is malformed or outside runtime-selected coverage.
    #[error("checkpoint source is outside coverage")]
    Source,
    /// Canonical rendering exceeds its context-relative token budget.
    #[error("checkpoint exceeds rendered token budget")]
    RenderLimit,
    /// The replacement does not reclaim at least ten percent of eligible tokens.
    #[error("checkpoint reclaims insufficient context")]
    InsufficientReclaim,
}

impl Checkpoint {
    /// Parse bounded JSON and validate each reference against runtime coverage.
    ///
    /// `source_is_covered` must recognize only issued identifiers for this fold
    /// and its validated parent. It must not open model-supplied paths.
    pub fn parse(
        payload: &str,
        source_is_covered: impl Fn(&str) -> bool,
    ) -> Result<Self, CheckpointError> {
        if payload.len() > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::PayloadLimit);
        }
        let checkpoint: Self =
            serde_json::from_str(payload).map_err(|_| CheckpointError::Schema)?;
        checkpoint.validate(source_is_covered)?;
        Ok(checkpoint)
    }

    fn collections(&self) -> [&[CheckpointItem]; 9] {
        [
            &self.constraints,
            &self.completed,
            &self.in_progress,
            &self.pending,
            &self.decisions,
            &self.facts,
            &self.assumptions,
            &self.resources,
            &self.open_loops,
        ]
    }

    fn validate(&self, source_is_covered: impl Fn(&str) -> bool) -> Result<(), CheckpointError> {
        if self.schema_version != 1 {
            return Err(CheckpointError::Version);
        }
        let collections = self.collections();
        if collections.iter().any(|items| items.len() > 32) {
            return Err(CheckpointError::CollectionLimit);
        }
        let mut refs = 0usize;
        for item in [&self.summary, &self.goal, &self.next_step]
            .into_iter()
            .chain(collections.into_iter().flatten())
        {
            if item.text.trim().is_empty() || item.text.len() > 4096 {
                return Err(CheckpointError::Text);
            }
            refs += item.sources.len();
            if item.sources.len() > 8 || refs > 256 {
                return Err(CheckpointError::CollectionLimit);
            }
            for source in &item.sources {
                if source.is_empty() || source.len() > 128 || !source_is_covered(source) {
                    return Err(CheckpointError::Source);
                }
            }
        }
        Ok(())
    }

    /// Render validated fields in fixed schema order with JSON escaping.
    ///
    /// JSON keeps embedded newlines/section-like strings inside their fields.
    /// The output remains untrusted model context and must never be a system instruction.
    pub fn render(&self, context_limit: u32) -> Result<String, CheckpointError> {
        // Recheck structural bounds because callers can construct public fields.
        self.validate(|_| true)?;
        let rendered = serde_json::to_string(self).map_err(|_| CheckpointError::Schema)?;
        if rendered.len() > MAX_CHECKPOINT_BYTES {
            return Err(CheckpointError::PayloadLimit);
        }
        if TokenEstimator::estimate_text(&rendered) > 4096.min(context_limit / 10) {
            return Err(CheckpointError::RenderLimit);
        }
        Ok(rendered)
    }

    /// Require final-prefix reclamation; intermediate folds must not use this gate.
    /// Full ordinary-request admission is a separate mandatory check at application.
    pub fn check_reclamation(old_tokens: u32, new_tokens: u32) -> Result<(), CheckpointError> {
        if old_tokens == 0 || u64::from(new_tokens) * 10 > u64::from(old_tokens) * 9 {
            return Err(CheckpointError::InsufficientReclaim);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> serde_json::Value {
        let item =
            serde_json::json!({"text": "Keep original constraints", "sources": ["raw:0:0:25"]});
        serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
            "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
            "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
    }

    fn parse(value: &serde_json::Value) -> Result<Checkpoint, CheckpointError> {
        Checkpoint::parse(&value.to_string(), |s| s == "raw:0:0:25")
    }

    #[test]
    fn canonical_render_retains_semantics_and_source_refs() {
        let mut value = payload();
        value["open_loops"] =
            serde_json::json!([{"text":"failed attempt: preserve it", "sources":["raw:0:0:25"]}]);
        let checkpoint = parse(&value).expect("valid checkpoint test fixture");
        let rendered = checkpoint
            .render(100_000)
            .expect("valid checkpoint test fixture");
        assert_eq!(
            Checkpoint::parse(&rendered, |_| true).expect("valid checkpoint test fixture"),
            checkpoint
        );
        assert_eq!(
            checkpoint
                .render(100_000)
                .expect("valid checkpoint test fixture"),
            rendered
        );
        assert!(
            rendered
                .find("summary")
                .expect("valid checkpoint test fixture")
                < rendered
                    .find("goal")
                    .expect("valid checkpoint test fixture")
        );
        assert!(rendered.contains("failed attempt: preserve it"));
    }

    #[test]
    fn rejects_missing_unknown_duplicate_and_unsupported_schema() {
        let mut value = payload();
        value
            .as_object_mut()
            .expect("valid checkpoint test fixture")
            .remove("pending");
        assert_eq!(parse(&value), Err(CheckpointError::Schema));
        let mut value = payload();
        value["raw_end"] = 999.into();
        assert_eq!(parse(&value), Err(CheckpointError::Schema));
        let duplicate = payload()
            .to_string()
            .replacen('{', "{\"schema_version\":1,", 1);
        assert_eq!(
            Checkpoint::parse(&duplicate, |_| true),
            Err(CheckpointError::Schema)
        );
        let mut value = payload();
        value["schema_version"] = 2.into();
        assert_eq!(parse(&value), Err(CheckpointError::Version));
    }

    #[test]
    fn enforces_utf8_byte_bounds_and_covered_sources() {
        let mut value = payload();
        value["summary"]["text"] = "界".repeat(1366).into();
        assert_eq!(parse(&value), Err(CheckpointError::Text));
        value["summary"]["text"] = " \n".into();
        assert_eq!(parse(&value), Err(CheckpointError::Text));
        value = payload();
        value["goal"]["sources"] = serde_json::json!(["raw:999:0:25"]);
        assert_eq!(parse(&value), Err(CheckpointError::Source));
        assert_eq!(
            Checkpoint::parse(&"x".repeat(MAX_CHECKPOINT_BYTES + 1), |_| true),
            Err(CheckpointError::PayloadLimit)
        );
    }

    #[test]
    fn collection_and_reference_limits_are_aggregate() {
        let mut value = payload();
        value["facts"] = vec![value["summary"].clone(); 33].into();
        assert_eq!(parse(&value), Err(CheckpointError::CollectionLimit));
        value = payload();
        value["summary"]["sources"] = vec!["raw:0:0:25"; 9].into();
        assert_eq!(parse(&value), Err(CheckpointError::CollectionLimit));
        value["summary"]["sources"] = vec!["raw:0:0:25"; 8].into();
        value["facts"] = vec![value["summary"].clone(); 32].into();
        assert_eq!(parse(&value), Err(CheckpointError::CollectionLimit));
    }

    #[test]
    fn final_reclamation_and_render_limits_do_not_round_down() {
        assert!(Checkpoint::check_reclamation(100, 90).is_ok());
        assert_eq!(
            Checkpoint::check_reclamation(99, 90),
            Err(CheckpointError::InsufficientReclaim)
        );
        assert_eq!(
            Checkpoint::check_reclamation(0, 0),
            Err(CheckpointError::InsufficientReclaim)
        );
        assert!(Checkpoint::check_reclamation(u32::MAX, u32::MAX / 2).is_ok());
        assert_eq!(
            parse(&payload())
                .expect("valid checkpoint test fixture")
                .render(1),
            Err(CheckpointError::RenderLimit)
        );
    }
}
