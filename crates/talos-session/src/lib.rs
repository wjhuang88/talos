//! Talos session management — JSONL-based session logging with tree-branching support.
//!
//! Sessions are stored as append-only JSONL files, organized by working directory.
//! Each line in a JSONL file is a JSON object representing a [`SessionEntry`] with
//! fields for `id`, `parent_id`, `timestamp`, `role`, `content`, and optional `metadata`.
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
//! entry, enabling tree-structured conversation histories.
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

mod error;
mod jsonl;
mod manager;
pub mod sqlite;
mod store;
pub mod todo;
mod topology;
mod types;

pub use error::SessionError;
pub use manager::{
    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,
};
pub use sqlite::{ForkInfo, IndexError, SearchResult, SessionIndex};
pub use store::{JsonlSessionStore, SessionStore};
pub use todo::{
    CreateTodo, TodoAddDependencyTool, TodoCreateBatchInput, TodoCreateBatchTool, TodoCreateInput,
    TodoCreateTool, TodoDeleteInput, TodoDeleteTool, TodoDependency, TodoDependencyInput,
    TodoError, TodoItem, TodoPriority, TodoQuery, TodoQueryInput, TodoQueryTool,
    TodoRemoveDependencyTool, TodoRepository, TodoStatus, TodoUpdate, TodoUpdateBatchInput,
    TodoUpdateBatchTool, TodoUpdateInput, TodoUpdateStatusInput, TodoUpdateStatusTool,
    TodoUpdateTool, status_icon,
};
pub use types::{Session, SessionBranch, SessionEntry, SessionInfo, SessionMetadata};

#[cfg(test)]
#[allow(warnings)]
mod tests;
