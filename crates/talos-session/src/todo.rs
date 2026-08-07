//! Session-scoped todo storage for orchestration state.
//!
//! The todo repository is separate from the append-only JSONL transcript. It stores structured,
//! session-owned planning data in SQLite so later TUI views, tools, and prompt integration can share
//! one durable source of truth.

mod formatting;
mod model;
mod repository;
mod tools;

pub use formatting::status_icon;
pub use model::{
    CreateTodo, TodoCreateInput, TodoDeleteInput, TodoDependency, TodoDependencyInput, TodoError,
    TodoItem, TodoPriority, TodoQuery, TodoQueryInput, TodoStatus, TodoUpdate, TodoUpdateInput,
    TodoUpdateStatusInput,
};
pub use repository::TodoRepository;
pub use tools::{
    TodoAddDependencyTool, TodoCreateBatchInput, TodoCreateBatchTool, TodoCreateTool,
    TodoDeleteTool, TodoQueryTool, TodoRemoveDependencyTool, TodoUpdateBatchInput,
    TodoUpdateBatchTool, TodoUpdateStatusTool, TodoUpdateTool,
};

#[cfg(test)]
mod tests;
