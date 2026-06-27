//! 5-layer context compaction for agent sessions.
//!
//! When the conversation context approaches the model's token limit, this module
//! applies progressive compaction layers to reduce context size while preserving
//! recent conversation fidelity.
//!
//! # Compaction Layers
//!
//! Layers are applied in order, stopping as soon as the context fits:
//!
//! 1. **Budget** — Cap individual tool results to 4000 characters, truncating
//!    with `"... [truncated]"`.
//! 2. **Trim** — Remove tool results from turns older than 20.
//! 3. **Microcompact** — For each tool call ID, keep only the last result.
//! 4. **Collapse** — Summarize turns older than 10 into a single summary message
//!    using the LLM.
//! 5. **Autocompact** — Use the LLM to summarize the entire conversation history.
//!
//! # Preservation Guarantee
//!
//! The last 10 turns are **never** compacted by layers 4 or 5. They are always
//! preserved verbatim to maintain conversation continuity.
//!
//! # Circuit Breaker
//!
//! If compaction fails 3 consecutive times, the circuit breaker trips and
//! subsequent compaction attempts return [`CompactionError::CircuitBreakerTripped`]
//! immediately.

mod constants;
mod engine;
mod policy;
mod types;

#[cfg(test)]
#[allow(warnings)]
mod tests;

pub use engine::Compactor;
pub use policy::CompactionPolicy;
pub use types::{CompactionError, CompactionResult, CompactionStatus};
