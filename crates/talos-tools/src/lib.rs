//! Built-in agent tools for Talos.
//!
//! This crate provides implementations of the [`AgentTool`] trait for common
//! agent operations such as shell command execution and file operations.

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
}

/// Resolves a relative path against the workspace root and validates
/// that the resulting path stays within the workspace.
fn resolve_workspace_path(workspace_root: &Path, relative: &str) -> Result<PathBuf, FileToolError> {
    let canon_root = workspace_root.canonicalize()?;
    let joined = workspace_root.join(relative);

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
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub start_line: Option<u32>,
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub end_line: Option<u32>,
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
        "Read file content with optional line range"
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
        let read_input: ReadInput =
            serde_json::from_value(input).map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &read_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(read_input.path));
        }

        if is_binary_file(&path)? {
            return Err(FileToolError::BinaryFile(read_input.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();

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

        let selected = &lines[start.min(lines.len())..end.min(lines.len())];

        let mut output = String::new();
        for (i, line) in selected.iter().enumerate() {
            let line_num = start + i + 1;
            output.push_str(&format!("{line_num}: {line}\n"));
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

/// A tool that creates or overwrites a file with the given content.
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
        "Create or overwrite a file"
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
        let write_input: WriteInput =
            serde_json::from_value(input).map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &write_input.path)?;

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
        let edit_input: EditInput =
            serde_json::from_value(input).map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

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
        fs::write(temp_dir.path().join("test.txt"), "line1\nline2\nline3\nline4\n").unwrap();

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
    async fn test_write_tool_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("existing.txt");
        fs::write(&file, "old content").unwrap();

        let tool = WriteTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "existing.txt", "content": "new content" }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "new content");
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
