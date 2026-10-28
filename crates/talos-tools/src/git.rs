//! Built-in Git tools using the gix crate (pure Rust Git implementation).
//!
//! Read-only tools are implemented natively via gix.
//! Write tools use a structured host git fallback while native gix write
//! orchestration remains under evaluation.

use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use gix::bstr::ByteSlice;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;
use thiserror::Error;
use tokio::io::AsyncReadExt;

const HOST_DIFF_OUTPUT_LIMIT: u64 = 256 * 1024;

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
    GitAddInput, GitAddTool, GitCheckoutInput, GitCheckoutTool, GitCommitInput, GitCommitTool,
    GitPullInput, GitPullTool, GitPushInput, GitPushTool,
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

        if git_input.base_ref.is_some() != git_input.head_ref.is_some() {
            return Err(GitToolError::InvalidInput(
                "base_ref and head_ref must be provided together".into(),
            ));
        }
        if let (Some(base), Some(head)) = (&git_input.base_ref, &git_input.head_ref) {
            let (workdir, base_id, head_id) = {
                let repo = discover_repo(&self.workspace_root)?;
                let workdir = repo
                    .workdir()
                    .ok_or_else(|| GitToolError::Git("no workdir (bare repo?)".into()))?
                    .to_path_buf();
                let resolve = |reference: &str| {
                    if reference.is_empty() || reference.starts_with('-') {
                        return Err(GitToolError::InvalidInput("invalid diff ref".into()));
                    }
                    repo.rev_parse_single(reference)
                        .map(|id| id.to_hex().to_string())
                        .map_err(|e| GitToolError::Git(format!("diff ref not found: {e}")))
                };
                (workdir, resolve(base)?, resolve(head)?)
            };

            let mut cmd = tokio::process::Command::new("git");
            cmd.arg("--literal-pathspecs")
                .arg("diff")
                .arg("--no-ext-diff")
                .arg("--no-textconv")
                .arg("--no-color")
                .arg(&base_id)
                .arg(&head_id)
                .current_dir(&workdir)
                .kill_on_drop(true);
            if let Some(p) = path_filter {
                cmd.arg("--").arg(p);
            }
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
            let mut child = cmd
                .spawn()
                .map_err(|e| GitToolError::Git(format!("host git not available: {e}")))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| GitToolError::Git("missing git stdout pipe".into()))?;
            let stderr = child
                .stderr
                .take()
                .ok_or_else(|| GitToolError::Git("missing git stderr pipe".into()))?;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
            let read_output = async {
                let stdout_read = async {
                    let mut bytes = Vec::new();
                    stdout
                        .take(HOST_DIFF_OUTPUT_LIMIT + 1)
                        .read_to_end(&mut bytes)
                        .await?;
                    if bytes.len() as u64 > HOST_DIFF_OUTPUT_LIMIT {
                        return Err(std::io::Error::other(
                            "git diff output exceeds 256 KiB limit",
                        ));
                    }
                    Ok::<_, std::io::Error>(bytes)
                };
                let stderr_read = async {
                    let mut bytes = Vec::new();
                    stderr
                        .take(HOST_DIFF_OUTPUT_LIMIT + 1)
                        .read_to_end(&mut bytes)
                        .await?;
                    if bytes.len() as u64 > HOST_DIFF_OUTPUT_LIMIT {
                        return Err(std::io::Error::other(
                            "git diff output exceeds 256 KiB limit",
                        ));
                    }
                    Ok::<_, std::io::Error>(bytes)
                };
                tokio::try_join!(stdout_read, stderr_read)
            };
            let (stdout, stderr) = match tokio::time::timeout_at(deadline, read_output).await {
                Ok(Ok(output)) => output,
                Ok(Err(error)) => {
                    let _ = child.kill().await;
                    return Err(GitToolError::Git(format!(
                        "cannot read git diff output: {error}"
                    )));
                }
                Err(_) => {
                    let _ = child.kill().await;
                    return Err(GitToolError::Git(format!(
                        "git diff {base}..{head} timed out after 10 seconds"
                    )));
                }
            };
            let status = tokio::time::timeout_at(deadline, child.wait())
                .await
                .map_err(|_| GitToolError::Git("git diff process timed out".into()))?
                .map_err(|e| GitToolError::Git(format!("cannot wait for git diff: {e}")))?;
            if !status.success() {
                let stderr = String::from_utf8_lossy(&stderr);
                return Err(GitToolError::Git(format!(
                    "git diff {base}..{head} failed: {stderr}"
                )));
            }
            let stdout = String::from_utf8_lossy(&stdout);
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

        let trust_file_mode = repo
            .config_snapshot()
            .try_boolean("core.fileMode")
            .map_err(|error| GitToolError::Git(format!("invalid core.fileMode: {error}")))?
            .unwrap_or(true);

        let mut paths = std::collections::BTreeSet::new();
        let mut submodule_content = std::collections::BTreeMap::new();
        if staged {
            let index = repo
                .index_or_empty()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            for entry in index.entries() {
                paths.insert(entry.path(&index).to_vec());
            }
            let head = repo.head().map_err(|e| GitToolError::Git(e.to_string()))?;
            if !head.is_unborn() {
                let tree = repo
                    .head_tree()
                    .map_err(|e| GitToolError::Git(e.to_string()))?;
                for entry in tree
                    .traverse()
                    .breadthfirst
                    .files()
                    .map_err(|e| GitToolError::Git(e.to_string()))?
                {
                    if !entry.mode.is_tree() {
                        paths.insert(entry.filepath.to_vec());
                    }
                }
            }
        } else {
            let platform = repo
                .status(gix::progress::Discard)
                .map_err(|e| GitToolError::Git(e.to_string()))?
                .untracked_files(gix::status::UntrackedFiles::None)
                .index_worktree_submodules(None);
            let iter = platform
                .into_index_worktree_iter(Vec::<gix::bstr::BString>::new())
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            for item in iter {
                let item = item.map_err(|e| GitToolError::Git(e.to_string()))?;
                paths.insert(item.rela_path().to_vec());
            }
            if let Some(submodules) = repo
                .submodules()
                .map_err(|error| GitToolError::Git(error.to_string()))?
            {
                use gix::submodule::config::Ignore;
                let global_ignore = repo
                    .config_snapshot()
                    .string("diff.ignoreSubmodules")
                    .map(|value| {
                        Ignore::try_from(value.as_bstr())
                            .map_err(|()| GitToolError::Git("invalid diff.ignoreSubmodules".into()))
                    })
                    .transpose()?;
                let index = repo
                    .index_or_empty()
                    .map_err(|error| GitToolError::Git(error.to_string()))?;
                for submodule in submodules {
                    let path = submodule
                        .path()
                        .map_err(|error| GitToolError::Git(error.to_string()))?
                        .to_vec();
                    if !diff_path_matches(&path, path_filter) {
                        continue;
                    }
                    // .gitmodules may outlive removal or a staged type change.
                    // Only current, resolved gitlinks participate in child status.
                    if !index.entry_by_path(path.as_bstr()).is_some_and(|entry| {
                        entry.stage_raw() == 0 && entry.mode.bits() == 0o160000
                    }) {
                        continue;
                    }
                    // Git diff defaults to ignoring untracked child files and
                    // lets each module override the global diff setting.
                    let ignore = submodule
                        .ignore()
                        .map_err(|error| GitToolError::Git(error.to_string()))?
                        .or(global_ignore)
                        .unwrap_or(Ignore::Untracked);
                    let status = submodule
                        .status(ignore, false)
                        .map_err(|error| GitToolError::Git(error.to_string()))?;
                    if status.is_dirty() != Some(true) {
                        continue;
                    }
                    let id = status.checked_out_head_id.ok_or_else(|| {
                        GitToolError::Git("cannot resolve changed submodule HEAD".into())
                    })?;
                    let suffix = if status
                        .changes
                        .as_ref()
                        .is_some_and(|changes| !changes.is_empty())
                    {
                        "-dirty"
                    } else {
                        ""
                    };
                    paths.insert(path.clone());
                    submodule_content.insert(
                        path,
                        format!("Subproject commit {id}{suffix}\n").into_bytes(),
                    );
                }
            }
        }

        let mut output = String::new();
        let mut line_count = 0usize;

        for path_bytes in paths {
            if !diff_path_matches(&path_bytes, path_filter) {
                continue;
            }

            // Git paths are byte strings. Never use a lossy display conversion for
            // an object lookup or a filesystem access: distinct entries may collapse.
            let path = gix::path::from_bstr(path_bytes.as_bstr())
                .map_err(|e| GitToolError::Git(format!("unrepresentable diff path: {e}")))?;
            let path_str = diff_path_label(&path_bytes);

            let old_content = if staged {
                read_head_blob_content(&repo, &path_bytes)?
            } else {
                read_index_blob_content(&repo, &path_bytes)?
            };
            let new_content = if staged {
                read_index_blob_content(&repo, &path_bytes)?
            } else if let Some(content) = submodule_content.get(&path_bytes) {
                Some(content.clone())
            } else {
                read_worktree_blob_content(&workdir, path.as_ref())?
            };
            let old_mode = if staged {
                read_head_mode(&repo, &path_bytes)?
            } else {
                read_index_mode(&repo, &path_bytes)?
            };
            let new_mode = if staged {
                read_index_mode(&repo, &path_bytes)?
            } else if submodule_content.contains_key(&path_bytes) {
                Some(0o160000)
            } else {
                read_worktree_mode(&workdir, path.as_ref())?
            };

            // core.fileMode affects only executable-bit comparisons for ordinary
            // worktree files, never staged modes or file/symlink type changes.
            // Platforms without Unix mode bits retain the index executable bit,
            // while still reporting the worktree entry's file type.
            let new_mode = if !staged
                && (!trust_file_mode || !cfg!(unix))
                && matches!(old_mode, Some(0o100644 | 0o100755))
                && matches!(new_mode, Some(0o100644 | 0o100755))
            {
                old_mode
            } else {
                new_mode
            };

            if old_content == new_content && (new_mode.is_none() || old_mode == new_mode) {
                continue;
            }
            let old_label = if old_content.is_some() {
                format!("a/{path_str}")
            } else {
                "/dev/null".into()
            };
            let new_label = if new_content.is_some() {
                format!("b/{path_str}")
            } else {
                "/dev/null".into()
            };
            let old_bytes = old_content.as_deref().unwrap_or_default();
            let new_bytes = new_content.as_deref().unwrap_or_default();
            let mut file_diff = format!("diff --git a/{path_str} b/{path_str}\n");
            match (old_mode, new_mode) {
                (None, Some(mode)) => file_diff.push_str(&format!("new file mode {mode:06o}\n")),
                (Some(mode), None) if new_content.is_none() => {
                    file_diff.push_str(&format!("deleted file mode {mode:06o}\n"));
                }
                (Some(old), Some(new)) if old != new => {
                    file_diff.push_str(&format!("old mode {old:06o}\nnew mode {new:06o}\n"));
                }
                _ => {}
            }
            if old_content != new_content {
                match (
                    std::str::from_utf8(old_bytes),
                    std::str::from_utf8(new_bytes),
                ) {
                    (Ok(old), Ok(new)) if !old_bytes.contains(&0) && !new_bytes.contains(&0) => {
                        let diff = similar::TextDiff::from_lines(old, new);
                        let patch = diff
                            .unified_diff()
                            .header(&old_label, &new_label)
                            .context_radius(3)
                            .to_string();

                        file_diff.push_str(&patch);
                    }
                    _ => {
                        file_diff.push_str(&format!(
                            "Binary files {old_label} and {new_label} differ\n"
                        ));
                    }
                }
            }
            for line in file_diff.lines() {
                if line_count >= max_lines {
                    output.push_str(&format!("... (truncated at {max_lines} lines)\n"));
                    return Ok(output.trim_end().to_string());
                }
                output.push_str(line);
                output.push('\n');
                line_count += 1;
            }
        }

        if output.is_empty() {
            Ok("no changes".to_string())
        } else {
            Ok(output.trim_end().to_string())
        }
    }
}

fn diff_path_matches(path: &[u8], filter: Option<&str>) -> bool {
    let Some(filter) = filter else {
        return true;
    };
    let filter = filter.trim_end_matches('/').as_bytes();
    path == filter
        || path
            .strip_prefix(filter)
            .is_some_and(|suffix| suffix.starts_with(b"/"))
}

// Display escapes are deliberately separate from the raw identity used above.
fn diff_path_label(path: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(path)
        && !text.chars().any(|c| c.is_control() || c == '\\')
    {
        return text.to_owned();
    }
    let mut label = String::new();
    for &byte in path {
        if byte.is_ascii_graphic() && byte != b'\\' {
            label.push(char::from(byte));
        } else {
            label.push_str(&format!("\\{byte:03o}"));
        }
    }
    label
}

fn read_head_blob_content(
    repo: &gix::Repository,
    path: &[u8],
) -> Result<Option<Vec<u8>>, GitToolError> {
    if repo
        .head()
        .map_err(|e| GitToolError::Git(e.to_string()))?
        .is_unborn()
    {
        return Ok(None);
    }
    let tree = repo
        .head_tree()
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    let Some(entry) = tree
        .lookup_entry(path.split(|byte| *byte == b'/'))
        .map_err(|e| GitToolError::Git(e.to_string()))?
    else {
        return Ok(None);
    };
    if u32::from(entry.mode().value()) == 0o160000 {
        return Ok(Some(
            format!("Subproject commit {}\n", entry.object_id()).into_bytes(),
        ));
    }
    let blob = entry
        .object()
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    Ok(Some(blob.data.to_vec()))
}

fn read_head_mode(repo: &gix::Repository, path: &[u8]) -> Result<Option<u32>, GitToolError> {
    if repo
        .head()
        .map_err(|e| GitToolError::Git(e.to_string()))?
        .is_unborn()
    {
        return Ok(None);
    }
    let tree = repo
        .head_tree()
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    tree.lookup_entry(path.split(|byte| *byte == b'/'))
        .map(|entry| entry.map(|entry| u32::from(entry.mode().value())))
        .map_err(|e| GitToolError::Git(e.to_string()))
}

fn read_index_blob_content(
    repo: &gix::Repository,
    path: &[u8],
) -> Result<Option<Vec<u8>>, GitToolError> {
    use gix::bstr::ByteSlice;
    let index = repo
        .index_or_empty()
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    let Some(entry) = index.entry_by_path(path.as_bstr()) else {
        return Ok(None);
    };
    if entry.stage_raw() != 0 {
        return Err(GitToolError::Git(format!(
            "cannot show ordinary diff for unresolved index path {}",
            diff_path_label(path)
        )));
    }
    if entry.mode.bits() == 0o160000 {
        return Ok(Some(
            format!("Subproject commit {}\n", entry.id).into_bytes(),
        ));
    }
    let blob = repo
        .find_object(entry.id)
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    Ok(Some(blob.data.to_vec()))
}

fn read_index_mode(repo: &gix::Repository, path: &[u8]) -> Result<Option<u32>, GitToolError> {
    use gix::bstr::ByteSlice;
    let index = repo
        .index_or_empty()
        .map_err(|e| GitToolError::Git(e.to_string()))?;
    let Some(entry) = index.entry_by_path(path.as_bstr()) else {
        return Ok(None);
    };
    if entry.stage_raw() != 0 {
        return Err(GitToolError::Git(format!(
            "cannot show ordinary diff for unresolved index path {}",
            diff_path_label(path)
        )));
    }
    Ok(Some(entry.mode.bits()))
}

fn validate_worktree_ancestors(workdir: &Path, path: &Path) -> Result<bool, GitToolError> {
    let relative = path
        .strip_prefix(workdir)
        .map_err(|_| GitToolError::Git("diff path escapes the canonical worktree".into()))?;
    let mut current = workdir.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(GitToolError::Git(
                "diff path contains an invalid component".into(),
            ));
        };
        current.push(name);
        if current != path {
            let metadata = match std::fs::symlink_metadata(&current) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
                Err(error) => {
                    return Err(GitToolError::Git(format!(
                        "cannot inspect diff ancestor {}: {error}",
                        current.display()
                    )));
                }
            };
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(GitToolError::Git(format!(
                    "diff path ancestor {} is not a regular directory",
                    current.display()
                )));
            }
        }
    }
    Ok(true)
}

fn read_worktree_mode(workdir: &Path, path: &Path) -> Result<Option<u32>, GitToolError> {
    let path = workdir.join(path);
    if !validate_worktree_ancestors(workdir, &path)? {
        return Ok(None);
    }
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(GitToolError::Git(format!(
                "cannot inspect diff path {}: {error}",
                path.display()
            )));
        }
    };
    if metadata.file_type().is_symlink() {
        return Ok(Some(0o120000));
    }
    if !metadata.is_file() {
        return Err(GitToolError::Git(format!(
            "diff path {} is not a regular file or symlink",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(Some(if metadata.permissions().mode() & 0o100 == 0 {
            0o100644
        } else {
            0o100755
        }))
    }
    #[cfg(not(unix))]
    Ok(Some(0o100644))
}

fn read_worktree_blob_content(
    workdir: &Path,
    relative: &Path,
) -> Result<Option<Vec<u8>>, GitToolError> {
    let path = workdir.join(relative);
    if !validate_worktree_ancestors(workdir, &path)? {
        return Ok(None);
    }
    let metadata = match std::fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(GitToolError::Git(format!(
                "cannot inspect diff path {}: {error}",
                path.display()
            )));
        }
    };
    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(&path).map_err(|error| {
            GitToolError::Git(format!(
                "cannot read diff symlink {}: {error}",
                path.display()
            ))
        })?;
        return Ok(Some(target.as_os_str().as_encoded_bytes().to_vec()));
    }
    if !metadata.is_file() {
        return Err(GitToolError::Git(format!(
            "diff path {} is not a regular file or symlink",
            path.display()
        )));
    }
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(GitToolError::Git(format!(
            "cannot read diff path {}: {error}",
            path.display()
        ))),
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
