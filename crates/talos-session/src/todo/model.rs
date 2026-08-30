//! Todo domain types and tool input schemas.

use chrono::{DateTime, Utc};
use rusqlite;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while reading or writing session todos.
#[derive(Debug, Error)]
pub enum TodoError {
    /// A database operation failed.
    #[error("todo database error: {0}")]
    Database(String),

    /// JSON metadata could not be serialized or parsed.
    #[error("todo metadata JSON error: {0}")]
    Json(String),

    /// A todo id did not exist in the target session.
    #[error("todo item not found: {0}")]
    NotFound(Uuid),

    /// A dependency would create a cycle.
    #[error("todo dependency would create a cycle: {parent_id} -> {child_id}")]
    DependencyCycle {
        /// Parent todo id from the attempted dependency edge.
        parent_id: Uuid,
        /// Child todo id from the attempted dependency edge.
        child_id: Uuid,
    },

    /// A todo cannot depend on itself.
    #[error("todo item cannot depend on itself: {0}")]
    SelfDependency(Uuid),

    /// The persisted revision cannot advance further.
    #[error("todo revision exhausted for item: {0}")]
    RevisionExhausted(Uuid),

    /// A concurrent writer advanced the canonical revision.
    #[error("todo revision changed concurrently for item: {0}")]
    RevisionConflict(Uuid),

    /// The Todo database schema or stored value cannot be migrated losslessly.
    #[error("unsupported or lossy todo schema: {0}")]
    Migration(String),
}

impl From<rusqlite::Error> for TodoError {
    fn from(err: rusqlite::Error) -> Self {
        TodoError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for TodoError {
    fn from(err: serde_json::Error) -> Self {
        TodoError::Json(err.to_string())
    }
}

/// Status for a session todo item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    /// Not started.
    Todo,
    /// Currently being worked.
    InProgress,
    /// Completed.
    Completed,
    /// Blocked by an external condition.
    Blocked,
}

impl TodoStatus {
    /// Return the stable snake_case representation used in storage and prompts.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TodoStatus::Todo => "todo",
            TodoStatus::InProgress => "in_progress",
            TodoStatus::Completed => "completed",
            TodoStatus::Blocked => "blocked",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "todo" => Some(TodoStatus::Todo),
            "in_progress" => Some(TodoStatus::InProgress),
            "completed" => Some(TodoStatus::Completed),
            "blocked" => Some(TodoStatus::Blocked),
            _ => None,
        }
    }
}

/// Priority for a session todo item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    Medium,
    /// High priority.
    High,
    /// Critical priority.
    Critical,
}

impl TodoPriority {
    /// Return the stable snake_case representation used in storage and prompts.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            TodoPriority::Low => "low",
            TodoPriority::Medium => "medium",
            TodoPriority::High => "high",
            TodoPriority::Critical => "critical",
        }
    }

    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value {
            "low" => Some(TodoPriority::Low),
            "medium" => Some(TodoPriority::Medium),
            "high" => Some(TodoPriority::High),
            "critical" => Some(TodoPriority::Critical),
            _ => None,
        }
    }
}

/// A structured todo item owned by one session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoItem {
    /// Unique todo id.
    pub id: Uuid,
    /// Owning session id.
    pub session_id: Uuid,
    /// Short title.
    pub title: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Current status.
    pub status: TodoStatus,
    /// Planning priority.
    pub priority: TodoPriority,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Completion timestamp, set when status is completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Optional turn id that owns or last selected this item.
    pub assigned_to_turn: Option<String>,
    /// User/model tags for filtering.
    pub tags: Vec<String>,
}

impl TodoItem {
    /// Project this legacy Todo record into the canonical storage-neutral WorkUnit domain value.
    ///
    /// The projection is lossless for the fields currently owned by Todo. Persistence remains
    /// owned by `TodoRepository`; this method does not create a second repository or mutate data.
    #[must_use]
    pub fn as_work_unit(&self, revision: u64) -> talos_core::work::WorkNode {
        use talos_core::work::{WorkPriority, WorkStatus};
        talos_core::work::WorkNode {
            identity: talos_core::work::WorkIdentity {
                id: self.id,
                kind: talos_core::work::WorkKind::WorkUnit,
                revision,
            },
            parent_id: None,
            title: self.title.clone(),
            description: self.description.clone(),
            status: match self.status {
                TodoStatus::Todo => WorkStatus::Todo,
                TodoStatus::InProgress => WorkStatus::InProgress,
                TodoStatus::Completed => WorkStatus::Completed,
                TodoStatus::Blocked => WorkStatus::Blocked,
            },
            priority: match self.priority {
                TodoPriority::Low => WorkPriority::Low,
                TodoPriority::Medium => WorkPriority::Medium,
                TodoPriority::High => WorkPriority::High,
                TodoPriority::Critical => WorkPriority::Critical,
            },
            tags: self.tags.clone(),
        }
    }
}

/// A dependency edge between two todo items in one session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TodoDependency {
    /// Owning session id.
    pub session_id: Uuid,
    /// Parent todo that must be handled before the child.
    pub parent_id: Uuid,
    /// Child todo that depends on the parent.
    pub child_id: Uuid,
}

/// Parameters for creating a todo item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodo {
    /// Owning session id.
    pub session_id: Uuid,
    /// Short title.
    pub title: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Planning priority.
    pub priority: TodoPriority,
    /// Optional turn id assignment.
    pub assigned_to_turn: Option<String>,
    /// Tags for filtering.
    pub tags: Vec<String>,
}

/// Parameters for updating todo item fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoUpdate {
    /// New title.
    pub title: Option<String>,
    /// New description. `Some(None)` clears it.
    pub description: Option<Option<String>>,
    /// New priority.
    pub priority: Option<TodoPriority>,
    /// New turn assignment. `Some(None)` clears it.
    pub assigned_to_turn: Option<Option<String>>,
    /// New complete tag set.
    pub tags: Option<Vec<String>>,
}

/// Filter for querying todos.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoQuery {
    /// Restrict to one status.
    pub status: Option<TodoStatus>,
    /// Restrict to one priority.
    pub priority: Option<TodoPriority>,
    /// Require one tag.
    pub tag: Option<String>,
}

/// Input for the `todo_create` tool.
///
/// `session_id` is intentionally absent: the owning tool resolves it from the
/// active session at construction time so the model never has to track it.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoCreateInput {
    /// Short title.
    pub title: String,
    /// Optional longer description.
    #[serde(default)]
    pub description: Option<String>,
    /// Planning priority. Defaults to medium when omitted.
    #[serde(default = "default_priority")]
    pub priority: TodoPriority,
    /// Optional turn id assignment.
    #[serde(default)]
    pub assigned_to_turn: Option<String>,
    /// Tags for filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Input for the `todo_update_status` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoUpdateStatusInput {
    /// Todo item id.
    pub id: String,
    /// New status.
    pub status: TodoStatus,
}

/// Input for the `todo_update` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoUpdateInput {
    /// Todo item id.
    pub id: String,
    /// New title.
    #[serde(default)]
    pub title: Option<String>,
    /// New description.
    #[serde(default)]
    pub description: Option<String>,
    /// Clear the existing description.
    #[serde(default)]
    pub clear_description: bool,
    /// New priority.
    #[serde(default)]
    pub priority: Option<TodoPriority>,
    /// New turn assignment.
    #[serde(default)]
    pub assigned_to_turn: Option<String>,
    /// Clear the existing turn assignment.
    #[serde(default)]
    pub clear_assigned_to_turn: bool,
    /// Replace tags with this complete set.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Input for the `todo_delete` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoDeleteInput {
    /// Todo item id.
    pub id: String,
}

/// Input for todo dependency mutation tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoDependencyInput {
    /// Parent todo that must be handled before the child.
    pub parent_id: String,
    /// Child todo that depends on the parent.
    pub child_id: String,
}

/// Input for the `todo_query` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoQueryInput {
    /// Restrict to one status.
    #[serde(default)]
    pub status: Option<TodoStatus>,
    /// Restrict to one priority.
    #[serde(default)]
    pub priority: Option<TodoPriority>,
    /// Require one tag.
    #[serde(default)]
    pub tag: Option<String>,
}

fn default_priority() -> TodoPriority {
    TodoPriority::Medium
}
