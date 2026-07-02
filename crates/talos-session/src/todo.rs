//! Session-scoped todo storage for orchestration state.
//!
//! The todo repository is separate from the append-only JSONL transcript. It stores structured,
//! session-owned planning data in SQLite so later TUI views, tools, and prompt integration can share
//! one durable source of truth.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Result as RusqliteResult, params};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;
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

    fn from_str(value: &str) -> Self {
        match value {
            "in_progress" => TodoStatus::InProgress,
            "completed" => TodoStatus::Completed,
            "blocked" => TodoStatus::Blocked,
            _ => TodoStatus::Todo,
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

    fn from_str(value: &str) -> Self {
        match value {
            "low" => TodoPriority::Low,
            "high" => TodoPriority::High,
            "critical" => TodoPriority::Critical,
            _ => TodoPriority::Medium,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoCreateInput {
    /// Owning session id.
    pub session_id: String,
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
    /// Owning session id.
    pub session_id: String,
    /// Todo item id.
    pub id: String,
    /// New status.
    pub status: TodoStatus,
}

/// Input for the `todo_update` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoUpdateInput {
    /// Owning session id.
    pub session_id: String,
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
    /// Owning session id.
    pub session_id: String,
    /// Todo item id.
    pub id: String,
}

/// Input for todo dependency mutation tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoDependencyInput {
    /// Owning session id.
    pub session_id: String,
    /// Parent todo that must be handled before the child.
    pub parent_id: String,
    /// Child todo that depends on the parent.
    pub child_id: String,
}

/// Input for the `todo_query` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoQueryInput {
    /// Owning session id.
    pub session_id: String,
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

/// SQLite repository for session todo state.
#[derive(Debug)]
pub struct TodoRepository {
    conn: Connection,
    db_path: PathBuf,
}

impl TodoRepository {
    /// Open or create a todo database at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error when the database file cannot be opened.
    pub fn new(path: &Path) -> Result<Self, TodoError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| TodoError::Database(err.to_string()))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(Self {
            conn,
            db_path: path.to_path_buf(),
        })
    }

    /// Return the path to the SQLite database.
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Initialize todo tables.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite rejects the schema.
    pub fn init_schema(&self) -> Result<(), TodoError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS todo_items (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                assigned_to_turn TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]'
            );

            CREATE INDEX IF NOT EXISTS idx_todo_items_session_status
                ON todo_items(session_id, status);

            CREATE TABLE IF NOT EXISTS todo_dependencies (
                session_id TEXT NOT NULL,
                parent_id TEXT NOT NULL,
                child_id TEXT NOT NULL,
                PRIMARY KEY (session_id, parent_id, child_id)
            );
            "#,
        )?;
        Ok(())
    }

    /// Create a todo item.
    ///
    /// # Errors
    ///
    /// Returns an error when the item cannot be persisted.
    pub fn create(&self, input: CreateTodo) -> Result<TodoItem, TodoError> {
        let item = TodoItem {
            id: Uuid::new_v4(),
            session_id: input.session_id,
            title: input.title,
            description: input.description,
            status: TodoStatus::Todo,
            priority: input.priority,
            created_at: Utc::now(),
            completed_at: None,
            assigned_to_turn: input.assigned_to_turn,
            tags: normalize_tags(input.tags),
        };
        self.insert_item(&item)?;
        Ok(item)
    }

    /// Get one todo item by id within a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn get(&self, session_id: Uuid, id: Uuid) -> Result<Option<TodoItem>, TodoError> {
        self.conn
            .query_row(
                r#"
                SELECT id, session_id, title, description, status, priority, created_at,
                       completed_at, assigned_to_turn, tags_json
                FROM todo_items
                WHERE session_id = ?1 AND id = ?2
                "#,
                params![session_id.to_string(), id.to_string()],
                map_todo_item,
            )
            .optional()
            .map_err(TodoError::from)
    }

    /// List todo items for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails or stored metadata cannot be parsed.
    pub fn list(&self, session_id: Uuid, query: TodoQuery) -> Result<Vec<TodoItem>, TodoError> {
        let mut items = self.list_all(session_id)?;
        if let Some(status) = query.status {
            items.retain(|item| item.status == status);
        }
        if let Some(priority) = query.priority {
            items.retain(|item| item.priority == priority);
        }
        if let Some(tag) = query.tag {
            items.retain(|item| item.tags.iter().any(|candidate| candidate == &tag));
        }
        Ok(items)
    }

    /// Update mutable todo fields.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::NotFound`] when the item does not exist in the session.
    pub fn update(
        &self,
        session_id: Uuid,
        id: Uuid,
        update: TodoUpdate,
    ) -> Result<TodoItem, TodoError> {
        let mut item = self.get(session_id, id)?.ok_or(TodoError::NotFound(id))?;
        if let Some(title) = update.title {
            item.title = title;
        }
        if let Some(description) = update.description {
            item.description = description;
        }
        if let Some(priority) = update.priority {
            item.priority = priority;
        }
        if let Some(assigned_to_turn) = update.assigned_to_turn {
            item.assigned_to_turn = assigned_to_turn;
        }
        if let Some(tags) = update.tags {
            item.tags = normalize_tags(tags);
        }
        self.replace_item(&item)?;
        Ok(item)
    }

    /// Update item status and maintain `completed_at`.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::NotFound`] when the item does not exist in the session.
    pub fn update_status(
        &self,
        session_id: Uuid,
        id: Uuid,
        status: TodoStatus,
    ) -> Result<TodoItem, TodoError> {
        let mut item = self.get(session_id, id)?.ok_or(TodoError::NotFound(id))?;
        item.status = status;
        item.completed_at = if status == TodoStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };
        self.replace_item(&item)?;
        Ok(item)
    }

    /// Delete an item and any dependency edges that reference it.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn delete(&mut self, session_id: Uuid, id: Uuid) -> Result<bool, TodoError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND (parent_id = ?2 OR child_id = ?2)",
            params![session_id.to_string(), id.to_string()],
        )?;
        let deleted = tx.execute(
            "DELETE FROM todo_items WHERE session_id = ?1 AND id = ?2",
            params![session_id.to_string(), id.to_string()],
        )?;
        tx.commit()?;
        Ok(deleted > 0)
    }

    /// Add a dependency edge after validating item existence and acyclicity.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::DependencyCycle`] if adding the edge would create a cycle.
    pub fn add_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<TodoDependency, TodoError> {
        if parent_id == child_id {
            return Err(TodoError::SelfDependency(parent_id));
        }
        self.require_item(session_id, parent_id)?;
        self.require_item(session_id, child_id)?;
        if self.path_exists(session_id, child_id, parent_id)? {
            return Err(TodoError::DependencyCycle {
                parent_id,
                child_id,
            });
        }
        self.conn.execute(
            r#"
            INSERT OR IGNORE INTO todo_dependencies (session_id, parent_id, child_id)
            VALUES (?1, ?2, ?3)
            "#,
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
            ],
        )?;
        Ok(TodoDependency {
            session_id,
            parent_id,
            child_id,
        })
    }

    /// Remove a dependency edge.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn remove_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<bool, TodoError> {
        let deleted = self.conn.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND parent_id = ?2 AND child_id = ?3",
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
            ],
        )?;
        Ok(deleted > 0)
    }

    /// List all dependency edges for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn list_dependencies(&self, session_id: Uuid) -> Result<Vec<TodoDependency>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, parent_id, child_id FROM todo_dependencies WHERE session_id = ?1",
        )?;
        let deps = stmt
            .query_map(params![session_id.to_string()], |row| {
                Ok(TodoDependency {
                    session_id: parse_uuid_column(row.get::<_, String>(0)?, 0)?,
                    parent_id: parse_uuid_column(row.get::<_, String>(1)?, 1)?,
                    child_id: parse_uuid_column(row.get::<_, String>(2)?, 2)?,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(deps)
    }

    fn insert_item(&self, item: &TodoItem) -> Result<(), TodoError> {
        self.conn.execute(
            r#"
            INSERT INTO todo_items (
                id, session_id, title, description, status, priority, created_at, completed_at,
                assigned_to_turn, tags_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params_for_item(item)?,
        )?;
        Ok(())
    }

    fn replace_item(&self, item: &TodoItem) -> Result<(), TodoError> {
        self.conn.execute(
            r#"
            UPDATE todo_items
            SET title = ?3,
                description = ?4,
                status = ?5,
                priority = ?6,
                created_at = ?7,
                completed_at = ?8,
                assigned_to_turn = ?9,
                tags_json = ?10
            WHERE id = ?1 AND session_id = ?2
            "#,
            params_for_item(item)?,
        )?;
        Ok(())
    }

    fn list_all(&self, session_id: Uuid) -> Result<Vec<TodoItem>, TodoError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, title, description, status, priority, created_at,
                   completed_at, assigned_to_turn, tags_json
            FROM todo_items
            WHERE session_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let items = stmt
            .query_map(params![session_id.to_string()], map_todo_item)?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(items)
    }

    fn require_item(&self, session_id: Uuid, id: Uuid) -> Result<(), TodoError> {
        if self.get(session_id, id)?.is_some() {
            Ok(())
        } else {
            Err(TodoError::NotFound(id))
        }
    }

    fn path_exists(&self, session_id: Uuid, from: Uuid, to: Uuid) -> Result<bool, TodoError> {
        let deps = self.list_dependencies(session_id)?;
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for dep in deps {
            graph.entry(dep.parent_id).or_default().push(dep.child_id);
        }
        let mut stack = vec![from];
        let mut seen = HashSet::new();
        while let Some(node) = stack.pop() {
            if node == to {
                return Ok(true);
            }
            if !seen.insert(node) {
                continue;
            }
            if let Some(children) = graph.get(&node) {
                stack.extend(children.iter().copied());
            }
        }
        Ok(false)
    }
}

/// Agent tool that creates a session todo item.
#[derive(Debug, Clone)]
pub struct TodoCreateTool {
    db_path: PathBuf,
}

impl TodoCreateTool {
    /// Create a todo creation tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo creation tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoCreateTool {
    fn name(&self) -> &str {
        "todo_create"
    }

    fn description(&self) -> &str {
        "Create a session-scoped todo item for agent planning"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoCreateInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoCreateInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return ToolResult::error(format!("Invalid todo_create input: {err}")),
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let session_id = match parse_tool_uuid("session_id", &input.session_id) {
            Ok(session_id) => session_id,
            Err(err) => return ToolResult::error(err),
        };
        let created = match repo.create(CreateTodo {
            session_id,
            title: input.title,
            description: input.description,
            priority: input.priority,
            assigned_to_turn: input.assigned_to_turn,
            tags: input.tags,
        }) {
            Ok(item) => item,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        json_tool_result(&created)
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "title", "priority"]
    }
}

/// Agent tool that updates a session todo status.
#[derive(Debug, Clone)]
pub struct TodoUpdateStatusTool {
    db_path: PathBuf,
}

impl TodoUpdateStatusTool {
    /// Create a todo status update tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo status update tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoUpdateStatusTool {
    fn name(&self) -> &str {
        "todo_update_status"
    }

    fn description(&self) -> &str {
        "Update the status of a session-scoped todo item"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoUpdateStatusInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoUpdateStatusInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => {
                return ToolResult::error(format!("Invalid todo_update_status input: {err}"));
            }
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let session_id = match parse_tool_uuid("session_id", &input.session_id) {
            Ok(session_id) => session_id,
            Err(err) => return ToolResult::error(err),
        };
        let id = match parse_tool_uuid("id", &input.id) {
            Ok(id) => id,
            Err(err) => return ToolResult::error(err),
        };
        match repo.update_status(session_id, id, input.status) {
            Ok(item) => json_tool_result(&item),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "id", "status"]
    }
}

/// Agent tool that updates mutable fields on a session todo item.
#[derive(Debug, Clone)]
pub struct TodoUpdateTool {
    db_path: PathBuf,
}

impl TodoUpdateTool {
    /// Create a todo update tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo update tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoUpdateTool {
    fn name(&self) -> &str {
        "todo_update"
    }

    fn description(&self) -> &str {
        "Update mutable fields on a session-scoped todo item"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoUpdateInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoUpdateInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return ToolResult::error(format!("Invalid todo_update input: {err}")),
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let session_id = match parse_tool_uuid("session_id", &input.session_id) {
            Ok(session_id) => session_id,
            Err(err) => return ToolResult::error(err),
        };
        let id = match parse_tool_uuid("id", &input.id) {
            Ok(id) => id,
            Err(err) => return ToolResult::error(err),
        };
        let update = TodoUpdate {
            title: input.title,
            description: if input.clear_description {
                Some(None)
            } else {
                input.description.map(Some)
            },
            priority: input.priority,
            assigned_to_turn: if input.clear_assigned_to_turn {
                Some(None)
            } else {
                input.assigned_to_turn.map(Some)
            },
            tags: input.tags,
        };
        match repo.update(session_id, id, update) {
            Ok(item) => json_tool_result(&item),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "id", "title", "priority"]
    }
}

/// Agent tool that deletes a session todo item.
#[derive(Debug, Clone)]
pub struct TodoDeleteTool {
    db_path: PathBuf,
}

impl TodoDeleteTool {
    /// Create a todo delete tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo delete tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoDeleteTool {
    fn name(&self) -> &str {
        "todo_delete"
    }

    fn description(&self) -> &str {
        "Delete a session-scoped todo item and its dependency edges"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoDeleteInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoDeleteInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return ToolResult::error(format!("Invalid todo_delete input: {err}")),
        };
        let mut repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let session_id = match parse_tool_uuid("session_id", &input.session_id) {
            Ok(session_id) => session_id,
            Err(err) => return ToolResult::error(err),
        };
        let id = match parse_tool_uuid("id", &input.id) {
            Ok(id) => id,
            Err(err) => return ToolResult::error(err),
        };
        match repo.delete(session_id, id) {
            Ok(deleted) => json_tool_result(&serde_json::json!({ "deleted": deleted })),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "id"]
    }
}

/// Agent tool that adds a dependency edge between session todo items.
#[derive(Debug, Clone)]
pub struct TodoAddDependencyTool {
    db_path: PathBuf,
}

impl TodoAddDependencyTool {
    /// Create a todo dependency-add tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo dependency-add tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoAddDependencyTool {
    fn name(&self) -> &str {
        "todo_add_dependency"
    }

    fn description(&self) -> &str {
        "Add an acyclic dependency edge between two session-scoped todo items"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoDependencyInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoDependencyInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => {
                return ToolResult::error(format!("Invalid todo_add_dependency input: {err}"));
            }
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let ids = match parse_dependency_input(&input) {
            Ok(ids) => ids,
            Err(err) => return ToolResult::error(err),
        };
        match repo.add_dependency(ids.session_id, ids.parent_id, ids.child_id) {
            Ok(dep) => json_tool_result(&dep),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "parent_id", "child_id"]
    }
}

/// Agent tool that removes a dependency edge between session todo items.
#[derive(Debug, Clone)]
pub struct TodoRemoveDependencyTool {
    db_path: PathBuf,
}

impl TodoRemoveDependencyTool {
    /// Create a todo dependency-remove tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo dependency-remove tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoRemoveDependencyTool {
    fn name(&self) -> &str {
        "todo_remove_dependency"
    }

    fn description(&self) -> &str {
        "Remove a dependency edge between two session-scoped todo items"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoDependencyInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoDependencyInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => {
                return ToolResult::error(format!("Invalid todo_remove_dependency input: {err}"));
            }
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let ids = match parse_dependency_input(&input) {
            Ok(ids) => ids,
            Err(err) => return ToolResult::error(err),
        };
        match repo.remove_dependency(ids.session_id, ids.parent_id, ids.child_id) {
            Ok(removed) => json_tool_result(&serde_json::json!({ "removed": removed })),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Write, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "parent_id", "child_id"]
    }
}

/// Agent tool that queries session todo items.
#[derive(Debug, Clone)]
pub struct TodoQueryTool {
    db_path: PathBuf,
}

impl TodoQueryTool {
    /// Create a todo query tool backed by a SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Create a todo query tool using the standard database under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"))
    }
}

#[async_trait]
impl AgentTool for TodoQueryTool {
    fn name(&self) -> &str {
        "todo_query"
    }

    fn description(&self) -> &str {
        "Query session-scoped todo items without modifying them"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoQueryInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoQueryInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => return ToolResult::error(format!("Invalid todo_query input: {err}")),
        };
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let session_id = match parse_tool_uuid("session_id", &input.session_id) {
            Ok(session_id) => session_id,
            Err(err) => return ToolResult::error(err),
        };
        match repo.list(
            session_id,
            TodoQuery {
                status: input.status,
                priority: input.priority,
                tag: input.tag,
            },
        ) {
            Ok(items) => json_tool_result(&items),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(ToolNature::Read, input)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["session_id", "status", "priority", "tag"]
    }
}

fn open_tool_repo(db_path: &Path) -> Result<TodoRepository, TodoError> {
    let repo = TodoRepository::new(db_path)?;
    repo.init_schema()?;
    Ok(repo)
}

fn parse_tool_uuid(field: &str, value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|err| format!("Invalid {field} UUID: {err}"))
}

struct ParsedDependencyInput {
    session_id: Uuid,
    parent_id: Uuid,
    child_id: Uuid,
}

fn parse_dependency_input(input: &TodoDependencyInput) -> Result<ParsedDependencyInput, String> {
    Ok(ParsedDependencyInput {
        session_id: parse_tool_uuid("session_id", &input.session_id)?,
        parent_id: parse_tool_uuid("parent_id", &input.parent_id)?,
        child_id: parse_tool_uuid("child_id", &input.child_id)?,
    })
}

fn json_tool_result(value: &impl Serialize) -> ToolResult {
    match serde_json::to_string_pretty(value) {
        Ok(json) => ToolResult::success(json),
        Err(err) => ToolResult::error(format!("Failed to serialize todo result: {err}")),
    }
}

fn todo_permission_facet(nature: ToolNature, input: &Value) -> ToolPermissionFacet {
    let session = input
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    ToolPermissionFacet::with_resource(
        nature,
        format!("session:{session}:todos"),
        ToolResourceKind::Remote,
    )
    .with_description("session todo list")
}

fn params_for_item(item: &TodoItem) -> Result<[String; 10], TodoError> {
    Ok([
        item.id.to_string(),
        item.session_id.to_string(),
        item.title.clone(),
        item.description.clone().unwrap_or_default(),
        item.status.as_str().to_string(),
        item.priority.as_str().to_string(),
        item.created_at.to_rfc3339(),
        item.completed_at
            .map(|completed_at| completed_at.to_rfc3339())
            .unwrap_or_default(),
        item.assigned_to_turn.clone().unwrap_or_default(),
        serde_json::to_string(&item.tags)?,
    ])
}

fn map_todo_item(row: &rusqlite::Row<'_>) -> RusqliteResult<TodoItem> {
    let id = parse_uuid_column(row.get::<_, String>(0)?, 0)?;
    let session_id = parse_uuid_column(row.get::<_, String>(1)?, 1)?;
    let created_at = parse_datetime_column(row.get::<_, String>(6)?, 6)?;
    let completed_at = match row.get::<_, String>(7)?.as_str() {
        "" => None,
        value => Some(parse_datetime_column(value.to_string(), 7)?),
    };
    let tags_json: String = row.get(9)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).map_err(|_| {
        rusqlite::Error::InvalidColumnType(9, tags_json, rusqlite::types::Type::Text)
    })?;
    let description = empty_string_to_none(row.get::<_, String>(3)?);
    let assigned_to_turn = empty_string_to_none(row.get::<_, String>(8)?);

    Ok(TodoItem {
        id,
        session_id,
        title: row.get(2)?,
        description,
        status: TodoStatus::from_str(&row.get::<_, String>(4)?),
        priority: TodoPriority::from_str(&row.get::<_, String>(5)?),
        created_at,
        completed_at,
        assigned_to_turn,
        tags,
    })
}

fn parse_uuid_column(value: String, column: usize) -> RusqliteResult<Uuid> {
    Uuid::parse_str(&value)
        .map_err(|_| rusqlite::Error::InvalidColumnType(column, value, rusqlite::types::Type::Text))
}

fn parse_datetime_column(value: String, column: usize) -> RusqliteResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| rusqlite::Error::InvalidColumnType(column, value, rusqlite::types::Type::Text))
}

fn empty_string_to_none(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut tags: Vec<String> = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::AgentTool;
    use tempfile::tempdir;

    fn repo() -> TodoRepository {
        let dir = tempdir().expect("temp dir");
        let repo = TodoRepository::new(&dir.path().join("todos.sqlite")).expect("repo");
        repo.init_schema().expect("schema");
        repo
    }

    fn create(repo: &TodoRepository, session_id: Uuid, title: &str) -> TodoItem {
        repo.create(CreateTodo {
            session_id,
            title: title.to_string(),
            description: None,
            priority: TodoPriority::Medium,
            assigned_to_turn: None,
            tags: vec![],
        })
        .expect("create todo")
    }

    #[test]
    fn create_and_get_round_trips_item() {
        let repo = repo();
        let session_id = Uuid::new_v4();

        let item = repo
            .create(CreateTodo {
                session_id,
                title: "Implement repository".to_string(),
                description: Some("SQLite CRUD".to_string()),
                priority: TodoPriority::High,
                assigned_to_turn: Some("turn-1".to_string()),
                tags: vec!["session".to_string(), " session ".to_string()],
            })
            .expect("create");

        let loaded = repo
            .get(session_id, item.id)
            .expect("get")
            .expect("item exists");
        assert_eq!(loaded.title, "Implement repository");
        assert_eq!(loaded.description.as_deref(), Some("SQLite CRUD"));
        assert_eq!(loaded.priority, TodoPriority::High);
        assert_eq!(loaded.assigned_to_turn.as_deref(), Some("turn-1"));
        assert_eq!(loaded.tags, vec!["session"]);
    }

    #[test]
    fn list_filters_by_status_priority_and_tag() {
        let repo = repo();
        let session_id = Uuid::new_v4();
        let other_session = Uuid::new_v4();
        let first = create(&repo, session_id, "first");
        let second = repo
            .create(CreateTodo {
                session_id,
                title: "second".to_string(),
                description: None,
                priority: TodoPriority::Critical,
                assigned_to_turn: None,
                tags: vec!["release".to_string()],
            })
            .expect("create second");
        create(&repo, other_session, "other");
        repo.update_status(session_id, first.id, TodoStatus::Completed)
            .expect("status");

        let results = repo
            .list(
                session_id,
                TodoQuery {
                    status: Some(TodoStatus::Todo),
                    priority: Some(TodoPriority::Critical),
                    tag: Some("release".to_string()),
                },
            )
            .expect("list");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, second.id);
    }

    #[test]
    fn update_status_sets_and_clears_completed_at() {
        let repo = repo();
        let session_id = Uuid::new_v4();
        let item = create(&repo, session_id, "done");

        let completed = repo
            .update_status(session_id, item.id, TodoStatus::Completed)
            .expect("complete");
        assert!(completed.completed_at.is_some());

        let reopened = repo
            .update_status(session_id, item.id, TodoStatus::InProgress)
            .expect("reopen");
        assert!(reopened.completed_at.is_none());
    }

    #[test]
    fn update_changes_optional_fields() {
        let repo = repo();
        let session_id = Uuid::new_v4();
        let item = create(&repo, session_id, "old");

        let updated = repo
            .update(
                session_id,
                item.id,
                TodoUpdate {
                    title: Some("new".to_string()),
                    description: Some(Some("details".to_string())),
                    priority: Some(TodoPriority::Low),
                    assigned_to_turn: Some(Some("turn-2".to_string())),
                    tags: Some(vec!["b".to_string(), "a".to_string(), "b".to_string()]),
                },
            )
            .expect("update");

        assert_eq!(updated.title, "new");
        assert_eq!(updated.description.as_deref(), Some("details"));
        assert_eq!(updated.priority, TodoPriority::Low);
        assert_eq!(updated.assigned_to_turn.as_deref(), Some("turn-2"));
        assert_eq!(updated.tags, vec!["a", "b"]);
    }

    #[test]
    fn delete_removes_item_and_dependency_edges() {
        let mut repo = repo();
        let session_id = Uuid::new_v4();
        let parent = create(&repo, session_id, "parent");
        let child = create(&repo, session_id, "child");
        repo.add_dependency(session_id, parent.id, child.id)
            .expect("dependency");

        assert!(repo.delete(session_id, parent.id).expect("delete"));
        assert!(
            repo.list_dependencies(session_id)
                .expect("dependencies")
                .is_empty()
        );
    }

    #[test]
    fn dependency_cycle_is_rejected() {
        let repo = repo();
        let session_id = Uuid::new_v4();
        let first = create(&repo, session_id, "first");
        let second = create(&repo, session_id, "second");
        let third = create(&repo, session_id, "third");

        repo.add_dependency(session_id, first.id, second.id)
            .expect("first edge");
        repo.add_dependency(session_id, second.id, third.id)
            .expect("second edge");

        let err = repo
            .add_dependency(session_id, third.id, first.id)
            .expect_err("cycle");
        assert!(matches!(err, TodoError::DependencyCycle { .. }));
    }

    #[test]
    fn dependency_requires_items_in_same_session() {
        let repo = repo();
        let session_id = Uuid::new_v4();
        let other_session = Uuid::new_v4();
        let parent = create(&repo, session_id, "parent");
        let child = create(&repo, other_session, "child");

        let err = repo
            .add_dependency(session_id, parent.id, child.id)
            .expect_err("missing child");
        assert!(matches!(err, TodoError::NotFound(id) if id == child.id));
    }

    #[test]
    fn session_manager_opens_initialized_todo_repository() {
        let dir = tempdir().expect("temp dir");
        let manager = crate::SessionManager::with_dir(dir.path().to_path_buf());
        let repo = manager.todo_repository().expect("todo repository");
        let session_id = Uuid::new_v4();

        let item = create(&repo, session_id, "manager");

        assert_eq!(repo.db_path(), &dir.path().join("todos.sqlite"));
        assert!(repo.get(session_id, item.id).expect("get").is_some());
    }

    #[tokio::test]
    async fn todo_tools_create_query_and_update_status() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("todos.sqlite");
        let create_tool = TodoCreateTool::new(db_path.clone());
        let query_tool = TodoQueryTool::new(db_path.clone());
        let update_fields_tool = TodoUpdateTool::new(db_path.clone());
        let add_dep_tool = TodoAddDependencyTool::new(db_path.clone());
        let remove_dep_tool = TodoRemoveDependencyTool::new(db_path.clone());
        let delete_tool = TodoDeleteTool::new(db_path.clone());
        let update_tool = TodoUpdateStatusTool::new(db_path);
        let session_id = Uuid::new_v4();

        let created = create_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "title": "tool item",
                "priority": "high",
                "tags": ["tool"]
            }))
            .await;
        assert!(!created.is_error, "{}", created.content);
        let item: TodoItem = serde_json::from_str(&created.content).expect("created item");

        let queried = query_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "tag": "tool"
            }))
            .await;
        assert!(!queried.is_error, "{}", queried.content);
        let items: Vec<TodoItem> = serde_json::from_str(&queried.content).expect("query items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item.id);

        let updated = update_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "id": item.id.to_string(),
                "status": "completed"
            }))
            .await;
        assert!(!updated.is_error, "{}", updated.content);
        let item: TodoItem = serde_json::from_str(&updated.content).expect("updated item");
        assert_eq!(item.status, TodoStatus::Completed);
        assert!(item.completed_at.is_some());

        let field_updated = update_fields_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "id": item.id.to_string(),
                "title": "renamed",
                "clear_description": true,
                "priority": "critical",
                "tags": ["next"]
            }))
            .await;
        assert!(!field_updated.is_error, "{}", field_updated.content);
        let item: TodoItem = serde_json::from_str(&field_updated.content).expect("field item");
        assert_eq!(item.title, "renamed");
        assert_eq!(item.priority, TodoPriority::Critical);
        assert_eq!(item.tags, vec!["next"]);

        let child = create_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "title": "child"
            }))
            .await;
        assert!(!child.is_error, "{}", child.content);
        let child: TodoItem = serde_json::from_str(&child.content).expect("child item");

        let dep = add_dep_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "parent_id": item.id.to_string(),
                "child_id": child.id.to_string()
            }))
            .await;
        assert!(!dep.is_error, "{}", dep.content);

        let cycle = add_dep_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "parent_id": child.id.to_string(),
                "child_id": item.id.to_string()
            }))
            .await;
        assert!(cycle.is_error);
        assert!(cycle.content.contains("cycle"));

        let removed = remove_dep_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "parent_id": item.id.to_string(),
                "child_id": child.id.to_string()
            }))
            .await;
        assert!(!removed.is_error, "{}", removed.content);
        assert!(removed.content.contains("\"removed\": true"));

        let deleted = delete_tool
            .execute(serde_json::json!({
                "session_id": session_id.to_string(),
                "id": child.id.to_string()
            }))
            .await;
        assert!(!deleted.is_error, "{}", deleted.content);
        assert!(deleted.content.contains("\"deleted\": true"));
    }

    #[test]
    fn todo_tools_expose_permission_profiles() {
        let dir = tempdir().expect("temp dir");
        let session_id = Uuid::new_v4();
        let create_tool = TodoCreateTool::from_sessions_dir(dir.path());
        let query_tool = TodoQueryTool::from_sessions_dir(dir.path());

        let write_profile =
            create_tool.permission_profile(&serde_json::json!({ "session_id": session_id }));
        let read_profile =
            query_tool.permission_profile(&serde_json::json!({ "session_id": session_id }));

        assert_eq!(write_profile[0].nature, ToolNature::Write);
        assert_eq!(read_profile[0].nature, ToolNature::Read);
        let expected = format!("session:{session_id}:todos");
        assert_eq!(
            write_profile[0].resource.as_deref(),
            Some(expected.as_str())
        );
    }
}
