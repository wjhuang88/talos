//! Bash tool for shell command execution.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
    /// Optional timeout in seconds. Clamped to [1, 600]. Defaults to 120 if omitted.
    #[serde(default)]
    #[schemars(range(min = 1, max = 600))]
    pub timeout_secs: Option<u64>,
}

/// A tool that executes shell commands via `sh -c`.
///
/// Commands are run with a configurable timeout and working directory.
/// Output is captured from both stdout and stderr.
pub struct BashTool {
    working_dir: PathBuf,
    timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BashCommandClass {
    ReadOnlyInspection,
    ValidationBuild,
    PackageManagerOrNetwork,
    WriteOrMutating,
    ComplexShell,
}

impl BashCommandClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnlyInspection => "read_only_inspection",
            Self::ValidationBuild => "validation_build",
            Self::PackageManagerOrNetwork => "package_manager_or_network",
            Self::WriteOrMutating => "write_or_mutating",
            Self::ComplexShell => "complex_shell",
        }
    }
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

    fn permission_resource_for_command(&self, command: &str) -> String {
        bash_permission_resource(command, &self.working_dir)
    }

    async fn run_command(&self, command: &str, timeout_duration: Duration) -> ToolResult {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

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

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return ToolResult::error(format!("failed to spawn shell: {e}")),
        };

        let stdout_pipe = child.stdout.take().expect("stdout is piped");
        let stderr_pipe = child.stderr.take().expect("stderr is piped");

        let mut stdout_reader = BufReader::new(stdout_pipe).lines();
        let mut stderr_reader = BufReader::new(stderr_pipe).lines();

        let mut output = String::new();
        output.push_str(&format!("$ {command}\n"));

        let exit_status = loop {
            tokio::select! {
                line_result = stdout_reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            output.push_str(&line);
                            output.push('\n');
                        }
                        Ok(None) => {} // stdout closed
                        Err(e) => {
                            output.push_str(&format!("[stdout error: {e}]\n"));
                        }
                    }
                }
                line_result = stderr_reader.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            output.push_str(&line);
                            output.push('\n');
                        }
                        Ok(None) => {} // stderr closed
                        Err(e) => {
                            output.push_str(&format!("[stderr error: {e}]\n"));
                        }
                    }
                }
                status = child.wait() => {
                    // Drain any remaining output after child exits
                    while let Ok(Some(line)) = stdout_reader.next_line().await {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    while let Ok(Some(line)) = stderr_reader.next_line().await {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    break status;
                }
                _ = tokio::time::sleep(timeout_duration) => {
                    let _ = child.kill().await;
                    let _ = child.wait().await;
                    // Drain any remaining output after kill
                    while let Ok(Some(line)) = stdout_reader.next_line().await {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    while let Ok(Some(line)) = stderr_reader.next_line().await {
                        output.push_str(&line);
                        output.push('\n');
                    }
                    output.push_str("[timeout]");
                    return ToolResult::error(output);
                }
            }
        };

        let exit_status = match exit_status {
            Ok(s) => s,
            Err(e) => return ToolResult::error(format!("failed to wait for child: {e}")),
        };

        let exit_code = exit_status.code().unwrap_or(-1);
        output.push_str(&format!("[exit {exit_code}]"));

        ToolResult {
            content: output,
            is_error: !exit_status.success(),
            continuations: Vec::new(),
        }
    }
}

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell command (fallback: prefer grep/glob/ls/read/write/edit/delete for file ops)"
    }

    fn parameters(&self) -> Value {
        talos_core::tool_parameters!(BashInput)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn nature(&self) -> talos_core::tool::ToolNature {
        talos_core::tool::ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Shell
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let class = classify_bash_command(command);
        vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                self.permission_resource_for_command(command),
                ToolResourceKind::Command,
            )
            .with_description(format!("bash {}", class.as_str())),
        ]
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["command"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let bash_input = match parse_input(input) {
            Ok(i) => i,
            Err(e) => return ToolResult::error(e.to_string()),
        };

        const MAX_TIMEOUT_SECS: u64 = 600;
        let timeout_duration = bash_input
            .timeout_secs
            .map(|s| Duration::from_secs(s.clamp(1, MAX_TIMEOUT_SECS)))
            .unwrap_or(self.timeout);

        self.run_command(&bash_input.command, timeout_duration)
            .await
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

    let timeout_secs = obj.get("timeout_secs").and_then(Value::as_u64);

    Ok(BashInput {
        command: command.to_owned(),
        timeout_secs,
    })
}

fn bash_permission_resource(command: &str, cwd: &PathBuf) -> String {
    let normalized = normalize_bash_command(command);
    let class = classify_bash_command(&normalized);
    let mut hasher = DefaultHasher::new();
    cwd.hash(&mut hasher);
    normalized.hash(&mut hasher);
    format!("bash:{}:{:016x}", class.as_str(), hasher.finish())
}

fn normalize_bash_command(command: &str) -> String {
    command.trim().to_string()
}

fn classify_bash_command(command: &str) -> BashCommandClass {
    let normalized = normalize_bash_command(command);
    if normalized.is_empty() || has_shell_control_syntax(&normalized) {
        return BashCommandClass::ComplexShell;
    }

    let mut parts = normalized.split_whitespace();
    let Some(program) = parts.next() else {
        return BashCommandClass::ComplexShell;
    };
    let args: Vec<&str> = parts.collect();

    if is_env_assignment(program) {
        return BashCommandClass::ComplexShell;
    }

    match program {
        "ls" | "pwd" | "cat" | "head" | "tail" | "wc" | "grep" | "rg" | "find" | "stat"
        | "diff" | "sed" | "awk" => BashCommandClass::ReadOnlyInspection,
        "cargo" => classify_cargo(&args),
        "npm" | "pnpm" | "yarn" | "bun" => classify_javascript_tool(program, &args),
        "go" => classify_go(&args),
        "pytest" | "mvn" | "gradle" => BashCommandClass::ValidationBuild,
        "curl" | "wget" | "ssh" | "scp" | "rsync" => BashCommandClass::PackageManagerOrNetwork,
        "rm" | "mv" | "cp" | "mkdir" | "rmdir" | "touch" | "tee" | "chmod" | "chown" | "ln" => {
            BashCommandClass::WriteOrMutating
        }
        "git" => classify_git(&args),
        _ => BashCommandClass::ComplexShell,
    }
}

fn has_shell_control_syntax(command: &str) -> bool {
    command.contains('|')
        || command.contains(';')
        || command.contains('\n')
        || command.contains("&&")
        || command.contains("||")
        || command.contains("$(")
        || command.contains('`')
        || command.contains('>')
        || command.contains('<')
        || command.ends_with('&')
}

fn is_env_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && !name.as_bytes()[0].is_ascii_digit()
}

fn classify_cargo(args: &[&str]) -> BashCommandClass {
    match args.first().copied() {
        Some("test" | "check" | "build" | "clippy") => BashCommandClass::ValidationBuild,
        Some("fmt" | "fix") => BashCommandClass::WriteOrMutating,
        Some("install" | "publish" | "search" | "update") => {
            BashCommandClass::PackageManagerOrNetwork
        }
        _ => BashCommandClass::ComplexShell,
    }
}

fn classify_javascript_tool(program: &str, args: &[&str]) -> BashCommandClass {
    match (program, args.first().copied()) {
        ("bun", Some("test")) => BashCommandClass::ValidationBuild,
        ("bun", Some("install" | "add" | "remove" | "update" | "publish")) => {
            BashCommandClass::PackageManagerOrNetwork
        }
        ("npm" | "pnpm" | "yarn", Some("test" | "run")) => BashCommandClass::ValidationBuild,
        ("npm" | "pnpm" | "yarn", Some("install" | "add" | "remove" | "update" | "publish")) => {
            BashCommandClass::PackageManagerOrNetwork
        }
        _ => BashCommandClass::ComplexShell,
    }
}

fn classify_go(args: &[&str]) -> BashCommandClass {
    match args.first().copied() {
        Some("test" | "build" | "vet") => BashCommandClass::ValidationBuild,
        Some("get" | "install") => BashCommandClass::PackageManagerOrNetwork,
        _ => BashCommandClass::ComplexShell,
    }
}

fn classify_git(args: &[&str]) -> BashCommandClass {
    match args.first().copied() {
        Some("status" | "log" | "show" | "diff" | "branch") => BashCommandClass::ReadOnlyInspection,
        Some("fetch" | "pull" | "push" | "clone") => BashCommandClass::PackageManagerOrNetwork,
        Some(
            "add" | "commit" | "checkout" | "switch" | "merge" | "rebase" | "reset" | "restore"
            | "rm" | "mv" | "tag",
        ) => BashCommandClass::WriteOrMutating,
        _ => BashCommandClass::ComplexShell,
    }
}

#[cfg(test)]
#[allow(warnings)]
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
        assert!(result.content.contains("hello"));
        assert!(result.content.starts_with("$ echo hello\n"));
        assert!(result.content.ends_with("[exit 0]"));
    }

    #[tokio::test]
    async fn test_invalid_command_returns_error() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "nonexistent_command_xyz_123" }))
            .await;

        assert!(result.is_error);
        assert!(
            result
                .content
                .starts_with("$ nonexistent_command_xyz_123\n")
        );
    }

    #[tokio::test]
    async fn test_timeout_works() {
        let tool = BashTool::new(test_dir()).with_timeout(Duration::from_millis(100));
        let result = tool
            .execute(serde_json::json!({ "command": "sleep 10" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("[timeout]"));
    }

    #[tokio::test]
    async fn test_shell_metacharacters_pipe() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "echo hello | tr a-z A-Z" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("HELLO"));
        assert!(result.content.starts_with("$ echo hello | tr a-z A-Z\n"));
        assert!(result.content.ends_with("[exit 0]"));
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
        assert!(result.content.contains("test123"));
        assert!(result.content.ends_with("[exit 0]"));
    }

    #[tokio::test]
    async fn test_working_directory_restriction() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "basename $(pwd)" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("talos-tools"));
        assert!(result.content.ends_with("[exit 0]"));
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
        assert!(tool.description().contains("shell command"));
        assert!(tool.description().contains("fallback"));
    }

    #[test]
    fn test_bash_tool_not_read_only() {
        let tool = BashTool::new(test_dir());
        assert!(!tool.is_read_only());
    }

    #[test]
    fn test_bash_permission_profile_uses_stable_exact_resource() {
        let tool = BashTool::new(test_dir());
        let input = serde_json::json!({ "command": "git status" });

        let first = tool.permission_profile(&input);
        let second = tool.permission_profile(&input);

        assert_eq!(first, second);
        assert_eq!(first[0].nature, ToolNature::Execute);
        assert_eq!(first[0].resource_kind, Some(ToolResourceKind::Command));
        assert!(
            first[0]
                .resource
                .as_deref()
                .unwrap()
                .starts_with("bash:read_only_inspection:")
        );
    }

    #[test]
    fn test_bash_permission_profile_repeated_command_shares_resource() {
        let tool = BashTool::new(test_dir());

        let status = tool.permission_profile(&serde_json::json!({ "command": "git status" }));
        let repeated = tool.permission_profile(&serde_json::json!({ "command": " git status " }));

        assert_eq!(status[0].resource, repeated[0].resource);
    }

    #[test]
    fn test_bash_permission_profile_different_subcommands_do_not_share_resource() {
        let tool = BashTool::new(test_dir());

        let search =
            tool.permission_profile(&serde_json::json!({ "command": "cargo search serde" }));
        let publish = tool.permission_profile(&serde_json::json!({ "command": "cargo publish" }));
        let add = tool.permission_profile(&serde_json::json!({ "command": "git add ." }));
        let reset = tool.permission_profile(&serde_json::json!({ "command": "git reset --hard" }));

        assert_ne!(search[0].resource, publish[0].resource);
        assert_ne!(add[0].resource, reset[0].resource);
    }

    #[test]
    fn test_bash_permission_profile_same_command_across_directories_is_distinct() {
        let first_tool = BashTool::new(PathBuf::from("/tmp/project-a"));
        let second_tool = BashTool::new(PathBuf::from("/tmp/project-b"));
        let input = serde_json::json!({ "command": "git status" });

        let first = first_tool.permission_profile(&input);
        let second = second_tool.permission_profile(&input);

        assert_ne!(first[0].resource, second[0].resource);
    }

    #[test]
    fn test_bash_command_with_shell_operator_is_complex() {
        let tool = BashTool::new(test_dir());
        let profile =
            tool.permission_profile(&serde_json::json!({ "command": "git status && git log" }));

        assert!(
            profile[0]
                .resource
                .as_deref()
                .unwrap()
                .starts_with("bash:complex_shell:")
        );
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

    #[tokio::test]
    async fn test_streaming_command_header() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "echo test" }))
            .await;

        assert!(result.content.starts_with("$ echo test\n"));
    }

    #[tokio::test]
    async fn test_streaming_exit_code_success() {
        let tool = BashTool::new(test_dir());
        let result = tool.execute(serde_json::json!({ "command": "true" })).await;

        assert!(!result.is_error);
        assert!(result.content.ends_with("[exit 0]"));
    }

    #[tokio::test]
    async fn test_streaming_exit_code_failure() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "false" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.ends_with("[exit 1]"));
    }

    #[tokio::test]
    async fn test_streaming_multiline_output() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "printf 'line1\\nline2\\nline3\\n'" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("line1"));
        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
    }

    #[tokio::test]
    async fn test_streaming_stderr_captured() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "echo stderr_msg >&2" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("stderr_msg"));
    }

    #[tokio::test]
    async fn test_streaming_timeout_preserves_partial_output() {
        let tool = BashTool::new(test_dir()).with_timeout(Duration::from_millis(500));
        let result = tool
            .execute(serde_json::json!({
                "command": "echo before_sleep && sleep 10 && echo after_sleep"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("before_sleep"));
        assert!(result.content.contains("[timeout]"));
    }

    #[tokio::test]
    async fn test_streaming_empty_output() {
        let tool = BashTool::new(test_dir());
        let result = tool.execute(serde_json::json!({ "command": "true" })).await;

        assert!(!result.is_error);
        assert!(result.content.starts_with("$ true\n"));
        assert!(result.content.ends_with("[exit 0]"));
    }

    #[tokio::test]
    async fn test_streaming_timeout_input_clamped() {
        let tool = BashTool::new(test_dir());
        let result = tool
            .execute(serde_json::json!({ "command": "sleep 10", "timeout_secs": 0 }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("[timeout]"));
    }
}
