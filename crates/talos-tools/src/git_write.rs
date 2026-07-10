//! Git write tools — host-git-backed operations (add, commit, push, pull, checkout).

use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;

use super::{GitToolError, discover_repo};

// ─── Host git helpers ───

async fn get_workdir(workspace_root: &std::path::Path) -> Result<PathBuf, GitToolError> {
    let repo = discover_repo(workspace_root)?;
    Ok(repo
        .workdir()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| workspace_root.to_path_buf()))
}

pub(crate) async fn run_host_git_with_program(
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
