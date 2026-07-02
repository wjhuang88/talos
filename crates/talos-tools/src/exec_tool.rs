//! Direct argv-style subprocess execution tool.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::file_tools::{FileToolError, resolve_workspace_path};

const DEFAULT_TIMEOUT_SECS: u64 = 120;
const MAX_TIMEOUT_SECS: u64 = 600;
const MAX_STREAM_BYTES: usize = 32 * 1024;

/// Errors that can occur during direct exec execution.
#[derive(Debug, Error)]
pub enum ExecError {
    /// The input does not conform to the expected schema.
    #[error("invalid exec input: {0}")]
    InvalidInput(String),
    /// The command field is empty.
    #[error("command must not be empty")]
    EmptyCommand,
    /// A provided environment variable name is not allowed.
    #[error("sensitive env var names are not allowed: {0}")]
    SensitiveEnv(String),
    /// The requested working directory is invalid.
    #[error("invalid cwd: {0}")]
    InvalidCwd(String),
}

/// Input parameters for [`ExecTool`].
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExecInput {
    /// Program name or path to execute directly.
    pub command: String,
    /// Arguments passed as argv elements. No shell parsing is performed.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional working directory, resolved inside the workspace root.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Optional environment additions or overrides. Sensitive names are denied.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Optional timeout in seconds. Clamped to [1, 600]. Defaults to 120.
    #[serde(default)]
    #[schemars(range(min = 1, max = 600))]
    pub timeout_secs: Option<u64>,
}

/// A direct subprocess tool that executes one program with argv arguments.
pub struct ExecTool {
    workspace_root: PathBuf,
    timeout: Duration,
    max_stream_bytes: usize,
}

impl ExecTool {
    /// Creates a new `ExecTool` rooted at the given workspace.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_stream_bytes: MAX_STREAM_BYTES,
        }
    }

    /// Sets a custom default timeout for command execution.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum retained bytes for stdout and stderr.
    #[must_use]
    pub fn with_max_stream_bytes(mut self, max_stream_bytes: usize) -> Self {
        self.max_stream_bytes = max_stream_bytes;
        self
    }

    fn parse_input(&self, input: Value) -> Result<ExecInput, ExecError> {
        let parsed: ExecInput =
            serde_json::from_value(input).map_err(|e| ExecError::InvalidInput(e.to_string()))?;
        if parsed.command.trim().is_empty() {
            return Err(ExecError::EmptyCommand);
        }
        if let Some(name) = parsed.env.keys().find(|name| is_sensitive_env_name(name)) {
            return Err(ExecError::SensitiveEnv(name.clone()));
        }
        Ok(parsed)
    }

    fn resolve_cwd(&self, cwd: Option<&str>) -> Result<PathBuf, ExecError> {
        let cwd = cwd.unwrap_or(".");
        let resolved = resolve_workspace_path(&self.workspace_root, cwd)
            .map_err(|e| ExecError::InvalidCwd(e.to_string()))?;
        if !resolved.is_dir() {
            return Err(ExecError::InvalidCwd(
                FileToolError::FileNotFound(cwd.to_string()).to_string(),
            ));
        }
        Ok(resolved)
    }

    async fn run(&self, input: ExecInput) -> ToolResult {
        let cwd = match self.resolve_cwd(input.cwd.as_deref()) {
            Ok(cwd) => cwd,
            Err(e) => return ToolResult::error(e.to_string()),
        };
        let timeout = input
            .timeout_secs
            .map(|secs| Duration::from_secs(secs.clamp(1, MAX_TIMEOUT_SECS)))
            .unwrap_or(self.timeout);

        let mut cmd = Command::new(&input.command);
        cmd.args(&input.args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names() {
            cmd.env_remove(name);
        }
        for (name, value) in &input.env {
            cmd.env(name, value);
        }

        let started = Instant::now();
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return ToolResult::error(format!("failed to spawn '{}': {e}", input.command));
            }
        };

        let stdout = child.stdout.take().expect("stdout is piped");
        let stderr = child.stderr.take().expect("stderr is piped");
        let stdout_limit = self.max_stream_bytes;
        let stderr_limit = self.max_stream_bytes;
        let stdout_task = tokio::spawn(read_bounded(stdout, stdout_limit));
        let stderr_task = tokio::spawn(read_bounded(stderr, stderr_limit));

        let wait_status = tokio::select! {
            status = child.wait() => {
                match status {
                    Ok(status) => Some(status),
                    Err(e) => return ToolResult::error(format!("failed to wait for child: {e}")),
                }
            }
            _ = tokio::time::sleep(timeout) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                None
            }
        };

        let stdout = stdout_task
            .await
            .unwrap_or_else(|e| BoundedOutput::from_error(format!("stdout reader failed: {e}")));
        let stderr = stderr_task
            .await
            .unwrap_or_else(|e| BoundedOutput::from_error(format!("stderr reader failed: {e}")));
        let duration_ms = started.elapsed().as_millis();

        let Some(status) = wait_status else {
            return ToolResult::error(format_exec_output(
                &input,
                &cwd,
                duration_ms,
                None,
                &stdout,
                &stderr,
                Some("timeout"),
            ));
        };
        let exit_code = status.code().unwrap_or(-1);
        let content = format_exec_output(
            &input,
            &cwd,
            duration_ms,
            Some(exit_code),
            &stdout,
            &stderr,
            None,
        );

        ToolResult {
            content,
            is_error: !status.success(),
            continuations: Vec::new(),
        }
    }
}

#[async_trait]
impl AgentTool for ExecTool {
    fn name(&self) -> &str {
        "exec"
    }

    fn description(&self) -> &str {
        "Execute one program with argv arguments. No shell parsing, pipelines, redirection, glob expansion, or background jobs are performed."
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(ExecInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Shell
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let mut profile = Vec::new();
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        profile.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                command,
                ToolResourceKind::Command,
            )
            .with_description("direct process execution"),
        );
        if let Some(cwd) = input.get("cwd").and_then(Value::as_str) {
            profile.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    cwd.to_string(),
                    ToolResourceKind::Path,
                )
                .with_description("working directory"),
            );
        }
        profile
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["command", "cwd"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed = match self.parse_input(input) {
            Ok(parsed) => parsed,
            Err(e) => return ToolResult::error(e.to_string()),
        };
        self.run(parsed).await
    }
}

struct BoundedOutput {
    content: Vec<u8>,
    total_bytes: usize,
}

impl BoundedOutput {
    fn from_error(message: String) -> Self {
        Self {
            content: message.into_bytes(),
            total_bytes: 0,
        }
    }

    fn as_text(&self) -> String {
        String::from_utf8_lossy(&self.content).to_string()
    }

    fn truncated(&self) -> bool {
        self.total_bytes > self.content.len()
    }
}

async fn read_bounded<R>(mut reader: R, limit: usize) -> BoundedOutput
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut retained = Vec::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if retained.len() < limit {
                    let remaining = limit - retained.len();
                    retained.extend_from_slice(&buffer[..n.min(remaining)]);
                }
            }
            Err(e) => {
                let message = format!("[read error: {e}]");
                total += message.len();
                if retained.len() < limit {
                    let remaining = limit - retained.len();
                    retained.extend_from_slice(&message.as_bytes()[..message.len().min(remaining)]);
                }
                break;
            }
        }
    }
    BoundedOutput {
        content: retained,
        total_bytes: total,
    }
}

fn format_exec_output(
    input: &ExecInput,
    cwd: &std::path::Path,
    duration_ms: u128,
    exit_code: Option<i32>,
    stdout: &BoundedOutput,
    stderr: &BoundedOutput,
    marker: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("command: {}\n", input.command));
    output.push_str(&format!("args: {}\n", input.args.len()));
    output.push_str(&format!("cwd: {}\n", cwd.display()));
    if !input.env.is_empty() {
        let names = input.env.keys().cloned().collect::<Vec<_>>().join(", ");
        output.push_str(&format!("env: {names} (values redacted)\n"));
    }
    output.push_str(&format!("duration_ms: {duration_ms}\n"));
    if let Some(exit_code) = exit_code {
        output.push_str(&format!("exit_code: {exit_code}\n"));
    }
    if let Some(marker) = marker {
        output.push_str(&format!("[{marker}]\n"));
    }
    output.push_str(&format!("\nstdout ({} bytes):\n", stdout.total_bytes));
    output.push_str(&stdout.as_text());
    if stdout.truncated() {
        output.push_str(&format!(
            "\n[stdout truncated at {} bytes]",
            stdout.content.len()
        ));
    }
    output.push_str(&format!("\n\nstderr ({} bytes):\n", stderr.total_bytes));
    output.push_str(&stderr.as_text());
    if stderr.truncated() {
        output.push_str(&format!(
            "\n[stderr truncated at {} bytes]",
            stderr.content.len()
        ));
    }
    output
}

fn is_sensitive_env_name(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    name.contains("KEY")
        || name.contains("TOKEN")
        || name.contains("SECRET")
        || name.contains("PASSWORD")
        || name.contains("CREDENTIAL")
        || name.contains("COOKIE")
        || name.contains("AUTH")
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::ToolResourceKind;
    use tempfile::TempDir;

    fn tool_with_temp_workspace() -> (ExecTool, TempDir) {
        let temp = TempDir::new().expect("tempdir");
        (ExecTool::new(temp.path().to_path_buf()), temp)
    }

    #[test]
    fn metadata_matches_policy() {
        let (tool, _temp) = tool_with_temp_workspace();
        assert_eq!(tool.name(), "exec");
        assert_eq!(tool.nature(), ToolNature::Execute);
        assert_eq!(tool.family(), ToolFamily::Shell);
        assert!(!tool.is_read_only());
    }

    #[test]
    fn permission_profile_exposes_command_and_cwd_facets() {
        let (tool, _temp) = tool_with_temp_workspace();
        let profile = tool.permission_profile(&serde_json::json!({
            "command": "printf",
            "args": ["hello"],
            "cwd": "."
        }));

        assert_eq!(profile.len(), 2);
        assert_eq!(profile[0].nature, ToolNature::Execute);
        assert_eq!(profile[0].resource.as_deref(), Some("printf"));
        assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Command));
        assert_eq!(profile[1].nature, ToolNature::Read);
        assert_eq!(profile[1].resource.as_deref(), Some("."));
        assert_eq!(profile[1].resource_kind, Some(ToolResourceKind::Path));
    }

    #[tokio::test]
    async fn sensitive_env_names_are_denied_before_spawn() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "definitely-not-spawned",
                "env": {"API_KEY": "sk-live"}
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("sensitive env var names"));
        assert!(!result.content.contains("sk-live"));
    }

    #[tokio::test]
    async fn cwd_must_stay_in_workspace() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "cwd": "/"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn executes_successfully_without_shell() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "args": ["%s", "hello"]
            }))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("exit_code: 0"));
        assert!(result.content.contains("hello"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn non_zero_exit_is_error() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "false"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("exit_code: 1"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn shell_metacharacters_are_literal_args() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "args": ["%s", "hello; echo injected"]
            }))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("hello; echo injected"));
        assert!(!result.content.contains("\ninjected"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn timeout_kills_child() {
        let temp = TempDir::new().unwrap();
        let tool = ExecTool::new(temp.path().to_path_buf()).with_timeout(Duration::from_secs(30));
        let result = tool
            .execute(serde_json::json!({
                "command": "sh",
                "args": ["-c", "sleep 2"],
                "timeout_secs": 1
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("[timeout]"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stdout_is_bounded() {
        let (tool, _temp) = tool_with_temp_workspace();
        let tool = tool.with_max_stream_bytes(8);
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "args": ["%s", "abcdefghijklmnop"]
            }))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("[stdout truncated at 8 bytes]"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn env_values_are_not_echoed_in_metadata() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "args": ["%s", "ok"],
                "env": {"TALOS_EXEC_TEST": "visible-secret-value"}
            }))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("TALOS_EXEC_TEST"));
        assert!(!result.content.contains("visible-secret-value"));
    }
}
