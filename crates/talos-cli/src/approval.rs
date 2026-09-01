//! Interactive approval prompt for tool calls requiring permission.
//!
//! When a tool call triggers [`PermissionDecision::Ask`], this module presents
//! a prompt to the user in the terminal. The permission-aware wrapper owns
//! authority compilation and commit; this module only renders the choice.
//!
//! # Print Mode Behavior
//!
//! In print mode (`-p` flag), interactive prompts are not available. The caller
//! should treat [`PermissionDecision::Ask`] as [`PermissionDecision::Deny`]
//! without invoking [`ApprovalPrompt::prompt`].

use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
#[cfg(test)]
use talos_agent::permission_pipeline::PermissionBinding;
use talos_agent::permission_pipeline::{
    ApprovalResolver, ApprovalResolverError, PermissionApprovalRequest,
};
use talos_core::ApprovalChoice;
use talos_core::tool::ToolNature;
use talos_permission::{GrantPreview, PermissionEngine, PermissionSessionState};

#[cfg(test)]
use std::io::BufRead;

/// Maximum length for formatted tool input before truncation.
const MAX_INPUT_LENGTH: usize = 200;

/// Truncation suffix appended when input is truncated.
const TRUNCATION_SUFFIX: &str = "... (truncated)";

/// Interactive approval prompt for tool calls requiring user permission.
///
/// Provides a terminal-based choice renderer and owns the in-memory
/// permission Session shared by wrappers in this composition root.
///
/// # Thread Safety
///
/// This struct is designed to be shared across threads via `Arc<Mutex<ApprovalPrompt>>`.
/// The in-memory [`PermissionSessionState`] owns explicit Session grants; configured policy
/// remains separate and is never mutated by an approval choice.
pub struct ApprovalPrompt {
    state: Arc<PermissionSessionState>,
}

/// One approval request consumed by the interactive event loop's sole stdin reader.
pub(crate) struct TerminalApprovalRequest {
    pub(crate) id: uuid::Uuid,
    pub(crate) request: PermissionApprovalRequest,
    pub(crate) response: tokio::sync::oneshot::Sender<ApprovalChoice>,
}

/// Terminal adapter that delegates input ownership to the interactive event loop.
pub(crate) struct TerminalApprovalResolver {
    request_tx: tokio::sync::mpsc::UnboundedSender<TerminalApprovalRequest>,
}

pub(crate) fn terminal_approval_channel() -> (
    Arc<TerminalApprovalResolver>,
    tokio::sync::mpsc::UnboundedReceiver<TerminalApprovalRequest>,
) {
    let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();
    (
        Arc::new(TerminalApprovalResolver { request_tx }),
        request_rx,
    )
}

#[async_trait]
impl ApprovalResolver for TerminalApprovalResolver {
    async fn resolve(
        &self,
        request: PermissionApprovalRequest,
        remaining: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        let (response, response_rx) = tokio::sync::oneshot::channel();
        self.request_tx
            .send(TerminalApprovalRequest {
                id: uuid::Uuid::new_v4(),
                request,
                response,
            })
            .map_err(|_| ApprovalResolverError::new("terminal approval channel unavailable"))?;
        tokio::time::timeout(remaining, response_rx)
            .await
            .map_err(|_| ApprovalResolverError::new("terminal approval deadline exceeded"))?
            .map_err(|_| ApprovalResolverError::new("terminal approval cancelled"))
    }
}

impl ApprovalPrompt {
    pub(crate) fn render_choice_prompt(
        tool_name: &str,
        input: &serde_json::Value,
        preview: &GrantPreview,
    ) -> Result<()> {
        let formatted = Self::format_input(input);
        eprintln!();
        eprintln!("⚠ Tool requires approval: {tool_name}");
        eprintln!("Arguments: {formatted}");
        if !preview.facets().is_empty() {
            eprintln!("Always approve scope:");
            for facet in preview.facets() {
                eprintln!(
                    "  - session allow: {:?} {:?} `{}`; configured deny rules still win",
                    facet.nature, facet.resource_kind, facet.normalized_scope
                );
            }
        }
        eprintln!();
        eprintln!("[y] Approve once  [a] Always approve  [n] Deny");
        eprint!("> ");
        io::stderr().flush().context("failed to flush stderr")?;
        Ok(())
    }

    /// Creates a new approval prompt with the given permission engine.
    pub fn new(engine: PermissionEngine) -> Self {
        Self {
            state: Arc::new(PermissionSessionState::new(engine)),
        }
    }

    /// Returns the shared in-memory permission Session.
    pub(crate) fn session_state(&self) -> Arc<PermissionSessionState> {
        self.state.clone()
    }

    /// Presents an approval prompt for a multi-facet tool permission profile.
    ///
    /// Prints a formatted prompt to stderr showing the tool name, arguments,
    /// and available actions. Reads a single character from stdin:
    /// - `y` — approve once, returns [`PermissionDecision::Allow`]
    /// - `a` — always approve, installs an in-memory Session grant for all facets and returns
    ///   [`PermissionDecision::Allow`]
    /// - `n` — deny, returns [`PermissionDecision::Deny`]
    ///
    /// Invalid input causes the prompt to be re-displayed.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from stdin fails.
    #[cfg(test)]
    pub fn prompt_choice(
        tool_name: &str,
        input: &serde_json::Value,
        preview: &GrantPreview,
    ) -> Result<ApprovalChoice> {
        loop {
            Self::render_choice_prompt(tool_name, input, preview)?;

            let mut line = String::new();
            io::stdin()
                .lock()
                .read_line(&mut line)
                .context("failed to read from stdin")?;

            match line.trim() {
                "y" => return Ok(ApprovalChoice::ApproveOnce),
                "a" => return Ok(ApprovalChoice::AlwaysApprove),
                "n" => return Ok(ApprovalChoice::Deny),
                _ => {
                    eprintln!("Invalid input. Please enter y, a, or n.");
                    continue;
                }
            }
        }
    }

    /// Formats a JSON value for display in the approval prompt.
    ///
    /// Pretty-prints the JSON value. If the formatted output exceeds
    /// [`MAX_INPUT_LENGTH`] characters, it is truncated with a suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use talos_cli::approval::ApprovalPrompt;
    ///
    /// let input = serde_json::json!({"path": "src/main.rs", "content": "hello"});
    /// let formatted = ApprovalPrompt::format_input(&input);
    /// assert!(formatted.contains("path"));
    /// ```
    pub fn format_input(input: &serde_json::Value) -> String {
        let pretty = serde_json::to_string_pretty(input).unwrap_or_else(|_| input.to_string());

        if pretty.len() <= MAX_INPUT_LENGTH {
            pretty
        } else {
            let truncated = pretty.chars().take(MAX_INPUT_LENGTH).collect::<String>();
            format!("{truncated}{TRUNCATION_SUFFIX}")
        }
    }

    /// Returns approval info for TUI mode without blocking.
    ///
    /// The TUI renders the approval overlay and handles user interaction.
    /// This method simply returns the tool name and formatted arguments
    /// so the TUI can display them.
    #[allow(dead_code)]
    pub fn prompt_tui(
        tool_name: &str,
        nature: ToolNature,
        input: &serde_json::Value,
    ) -> (String, String, ToolNature) {
        let formatted = Self::format_input(input);
        (tool_name.to_string(), formatted, nature)
    }
}

pub(crate) fn format_grant_preview(preview: &GrantPreview) -> String {
    preview
        .facets()
        .iter()
        .map(|facet| {
            format!(
                "{:?} {:?}: {}",
                facet.nature, facet.resource_kind, facet.normalized_scope
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
    use talos_permission::{
        InteractionCapability, PermissionContext, PermissionInvocation, PermissionMode,
        PermissionRequest,
    };

    fn approval_request() -> PermissionApprovalRequest {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("target.txt");
        std::fs::write(&target, b"fixture").expect("fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let target_text = target.display().to_string();
        let input = serde_json::json!({"path": target_text.clone()});
        let profile = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            target_text,
            ToolResourceKind::Path,
        )];
        let permission_request =
            PermissionRequest::new("write", ToolProvenance::Native, &profile, &input);
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let PermissionInvocation::Ask { session, .. } = state
            .begin_invocation(&permission_request, &context)
            .expect("approval proposal")
        else {
            panic!("write should require approval")
        };
        PermissionApprovalRequest {
            tool_name: "write".to_owned(),
            provenance: ToolProvenance::Native,
            arguments: input,
            summary_fields: vec!["path".to_owned()],
            preview: session.preview().clone(),
            binding: PermissionBinding {
                session_id: state.session_id().expect("session id").stable_id(),
                revisions: state
                    .state_snapshot()
                    .expect("snapshot")
                    .revisions
                    .as_array(),
                mode: context.mode(),
                interaction: context.interaction(),
            },
        }
    }

    #[test]
    fn test_format_input_simple_object() {
        let input = serde_json::json!({
            "path": "src/main.rs",
            "content": "hello world"
        });
        let formatted = ApprovalPrompt::format_input(&input);
        assert!(formatted.contains("path"));
        assert!(formatted.contains("src/main.rs"));
        assert!(formatted.contains("content"));
        assert!(formatted.contains("hello world"));
    }

    #[tokio::test]
    async fn expired_terminal_request_cannot_consume_input_and_next_request_resolves() {
        let (resolver, mut requests) = terminal_approval_channel();
        let first_resolver = resolver.clone();
        let first_request = approval_request();
        let first = tokio::spawn(async move {
            first_resolver
                .resolve(first_request, Duration::from_millis(5))
                .await
        });
        let expired = tokio::time::timeout(Duration::from_secs(1), requests.recv())
            .await
            .expect("first request arrives")
            .expect("first request queued");
        assert!(first.await.expect("first task").is_err());
        assert!(expired.response.is_closed());

        let second_resolver = resolver.clone();
        let second_request = approval_request();
        let second = tokio::spawn(async move {
            second_resolver
                .resolve(second_request, Duration::from_secs(1))
                .await
        });
        let current = tokio::time::timeout(Duration::from_secs(1), requests.recv())
            .await
            .expect("second request arrives")
            .expect("second request queued");
        current
            .response
            .send(ApprovalChoice::ApproveOnce)
            .expect("second response accepted");
        assert_eq!(
            second.await.expect("second task").expect("approval"),
            ApprovalChoice::ApproveOnce
        );
    }

    #[test]
    fn test_format_input_long_json_truncation() {
        // Create a JSON object that exceeds MAX_INPUT_LENGTH characters
        let mut input = serde_json::Map::new();
        for i in 0..50 {
            input.insert(
                format!("key_{i:03}"),
                serde_json::Value::String(format!("value_{i:03}_with_some_extra_text")),
            );
        }
        let input = serde_json::Value::Object(input);
        let formatted = ApprovalPrompt::format_input(&input);
        assert!(formatted.len() <= MAX_INPUT_LENGTH + TRUNCATION_SUFFIX.len());
        assert!(formatted.ends_with(TRUNCATION_SUFFIX));
    }

    #[test]
    fn test_format_input_empty_object() {
        let input = serde_json::json!({});
        let formatted = ApprovalPrompt::format_input(&input);
        assert_eq!(formatted, "{}");
    }

    #[test]
    fn test_format_input_nested_object() {
        let input = serde_json::json!({
            "path": "src/main.rs",
            "nested": {
                "key": "value",
                "array": [1, 2, 3]
            }
        });
        let formatted = ApprovalPrompt::format_input(&input);
        assert!(formatted.contains("nested"));
        assert!(formatted.contains("array"));
    }

    #[test]
    fn test_format_input_array() {
        let input = serde_json::json!(["item1", "item2", "item3"]);
        let formatted = ApprovalPrompt::format_input(&input);
        assert!(formatted.contains("item1"));
        assert!(formatted.contains("item2"));
    }

    #[test]
    fn test_format_input_short_no_truncation() {
        let input = serde_json::json!({ "data": "short value" });
        let formatted = ApprovalPrompt::format_input(&input);
        assert!(!formatted.ends_with(TRUNCATION_SUFFIX));
        assert!(formatted.contains("short value"));
    }
}
