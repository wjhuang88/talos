//! Git write tools — host-git-backed operations (add, commit, push, pull, checkout).

#[cfg(windows)]
use crate::process_boundary::windows_foreground::{OwnedCommandExecutor, OwnedCommandOperation};
#[cfg(unix)]
use talos_sandbox::{OwnedCommandExecutor, OwnedCommandOperation};

use std::path::{Component, Path, PathBuf};
#[cfg(all(test, not(unix)))]
use std::process::Stdio;
#[cfg(test)]
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;
#[cfg(test)]
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{GitToolError, discover_repo};

#[path = "git_write/snapshot.rs"]
mod snapshot;

#[path = "git_write/repository_state.rs"]
mod repository_state;

#[path = "git_write/prepared_add.rs"]
mod prepared_add;

#[path = "git_write/config_state.rs"]
mod config_state;

#[path = "git_write/target_config.rs"]
mod target_config;

#[path = "git_write/checkout_state.rs"]
mod checkout_state;

#[path = "git_write/checkout_deletion.rs"]
mod checkout_deletion;

#[path = "git_write/status_state.rs"]
mod status_state;

#[path = "git_write/remote_state.rs"]
mod remote_state;

#[path = "git_write/receiver_state.rs"]
mod receiver_state;

#[path = "git_write/receiver_storage.rs"]
mod receiver_storage;

// Component resolution is separately authorized from target content reads.
#[path = "git_write/pull_resolution.rs"]
mod pull_resolution;

#[path = "git_write/auxiliary_paths.rs"]
mod auxiliary_paths;

#[path = "git_write/pull_inventory.rs"]
mod pull_inventory;

// ─── Host git helpers ───

#[cfg(test)]
pub(crate) async fn run_host_git_with_program(
    program: &str,
    workdir: &std::path::Path,
    args: &[&str],
) -> Result<String, GitToolError> {
    run_host_git_bounded(program, workdir, args, Duration::from_secs(300), 256 * 1024).await
}

#[cfg(test)]
async fn read_host_pipe(
    pipe: impl AsyncRead + Unpin,
    limit: u64,
) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    pipe.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::other("host git output limit exceeded"));
    }
    Ok(bytes)
}

#[cfg(test)]
async fn run_host_git_bounded(
    program: &str,
    workdir: &std::path::Path,
    args: &[&str],
    budget: Duration,
    output_limit: u64,
) -> Result<String, GitToolError> {
    let mut command = tokio::process::Command::new(program);
    command.args(args).current_dir(workdir);
    run_host_command(command, budget, output_limit)
        .await
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(all(unix, test))]
async fn run_host_command(
    mut command: tokio::process::Command,
    budget: Duration,
    output_limit: u64,
) -> Result<Vec<u8>, GitToolError> {
    crate::process_boundary::apply_process_hardening(&mut command);
    let output = talos_sandbox::execute_owned_command(
        command.into_std(), budget, usize::try_from(output_limit).unwrap_or(usize::MAX),
    ).await.map_err(|error| GitToolError::Git(format!("host git execution failure: {error}; verify Git is installed and executable, or use read-only tools; Git may have partially changed the repository; inspect its state before retrying")))?;
    if !output.status.success() {
        let bytes = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        return Err(GitToolError::Git(
            String::from_utf8_lossy(bytes).trim().to_owned(),
        ));
    }
    Ok(output.stdout)
}

#[cfg(all(test, not(unix)))]
async fn run_host_command_output(
    mut command: tokio::process::Command,
    budget: Duration,
    output_limit: u64,
) -> Result<std::process::Output, GitToolError> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GitToolError::Git("git not installed. Install git or use read-only tools.".into())
            } else {
                GitToolError::Git(format!("failed to spawn git: {e}"))
            }
        })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| GitToolError::Git("missing git stdout".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| GitToolError::Git("missing git stderr".into()))?;
    // One deadline covers both exit and inherited pipe EOF. Never retry a mutation
    // whose effects are unknown. kill_on_drop also handles cancellation of this future.
    let collected = tokio::time::timeout(budget, async {
        tokio::try_join!(
            child.wait(),
            read_host_pipe(stdout, output_limit),
            read_host_pipe(stderr, output_limit),
        )
    })
    .await;
    let (status, stdout, stderr) = match collected {
        Ok(Ok(output)) => output,
        failure => {
            // Do not await cleanup without a bound: descendant pipes or OS cleanup
            // must not turn the caller's deadline into another unbounded wait.
            let _ = child.start_kill();
            let reason = match failure {
                Ok(Err(error)) => error.to_string(),
                Err(_) => "host git deadline exceeded".to_string(),
                Ok(Ok(_)) => unreachable!(),
            };
            return Err(GitToolError::Git(format!(
                "{reason}; Git may have partially changed the repository; inspect its state before retrying"
            )));
        }
    };
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}

#[cfg(all(test, not(unix)))]
async fn run_host_command(
    command: tokio::process::Command,
    budget: Duration,
    output_limit: u64,
) -> Result<Vec<u8>, GitToolError> {
    let output = run_host_command_output(command, budget, output_limit).await?;
    if !output.status.success() {
        let bytes = if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        return Err(GitToolError::Git(
            String::from_utf8_lossy(bytes).trim().to_owned(),
        ));
    }
    Ok(output.stdout)
}

fn capture_add_sources(
    workdir: &Path,
    paths: &[String],
) -> Result<Vec<snapshot::Entry>, GitToolError> {
    let mut budget = snapshot::Budget {
        entries: 4096,
        bytes: 16 * 1024 * 1024,
    };
    paths
        .iter()
        .map(|path| {
            let entry = workdir.join(path);
            let captured = if path == "." {
                snapshot::Entry::capture_worktree(workdir, &mut budget)
            } else {
                snapshot::Entry::capture(&entry, &mut budget)
            };
            captured.map_err(|error| GitToolError::InvalidInput(error.to_string()))
        })
        .collect()
}

/// Resolve workspace-relative entries without dereferencing the final symlink.
/// Deleted entries are valid selections; existing ancestors must stay inside
/// the workspace. This is path interpretation, not permission admission or
/// race-free filesystem confinement.
fn literal_add_paths(
    workspace: &Path,
    workdir: &Path,
    paths: &[String],
) -> Result<Vec<String>, GitToolError> {
    let invalid = |message: &str| GitToolError::InvalidInput(message.to_owned());
    let workspace_spelling = workspace;
    let workspace = workspace
        .canonicalize()
        .map_err(|_| invalid("workspace cannot be resolved"))?;
    let workdir = workdir
        .canonicalize()
        .map_err(|_| invalid("repository worktree cannot be resolved"))?;
    if !workspace.starts_with(&workdir) {
        return Err(invalid("workspace is outside repository worktree"));
    }
    if paths.is_empty() {
        return Err(invalid("paths cannot be empty"));
    }
    paths.iter().map(|value| {
        if value.is_empty() || value.contains('\0') {
            return Err(invalid("Git add path must be nonempty and contain no NUL"));
        }
        let input = Path::new(value);
        let relative = if input.is_absolute() {
            input.strip_prefix(&workspace)
                .or_else(|_| input.strip_prefix(workspace_spelling))
                .map_err(|_| invalid("Git add path is outside workspace"))?
        } else {
            input
        };
        let mut entry = workspace.clone();
        for component in relative.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(name) => {
                    if name.to_str().is_some_and(|name| name.eq_ignore_ascii_case(".git")) {
                        return Err(invalid("Git metadata is not a staging source"));
                    }
                    entry.push(name);
                }
                Component::ParentDir if entry != workspace => { entry.pop(); }
                _ => return Err(invalid("Git add path escapes workspace")),
            }
        }
        // Canonicalize the nearest existing ancestor, not the leaf: staging a
        // symlink records its link text and must not grant access to its target.
        if entry != workspace {
            let mut ancestor = entry.parent().ok_or_else(|| invalid("invalid Git add path"))?;
            loop {
                match std::fs::symlink_metadata(ancestor) {
                    Ok(_) => {
                        let resolved = ancestor.canonicalize()
                            .map_err(|_| invalid("Git add ancestor cannot be resolved"))?;
                        if !resolved.starts_with(&workspace) {
                            return Err(invalid("Git add ancestor escapes workspace"));
                        }
                        // Git itself rejects traversal through a symlink path;
                        // retain that behavior rather than silently retargeting.
                        if resolved != ancestor {
                            return Err(invalid("Git add ancestor changed through a symlink; select its workspace-contained target explicitly"));
                        }
                        break;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        ancestor = ancestor.parent().ok_or_else(|| invalid("Git add ancestor is missing"))?;
                    }
                    Err(_) => return Err(invalid("Git add ancestor cannot be inspected")),
                }
            }
        }
        let relative = entry.strip_prefix(&workdir)
            .map_err(|_| invalid("Git add path is outside repository"))?;
        if relative.as_os_str().is_empty() {
            Ok(".".to_owned())
        } else {
            relative.to_str().map(str::to_owned)
                .ok_or_else(|| invalid("Git add path cannot be represented losslessly"))
        }
    }).collect()
}

#[cfg(test)]
mod host_boundary_tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn host_cancellation_owns_descendants_after_leader_exit() {
        let root = tempfile::tempdir().expect("fixture");
        let mut command = tokio::process::Command::new("sh");
        command.current_dir(root.path()).args([
            "-c",
            "(printf ready > ready; sleep 0.5; printf late > forbidden) & exit 0",
        ]);
        let running = tokio::spawn(run_host_command(command, Duration::from_secs(5), 1024));
        tokio::time::timeout(Duration::from_secs(2), async {
            while !root.path().join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("descendant ready");
        running.abort();
        assert!(running.await.expect_err("cancelled").is_cancelled());
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert!(!root.path().join("forbidden").exists());
    }

    fn git_fixture_command(root: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("fixture Git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("fixture UTF-8")
    }

    #[tokio::test]
    async fn literal_add_resolves_subworkspace_and_preserves_other_index_entries() {
        let root = tempfile::tempdir().expect("fixture");
        git_fixture_command(root.path(), &["init"]);
        std::fs::create_dir(root.path().join("sub")).expect("subdir");
        for name in [
            "outside.txt",
            "sub/first.txt",
            "sub/second.txt",
            "sub/untouched.txt",
        ] {
            std::fs::write(root.path().join(name), "fixture\n").expect("file");
        }
        git_fixture_command(root.path(), &["add", "outside.txt"]);
        let tool = GitAddTool::new(root.path().join("sub"));
        let result = tool
            .execute_inner(serde_json::json!({"paths":["first.txt", "second.txt"]}))
            .await
            .expect("stage");
        assert_eq!(result, "staged 2 file(s)");
        let staged = git_fixture_command(root.path(), &["ls-files", "-z"]);
        assert_eq!(staged, "outside.txt\0sub/first.txt\0sub/second.txt\0");
    }

    #[tokio::test]
    async fn literal_add_does_not_expand_options_or_pathspecs() {
        let root = tempfile::tempdir().expect("fixture");
        git_fixture_command(root.path(), &["init"]);
        std::fs::write(root.path().join("--all"), "literal\n").expect("file");
        std::fs::write(root.path().join("unselected"), "no\n").expect("file");
        let tool = GitAddTool::new(root.path().to_owned());
        tool.execute_inner(serde_json::json!({"paths":["--all"]}))
            .await
            .expect("stage literal option");
        assert_eq!(
            git_fixture_command(root.path(), &["ls-files", "-z"]),
            "--all\0"
        );
        let before = std::fs::read(root.path().join(".git/index")).expect("index");
        assert!(
            tool.execute_inner(serde_json::json!({"paths":["unselected", "../outside"]}))
                .await
                .is_err()
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            before
        );
        assert!(
            tool.execute_inner(serde_json::json!({"paths":[".git/config"]}))
                .await
                .is_err()
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn literal_add_accepts_wildcard_filename_and_broken_symlink_not_escaping_parent() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().expect("fixture");
        let outside = tempfile::tempdir().expect("outside");
        git_fixture_command(root.path(), &["init"]);
        std::fs::write(root.path().join("*.txt"), "literal\n").expect("file");
        std::fs::write(root.path().join("other.txt"), "no\n").expect("file");
        symlink(outside.path().join("missing"), root.path().join("broken")).expect("broken link");
        symlink(outside.path(), root.path().join("escape")).expect("parent link");
        let tool = GitAddTool::new(root.path().to_owned());
        tool.execute_inner(serde_json::json!({"paths":["*.txt", "broken"]}))
            .await
            .expect("literal entries");
        assert_eq!(
            git_fixture_command(root.path(), &["ls-files", "-z"]),
            "*.txt\0broken\0"
        );
        let before = std::fs::read(root.path().join(".git/index")).expect("index");
        assert!(
            tool.execute_inner(serde_json::json!({"paths":["escape/missing"]}))
                .await
                .is_err()
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            before
        );
    }

    #[tokio::test]
    async fn literal_add_supports_directory_deletions_and_contained_absolute_paths() {
        let root = tempfile::tempdir().expect("fixture");
        git_fixture_command(root.path(), &["init"]);
        std::fs::create_dir(root.path().join("sub")).expect("subdir");
        std::fs::write(root.path().join("sub/deleted.txt"), "old\n").expect("file");
        git_fixture_command(root.path(), &["add", "."]);
        std::fs::remove_file(root.path().join("sub/deleted.txt")).expect("delete");
        std::fs::write(root.path().join("sub/new.txt"), "new\n").expect("file");
        let tool = GitAddTool::new(root.path().to_owned());
        let selected = root.path().canonicalize().expect("root").join("sub");
        tool.execute_inner(serde_json::json!({"paths":[selected]}))
            .await
            .expect("stage subtree");
        assert_eq!(
            git_fixture_command(root.path(), &["ls-files", "-z"]),
            "sub/new.txt\0"
        );
        tool.execute_inner(serde_json::json!({"paths":[root.path().join("sub/new.txt")]}))
            .await
            .expect("original absolute spelling");
    }

    #[tokio::test]
    async fn output_boundary_accepts_exact_limit_and_rejects_excess() {
        assert_eq!(
            read_host_pipe(&b"1234"[..], 4).await.expect("exact"),
            b"1234"
        );
        assert!(read_host_pipe(&b"12345"[..], 4).await.is_err());
        assert!(read_host_pipe(&b""[..], 0).await.expect("empty").is_empty());
    }

    #[tokio::test]
    async fn real_host_output_limit_reports_ambiguous_effects() {
        let dir = tempfile::tempdir().expect("fixture");
        let error = run_host_git_bounded(
            "git",
            dir.path(),
            &["--version"],
            Duration::from_secs(10),
            1,
        )
        .await
        .expect_err("bounded output");
        assert!(
            error.to_string().contains("output limit exceeded"),
            "{error}"
        );
        assert!(
            error
                .to_string()
                .contains("inspect its state before retrying")
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn inherited_pipe_does_not_extend_host_deadline() {
        let dir = tempfile::tempdir().expect("fixture");
        // The shell exits while a short-lived child retains its output pipes.
        // This proves the caller deadline, not process-tree cancellation.
        let error = tokio::time::timeout(
            Duration::from_secs(2),
            run_host_git_bounded(
                "sh",
                dir.path(),
                &["-c", "sleep 1 & exit 0"],
                Duration::from_millis(20),
                1024,
            ),
        )
        .await
        .expect("caller must return without pipe EOF")
        .expect_err("deadline");
        assert!(
            error.to_string().contains("timed out")
                || error.to_string().contains("deadline exceeded")
        );
        assert!(error.to_string().contains("partially changed"));
    }
}

// ─── git_add ───

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitAddInput {
    pub paths: Vec<String>,
}

pub struct GitAddTool {
    workspace_root: PathBuf,
    #[cfg(any(unix, windows))]
    executor: std::sync::Arc<OwnedCommandExecutor>,
}

impl GitAddTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(any(unix, windows))]
            executor: std::sync::Arc::new(OwnedCommandExecutor::default()),
        }
    }
}

#[async_trait]
impl AgentTool for GitAddTool {
    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        Some(self.executor.clone())
    }

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

    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_add::AddPlan::prepare(&self.workspace_root, input, provenance)
            .map(|plan| {
                #[cfg(any(unix, windows))]
                let plan = plan.bind_operation(self.executor.operation());
                Some(Box::new(plan) as Box<dyn talos_core::tool::PreparedNormalToolInvocation>)
            })
            .map_err(|error| error.to_string())
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::error(
            "git_add requires a prepared invocation through the Agent or MCP permission pipeline; direct execution is not authorized",
        )
    }
}

#[cfg(test)]
impl GitAddTool {
    async fn execute_inner(&self, input: Value) -> Result<String, GitToolError> {
        let plan = prepared_add::AddPlan::prepare(
            &self.workspace_root,
            &input,
            talos_core::tool::ToolProvenance::Native,
        )?;
        let output = prepared_add::tests::execute_approved(plan).await;
        if output.result.is_error {
            Err(GitToolError::Git(output.result.content))
        } else {
            Ok(output.result.content)
        }
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
    #[cfg(any(unix, windows))]
    executor: std::sync::Arc<OwnedCommandExecutor>,
}

impl GitCommitTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(any(unix, windows))]
            executor: std::sync::Arc::new(OwnedCommandExecutor::default()),
        }
    }
}

#[async_trait]
impl AgentTool for GitCommitTool {
    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        Some(self.executor.clone())
    }

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

    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_add::AddPlan::prepare_commit(&self.workspace_root, input, provenance)
            .map(|plan| {
                #[cfg(any(unix, windows))]
                let plan = plan.bind_operation(self.executor.operation());
                Some(Box::new(plan) as Box<dyn talos_core::tool::PreparedNormalToolInvocation>)
            })
            .map_err(|error| error.to_string())
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::error(
            "git_commit requires a prepared invocation through the Agent or MCP permission pipeline; direct execution is not authorized",
        )
    }
}

// Positional values must not become host Git options. Keep ordinary remote
// URLs and refspec syntax intact; this is not a substitute for effect admission.
fn validate_git_positional(value: &str) -> Result<(), GitToolError> {
    if value.is_empty() || value.starts_with('-') || value.contains('\0') {
        return Err(GitToolError::InvalidInput(
            "Git positional argument must be nonempty and cannot start with '-' or contain NUL"
                .into(),
        ));
    }
    Ok(())
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
    #[cfg(any(unix, windows))]
    executor: std::sync::Arc<OwnedCommandExecutor>,
}

impl GitPushTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(any(unix, windows))]
            executor: std::sync::Arc::new(OwnedCommandExecutor::default()),
        }
    }
    #[cfg(test)]
    async fn execute_prepared(&self, input: Value) -> ToolResult {
        match prepared_add::AddPlan::prepare_push(
            &self.workspace_root,
            &input,
            talos_core::tool::ToolProvenance::Native,
        ) {
            Ok(plan) => prepared_add::tests::execute_approved(plan).await.result,
            Err(error) => ToolResult::error(error.to_string()),
        }
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
    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }
    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        Some(self.executor.clone())
    }
    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_add::AddPlan::prepare_push(&self.workspace_root, input, provenance)
            .map(|plan| {
                #[cfg(any(unix, windows))]
                let plan = plan.bind_operation(self.executor.operation());
                Some(Box::new(plan) as Box<dyn talos_core::tool::PreparedNormalToolInvocation>)
            })
            .map_err(|error| error.to_string())
    }
    async fn execute(&self, _: Value) -> ToolResult {
        ToolResult::error(
            "git_push requires a prepared invocation through the Agent or MCP permission pipeline",
        )
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
    #[cfg(any(unix, windows))]
    executor: std::sync::Arc<OwnedCommandExecutor>,
}

impl GitPullTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(any(unix, windows))]
            executor: std::sync::Arc::new(OwnedCommandExecutor::default()),
        }
    }
}

#[async_trait]
impl AgentTool for GitPullTool {
    fn name(&self) -> &str {
        "git_pull"
    }
    fn description(&self) -> &str {
        "Pull using configured native Git behavior after configuration discovery and whole-command authorization; may run configured programs and leave partial effects"
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

    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        Some(self.executor.clone())
    }

    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_add::AddPlan::prepare_pull(&self.workspace_root, input, provenance)
            .map(|plan| {
                #[cfg(any(unix, windows))]
                let plan = plan.bind_operation(self.executor.operation());
                Some(Box::new(plan) as Box<dyn talos_core::tool::PreparedNormalToolInvocation>)
            })
            .map_err(|error| error.to_string())
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
        let input: GitPullInput = serde_json::from_value(input)
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        validate_git_positional(input.remote.as_deref().unwrap_or("origin"))?;
        Err(GitToolError::InvalidInput(
            "git_pull requires a prepared invocation; fetch, merge/rebase, hooks, worktree and remote effects are not yet represented".into(),
        ))
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
    #[cfg(any(unix, windows))]
    executor: std::sync::Arc<OwnedCommandExecutor>,
}

impl GitCheckoutTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            #[cfg(any(unix, windows))]
            executor: std::sync::Arc::new(OwnedCommandExecutor::default()),
        }
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

    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        Some(self.executor.clone())
    }
    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: talos_core::tool::ToolProvenance,
    ) -> Result<Option<Box<dyn talos_core::tool::PreparedNormalToolInvocation>>, String> {
        prepared_add::AddPlan::prepare_checkout(&self.workspace_root, input, provenance)
            .map(|plan| {
                #[cfg(any(unix, windows))]
                let plan = plan.bind_operation(self.executor.operation());
                Some(Box::new(plan) as Box<dyn talos_core::tool::PreparedNormalToolInvocation>)
            })
            .map_err(|error| error.to_string())
    }
    async fn execute(&self, _: Value) -> ToolResult {
        ToolResult::error(
            "git_checkout requires a prepared invocation through the Agent or MCP permission pipeline",
        )
    }
}

#[cfg(test)]
impl GitCheckoutTool {
    async fn execute_prepared(&self, input: Value) -> ToolResult {
        let plan = match prepared_add::AddPlan::prepare_checkout(
            &self.workspace_root,
            &input,
            talos_core::tool::ToolProvenance::Native,
        ) {
            Ok(plan) => plan,
            Err(error) => return ToolResult::error(error.to_string()),
        };
        prepared_add::tests::execute_approved(plan).await.result
    }
}

#[cfg(test)]
mod positional_argument_tests {
    use super::*;

    #[tokio::test]
    async fn checkout_branch_name_never_restores_a_matching_file() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("host Git");
            assert!(output.status.success(), "{output:?}");
            output.stdout
        };
        git(&["init", "-b", "main"]);
        let path = root.path().join("topic");
        std::fs::write(&path, "committed\n").expect("file");
        git(&["add", "topic"]);
        git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "-m",
            "initial",
        ]);
        std::fs::write(&path, "uncommitted work\n").expect("edit");
        let index = std::fs::read(root.path().join(".git/index")).expect("index");
        let tool = GitCheckoutTool::new(root.path().to_path_buf());
        let result = tool
            .execute_prepared(serde_json::json!({"branch":"topic"}))
            .await;
        assert!(
            result.is_error,
            "must reject a file-only name: {}",
            result.content
        );
        assert_eq!(
            std::fs::read(&path).expect("preserved file"),
            b"uncommitted work\n"
        );
        assert_eq!(
            std::fs::read(root.path().join(".git/index")).expect("index"),
            index
        );
        assert_eq!(git(&["branch", "--show-current"]), b"main\n");
        let result = tool
            .execute_prepared(serde_json::json!({"branch":"topic", "create":true}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(git(&["branch", "--show-current"]), b"topic\n");
        assert_eq!(
            std::fs::read(&path).expect("preserved file"),
            b"uncommitted work\n"
        );
    }

    #[tokio::test]
    async fn checkout_preserves_unique_remote_tracking_guess() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("host Git");
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout).expect("text")
        };
        git(&["init", "-b", "main"]);
        git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        // Configure a local remote without performing any fetch or network call.
        git(&["remote", "add", "origin", "."]);
        git(&["update-ref", "refs/remotes/origin/topic", "HEAD"]);
        let result = GitCheckoutTool::new(root.path().to_path_buf())
            .execute_prepared(serde_json::json!({"branch":"topic"}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(git(&["branch", "--show-current"]).trim(), "topic");
        assert_eq!(git(&["config", "branch.topic.remote"]).trim(), "origin");
        assert_eq!(
            git(&["config", "branch.topic.merge"]).trim(),
            "refs/heads/topic"
        );
    }

    #[tokio::test]
    async fn checkout_preserves_guess_with_negative_fetch_mapping() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("host Git");
            assert!(output.status.success(), "{output:?}");
            String::from_utf8(output.stdout).expect("text")
        };
        git(&["init", "-b", "main"]);
        git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        // Configure a local remote without performing any fetch or network call.
        git(&["remote", "add", "origin", "."]);
        git(&["update-ref", "refs/remotes/origin/topic", "HEAD"]);
        git(&[
            "config",
            "--add",
            "remote.origin.fetch",
            "^refs/heads/topic",
        ]);
        let result = GitCheckoutTool::new(root.path().to_path_buf())
            .execute_prepared(serde_json::json!({"branch":"topic"}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(git(&["branch", "--show-current"]).trim(), "topic");
        let tracking = std::process::Command::new("git")
            .args(["config", "branch.topic.remote"])
            .current_dir(root.path())
            .output()
            .expect("tracking");
        assert!(
            !tracking.status.success(),
            "negative mapping suppresses automatic tracking configuration"
        );
    }

    #[tokio::test]
    async fn checkout_previous_branch_shorthand_preserves_branch_switch() {
        let root = tempfile::tempdir().expect("repository workspace");
        let run = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("host Git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).expect("Git fixture output")
        };
        run(&["init", "-b", "main"]);
        run(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ]);
        run(&["checkout", "-b", "topic"]);
        let tool = GitCheckoutTool::new(root.path().to_path_buf());
        let result = tool
            .execute_prepared(serde_json::json!({"branch":"-"}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(run(&["branch", "--show-current"]).trim(), "main");
        let result = tool
            .execute_prepared(serde_json::json!({"branch":"-"}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(run(&["branch", "--show-current"]).trim(), "topic");
        let result = tool
            .execute_prepared(serde_json::json!({"branch":"-", "create":true}))
            .await;
        assert!(result.is_error);
        assert!(result.content.contains("Git positional argument"));
        assert_eq!(run(&["branch", "--show-current"]).trim(), "topic");
    }

    #[tokio::test]
    async fn options_are_rejected_before_repository_or_host_access() {
        let root = tempfile::tempdir().expect("non-repository workspace");
        for value in ["", "--upload-pack=unexpected", "-f", "bad\0argument"] {
            for result in [
                GitPushTool::new(root.path().to_path_buf())
                    .execute_prepared(serde_json::json!({"remote":value}))
                    .await,
                GitPushTool::new(root.path().to_path_buf())
                    .execute_prepared(serde_json::json!({"branch":value}))
                    .await,
                GitPullTool::new(root.path().to_path_buf())
                    .execute(serde_json::json!({"remote":value}))
                    .await,
                GitCheckoutTool::new(root.path().to_path_buf())
                    .execute_prepared(serde_json::json!({"branch":value}))
                    .await,
                GitCheckoutTool::new(root.path().to_path_buf())
                    .execute_prepared(serde_json::json!({"branch":value,"create":true}))
                    .await,
            ] {
                assert!(result.is_error);
                assert!(
                    result.content.contains("Git positional argument"),
                    "{}",
                    result.content
                );
            }
        }
    }

    #[test]
    fn ordinary_remote_and_refspec_values_remain_valid() {
        for value in [
            "origin",
            "https://example.invalid/repo",
            "git@example.invalid:repo",
            "main",
            "HEAD:refs/heads/topic",
        ] {
            assert!(validate_git_positional(value).is_ok());
        }
    }
}
