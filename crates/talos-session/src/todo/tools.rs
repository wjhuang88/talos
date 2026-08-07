//! AgentTool adapters for session-scoped Todo operations.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;
use uuid::Uuid;

use super::formatting::{
    format_created, format_mutation_result, format_query_result, format_updated,
};
use super::model::{
    CreateTodo, TodoCreateInput, TodoDeleteInput, TodoDependencyInput, TodoError, TodoQuery,
    TodoQueryInput, TodoUpdate, TodoUpdateInput, TodoUpdateStatusInput,
};
use super::repository::TodoRepository;

/// Agent tool that creates a session todo item.
#[derive(Debug, Clone)]
pub struct TodoCreateTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoCreateTool {
    /// Create a todo creation tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo creation tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        match repo.create(CreateTodo {
            session_id: self.session_id,
            title: input.title,
            description: input.description,
            priority: input.priority,
            assigned_to_turn: input.assigned_to_turn,
            tags: input.tags,
        }) {
            Ok(item) => {
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&format_created(&item), &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["title", "priority"]
    }
}

/// Input for the `todo_create_batch` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoCreateBatchInput {
    /// Items to create (idempotent per title within the session).
    #[serde(default)]
    pub items: Vec<TodoCreateInput>,
}

/// Agent tool that creates multiple session todo items in one call.
#[derive(Debug, Clone)]
pub struct TodoCreateBatchTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoCreateBatchTool {
    /// Create a batch todo creation tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a batch todo creation tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
    }
}

#[async_trait]
impl AgentTool for TodoCreateBatchTool {
    fn name(&self) -> &str {
        "todo_create_batch"
    }

    fn description(&self) -> &str {
        "Create multiple session-scoped todo items in one call (idempotent per title)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoCreateBatchInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoCreateBatchInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => {
                return ToolResult::error(format!("Invalid todo_create_batch input: {err}"));
            }
        };
        if input.items.is_empty() {
            return ToolResult::error("todo_create_batch requires at least one item");
        }
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let create_inputs: Vec<CreateTodo> = input
            .items
            .into_iter()
            .map(|item| CreateTodo {
                session_id: self.session_id,
                title: item.title,
                description: item.description,
                priority: item.priority,
                assigned_to_turn: item.assigned_to_turn,
                tags: item.tags,
            })
            .collect();
        match repo.create_batch(create_inputs) {
            Ok(items) => {
                let created_count = items.len();
                let action = format!("Created {created_count} todo(s)");
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&action, &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["items"]
    }
}

/// Input for the `todo_update_batch` tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoUpdateBatchInput {
    /// Items to update (each must include `id`; remaining fields are optional).
    #[serde(default)]
    pub items: Vec<TodoUpdateInput>,
}

/// Agent tool that updates multiple session todo items in one call.
#[derive(Debug, Clone)]
pub struct TodoUpdateBatchTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoUpdateBatchTool {
    /// Create a batch todo update tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a batch todo update tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
    }
}

#[async_trait]
impl AgentTool for TodoUpdateBatchTool {
    fn name(&self) -> &str {
        "todo_update_batch"
    }

    fn description(&self) -> &str {
        "Update mutable fields on multiple session-scoped todo items in one call"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TodoUpdateBatchInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let input: TodoUpdateBatchInput = match serde_json::from_value(input) {
            Ok(input) => input,
            Err(err) => {
                return ToolResult::error(format!("Invalid todo_update_batch input: {err}"));
            }
        };
        if input.items.is_empty() {
            return ToolResult::error("todo_update_batch requires at least one item");
        }
        let repo = match open_tool_repo(&self.db_path) {
            Ok(repo) => repo,
            Err(err) => return ToolResult::error(err.to_string()),
        };
        let mut updated_count = 0usize;
        for item in input.items {
            let id = match parse_tool_uuid("id", &item.id) {
                Ok(id) => id,
                Err(err) => return ToolResult::error(err),
            };
            let update = TodoUpdate {
                title: item.title,
                description: if item.clear_description {
                    Some(None)
                } else {
                    item.description.map(Some)
                },
                priority: item.priority,
                assigned_to_turn: if item.clear_assigned_to_turn {
                    Some(None)
                } else {
                    item.assigned_to_turn.map(Some)
                },
                tags: item.tags,
            };
            match repo.update(self.session_id, id, update) {
                Ok(_) => updated_count += 1,
                Err(err) => return ToolResult::error(err.to_string()),
            }
        }
        let action = format!("Updated {updated_count} todo(s)");
        let all = repo.list_all(self.session_id).unwrap_or_default();
        ToolResult::success(format_mutation_result(&action, &all))
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["items"]
    }
}

/// Agent tool that updates a session todo status.
#[derive(Debug, Clone)]
pub struct TodoUpdateStatusTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoUpdateStatusTool {
    /// Create a todo status update tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo status update tool bound to one session, using the
    /// standard database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        let id = match parse_tool_uuid("id", &input.id) {
            Ok(id) => id,
            Err(err) => return ToolResult::error(err),
        };
        match repo.update_status(self.session_id, id, input.status) {
            Ok(item) => {
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&format_updated(&item), &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["id", "status"]
    }
}

/// Agent tool that updates mutable fields on a session todo item.
#[derive(Debug, Clone)]
pub struct TodoUpdateTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoUpdateTool {
    /// Create a todo update tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo update tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        match repo.update(self.session_id, id, update) {
            Ok(item) => {
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&format_updated(&item), &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["id", "title", "priority"]
    }
}

/// Agent tool that deletes a session todo item.
#[derive(Debug, Clone)]
pub struct TodoDeleteTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoDeleteTool {
    /// Create a todo delete tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo delete tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        let id = match parse_tool_uuid("id", &input.id) {
            Ok(id) => id,
            Err(err) => return ToolResult::error(err),
        };
        match repo.delete(self.session_id, id) {
            Ok(deleted) => {
                let action = if deleted {
                    format!("Deleted todo item {id}")
                } else {
                    "Todo item not found (already deleted?)".to_string()
                };
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&action, &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["id"]
    }
}

/// Agent tool that adds a dependency edge between session todo items.
#[derive(Debug, Clone)]
pub struct TodoAddDependencyTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoAddDependencyTool {
    /// Create a todo dependency-add tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo dependency-add tool bound to one session, using the
    /// standard database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        match repo.add_dependency(self.session_id, ids.parent_id, ids.child_id) {
            Ok(_dep) => {
                let action = format!("Added dependency: {} → {}", ids.parent_id, ids.child_id);
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&action, &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["parent_id", "child_id"]
    }
}

/// Agent tool that removes a dependency edge between session todo items.
#[derive(Debug, Clone)]
pub struct TodoRemoveDependencyTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoRemoveDependencyTool {
    /// Create a todo dependency-remove tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo dependency-remove tool bound to one session, using the
    /// standard database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        match repo.remove_dependency(self.session_id, ids.parent_id, ids.child_id) {
            Ok(removed) => {
                let action = if removed {
                    format!("Removed dependency: {} → {}", ids.parent_id, ids.child_id)
                } else {
                    "Dependency edge not found (already removed?)".to_string()
                };
                let all = repo.list_all(self.session_id).unwrap_or_default();
                ToolResult::success(format_mutation_result(&action, &all))
            }
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["parent_id", "child_id"]
    }
}

/// Agent tool that queries session todo items.
#[derive(Debug, Clone)]
pub struct TodoQueryTool {
    db_path: PathBuf,
    session_id: Uuid,
}

impl TodoQueryTool {
    /// Create a todo query tool bound to one session's SQLite database path.
    #[must_use]
    pub fn new(db_path: PathBuf, session_id: Uuid) -> Self {
        Self {
            db_path,
            session_id,
        }
    }

    /// Create a todo query tool bound to one session, using the standard
    /// database path under a sessions directory.
    #[must_use]
    pub fn from_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Self {
        Self::new(sessions_dir.join("todos.sqlite"), session_id)
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
        match repo.list(
            self.session_id,
            TodoQuery {
                status: input.status,
                priority: input.priority,
                tag: input.tag,
            },
        ) {
            Ok(items) => ToolResult::success(format_query_result(&items)),
            Err(err) => ToolResult::error(err.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn permission_profile(&self, _input: &Value) -> Vec<ToolPermissionFacet> {
        vec![todo_permission_facet(self.session_id)]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["status", "priority", "tag"]
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
    parent_id: Uuid,
    child_id: Uuid,
}

fn parse_dependency_input(input: &TodoDependencyInput) -> Result<ParsedDependencyInput, String> {
    Ok(ParsedDependencyInput {
        parent_id: parse_tool_uuid("parent_id", &input.parent_id)?,
        child_id: parse_tool_uuid("child_id", &input.child_id)?,
    })
}

fn todo_permission_facet(session_id: Uuid) -> ToolPermissionFacet {
    ToolPermissionFacet::with_resource(
        ToolNature::Internal,
        format!("session:{session_id}:todos"),
        ToolResourceKind::Remote,
    )
    .with_description("session todo list")
}
