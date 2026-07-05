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
    /// Single-command and multi-step fields were mixed.
    #[error("command and steps are mutually exclusive")]
    MixedCommandAndSteps,
    /// The requested execution mode is not supported by this slice.
    #[error("unsupported exec mode: {0}")]
    UnsupportedMode(String),
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
    /// Program name or path to execute directly in single-command mode.
    #[serde(default)]
    pub command: Option<String>,
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
    /// Sequential execution steps. When provided, top-level command fields must be omitted.
    #[serde(default)]
    pub steps: Vec<ExecStep>,
    /// Execution mode for steps. This slice supports only `sequential`.
    #[serde(default)]
    pub mode: ExecMode,
}

/// One direct argv-style execution step.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ExecStep {
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
    /// Optional timeout in seconds. Clamped to [1, 600]. Defaults to the request timeout.
    #[serde(default)]
    #[schemars(range(min = 1, max = 600))]
    pub timeout_secs: Option<u64>,
}

/// Multi-step execution mode.
#[derive(Debug, Default, Clone, Copy, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecMode {
    /// Run steps one after another, stopping on the first failure.
    #[default]
    Sequential,
    /// Reserved for a later TOOL-017 slice.
    Parallel,
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

        let has_command = parsed
            .command
            .as_deref()
            .is_some_and(|command| !command.trim().is_empty());
        if parsed.steps.is_empty() {
            if !has_command {
                return Err(ExecError::EmptyCommand);
            }
        } else {
            if has_command
                || !parsed.args.is_empty()
                || parsed.cwd.is_some()
                || !parsed.env.is_empty()
            {
                return Err(ExecError::MixedCommandAndSteps);
            }
            if parsed.mode != ExecMode::Sequential {
                return Err(ExecError::UnsupportedMode("parallel".to_string()));
            }
            if parsed
                .steps
                .iter()
                .any(|step| step.command.trim().is_empty())
            {
                return Err(ExecError::EmptyCommand);
            }
        }
        if let Some(name) = parsed.env.keys().find(|name| is_sensitive_env_name(name)) {
            return Err(ExecError::SensitiveEnv(name.clone()));
        }
        for step in &parsed.steps {
            if let Some(name) = step.env.keys().find(|name| is_sensitive_env_name(name)) {
                return Err(ExecError::SensitiveEnv(name.clone()));
            }
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
        if !input.steps.is_empty() {
            return self.run_steps(input).await;
        }

        let step = ExecStep {
            command: input.command.clone().unwrap_or_default(),
            args: input.args.clone(),
            cwd: input.cwd.clone(),
            env: input.env.clone(),
            timeout_secs: input.timeout_secs,
        };
        let result = match self
            .run_step(&step, input.timeout_secs, Instant::now())
            .await
        {
            Ok(result) => result,
            Err(result) => return result,
        };

        ToolResult {
            content: format_step_output(&step, &result),
            is_error: !result.success,
            continuations: Vec::new(),
        }
    }

    async fn run_steps(&self, input: ExecInput) -> ToolResult {
        let started = Instant::now();
        let mut content = String::new();
        content.push_str("mode: sequential\n");
        content.push_str(&format!("steps: {}\n", input.steps.len()));
        let mut is_error = false;

        for (index, step) in input.steps.iter().enumerate() {
            let step_started = Instant::now();
            let result = match self.run_step(step, input.timeout_secs, step_started).await {
                Ok(result) => result,
                Err(result) => return result,
            };
            content.push_str(&format!("\nstep[{index}]\n"));
            content.push_str("-------\n");
            content.push_str(&format_step_output(step, &result));
            content.push('\n');
            if !result.success {
                is_error = true;
                content.push_str(&format!(
                    "\n[stopped after failed step {index}; later steps were not executed]\n"
                ));
                break;
            }
        }
        content.insert_str(
            0,
            &format!("duration_ms: {}\n", started.elapsed().as_millis()),
        );

        ToolResult {
            content,
            is_error,
            continuations: Vec::new(),
        }
    }

    async fn run_step(
        &self,
        step: &ExecStep,
        request_timeout_secs: Option<u64>,
        started: Instant,
    ) -> Result<StepExecution, ToolResult> {
        let cwd = match self.resolve_cwd(step.cwd.as_deref()) {
            Ok(cwd) => cwd,
            Err(e) => return Err(ToolResult::error(e.to_string())),
        };
        let timeout = step
            .timeout_secs
            .or(request_timeout_secs)
            .map(|secs| Duration::from_secs(secs.clamp(1, MAX_TIMEOUT_SECS)))
            .unwrap_or(self.timeout);

        let mut cmd = Command::new(&step.command);
        cmd.args(&step.args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names() {
            cmd.env_remove(name);
        }
        for (name, value) in &step.env {
            cmd.env(name, value);
        }

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Err(ToolResult::error(format!(
                    "failed to spawn '{}': {e}",
                    step.command
                )));
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
                    Err(e) => return Err(ToolResult::error(format!("failed to wait for child: {e}"))),
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
            return Ok(StepExecution {
                cwd,
                duration_ms,
                exit_code: None,
                success: false,
                marker: Some("timeout"),
                stdout,
                stderr,
            });
        };
        let exit_code = status.code().unwrap_or(-1);
        Ok(StepExecution {
            cwd,
            duration_ms,
            exit_code: Some(exit_code),
            success: status.success(),
            marker: None,
            stdout,
            stderr,
        })
    }
}

#[async_trait]
impl AgentTool for ExecTool {
    fn name(&self) -> &str {
        "exec"
    }

    fn description(&self) -> &str {
        "Execute one program, or sequential argv-style steps, with direct process spawning. No shell parsing, pipelines, redirection, glob expansion, or background jobs are performed."
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
        if let Some(steps) = input.get("steps").and_then(Value::as_array) {
            for step in steps {
                push_step_permission_profile(&mut profile, step);
            }
        } else {
            push_step_permission_profile(&mut profile, input);
        }
        profile
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["command", "steps", "cwd"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let parsed = match self.parse_input(input) {
            Ok(parsed) => parsed,
            Err(e) => return ToolResult::error(e.to_string()),
        };
        self.run(parsed).await
    }
}

struct StepExecution {
    cwd: PathBuf,
    duration_ms: u128,
    exit_code: Option<i32>,
    success: bool,
    marker: Option<&'static str>,
    stdout: BoundedOutput,
    stderr: BoundedOutput,
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

fn format_step_output(step: &ExecStep, result: &StepExecution) -> String {
    let mut output = String::new();
    output.push_str(&format!("command: {}\n", step.command));
    output.push_str(&format!("args: {}\n", step.args.len()));
    output.push_str(&format!("cwd: {}\n", result.cwd.display()));
    if !step.env.is_empty() {
        let names = step.env.keys().cloned().collect::<Vec<_>>().join(", ");
        output.push_str(&format!("env: {names} (values redacted)\n"));
    }
    output.push_str(&format!("duration_ms: {}\n", result.duration_ms));
    if let Some(exit_code) = result.exit_code {
        output.push_str(&format!("exit_code: {exit_code}\n"));
    }
    if let Some(marker) = result.marker {
        output.push_str(&format!("[{marker}]\n"));
    }
    output.push_str(&format!(
        "\nstdout ({} bytes):\n",
        result.stdout.total_bytes
    ));
    output.push_str(&result.stdout.as_text());
    if result.stdout.truncated() {
        output.push_str(&format!(
            "\n[stdout truncated at {} bytes]",
            result.stdout.content.len()
        ));
    }
    output.push_str(&format!(
        "\n\nstderr ({} bytes):\n",
        result.stderr.total_bytes
    ));
    output.push_str(&result.stderr.as_text());
    if result.stderr.truncated() {
        output.push_str(&format!(
            "\n[stderr truncated at {} bytes]",
            result.stderr.content.len()
        ));
    }
    output
}

fn push_step_permission_profile(profile: &mut Vec<ToolPermissionFacet>, input: &Value) {
    let command = input
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    profile.push(
        ToolPermissionFacet::with_resource(ToolNature::Execute, command, ToolResourceKind::Command)
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

    #[test]
    fn permission_profile_exposes_each_sequential_step() {
        let (tool, _temp) = tool_with_temp_workspace();
        let profile = tool.permission_profile(&serde_json::json!({
            "steps": [
                {"command": "printf", "args": ["hello"], "cwd": "."},
                {"command": "cargo", "args": ["check"], "cwd": "crates/talos-tools"}
            ]
        }));

        assert_eq!(profile.len(), 4);
        assert_eq!(profile[0].nature, ToolNature::Execute);
        assert_eq!(profile[0].resource.as_deref(), Some("printf"));
        assert_eq!(profile[1].nature, ToolNature::Read);
        assert_eq!(profile[1].resource.as_deref(), Some("."));
        assert_eq!(profile[2].nature, ToolNature::Execute);
        assert_eq!(profile[2].resource.as_deref(), Some("cargo"));
        assert_eq!(profile[3].nature, ToolNature::Read);
        assert_eq!(profile[3].resource.as_deref(), Some("crates/talos-tools"));
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
    async fn sensitive_env_names_are_denied_in_steps_before_spawn() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "steps": [
                    {
                        "command": "definitely-not-spawned",
                        "env": {"API_TOKEN": "secret-token"}
                    }
                ]
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("sensitive env var names"));
        assert!(!result.content.contains("secret-token"));
    }

    #[tokio::test]
    async fn command_and_steps_are_mutually_exclusive() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "command": "printf",
                "steps": [{"command": "printf"}]
            }))
            .await;

        assert!(result.is_error);
        assert!(
            result
                .content
                .contains("command and steps are mutually exclusive")
        );
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
    async fn sequential_steps_run_in_order_without_shell() {
        let (tool, _temp) = tool_with_temp_workspace();
        let result = tool
            .execute(serde_json::json!({
                "steps": [
                    {"command": "printf", "args": ["%s", "first"]},
                    {"command": "printf", "args": ["%s", "second"]}
                ]
            }))
            .await;

        assert!(!result.is_error, "{}", result.content);
        assert!(result.content.contains("mode: sequential"));
        assert!(result.content.contains("step[0]"));
        assert!(result.content.contains("first"));
        assert!(result.content.contains("step[1]"));
        assert!(result.content.contains("second"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn sequential_steps_stop_after_first_failure() {
        let (tool, temp) = tool_with_temp_workspace();
        let marker = temp.path().join("should-not-exist");
        let result = tool
            .execute(serde_json::json!({
                "steps": [
                    {"command": "false"},
                    {"command": "touch", "args": ["should-not-exist"]}
                ]
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("step[0]"));
        assert!(!result.content.contains("step[1]"));
        assert!(result.content.contains("later steps were not executed"));
        assert!(!marker.exists());
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
