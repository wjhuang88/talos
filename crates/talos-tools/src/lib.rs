//! Built-in agent tools for Talos.
//!
//! This crate provides implementations of the [`AgentTool`] trait for common
//! agent operations such as shell command execution, file operations, and
//! AST-aware symbol queries.
//!
//! # Capability features
//!
//! The default surface is local read-only file and search tooling through the
//! `file-read` and `search` features. Mutating files, document extraction,
//! shell processes, Git, network/web, images, and code intelligence are opt-in.
//! Product assemblers can select `coding` to enable the complete Talos tool set.
//!
//! - `file-read` (default): local `read`, `ls`, and `tree` tools.
//! - `search` (default): local `glob` and `grep` tools.
//! - `file-write`: `write`, `edit`, and `delete`; also enables `file-read`.
//! - `document`: local document extraction; also enables `file-read`.
//! - `shell`: `exec` and the platform shell tool.
//! - `git`: Git inspection/mutation plus diff/stat tools.
//! - `network`: fetch, HTTP, browser-page, and web-search tools.
//! - `image`: validated image reading; also enables `file-read`.
//! - `code-intelligence`: symbol tools backed by tree-sitter grammars.
//! - `coding`: all capability features used by the Talos CLI product.
//!
//! Cargo features control which code and dependencies are compiled. They never
//! grant runtime permission: callers must still apply the Talos permission and
//! sandbox policies required by each tool.
//!
//! ## Migration from the broad pre-0.8 default
//!
//! Direct consumers that relied on write, shell, Git, network, image, document,
//! or symbol types being available implicitly must now enable the corresponding
//! features. Use `features = ["coding"]` only when the complete product-oriented
//! tool surface is intended.

#[cfg(feature = "shell")]
pub mod bash_tool;
#[cfg(feature = "network")]
pub mod browser_page;
#[cfg(any(
    feature = "file-write",
    feature = "shell",
    feature = "git",
    feature = "network",
    feature = "image",
    feature = "code-intelligence"
))]
pub mod contributions;
#[cfg(feature = "git")]
pub mod diff_stat;
#[cfg(feature = "document")]
pub mod document_extract;
#[cfg(feature = "shell")]
pub mod exec_tool;
#[cfg(feature = "network")]
pub mod fetch_url;
#[cfg(any(
    feature = "file-read",
    feature = "search",
    feature = "shell",
    feature = "git"
))]
pub mod file_tools;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "network")]
pub mod http_request;
#[cfg(feature = "image")]
pub mod image_validation;
#[cfg(feature = "shell")]
mod process_boundary;
#[cfg(feature = "image")]
pub mod read_image_tool;
#[cfg(all(feature = "network", feature = "file-write"))]
pub mod save_url;
#[cfg(feature = "search")]
pub mod search_engine;
#[cfg(feature = "search")]
pub mod search_tools;
#[cfg(feature = "code-intelligence")]
pub mod symbol;
#[cfg(feature = "file-read")]
pub mod tree;
#[cfg(feature = "network")]
pub mod web_search;

#[cfg(feature = "shell")]
pub use bash_tool::{BashError, BashInput, BashTool};
#[cfg(feature = "network")]
pub use browser_page::{
    BrowserPageConnector, BrowserPageLink, BrowserPageRecord, MockBrowserPageConnector,
};
#[cfg(feature = "network")]
pub use contributions::network_tool_contributions;
#[cfg(feature = "image")]
pub use contributions::read_image_tool_contribution;
#[cfg(feature = "code-intelligence")]
pub use contributions::symbol_tool_contributions;
#[cfg(all(feature = "file-read", feature = "search", feature = "git"))]
pub use contributions::workspace_non_document_tool_contributions;
#[cfg(all(
    feature = "file-read",
    feature = "search",
    feature = "git",
    feature = "document"
))]
pub use contributions::workspace_tool_contributions;
#[cfg(feature = "shell")]
pub use contributions::{bash_tool_contribution, shell_tool_contributions};
#[cfg(feature = "git")]
pub use contributions::{git_mutation_tool_contributions, git_read_tool_contributions};
#[cfg(all(feature = "file-read", feature = "file-write"))]
pub use contributions::{
    ordinary_file_tool_contributions, snapshot_aware_file_tool_contributions,
    snapshot_aware_file_tool_contributions_with_capability,
};
#[cfg(feature = "git")]
pub use diff_stat::{DiffInput, DiffTool, StatInput, StatTool};
#[cfg(feature = "document")]
pub use document_extract::{DocumentExtractError, DocumentExtractInput, DocumentExtractTool};
#[cfg(feature = "shell")]
pub use exec_tool::{ExecError, ExecInput, ExecTool};
#[cfg(feature = "network")]
pub use fetch_url::{FetchUrlError, FetchUrlInput, FetchUrlTool};
#[cfg(feature = "file-read")]
pub use file_tools::FileSnapshotRegistry;
#[cfg(all(feature = "file-read", feature = "file-write"))]
pub use file_tools::snapshot_aware_file_tools;
#[cfg(feature = "file-write")]
pub use file_tools::{
    CapStdAtomicCreateCapability, DeleteError, DeleteInput, DeleteTool, EditInput, EditTool,
    WriteInput, WriteTool,
};
#[cfg(any(
    feature = "file-read",
    feature = "search",
    feature = "shell",
    feature = "git"
))]
pub use file_tools::{FileToolError, is_skip_dir};
#[cfg(feature = "file-read")]
pub use file_tools::{LsInput, LsTool, ReadInput, ReadTool};
#[cfg(feature = "git")]
pub use git::{GitToolError, git_dirty_count};
#[cfg(feature = "network")]
pub use http_request::{HttpRequestError, HttpRequestInput, HttpRequestTool};
#[cfg(feature = "image")]
pub use read_image_tool::ReadImageTool;
#[cfg(all(feature = "network", feature = "file-write"))]
pub use save_url::{SaveUrlError, SaveUrlInput, SaveUrlTool};
#[cfg(feature = "search")]
pub use search_tools::{GlobInput, GlobTool, GrepInput, GrepTool};
#[cfg(feature = "file-read")]
pub use tree::TreeTool;
#[cfg(feature = "network")]
pub use web_search::{WebSearchError, WebSearchInput, WebSearchTool};

/// Returns a stable workspace-relative display path using `/` on every host.
#[cfg(any(feature = "file-read", feature = "search"))]
pub(crate) fn workspace_relative_display(path: &std::path::Path, root: &std::path::Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
