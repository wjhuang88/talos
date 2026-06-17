//! Built-in agent tools for Talos.
//!
//! This crate provides implementations of the [`AgentTool`] trait for common
//! agent operations such as shell command execution, file operations, and
//! AST-aware symbol queries.

pub mod symbol;

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

/// Errors that can occur during bash tool execution.
#[derive(Debug, Error)]
pub enum BashError {
    /// The input does not conform to the expected schema.
    #[error("invalid bash input: {0}")]
    InvalidInput(String),
}

/// Input parameters for the [`BashTool`].
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BashInput {
    /// The shell command to execute.
    pub command: String,
}

/// A tool that executes shell commands via `sh -c`.
///
/// Commands are run with a configurable timeout and working directory.
/// Output is captured from both stdout and stderr.
pub struct BashTool {
    working_dir: PathBuf,
    timeout: Duration,
}

impl BashTool {
    /// Creates a new `BashTool` with the given working directory.
    ///
    /// The default timeout is 120 seconds.
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            timeout: Duration::from_secs(120),
        }
    }

    /// Sets a custom timeout for command execution.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Returns the working directory for this tool.
    pub fn working_dir(&self) -> &PathBuf {
        &self.working_dir
    }

    /// Returns the timeout duration for this tool.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell command"
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(BashInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let bash_input = match parse_input(input) {
            Ok(i) => i,
            Err(e) => return ToolResult::error(e.to_string()),
        };

        let result = timeout(self.timeout, self.run_command(&bash_input.command)).await;

        match result {
            Ok(exec_result) => exec_result,
            Err(_) => ToolResult::error(format!(
                "command timed out after {}ms",
                self.timeout.as_millis()
            )),
        }
    }
}

impl BashTool {
    async fn run_command(&self, command: &str) -> ToolResult {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command).current_dir(&self.working_dir);

        #[cfg(unix)]
        {
            let c_names: Vec<std::ffi::CString> =
                talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names()
                    .into_iter()
                    .map(|s| std::ffi::CString::new(s).expect("valid env var name"))
                    .collect();

            // SAFETY: pre_exec closure runs post-fork/pre-exec, async-signal-safe per ADR-007.
            // Only libc::unsetenv and libc::setrlimit are called — both async-signal-safe.
            // No allocation, locking, formatting, or panics inside the closure.
            unsafe {
                cmd.pre_exec(move || {
                    for c_name in &c_names {
                        // SAFETY: c_name.as_ptr() is a valid NUL-terminated pointer.
                        // libc::unsetenv is async-signal-safe (POSIX.1-2008).
                        // ADR-007 pre-authorizes this unsafe site.
                        libc::unsetenv(c_name.as_ptr());
                    }

                    let rlim = libc::rlimit {
                        rlim_cur: 0,
                        rlim_max: 0,
                    };
                    // SAFETY: valid rlimit struct, well-defined POSIX constant.
                    // ADR-007 pre-authorizes this unsafe site.
                    libc::setrlimit(libc::RLIMIT_CORE, &rlim as *const _);

                    let rlim = libc::rlimit {
                        rlim_cur: 300,
                        rlim_max: 300,
                    };
                    // SAFETY: valid rlimit struct, well-defined POSIX constant.
                    // ADR-007 pre-authorizes this unsafe site.
                    libc::setrlimit(libc::RLIMIT_CPU, &rlim as *const _);

                    let rlim = libc::rlimit {
                        rlim_cur: 2 * 1024 * 1024 * 1024,
                        rlim_max: 2 * 1024 * 1024 * 1024,
                    };
                    // SAFETY: valid rlimit struct, well-defined POSIX constant.
                    // ADR-007 pre-authorizes this unsafe site.
                    libc::setrlimit(libc::RLIMIT_AS, &rlim as *const _);

                    Ok(())
                });
            }
        }

        let output = match cmd.output().await {
            Ok(o) => o,
            Err(e) => return ToolResult::error(format!("failed to spawn shell: {e}")),
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut content = String::new();
        if !stdout.is_empty() {
            content.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&stderr);
        }

        let is_error = !output.status.success();

        ToolResult { content, is_error }
    }
}

/// Parses a JSON [`Value`] into a [`BashInput`].
fn parse_input(input: Value) -> Result<BashInput, BashError> {
    let obj = input
        .as_object()
        .ok_or_else(|| BashError::InvalidInput("expected a JSON object".to_owned()))?;

    let command = obj
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| BashError::InvalidInput("missing required field 'command'".to_owned()))?;

    Ok(BashInput {
        command: command.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[tokio::test]
    async fn test_echo_returns_stdout() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "echo hello" }))
            .await;

        assert!(!result.is_error);
        assert_eq!(result.content.trim(), "hello");
    }

    #[tokio::test]
    async fn test_invalid_command_returns_error() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "nonexistent_command_xyz_123" }))
            .await;

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_timeout_works() {
        let tool = BashTool::new(test_dir()).with_timeout(Duration::from_millis(100));
        let result = tool
            .execute(serde_json::json!({ "command": "sleep 10" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("timed out"));
    }

    #[tokio::test]
    async fn test_shell_metacharacters_pipe() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "echo hello | tr a-z A-Z" }))
            .await;

        assert!(!result.is_error);
        assert_eq!(result.content.trim(), "HELLO");
    }

    #[tokio::test]
    async fn test_shell_metacharacters_redirect() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({
                "command": "echo test123 > /tmp/talos_bash_test.txt && cat /tmp/talos_bash_test.txt && rm /tmp/talos_bash_test.txt"
            }))
            .await;

        assert!(!result.is_error);
        assert_eq!(result.content.trim(), "test123");
    }

    #[tokio::test]
    async fn test_working_directory_restriction() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "basename $(pwd)" }))
            .await;

        assert!(!result.is_error);
        assert_eq!(result.content.trim(), "talos-tools");
    }

    #[tokio::test]
    async fn test_missing_command_field() {
        let tool = BashTool::new(test_dir());
        let result = tool.execute(serde_json::json!({})).await;

        assert!(result.is_error);
        assert!(result.content.contains("missing required field 'command'"));
    }

    #[tokio::test]
    async fn test_non_object_input() {
        let tool = BashTool::new(test_dir());
        let result = tool.execute(serde_json::json!("not an object")).await;

        assert!(result.is_error);
        assert!(result.content.contains("expected a JSON object"));
    }

    #[test]
    fn test_bash_tool_name() {
        let tool = BashTool::new(test_dir());
        assert_eq!(tool.name(), "bash");
    }

    #[test]
    fn test_bash_tool_description() {
        let tool = BashTool::new(test_dir());
        assert_eq!(tool.description(), "Execute a shell command");
    }

    #[test]
    fn test_bash_tool_not_read_only() {
        let tool = BashTool::new(test_dir());
        assert!(!tool.is_read_only());
    }

    #[test]
    fn test_bash_tool_parameters_schema() {
        let tool = BashTool::new(test_dir());
        let schema = tool.parameters();

        assert!(schema.is_object());
        let obj = schema.as_object().unwrap();
        assert!(obj.contains_key("properties"));
    }

    #[test]
    fn test_bash_tool_default_timeout() {
        let tool = BashTool::new(test_dir());
        assert_eq!(tool.timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_bash_tool_custom_timeout() {
        let tool = BashTool::new(test_dir()).with_timeout(Duration::from_secs(30));
        assert_eq!(tool.timeout(), Duration::from_secs(30));
    }
}

/// Size threshold for binary file detection (8KB).
const BINARY_CHECK_SIZE: usize = 8 * 1024;

/// Errors that can occur during file tool operations.
#[derive(Debug, Error)]
pub enum FileToolError {
    #[error("path escapes workspace root: {0}")]
    PathEscape(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("file appears to be binary: {0}")]
    BinaryFile(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("string not found in file: {0}")]
    StringNotFound(String),

    #[error("invalid line range: start_line ({start}) > end_line ({end})")]
    InvalidLineRange { start: u32, end: u32 },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("file already exists: {0}. Use the edit tool to modify existing files")]
    FileExists(String),
}

/// Resolves a relative path against the workspace root and validates
/// that the resulting path stays within the workspace.
fn resolve_workspace_path(workspace_root: &Path, relative: &str) -> Result<PathBuf, FileToolError> {
    let canon_root = workspace_root.canonicalize()?;

    let joined = if relative.starts_with('/') {
        PathBuf::from(relative)
    } else {
        workspace_root.join(relative)
    };

    let canonical = if joined.exists() {
        joined.canonicalize()?
    } else if let Some(parent) = joined.parent() {
        if parent.exists() {
            let canon_parent = parent.canonicalize()?;
            let file_name = joined
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            canon_parent.join(file_name)
        } else {
            canon_root.join(relative)
        }
    } else {
        canon_root.join(relative)
    };

    if !canonical.starts_with(&canon_root) {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }

    Ok(canonical)
}

/// Checks if a file appears to be binary by looking for null bytes
/// in the first 8KB of content.
fn is_binary_file(path: &Path) -> Result<bool, FileToolError> {
    let bytes = std::fs::read(path)?;
    let check_bytes = &bytes[..bytes.len().min(BINARY_CHECK_SIZE)];
    Ok(check_bytes.contains(&0u8))
}

/// Input parameters for the [`ReadTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadInput {
    pub path: String,
    /// Starting line number (1-based). Prefer `offset` for new code.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub start_line: Option<u32>,
    /// Ending line number (1-based, inclusive). Prefer `limit` for new code.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub end_line: Option<u32>,
    /// 0-based line offset for pagination. `offset=0` starts at line 1.
    /// Takes precedence over `start_line`/`end_line` when specified.
    #[serde(default)]
    #[schemars(range(min = 0))]
    pub offset: Option<u32>,
    /// Maximum number of lines to return. Defaults to 2000 when `offset` is set.
    /// Takes precedence over `start_line`/`end_line` when specified.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub limit: Option<u32>,
}

/// A tool that reads file content with optional line range support.
pub struct ReadTool {
    workspace_root: PathBuf,
}

impl ReadTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file content with optional line range or offset/limit pagination"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(ReadInput)
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
}

impl ReadTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let read_input: ReadInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &read_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(read_input.path));
        }

        if is_binary_file(&path)? {
            return Err(FileToolError::BinaryFile(read_input.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let (start, max_lines) = if read_input.offset.is_some() || read_input.limit.is_some() {
            let offset = read_input.offset.unwrap_or(0) as usize;
            let limit = read_input.limit.unwrap_or(2000) as usize;
            (offset, limit)
        } else if read_input.start_line.is_some() || read_input.end_line.is_some() {
            let start = read_input.start_line.unwrap_or(1).saturating_sub(1) as usize;
            let end = match read_input.end_line {
                Some(e) => e as usize,
                None => lines.len(),
            };
            if start > end {
                return Err(FileToolError::InvalidLineRange {
                    start: read_input.start_line.unwrap_or(1),
                    end: read_input.end_line.unwrap_or(lines.len() as u32),
                });
            }
            (start, end.saturating_sub(start))
        } else {
            (0, lines.len())
        };

        let start = start.min(total_lines);
        let end = (start + max_lines).min(total_lines);
        let selected = &lines[start..end];

        let mut output = String::new();
        for (i, line) in selected.iter().enumerate() {
            let line_num = start + i + 1;
            output.push_str(&format!("{line_num}: {line}\n"));
        }

        let remaining = total_lines.saturating_sub(end);
        if remaining > 0 {
            let next_offset = end;
            output.push_str(&format!(
                "\n... ({remaining} more lines, use offset={next_offset} to continue)"
            ));
        }

        Ok(output)
    }
}

/// Input parameters for the [`WriteTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteInput {
    pub path: String,
    pub content: String,
}

/// A tool that creates a new file with the given content.
/// Does not overwrite existing files — use the edit tool for modifications.
pub struct WriteTool {
    workspace_root: PathBuf,
}

impl WriteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Create a new file (does not overwrite existing files — use edit instead)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(WriteInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl WriteTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let write_input: WriteInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &write_input.path)?;

        if path.exists() {
            return Err(FileToolError::FileExists(write_input.path));
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, &write_input.content).await?;

        Ok(format!(
            "wrote {} bytes to {}",
            write_input.content.len(),
            write_input.path
        ))
    }
}

/// Input parameters for the [`EditTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

/// A tool that applies a string replacement in a file.
pub struct EditTool {
    workspace_root: PathBuf,
}

impl EditTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Apply a string replacement in a file"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(EditInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl EditTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let edit_input: EditInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &edit_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(edit_input.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;

        let Some(first_match_start) = content.find(&edit_input.old_string) else {
            return Err(FileToolError::StringNotFound(edit_input.old_string));
        };

        let first_match_end = first_match_start + edit_input.old_string.len();
        let mut new_content = String::with_capacity(
            content.len() + edit_input.new_string.len() - edit_input.old_string.len(),
        );
        new_content.push_str(&content[..first_match_start]);
        new_content.push_str(&edit_input.new_string);
        new_content.push_str(&content[first_match_end..]);

        tokio::fs::write(&path, &new_content).await?;

        Ok(format!("edited {}", edit_input.path))
    }
}

/// Directories that are skipped during recursive search.
fn is_skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

/// Input parameters for the [`GrepTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepInput {
    /// Regular expression pattern to search for.
    pub pattern: String,
    /// File or directory to search in. Defaults to workspace root.
    #[serde(default)]
    pub path: Option<String>,
    /// Glob pattern to filter files (e.g. `*.rs`). Only matching files are searched.
    #[serde(default)]
    pub include: Option<String>,
    /// Maximum number of matches to return. Default 50.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub max_results: Option<u32>,
}

/// A tool that searches file contents by regex across the workspace.
pub struct GrepTool {
    workspace_root: PathBuf,
}

impl GrepTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents by regex across the workspace"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(GrepInput)
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
}

impl GrepTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let grep_input: GrepInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let re = regex::Regex::new(&grep_input.pattern)
            .map_err(|e| FileToolError::InvalidInput(format!("invalid regex: {e}")))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let search_path = match &grep_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        if !search_path.exists() {
            return Err(FileToolError::FileNotFound(
                grep_input.path.unwrap_or_default(),
            ));
        }

        let include_pattern = grep_input
            .include
            .as_deref()
            .map(glob::Pattern::new)
            .transpose()
            .map_err(|e| FileToolError::InvalidInput(format!("invalid include glob: {e}")))?;

        let max_results = grep_input.max_results.unwrap_or(50) as usize;

        let files: Vec<PathBuf> = if search_path.is_file() {
            vec![search_path.clone()]
        } else {
            walkdir::WalkDir::new(&search_path)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    !(e.file_type().is_dir() && is_skip_dir(&e.file_name().to_string_lossy()))
                })
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        let root = canonical_root;
        let mut matches: Vec<(String, usize, String)> = Vec::new();

        for file_path in &files {
            if matches.len() >= max_results {
                break;
            }

            if let Some(ref pat) = include_pattern {
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !pat.matches(&file_name) {
                    continue;
                }
            }

            if is_binary_file(file_path)? {
                continue;
            }

            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let display_path = file_path
                .strip_prefix(&root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            for (i, line) in content.lines().enumerate() {
                if matches.len() >= max_results {
                    break;
                }
                if re.is_match(line) {
                    matches.push((display_path.clone(), i + 1, line.trim_end().to_string()));
                }
            }
        }

        if matches.is_empty() {
            return Ok(format!(
                "no matches found for pattern '{}'",
                grep_input.pattern
            ));
        }

        let mut output = String::new();
        for (file, line_num, line) in &matches {
            output.push_str(&format!("{file}:{line_num}: {line}\n"));
        }

        if matches.len() >= max_results {
            output.push_str(&format!(
                "\n... (showing first {max_results} matches, refine pattern for more)"
            ));
        }

        Ok(output)
    }
}

/// Input parameters for the [`GlobTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobInput {
    /// Glob pattern (e.g. `**/*.rs`, `src/**/*.ts`, `*.toml`).
    pub pattern: String,
    /// Base directory for the search. Defaults to workspace root.
    #[serde(default)]
    pub path: Option<String>,
}

/// A tool that finds files by name pattern using glob matching.
pub struct GlobTool {
    workspace_root: PathBuf,
}

impl GlobTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files by glob pattern (e.g. **/*.rs)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(GlobInput)
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
}

impl GlobTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let glob_input: GlobInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let base_path = match &glob_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        let full_pattern = base_path.join(&glob_input.pattern);
        let pattern_str = full_pattern.to_string_lossy().to_string();

        let opts = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut paths: Vec<String> = Vec::new();
        for entry in glob::glob_with(&pattern_str, opts)
            .map_err(|e| FileToolError::InvalidInput(format!("invalid glob pattern: {e}")))?
        {
            let path = entry
                .map_err(|e| FileToolError::InvalidInput(format!("glob error: {e}")))?;
            let display_path = path
                .strip_prefix(&canonical_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            paths.push(display_path);
        }

        if paths.is_empty() {
            return Ok(format!("no files matched pattern '{}'", glob_input.pattern));
        }

        paths.sort();

        let mut output = String::new();
        for p in &paths {
            output.push_str(p);
            output.push('\n');
        }
        output.push_str(&format!("\n{} file(s) matched", paths.len()));

        Ok(output)
    }
}

/// Input parameters for the [`LsTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LsInput {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub all: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub long: bool,
}

/// A tool that lists directory contents with file metadata.
pub struct LsTool {
    workspace_root: PathBuf,
}

impl LsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List directory contents (supports long format, hidden files, recursive)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(LsInput)
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
}

impl LsTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let ls_input: LsInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let target = match &ls_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        if !target.exists() {
            return Err(FileToolError::FileNotFound(
                ls_input.path.unwrap_or_default(),
            ));
        }

        if target.is_file() {
            return Ok(format_entry(&target, &canonical_root, ls_input.long));
        }

        if ls_input.recursive {
            let mut output = String::new();
            for entry in walkdir::WalkDir::new(&target)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    if !ls_input.all && is_hidden_entry(e.file_name()) {
                        return false;
                    }
                    !(e.file_type().is_dir() && is_skip_dir(&e.file_name().to_string_lossy()))
                })
                .filter_map(Result::ok)
                .filter(|e| e.depth() > 0)
            {
                output.push_str(&format_entry(entry.path(), &canonical_root, ls_input.long));
                output.push('\n');
            }
            return Ok(output.trim_end().to_string());
        }

        let mut entries: Vec<_> = std::fs::read_dir(&target)?
            .filter_map(Result::ok)
            .filter(|e| ls_input.all || !is_hidden_entry(&e.file_name()))
            .collect();

        entries.sort_by_key(|e| {
            let is_dir = e
                .file_type()
                .map(|ft| ft.is_dir())
                .unwrap_or(false);
            (!is_dir, e.file_name())
        });

        let mut output = String::new();
        for entry in &entries {
            output.push_str(&format_entry(&entry.path(), &canonical_root, ls_input.long));
            output.push('\n');
        }

        Ok(output.trim_end().to_string())
    }
}

fn is_hidden_entry(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn format_entry(path: &Path, root: &Path, long: bool) -> String {
    let display = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    if long {
        return format_entry_long(path, &display);
    }

    let meta = std::fs::symlink_metadata(path).ok();
    let (type_char, size) = match &meta {
        Some(m) => {
            let ft = m.file_type();
            if ft.is_dir() {
                ('d', None)
            } else if ft.is_symlink() {
                ('l', Some(m.len()))
            } else {
                ('-', Some(m.len()))
            }
        }
        None => ('?', None),
    };

    let size_str = match size {
        Some(s) => format!("{s:>8}"),
        None => format!("{:>8}", "-"),
    };

    format!("{type_char} {size_str}  {display}")
}

#[cfg(unix)]
fn format_entry_long(path: &Path, display: &str) -> String {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return format!("?---------    -      -      -        -  ????-??-?? ??:??  {display}"),
    };

    let ft = meta.file_type();
    let type_char = if ft.is_dir() {
        'd'
    } else if ft.is_symlink() {
        'l'
    } else {
        '-'
    };

    let perms = format_permissions(meta.mode());
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();
    let mtime = format_mtime(meta.mtime());

    format!(
        "{type_char}{perms}  {nlink:>3}  {uid:>5}  {gid:>5}  {size:>8}  {mtime}  {display}"
    )
}

#[cfg(not(unix))]
fn format_entry_long(path: &Path, display: &str) -> String {
    let meta = std::fs::symlink_metadata(path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    format!("-         {size:>8}  {display}")
}

#[cfg(unix)]
fn format_permissions(mode: u32) -> String {
    let bits: [(u32, char); 9] = [
        (0o400, 'r'), (0o200, 'w'), (0o100, 'x'),
        (0o040, 'r'), (0o020, 'w'), (0o010, 'x'),
        (0o004, 'r'), (0o002, 'w'), (0o001, 'x'),
    ];
    bits.iter()
        .map(|(bit, c)| if mode & bit != 0 { *c } else { '-' })
        .collect()
}

#[cfg(unix)]
fn format_mtime(mtime: i64) -> String {
    let secs = mtime.max(0) as u64;
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

/// Input parameters for the [`DeleteTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteInput {
    /// Path to the file or directory to delete.
    pub path: String,
}

/// Errors specific to the delete tool.
#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("refusing to delete workspace root")]
    WorkspaceRoot,
}

/// A tool that deletes files or directories within the workspace.
pub struct DeleteTool {
    workspace_root: PathBuf,
}

impl DeleteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for DeleteTool {
    fn name(&self) -> &str {
        "delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory (cannot delete workspace root)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(DeleteInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }
}

impl DeleteTool {
    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let delete_input: DeleteInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &delete_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(delete_input.path));
        }

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .map_err(FileToolError::Io)?;

        if path == canonical_root {
            return Err(FileToolError::InvalidInput(
                DeleteError::WorkspaceRoot.to_string(),
            ));
        }

        if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await?;
        } else {
            tokio::fs::remove_file(&path).await?;
        }

        let display = path
            .strip_prefix(&canonical_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        Ok(format!("deleted {display}"))
    }
}

#[cfg(test)]
mod file_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_read_tool_read_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: line1"));
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
    }

    #[tokio::test]
    async fn test_read_tool_line_range() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "line1\nline2\nline3\nline4\n",
        )
        .unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "start_line": 2, "end_line": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 1, "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_zero() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 1 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: a"));
        assert!(!result.content.contains("2: b"));
    }

    #[tokio::test]
    async fn test_read_tool_limit_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: line1"));
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("more lines"));
        assert!(result.content.contains("offset=2"));
    }

    #[tokio::test]
    async fn test_read_tool_pagination_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=10).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("7 more lines"));
        assert!(result.content.contains("offset=3"));
    }

    #[tokio::test]
    async fn test_read_tool_no_truncation_no_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 100 }))
            .await;

        assert!(!result.is_error);
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_takes_precedence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "test.txt",
                "start_line": 1,
                "end_line": 2,
                "offset": 2,
                "limit": 2
            }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("3: line3"));
        assert!(result.content.contains("4: line4"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("2: line2"));
    }

    #[tokio::test]
    async fn test_read_tool_backward_compat_no_params() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: a"));
        assert!(result.content.contains("2: b"));
        assert!(result.content.contains("3: c"));
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "../outside.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_read_tool_binary_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("binary.bin"), &[0u8, 1, 2, 3]).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "binary.bin" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("binary"));
    }

    #[tokio::test]
    async fn test_read_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[tokio::test]
    async fn test_write_tool_new_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({ "path": "new.txt", "content": "hello world" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("wrote"));

        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_write_tool_refuses_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("existing.txt");
        fs::write(&file, "old content").unwrap();

        let tool = WriteTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "existing.txt", "content": "new content" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("already exists"));
        assert!(result.content.contains("edit tool"));
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "old content");
    }

    #[tokio::test]
    async fn test_write_tool_create_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "a/b/c/deep.txt",
                "content": "deep content"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(temp_dir.path().join("a/b/c/deep.txt")).unwrap();
        assert_eq!(content, "deep content");
    }

    #[tokio::test]
    async fn test_edit_tool_replace_first_occurrence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "foo bar foo baz").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "foo",
                "new_string": "qux"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "qux bar foo baz");
    }

    #[tokio::test]
    async fn test_edit_tool_no_match() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "hello world").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "notfound",
                "new_string": "replacement"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("string not found"));
    }

    #[tokio::test]
    async fn test_edit_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "../outside.txt",
                "old_string": "secret",
                "new_string": "exposed"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_edit_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "nonexistent.txt",
                "old_string": "foo",
                "new_string": "bar"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[test]
    fn test_resolve_workspace_path_within_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("subdir/file.txt");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "content").unwrap();

        let resolved = resolve_workspace_path(temp_dir.path(), "subdir/file.txt").unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_workspace_path_escape_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = resolve_workspace_path(temp_dir.path(), "../outside.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileToolError::PathEscape(_)));
    }
}

#[cfg(test)]
mod grep_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn hello() {}\nfn world() {}\n").unwrap();
        fs::write(dir.path().join("b.txt"), "hello world\nfoo bar\n").unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(
            dir.path().join("sub/c.rs"),
            "hello from sub\nanother line\n",
        )
        .unwrap();
        dir
    }

    #[tokio::test]
    async fn test_grep_basic_match() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "hello" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:1:"));
        assert!(result.content.contains("b.txt:1:"));
        assert!(result.content.contains("sub/c.rs:1:"));
    }

    #[tokio::test]
    async fn test_grep_regex_pattern() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "fn \\w+\\(\\)" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:1: fn hello()"));
        assert!(result.content.contains("a.rs:2: fn world()"));
        assert!(!result.content.contains("b.txt"));
    }

    #[tokio::test]
    async fn test_grep_include_filter() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "include": "*.rs" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:1:"));
        assert!(result.content.contains("sub/c.rs:1:"));
        assert!(!result.content.contains("b.txt"));
    }

    #[tokio::test]
    async fn test_grep_single_file() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "foo", "path": "b.txt" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("b.txt:2: foo bar"));
    }

    #[tokio::test]
    async fn test_grep_no_match() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "nonexistent_pattern_xyz" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("no matches"));
    }

    #[tokio::test]
    async fn test_grep_max_results() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "max_results": 1 }))
            .await;

        assert!(!result.is_error);
        let match_count = result.content.lines().filter(|l| l.contains(": ") && !l.starts_with("...")).count();
        assert_eq!(match_count, 1);
        assert!(result.content.contains("showing first 1"));
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "[invalid" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("invalid regex"));
    }

    #[tokio::test]
    async fn test_grep_skips_hidden_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("visible.txt"), "target_text\n").unwrap();
        fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        fs::write(dir.path().join(".hidden/secret.txt"), "target_text\n").unwrap();

        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "target_text" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("visible.txt"));
        assert!(!result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_grep_path_not_found() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "path": "nonexistent_dir" }))
            .await;

        assert!(result.is_error);
    }

    #[test]
    fn test_grep_tool_is_read_only() {
        let tool = GrepTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
mod glob_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn lib() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;\n").unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(dir.path().join("tests/integration.rs"), "#[test]\nfn test() {}\n").unwrap();
        dir
    }

    #[tokio::test]
    async fn test_glob_recursive_rs() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("lib.rs"));
        assert!(result.content.contains("src/mod.rs"));
        assert!(result.content.contains("tests/integration.rs"));
        assert!(!result.content.contains("Cargo.toml"));
    }

    #[tokio::test]
    async fn test_glob_top_level_pattern() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("lib.rs"));
        assert!(!result.content.contains("src/mod.rs"));
    }

    #[tokio::test]
    async fn test_glob_toml_files() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "*.toml" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("Cargo.toml"));
        assert!(!result.content.contains(".rs"));
    }

    #[tokio::test]
    async fn test_glob_specific_dir() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "src/**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("src/mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_glob_no_match() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.py" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("no files matched"));
    }

    #[tokio::test]
    async fn test_glob_with_path_param() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "*.rs", "path": "src" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_glob_file_count() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("4 file(s) matched"));
    }

    #[test]
    fn test_glob_tool_is_read_only() {
        let tool = GlobTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
mod ls_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;\n").unwrap();
        fs::write(dir.path().join(".hidden"), "secret\n").unwrap();
        dir
    }

    #[tokio::test]
    async fn test_ls_flat() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("Cargo.toml"));
        assert!(result.content.contains("src"));
        assert!(!result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_show_hidden() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "all": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_recursive() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "recursive": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("src/mod.rs"));
    }

    #[tokio::test]
    async fn test_ls_specific_dir() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "src" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_dir_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let src_line = result
            .content
            .lines()
            .find(|l| l.contains("src"))
            .unwrap();
        assert!(src_line.starts_with('d'));
    }

    #[tokio::test]
    async fn test_ls_file_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let toml_line = result
            .content
            .lines()
            .find(|l| l.contains("Cargo.toml"))
            .unwrap();
        assert!(toml_line.starts_with('-'));
    }

    #[tokio::test]
    async fn test_ls_file_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        assert!(result.content.contains("11"));
    }

    #[tokio::test]
    async fn test_ls_not_found() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent" })).await;

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_ls_single_file() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "main.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_long_format() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let txt_line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap_or_else(|| panic!("no test.txt line in: {}", result.content));
        assert!(txt_line.starts_with('-'));
        assert!(txt_line.contains("rw"));
    }

    #[tokio::test]
    async fn test_ls_long_format_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let src_line = result
            .content
            .lines()
            .find(|l| l.contains("src"))
            .unwrap_or_else(|| panic!("no src line in: {}", result.content));
        assert!(src_line.starts_with('d'));
    }

    #[tokio::test]
    async fn test_ls_long_shows_permissions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "content").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let line = result.content.lines().find(|l| l.contains("test.txt")).unwrap();
        let perms_field = line.split_whitespace().nth(0).unwrap_or("");
        assert!(perms_field.starts_with('-'));
        assert!(perms_field.len() == 10);
    }

    #[test]
    fn test_ls_tool_is_read_only() {
        let tool = LsTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
mod delete_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_delete_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("temp.txt"), "content").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "temp.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("deleted"));
        assert!(!dir.path().join("temp.txt").exists());
    }

    #[tokio::test]
    async fn test_delete_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("subdir/nested")).unwrap();
        fs::write(dir.path().join("subdir/nested/file.txt"), "data").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "subdir" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("deleted"));
        assert!(!dir.path().join("subdir").exists());
    }

    #[tokio::test]
    async fn test_delete_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("empty")).unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "empty" })).await;

        assert!(!result.is_error);
        assert!(!dir.path().join("empty").exists());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[tokio::test]
    async fn test_delete_path_escape() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().parent().unwrap().join("outside_target.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "../outside_target.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
        assert!(outside.exists());
        let _ = fs::remove_file(&outside);
    }

    #[tokio::test]
    async fn test_delete_refuses_workspace_root() {
        let dir = tempfile::tempdir().unwrap();
        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "." })).await;

        assert!(result.is_error);
        assert!(result.content.contains("workspace root"));
        assert!(dir.path().exists());
    }

    #[tokio::test]
    async fn test_delete_not_read_only() {
        let tool = DeleteTool::new(PathBuf::from("."));
        assert!(!tool.is_read_only());
    }
}
