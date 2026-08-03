use serde::{Deserialize, Serialize};

pub(crate) const TURN_TRANSCRIPT_OUTCOME_PREFIX: &str = "__TALOS_TURN_TRANSCRIPT_OUTCOME__:";

/// Durable proof of the terminal transcript outcome for one runtime Turn.
///
/// This marker is appended only after every transcript message for the outcome
/// has been written. Startup recovery must not infer Success from ordinary or
/// partial transcript entries alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnTranscriptOutcome {
    Success,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnTranscriptOutcomeRecord {
    pub version: u8,
    pub turn_id: String,
    pub outcome: TurnTranscriptOutcome,
}

impl TurnTranscriptOutcomeRecord {
    #[must_use]
    pub fn new(turn_id: impl Into<String>, outcome: TurnTranscriptOutcome) -> Self {
        Self {
            version: 1,
            turn_id: turn_id.into(),
            outcome,
        }
    }
}

pub(crate) fn encode_turn_transcript_outcome(
    outcome: &TurnTranscriptOutcomeRecord,
) -> Result<String, serde_json::Error> {
    serde_json::to_string(outcome)
        .map(|encoded| format!("{TURN_TRANSCRIPT_OUTCOME_PREFIX}{encoded}"))
}

pub(crate) fn decode_turn_transcript_outcome(content: &str) -> Option<TurnTranscriptOutcomeRecord> {
    content
        .strip_prefix(TURN_TRANSCRIPT_OUTCOME_PREFIX)
        .and_then(|encoded| serde_json::from_str(encoded).ok())
}

pub(crate) fn is_turn_transcript_outcome_content(content: &str) -> bool {
    content.starts_with(TURN_TRANSCRIPT_OUTCOME_PREFIX)
}
