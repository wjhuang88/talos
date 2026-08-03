//! Talos session management — append-only session logging with tree-branching support.
//!
//! Sessions are stored as append-only files, organized by working directory.
//! New sessions use the compact text `.tlog` format by default; existing `.jsonl`
//! files are read transparently for backward compatibility. Each line in a session
//! file represents a [`SessionEntry`] with fields for `id`, `parent_id`,
//! `timestamp`, `role`, `content`, and optional `metadata`.
//!
//! # Directory Layout
//!
//! ```text
//! ~/.talos/sessions/
//!   <project>/
//!     <uuid>.jsonl
//! ```
//!
//! # Branching Model
//!
//! Each session supports multiple branches. A branch is a linear sequence of entries
//! rooted at a specific entry. The `fork` method creates a new branch from any existing
//! entry, enabling tree-structured conversations.
//!
//! # Crash Safety
//!
//! JSONL is append-only. If a crash occurs, only the last line may be corrupted,
//! which can be detected and skipped during reads.
//!
//! # Backward Compatibility
//!
//! Entries without `id` or `parent_id` fields (from older JSONL files) are treated
//! as part of a single linear branch. They are assigned synthetic IDs on load.

mod compact_text;
pub mod compaction_engine;
mod compression;
mod diagnostic;
mod durable;
mod durable_recovery;
mod error;
mod jsonl;
mod manager;
mod pending_submission;
mod segment_chain;
pub mod sqlite;
mod store;
pub mod todo;
mod tool_compression;
mod tool_contributions;
mod topology;
mod transcript;
pub use tool_compression::{ToolOutputCompression, compress_tool_output};
mod turn_outcome;
mod types;

pub use diagnostic::{ProviderTerminalDiagnostic, ProviderTerminalOutcome, ProviderTerminalSource};
pub use durable::{
    DurableSession, DurableTranscriptEntry, PersistencePolicy, SessionCapabilities, TurnCommit,
};
pub use error::SessionError;
pub use manager::{
    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,
};
pub use pending_submission::{
    PendingSubmissionError, PendingSubmissionRecord, PendingSubmissionStore,
};
pub use sqlite::{ForkInfo, IndexError, SearchResult, SessionIndex};
pub use store::{CompactTextSessionStore, JsonlSessionStore, SessionStore};
pub use todo::{
    CreateTodo, TodoAddDependencyTool, TodoCreateBatchInput, TodoCreateBatchTool, TodoCreateInput,
    TodoCreateTool, TodoDeleteInput, TodoDeleteTool, TodoDependency, TodoDependencyInput,
    TodoError, TodoItem, TodoPriority, TodoQuery, TodoQueryInput, TodoQueryTool,
    TodoRemoveDependencyTool, TodoRepository, TodoStatus, TodoUpdate, TodoUpdateBatchInput,
    TodoUpdateBatchTool, TodoUpdateInput, TodoUpdateStatusInput, TodoUpdateStatusTool,
    TodoUpdateTool, status_icon,
};
pub use tool_contributions::todo_tool_contributions_for_sessions_dir;
pub use transcript::{TranscriptEntry, export_json, export_markdown, read_transcript};
pub use turn_outcome::{TurnTranscriptOutcome, TurnTranscriptOutcomeRecord};
pub use types::{Session, SessionBranch, SessionEntry, SessionInfo, SessionMetadata};

#[cfg(test)]
#[allow(warnings)]
mod tests;
