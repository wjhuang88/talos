//! Built-in Git tools using the gix crate (pure Rust Git implementation).
//!
//! Read-only tools are implemented natively via gix.
//! Write tools use a structured host git fallback while native gix write
//! orchestration remains under evaluation.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolResult,
};
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

/// Git write tools module.
#[path = "git_write.rs"]
mod git_write;
pub use git_write::{
    GitAddInput, GitAddTool, GitCheckoutInput, GitCheckoutTool, GitCommitInput,
    GitCommitTool, GitPullInput, GitPullTool, GitPushInput, GitPushTool,
};

fn discover_repo(workspace_root: &std::path::Path) -> Result<gix::Repository, GitToolError> {
    gix::discover(workspace_root)
        .map_err(|_| GitToolError::NotARepository(workspace_root.display().to_string()))
}

// ─── git_status ───

/// Count dirty working-tree entries using the native `gix` status API.
///
/// This is used by runtime/governance status surfaces that need a small Git summary without
/// falling back to the host `git` executable.
pub fn git_dirty_count(workspace_root: &Path) -> Result<usize, GitToolError> {
    Ok(git_status_lines(workspace_root)?.len())
}

fn git_status_lines(search_path: &Path) -> Result<Vec<String>, GitToolError> {
    let repo = discover_repo(search_path)?;
    let platform = repo
        .status(gix::progress::Discard)
        .map_err(|e| GitToolError::Git(e.to_string()))?
        .untracked_files(gix::status::UntrackedFiles::Files);

    let iter = platform
        .into_index_worktree_iter(Vec::<gix::bstr::BString>::new())
        .map_err(|e| GitToolError::Git(e.to_string()))?;

    let mut output = Vec::new();
    for item in iter {
        let item = item.map_err(|e| GitToolError::Git(e.to_string()))?;
        let summary = item.summary();
        let path_str = item.rela_path().to_string();
        let status_char = match summary {
            Some(gix::status::index_worktree::iter::Summary::Modified) => "M",
            Some(gix::status::index_worktree::iter::Summary::Added) => "A",
            Some(gix::status::index_worktree::iter::Summary::Removed) => "D",
            Some(gix::status::index_worktree::iter::Summary::Renamed) => "R",
            Some(gix::status::index_worktree::iter::Summary::Copied) => "C",
            Some(gix::status::index_worktree::iter::Summary::TypeChange) => "T",
            Some(gix::status::index_worktree::iter::Summary::Conflict) => "!",
            Some(gix::status::index_worktree::iter::Summary::IntentToAdd) => "I",
            None => "?",
        };
        output.push(format!("{status_char} {path_str}"));
    }

    Ok(output)
}

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
    fn name(&self) -> &str {
        "git_status"
    }
    fn description(&self) -> &str {
        "Show working tree status"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitStatusInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }
}

impl GitStatusTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitStatusInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let search_path = match git_input.path {
            Some(ref p) => self.workspace_root.join(p),
            None => self.workspace_root.clone(),
        };

        let output = git_status_lines(&search_path)?;

        if output.is_empty() {
            Ok("clean working tree".to_string())
        } else {
            Ok(output.join("\n"))
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
    fn name(&self) -> &str {
        "git_log"
    }
    fn description(&self) -> &str {
        "Show commit history"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitLogInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["max_count"]
    }
    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }
}

impl GitLogTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitLogInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let max_count = git_input.max_count.unwrap_or(20) as usize;
        let repo = discover_repo(&self.workspace_root)?;
        let head = repo
            .rev_parse_single("HEAD")
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit = head
            .object()
            .map_err(|e| GitToolError::Git(e.to_string()))?
            .try_into_commit()
            .map_err(|e| GitToolError::Git(e.to_string()))?;

        let mut output = String::new();

        for (count, info) in repo
            .rev_walk([commit.id])
            .all()
            .map_err(|e| GitToolError::Git(e.to_string()))?
            .enumerate()
        {
            if count >= max_count {
                break;
            }
            let info = info.map_err(|e| GitToolError::Git(e.to_string()))?;
            let obj = info
                .object()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            let commit_ref = obj.decode().map_err(|e| GitToolError::Git(e.to_string()))?;
            let author = commit_ref
                .author()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            let author_name = author.name.to_string();
            let date = author.time.to_string();

            let hex_str = info.id.to_hex().to_string();
            let short = &hex_str[..7.min(hex_str.len())];
            let message = commit_ref.message.to_string();
            let first_line = message.lines().next().unwrap_or("(no message)");

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
    fn name(&self) -> &str {
        "git_branch_list"
    }
    fn description(&self) -> &str {
        "List branches"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitBranchListInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["remote"]
    }
    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }
}

impl GitBranchListTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitBranchListInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let repo = discover_repo(&self.workspace_root)?;
        let current_head = repo.head_name().ok().flatten();
        let current_branch = current_head.as_ref().map(|h| h.shorten().to_string());

        let mut output = String::new();
        let mut local_branches = Vec::new();
        let mut remote_branches = Vec::new();

        let ref_platform = repo
            .references()
            .map_err(|e| GitToolError::Git(e.to_string()))?;

        let local_iter = ref_platform
            .local_branches()
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
            let remote_iter = ref_platform
                .remote_branches()
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
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub base_ref: Option<String>,
    #[serde(default)]
    pub head_ref: Option<String>,
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
    fn name(&self) -> &str {
        "git_diff"
    }
    fn description(&self) -> &str {
        "Show changes (staged or unstaged)"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitDiffInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["staged", "max_lines"]
    }
    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }
}

impl GitDiffTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitDiffInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;
        let max_lines = git_input.max_lines.unwrap_or(200) as usize;
        let path_filter = git_input.path.as_deref();

        if let (Some(base), Some(head)) = (&git_input.base_ref, &git_input.head_ref) {
            let workdir = {
                let repo = discover_repo(&self.workspace_root)?;
                repo.workdir()
                    .ok_or_else(|| GitToolError::Git("no workdir (bare repo?)".into()))?
                    .to_path_buf()
            };

            let mut cmd = tokio::process::Command::new("git");
            cmd.arg("diff")
                .arg(format!("{base}..{head}"))
                .current_dir(&workdir);
            if let Some(p) = path_filter {
                cmd.arg("--").arg(p);
            }
            let output = cmd
                .output()
                .await
                .map_err(|e| GitToolError::Git(format!("host git not available: {e}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(GitToolError::Git(format!(
                    "git diff {base}..{head} failed: {stderr}"
                )));
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            if lines.len() > max_lines {
                let truncated = lines[..max_lines].join("\n");
                return Ok(format!("{truncated}\n... (truncated at {max_lines} lines)"));
            }
            return Ok(stdout.trim_end().to_string());
        }

        let staged = git_input.staged;

        let repo = discover_repo(&self.workspace_root)?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitToolError::Git("no workdir".into()))?
            .to_path_buf();

        let platform = repo
            .status(gix::progress::Discard)
            .map_err(|e| GitToolError::Git(e.to_string()))?
            .untracked_files(gix::status::UntrackedFiles::Files);

        let iter = platform
            .into_index_worktree_iter(Vec::<gix::bstr::BString>::new())
            .map_err(|e| GitToolError::Git(e.to_string()))?;

        let mut output = String::new();
        let mut line_count = 0usize;

        for item in iter {
            let item = item.map_err(|e| GitToolError::Git(e.to_string()))?;
            let path_str = item.rela_path().to_string();

            if let Some(filter) = path_filter
                && !path_str.starts_with(filter)
            {
                continue;
            }

            let old_content = read_blob_at_ref(&repo, "HEAD", &path_str);
            let new_content = if staged {
                read_index_blob_content(&repo, &path_str)
            } else {
                std::fs::read_to_string(workdir.join(&path_str)).ok()
            };

            match (old_content, new_content) {
                (Some(old), Some(new)) => {
                    if old == new {
                        continue;
                    }
                    let diff = similar::TextDiff::from_lines(&old, &new);
                    let patch = diff
                        .unified_diff()
                        .header(&format!("--- a/{path_str}"), &format!("+++ b/{path_str}"))
                        .context_radius(3)
                        .to_string();

                    output.push_str(&format!("diff --git a/{path_str} b/{path_str}\n"));
                    for line in patch.lines() {
                        if line_count >= max_lines {
                            output.push_str(&format!("\n... (truncated at {max_lines} lines)\n"));
                            return Ok(output.trim_end().to_string());
                        }
                        output.push_str(line);
                        output.push('\n');
                        line_count += 1;
                    }
                }
                _ => {
                    if line_count >= max_lines {
                        output.push_str(&format!("\n... (truncated at {max_lines} lines)\n"));
                        return Ok(output.trim_end().to_string());
                    }
                    output.push_str(&format!("diff -- {path_str} (binary or unreadable)\n"));
                    line_count += 1;
                }
            }
        }

        if output.is_empty() {
            Ok("no changes".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

fn read_blob_at_ref(repo: &gix::Repository, rev: &str, path: &str) -> Option<String> {
    use gix::bstr::ByteSlice;
    let spec = format!("{rev}:{path}");
    let blob_id = repo.rev_parse_single(spec.as_bytes().as_bstr()).ok()?;
    let blob = repo.find_object(blob_id).ok()?;
    String::from_utf8(blob.data.to_vec()).ok()
}

fn read_index_blob_content(repo: &gix::Repository, path: &str) -> Option<String> {
    use gix::bstr::ByteSlice;
    let blob_id = repo
        .rev_parse_single(format!(":{path}").as_bytes().as_bstr())
        .ok()?;
    let blob = repo.find_object(blob_id).ok()?;
    String::from_utf8(blob.data.to_vec()).ok()
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
    fn name(&self) -> &str {
        "git_show"
    }
    fn description(&self) -> &str {
        "Show details of a specific commit"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitShowInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["revision"]
    }
    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }
}

impl GitShowTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitShowInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let repo = discover_repo(&self.workspace_root)?;
        let rev = repo
            .rev_parse_single(git_input.revision.as_str())
            .map_err(|e| {
                GitToolError::Git(format!("revision '{}' not found: {e}", git_input.revision))
            })?;

        let obj = rev.object().map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit = obj
            .try_into_commit()
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        let commit_ref = commit
            .decode()
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        let author = commit_ref
            .author()
            .map_err(|e| GitToolError::Git(e.to_string()))?;
        let author_name = author.name.to_string();
        let date = author.time.to_string();

        let hex_str = commit.id().to_hex().to_string();
        let short = &hex_str[..7.min(hex_str.len())];
        let message = commit_ref.message.to_string();
        let first_line = message.lines().next().unwrap_or("(no message)");

        let mut output = format!("commit {short}\n");
        output.push_str(&format!("Author: {author_name}\n"));
        output.push_str(&format!("Date:   {date}\n"));
        output.push_str(&format!("\n    {first_line}\n"));

        Ok(output.trim_end().to_string())
    }
}


#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
