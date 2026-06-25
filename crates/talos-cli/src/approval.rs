//! Interactive approval prompt for tool calls requiring permission.
//!
//! When a tool call triggers [`PermissionDecision::Ask`], this module presents
//! a prompt to the user in the terminal. The user can approve once, approve
//! always (adding a rule to the engine), or deny.
//!
//! # Print Mode Behavior
//!
//! In print mode (`-p` flag), interactive prompts are not available. The caller
//! should treat [`PermissionDecision::Ask`] as [`PermissionDecision::Deny`]
//! without invoking [`ApprovalPrompt::prompt`].

use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};
use talos_core::tool::ToolNature;
use talos_permission::{
    PermissionDecision, PermissionEngine, PermissionRule, ResourceExtractor, ResourceKind,
};

/// Maximum length for formatted tool input before truncation.
const MAX_INPUT_LENGTH: usize = 200;

/// Truncation suffix appended when input is truncated.
const TRUNCATION_SUFFIX: &str = "... (truncated)";

/// Interactive approval prompt for tool calls requiring user permission.
///
/// Wraps a [`PermissionEngine`] and provides a terminal-based prompt that
/// allows the user to approve, always approve, or deny a tool call.
/// When the user chooses "always approve", a new rule is added to the engine.
///
/// # Thread Safety
///
/// This struct is designed to be shared across threads via `Arc<Mutex<ApprovalPrompt>>`.
/// The internal [`PermissionEngine`] uses interior mutability for rule addition.
pub struct ApprovalPrompt {
    /// The permission engine used for evaluation and rule management.
    engine: PermissionEngine,
}

impl ApprovalPrompt {
    /// Creates a new approval prompt with the given permission engine.
    pub fn new(engine: PermissionEngine) -> Self {
        Self { engine }
    }

    /// Returns a reference to the underlying permission engine.
    pub fn engine(&self) -> &PermissionEngine {
        &self.engine
    }

    /// Presents an approval prompt to the user for the given tool call.
    ///
    /// Prints a formatted prompt to stderr showing the tool name, arguments,
    /// and available actions. Reads a single character from stdin:
    /// - `y` — approve once, returns [`PermissionDecision::Allow`]
    /// - `a` — always approve, adds an allow rule to the engine and returns [`PermissionDecision::Allow`]
    /// - `n` — deny, returns [`PermissionDecision::Deny`]
    ///
    /// Invalid input causes the prompt to be re-displayed.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from stdin fails.
    pub fn prompt(
        &mut self,
        tool_name: &str,
        nature: ToolNature,
        input: &serde_json::Value,
    ) -> Result<PermissionDecision> {
        let formatted = Self::format_input(input);

        loop {
            eprintln!();
            eprintln!("⚠ Tool requires approval: {tool_name}");
            eprintln!("Arguments: {formatted}");
            eprintln!();
            eprintln!("[y] Approve once  [a] Always approve  [n] Deny");
            eprint!("> ");
            io::stderr().flush().context("failed to flush stderr")?;

            let mut line = String::new();
            io::stdin()
                .lock()
                .read_line(&mut line)
                .context("failed to read from stdin")?;

            match line.trim() {
                "y" => return Ok(PermissionDecision::Allow),
                "a" => {
                    let resource = ResourceExtractor::extract(nature, input);
                    let resource_kind = match nature {
                        ToolNature::Network => Some(ResourceKind::Domain),
                        _ => Some(ResourceKind::Path),
                    };
                    let rule = PermissionRule::new_nature(
                        nature,
                        resource,
                        resource_kind,
                        PermissionDecision::Allow,
                    );
                    self.engine.add_rule(rule);
                    return Ok(PermissionDecision::Allow);
                }
                "n" => {
                    return Ok(PermissionDecision::Deny("User denied".to_string()));
                }
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

#[cfg(test)]
mod tests {
    use super::*;

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
