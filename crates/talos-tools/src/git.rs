//! Built-in Git tools using the gix crate (pure Rust Git implementation).
//!
//! Read-only tools are implemented natively via gix.
//! Write tools use gix where possible (add, commit) and host git fallback
//! for operations gix doesn't support (push, pull, checkout).

use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitToolError {
    #[error("not a git repository: {0}")]
    NotARepository(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

fn discover_repo(workspace_root: &std::path::Path) -> Result<gix::Repository, GitToolError> {
    gix::discover(workspace_root).map_err(|_| GitToolError::NotARepository(
        workspace_root.display().to_string(),
    ))
}

// ─── git_status ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitStatusInput {
    #[serde(default)]
    pub path: Option<String>,
}

pub struct GitStatusTool {
    workspace_root: PathBuf,
}

impl GitStatusTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitStatusTool {
    fn name(&self) -> &str { "git_status" }
    fn description(&self) -> &str { "Show working tree status" }
    fn parameters(&self) -> Value { tool_parameters!(GitStatusInput) }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool { true }
}

impl GitStatusTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitStatusInput = serde_json::from_value(input)
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let search_path = match git_input.path {
            Some(ref p) => self.workspace_root.join(p),
            None => self.workspace_root.clone(),
        };

        let repo = discover_repo(&search_path)?;
        let platform = repo
            .status(gix::progress::Discard)
            .map_err(|e| GitToolError::Git(e.to_string()))?
            .untracked_files(gix::status::UntrackedFiles::Files);

        let iter = platform.into_index_worktree_iter(Vec::<gix::bstr::BString>::new()).map_err(|e| GitToolError::Git(e.to_string()))?;

        let mut output = String::new();
        for item in iter {
            let item = item.map_err(|e| GitToolError::Git(e.to_string()))?;
            let (status_char, path) = match item {
                gix::status::index_worktree::iter::Item::Modification { rela_path, .. } => {
                    ("M", rela_path.to_string())
                }
                gix::status::index_worktree::iter::Item::DirectoryContents { entry, .. } => {
                    ("??", entry.rela_path.to_string())
                }
                gix::status::index_worktree::iter::Item::Rewrite { dirwalk_entry, .. } => {
                    ("R", dirwalk_entry.rela_path.to_string())
                }
            };
            output.push_str(&format!("{status_char} {path}\n"));
        }

        if output.is_empty() {
            Ok("clean working tree".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

// ─── git_log ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitLogInput {
    #[serde(default)]
    pub max_count: Option<u32>,
}

pub struct GitLogTool {
    workspace_root: PathBuf,
}

impl GitLogTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitLogTool {
    fn name(&self) -> &str { "git_log" }
    fn description(&self) -> &str { "Show commit history" }
    fn parameters(&self) -> Value { tool_parameters!(GitLogInput) }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool { true }
}

impl GitLogTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitLogInput = serde_json::from_value(input)
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let max_count = git_input.max_count.unwrap_or(20) as usize;
        let repo = discover_repo(&self.workspace_root)?;
        let head = repo.rev_parse_single("HEAD")
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit = head.object().map_err(|e| GitToolError::Git(e.to_string()))?
            .try_into_commit().map_err(|e| GitToolError::Git(e.to_string()))?;

        let mut output = String::new();

        for (count, info) in repo.rev_walk([commit.id]).all().map_err(|e| GitToolError::Git(e.to_string()))?.enumerate() {
            if count >= max_count {
                break;
            }
            let info = info.map_err(|e| GitToolError::Git(e.to_string()))?;
            let obj = info.object().map_err(|e| GitToolError::Git(e.to_string()))?;
            let commit_ref = obj.decode().map_err(|e| GitToolError::Git(e.to_string()))?;
            let author = commit_ref.author;

            let hex_str = info.id.to_hex().to_string();
            let short = &hex_str[..7.min(hex_str.len())];
            let message = commit_ref.message.to_string();
            let first_line = message.lines().next().unwrap_or("(no message)");
            let actor = author.actor();
            let author_name = actor.name.to_string();
            let seconds = author.time.seconds;
            let date = format_unix_timestamp(seconds as u64);

            output.push_str(&format!("{short} {first_line} ({author_name}, {date})\n"));
        }

        if output.is_empty() {
            Ok("no commits".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

// ─── git_branch_list ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitBranchListInput {
    #[serde(default)]
    pub remote: bool,
}

pub struct GitBranchListTool {
    workspace_root: PathBuf,
}

impl GitBranchListTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitBranchListTool {
    fn name(&self) -> &str { "git_branch_list" }
    fn description(&self) -> &str { "List branches" }
    fn parameters(&self) -> Value { tool_parameters!(GitBranchListInput) }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool { true }
}

impl GitBranchListTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitBranchListInput = serde_json::from_value(input)
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let repo = discover_repo(&self.workspace_root)?;
        let current_head = repo.head_name().ok().flatten();
        let current_branch = current_head.as_ref().map(|h| h.shorten().to_string());

        let mut output = String::new();
        let mut local_branches = Vec::new();
        let mut remote_branches = Vec::new();

        let ref_platform = repo.references()
            .map_err(|e| GitToolError::Git(e.to_string()))?;

        let local_iter = ref_platform.local_branches()
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        for reference in local_iter {
            let reference = match reference {
                Ok(r) => r,
                Err(_) => continue,
            };
            let name = reference.name();
            let name_str = name.as_bstr().to_string();
            let branch_name = name_str.strip_prefix("refs/heads/").unwrap_or(&name_str);
            local_branches.push(branch_name.to_string());
        }

        if git_input.remote {
            let remote_iter = ref_platform.remote_branches()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            for reference in remote_iter {
                let reference = match reference {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let name = reference.name();
                let name_str = name.as_bstr().to_string();
                let branch_name = name_str.strip_prefix("refs/remotes/").unwrap_or(&name_str);
                remote_branches.push(branch_name.to_string());
            }
        }

        local_branches.sort();
        for branch in &local_branches {
            let marker = if current_branch.as_deref() == Some(branch.as_str()) {
                "* "
            } else {
                "  "
            };
            output.push_str(&format!("{marker}{branch}\n"));
        }

        if git_input.remote {
            remote_branches.sort();
            for branch in &remote_branches {
                output.push_str(&format!("  remotes/{branch}\n"));
            }
        }

        if output.is_empty() {
            Ok("no branches".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

// ─── git_diff ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitDiffInput {
    #[serde(default)]
    pub staged: bool,
    #[serde(default)]
    pub max_lines: Option<u32>,
}

pub struct GitDiffTool {
    workspace_root: PathBuf,
}

impl GitDiffTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitDiffTool {
    fn name(&self) -> &str { "git_diff" }
    fn description(&self) -> &str { "Show changes (staged or unstaged)" }
    fn parameters(&self) -> Value { tool_parameters!(GitDiffInput) }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool { true }
}

impl GitDiffTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitDiffInput = serde_json::from_value(input)
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;
        let max_lines = git_input.max_lines.unwrap_or(200) as usize;

        let repo = discover_repo(&self.workspace_root)?;
        let platform = repo
            .status(gix::progress::Discard)
            .map_err(|e| GitToolError::Git(e.to_string()))?
            .untracked_files(gix::status::UntrackedFiles::Files);

        let iter = platform.into_index_worktree_iter(Vec::<gix::bstr::BString>::new()).map_err(|e| GitToolError::Git(e.to_string()))?;

        let mut output = String::new();
        let mut line_count = 0;

        for item in iter {
            let item = item.map_err(|e| GitToolError::Git(e.to_string()))?;
            let path = match item {
                gix::status::index_worktree::iter::Item::Modification { rela_path, .. } => {
                    rela_path.to_string()
                }
                gix::status::index_worktree::iter::Item::DirectoryContents { entry, .. } => {
                    entry.rela_path.to_string()
                }
                gix::status::index_worktree::iter::Item::Rewrite { dirwalk_entry, .. } => {
                    dirwalk_entry.rela_path.to_string()
                }
            };

            output.push_str(&format!("diff -- {path}\n"));
            line_count += 1;

            if line_count >= max_lines {
                output.push_str(&format!(
                    "\n... (truncated at {max_lines} entries)"
                ));
                break;
            }
        }

        if output.is_empty() {
            Ok("no changes".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

// ─── git_show ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitShowInput {
    pub revision: String,
}

pub struct GitShowTool {
    workspace_root: PathBuf,
}

impl GitShowTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitShowTool {
    fn name(&self) -> &str { "git_show" }
    fn description(&self) -> &str { "Show details of a specific commit" }
    fn parameters(&self) -> Value { tool_parameters!(GitShowInput) }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool { true }
}

impl GitShowTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitShowInput = serde_json::from_value(input)
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let repo = discover_repo(&self.workspace_root)?;
        let rev = repo.rev_parse_single(git_input.revision.as_str())
            .map_err(|e| GitToolError::Git(format!("revision '{}' not found: {e}", git_input.revision)))?;

        let obj = rev.object().map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit = obj.try_into_commit().map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit_ref = commit.decode().map_err(|e| GitToolError::Git(e.to_string()))?;
        let author = commit_ref.author;

        let hex_str = commit.id().to_hex().to_string();
        let short = &hex_str[..7.min(hex_str.len())];
        let actor = author.actor();
        let author_name = actor.name.to_string();
        let seconds = author.time.seconds;
        let date = format_unix_timestamp(seconds as u64);
        let message = commit_ref.message.to_string();
        let first_line = message.lines().next().unwrap_or("(no message)");

        let mut output = format!("commit {short}\n");
        output.push_str(&format!("Author: {author_name}\n"));
        output.push_str(&format!("Date:   {date}\n"));
        output.push_str(&format!("\n    {first_line}\n"));

        Ok(output.trim_end().to_string())
    }
}

// ─── Helper ───

fn format_unix_timestamp(secs: u64) -> String {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;

    let z = days as i64 + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02} {hour:02}:{min:02}")
}
