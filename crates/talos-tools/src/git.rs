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
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
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

            let old_content = read_head_blob_content(&repo, &path_str);
            let new_content = std::fs::read_to_string(workdir.join(&path_str)).ok();

            match (old_content, new_content) {
                (Some(old), Some(new)) => {
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

fn read_head_blob_content(repo: &gix::Repository, path: &str) -> Option<String> {
    use gix::bstr::ByteSlice;
    let spec = format!("HEAD:{path}");
    let blob_id = repo.rev_parse_single(spec.as_bytes().as_bstr()).ok()?;
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

// ─── Host git helpers ───

async fn get_workdir(workspace_root: &std::path::Path) -> Result<PathBuf, GitToolError> {
    let repo = discover_repo(workspace_root)?;
    Ok(repo
        .workdir()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| workspace_root.to_path_buf()))
}

async fn run_host_git_with_program(
    program: &str,
    workdir: &std::path::Path,
    args: &[&str],
) -> Result<String, GitToolError> {
    let output = tokio::process::Command::new(program)
        .args(args)
        .current_dir(workdir)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GitToolError::Git("git not installed. Install git or use read-only tools.".into())
            } else {
                GitToolError::Git(format!("failed to spawn git: {e}"))
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        let err = if stderr.is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Err(GitToolError::Git(err));
    }

    Ok(stdout)
}

async fn run_host_git(workdir: &std::path::Path, args: &[&str]) -> Result<String, GitToolError> {
    run_host_git_with_program("git", workdir, args).await
}

// ─── git_add ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitAddInput {
    pub paths: Vec<String>,
}

pub struct GitAddTool {
    workspace_root: PathBuf,
}

impl GitAddTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitAddTool {
    fn name(&self) -> &str {
        "git_add"
    }
    fn description(&self) -> &str {
        "Stage files for commit"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitAddInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["paths"]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl GitAddTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitAddInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        if git_input.paths.is_empty() {
            return Err(GitToolError::InvalidInput("paths cannot be empty".into()));
        }

        let workdir = get_workdir(&self.workspace_root).await?;

        let mut args = vec!["add"];
        let path_refs: Vec<&str> = git_input.paths.iter().map(|s| s.as_str()).collect();
        args.extend(path_refs.iter().copied());

        run_host_git(&workdir, &args).await?;
        Ok(format!("staged {} file(s)", git_input.paths.len()))
    }
}

// ─── git_commit ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitCommitInput {
    pub message: String,
    #[serde(default)]
    pub all: bool,
}

pub struct GitCommitTool {
    workspace_root: PathBuf,
}

impl GitCommitTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitCommitTool {
    fn name(&self) -> &str {
        "git_commit"
    }
    fn description(&self) -> &str {
        "Create a commit with a message"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitCommitInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["message", "all"]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl GitCommitTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitCommitInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let workdir = get_workdir(&self.workspace_root).await?;

        let mut args = vec!["commit", "-m", &git_input.message];
        if git_input.all {
            args.push("-a");
        }

        run_host_git(&workdir, &args).await?;
        let log_output = run_host_git(&workdir, &["rev-parse", "--short", "HEAD"]).await?;
        let short_sha = log_output.trim();
        let first_line = git_input
            .message
            .lines()
            .next()
            .unwrap_or(&git_input.message);

        Ok(format!("committed: {short_sha} {first_line}"))
    }
}

// ─── git_push ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitPushInput {
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub force: bool,
}

pub struct GitPushTool {
    workspace_root: PathBuf,
}

impl GitPushTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitPushTool {
    fn name(&self) -> &str {
        "git_push"
    }
    fn description(&self) -> &str {
        "Push to remote repository"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitPushInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["remote", "branch", "force"]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        talos_core::tool::ToolNature::Execute
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let remote = input
            .get("remote")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("origin");

        vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "git",
                ToolResourceKind::Command,
            )
            .with_description("host git push command"),
            ToolPermissionFacet::with_resource(
                ToolNature::Network,
                remote,
                ToolResourceKind::Remote,
            )
            .with_description("remote repository mutation"),
        ]
    }
}

impl GitPushTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitPushInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let workdir = get_workdir(&self.workspace_root).await?;

        let remote = git_input.remote.as_deref().unwrap_or("origin");
        let mut args = vec!["push"];
        if git_input.force {
            args.push("--force");
        }
        args.push(remote);
        if let Some(ref branch) = git_input.branch {
            args.push(branch);
        }

        run_host_git(&workdir, &args).await?;
        Ok(format!("pushed to {remote}"))
    }
}

// ─── git_pull ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitPullInput {
    #[serde(default)]
    pub remote: Option<String>,
}

pub struct GitPullTool {
    workspace_root: PathBuf,
}

impl GitPullTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitPullTool {
    fn name(&self) -> &str {
        "git_pull"
    }
    fn description(&self) -> &str {
        "Pull from remote repository"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitPullInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["remote"]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        talos_core::tool::ToolNature::Execute
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let remote = input
            .get("remote")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("origin");

        vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "git",
                ToolResourceKind::Command,
            )
            .with_description("host git pull command"),
            ToolPermissionFacet::with_resource(
                ToolNature::Network,
                remote,
                ToolResourceKind::Remote,
            )
            .with_description("remote repository fetch"),
            ToolPermissionFacet::with_resource(
                ToolNature::Write,
                self.workspace_root.to_string_lossy(),
                ToolResourceKind::Path,
            )
            .with_description("workspace update"),
        ]
    }
}

impl GitPullTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitPullInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let workdir = get_workdir(&self.workspace_root).await?;

        let remote = git_input.remote.as_deref().unwrap_or("origin");
        let output = run_host_git(&workdir, &["pull", remote]).await?;

        if output.contains("Already up to date") || output.contains("Already up-to-date") {
            Ok("already up to date".to_string())
        } else {
            Ok(format!("pulled from {remote}"))
        }
    }
}

// ─── git_checkout ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitCheckoutInput {
    pub branch: String,
    #[serde(default)]
    pub create: bool,
}

pub struct GitCheckoutTool {
    workspace_root: PathBuf,
}

impl GitCheckoutTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GitCheckoutTool {
    fn name(&self) -> &str {
        "git_checkout"
    }
    fn description(&self) -> &str {
        "Switch branches"
    }
    fn parameters(&self) -> Value {
        tool_parameters!(GitCheckoutInput)
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["branch", "create"]
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Git
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl GitCheckoutTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let git_input: GitCheckoutInput =
            serde_json::from_value(input).map_err(|e| GitToolError::InvalidInput(e.to_string()))?;

        let workdir = get_workdir(&self.workspace_root).await?;

        let mut args = vec!["checkout"];
        if git_input.create {
            args.push("-b");
        }
        args.push(&git_input.branch);

        run_host_git(&workdir, &args).await?;
        Ok(format!("switched to branch {}", git_input.branch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_push_permission_profile_has_execute_and_remote_network() {
        let tool = GitPushTool::new(PathBuf::from("/workspace"));
        let profile = tool.permission_profile(&serde_json::json!({
            "remote": "upstream",
            "branch": "main"
        }));

        assert_eq!(profile.len(), 2);
        assert_eq!(profile[0].nature, ToolNature::Execute);
        assert_eq!(profile[0].resource.as_deref(), Some("git"));
        assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Command));
        assert_eq!(profile[1].nature, ToolNature::Network);
        assert_eq!(profile[1].resource.as_deref(), Some("upstream"));
        assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Remote));
    }

    #[test]
    fn git_pull_permission_profile_has_execute_remote_and_workspace_write() {
        let tool = GitPullTool::new(PathBuf::from("/workspace"));
        let profile = tool.permission_profile(&serde_json::json!({}));

        assert_eq!(profile.len(), 3);
        assert_eq!(profile[0].nature, ToolNature::Execute);
        assert_eq!(profile[0].resource.as_deref(), Some("git"));
        assert_eq!(profile[1].nature, ToolNature::Network);
        assert_eq!(profile[1].resource.as_deref(), Some("origin"));
        assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Remote));
        assert_eq!(profile[2].nature, ToolNature::Write);
        assert_eq!(profile[2].resource.as_deref(), Some("/workspace"));
        assert_eq!(profile[2].resource_kind, Some(ToolResourceKind::Path));
    }

    #[tokio::test]
    async fn host_git_unavailable_returns_actionable_error() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let err = run_host_git_with_program(
            "talos-host-git-that-should-not-exist",
            tempdir.path(),
            &["status"],
        )
        .await
        .expect_err("missing git executable should fail");

        assert_eq!(
            err.to_string(),
            "git error: git not installed. Install git or use read-only tools."
        );
    }

    #[tokio::test]
    async fn git_diff_produces_unified_diff_content() {
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_err()
        {
            eprintln!("skipping: host git not available");
            return;
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .output()
                .expect("git command")
        };
        run(&["init"]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test"]);

        std::fs::write(dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();
        run(&["add", "test.txt"]);
        run(&["commit", "-m", "initial"]);

        std::fs::write(dir.path().join("test.txt"), "line1\nmodified\nline3\n").unwrap();

        let tool = GitDiffTool::new(dir.path().to_path_buf());
        let result = tool.execute(serde_json::json!({})).await;

        assert!(!result.is_error, "{}", result.content);
        assert!(
            result.content.contains("diff --git a/test.txt b/test.txt"),
            "expected diff --git header, got: {}",
            result.content
        );
        assert!(
            result.content.contains("--- a/test.txt"),
            "expected --- header, got: {}",
            result.content
        );
        assert!(
            result.content.contains("+++ b/test.txt"),
            "expected +++ header, got: {}",
            result.content
        );
        assert!(
            result.content.contains("-line2"),
            "expected removed line, got: {}",
            result.content
        );
        assert!(
            result.content.contains("+modified"),
            "expected added line, got: {}",
            result.content
        );
    }
}
