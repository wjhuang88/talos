//! Built-in agent tools for Talos.
//!
//! This crate provides implementations of the [`AgentTool`] trait for common
//! agent operations such as shell command execution, file operations, and
//! AST-aware symbol queries.

pub mod bash_tool;
pub mod diff_stat;
pub mod file_tools;
pub mod git;
pub mod http_request;
pub mod search_tools;
pub mod symbol;
pub mod tree;
pub mod web_search;

pub use bash_tool::{BashError, BashInput, BashTool};
pub use diff_stat::{DiffInput, DiffTool, StatInput, StatTool};
pub use file_tools::is_skip_dir;
pub use file_tools::{
    DeleteError, DeleteInput, DeleteTool, EditInput, EditTool, FileToolError, LsInput, LsTool,
    ReadInput, ReadTool, WriteInput, WriteTool,
};
pub use http_request::{HttpRequestError, HttpRequestInput, HttpRequestTool};
pub use search_tools::{GlobInput, GlobTool, GrepInput, GrepTool};
pub use tree::TreeTool;
pub use web_search::{WebSearchError, WebSearchInput, WebSearchTool};
