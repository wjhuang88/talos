//! Structured checkpoints and request-pressure policy for session compaction.
//!
//! Live automatic/manual compaction uses the Agent's shared request pipeline,
//! which preserves raw history and validates budgets, privacy and source lineage.
//! `Compactor` retains only explicitly selected legacy deterministic utilities;
//! its deprecated model-assisted entry points refuse missing request authority.

mod checkpoint;
mod constants;
mod engine;
mod operations;
mod policy;
mod source;
mod summary_stream;
mod types;

#[cfg(test)]
#[allow(warnings)]
mod tests;

pub use checkpoint::{Checkpoint, CheckpointError, CheckpointItem, MAX_CHECKPOINT_BYTES};
pub use engine::Compactor;
pub use operations::{CompactAdmission, CompactOperations, CompactOutcome};
pub use policy::{CompactionPolicy, RequestPressure};
pub use source::HistorySource;
pub(crate) use summary_stream::read_summary_stream;
pub use types::{CompactionError, CompactionResult, CompactionStatus};
