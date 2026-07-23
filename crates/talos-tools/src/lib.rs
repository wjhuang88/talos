//! Built-in agent tools for Talos.
//!
//! This crate provides implementations of the [`AgentTool`] trait for common
//! agent operations such as shell command execution, file operations, and
//! AST-aware symbol queries.

pub mod bash_tool;
pub mod browser_page;
pub mod diff_stat;
pub mod document_extract;
pub mod exec_tool;
pub mod fetch_url;
pub mod file_tools;
pub mod git;
pub mod http_request;
pub mod image_validation;
pub mod read_image_tool;
pub mod save_url;
pub mod search_engine;
pub mod search_tools;
pub mod symbol;
pub mod tree;
pub mod web_search;

pub use bash_tool::{BashError, BashInput, BashTool};
pub use browser_page::{
    BrowserPageConnector, BrowserPageLink, BrowserPageRecord, MockBrowserPageConnector,
};
pub use diff_stat::{DiffInput, DiffTool, StatInput, StatTool};
pub use document_extract::{DocumentExtractError, DocumentExtractInput, DocumentExtractTool};
pub use exec_tool::{ExecError, ExecInput, ExecTool};
pub use fetch_url::{FetchUrlError, FetchUrlInput, FetchUrlTool};
pub use file_tools::is_skip_dir;
pub use file_tools::{
    DeleteError, DeleteInput, DeleteTool, EditInput, EditTool, FileSnapshotRegistry, FileToolError,
    LsInput, LsTool, ReadInput, ReadTool, WriteInput, WriteTool, snapshot_aware_file_tools,
};
pub use git::{GitToolError, git_dirty_count};
pub use http_request::{HttpRequestError, HttpRequestInput, HttpRequestTool};
pub use read_image_tool::ReadImageTool;
pub use save_url::{SaveUrlError, SaveUrlInput, SaveUrlTool};
pub use search_tools::{GlobInput, GlobTool, GrepInput, GrepTool};
pub use tree::TreeTool;
pub use web_search::{WebSearchError, WebSearchInput, WebSearchTool};
