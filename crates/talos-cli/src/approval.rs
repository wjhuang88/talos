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
use std::path::Path;

use anyhow::{Context, Result};
use talos_core::tool::{ToolNature, ToolPermissionFacet};
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

    /// Presents an approval prompt for a multi-facet tool permission profile.
    ///
    /// Prints a formatted prompt to stderr showing the tool name, arguments,
    /// and available actions. Reads a single character from stdin:
    /// - `y` — approve once, returns [`PermissionDecision::Allow`]
    /// - `a` — always approve, adds allow rules for all facets and returns
    ///   [`PermissionDecision::Allow`]
    /// - `n` — deny, returns [`PermissionDecision::Deny`]
    ///
    /// Invalid input causes the prompt to be re-displayed.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from stdin fails.
    pub fn prompt_profile(
        &mut self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        input: &serde_json::Value,
    ) -> Result<PermissionDecision> {
        let formatted = Self::format_input(input);
        let always_scopes = always_allow_rule_descriptions(tool_name, profile, input);

        loop {
            eprintln!();
            eprintln!("⚠ Tool requires approval: {tool_name}");
            eprintln!("Arguments: {formatted}");
            if !always_scopes.is_empty() {
                eprintln!("Always approve scope:");
                for scope in &always_scopes {
                    eprintln!("  - {scope}");
                }
            }
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
                    add_always_allow_rules(&mut self.engine, tool_name, profile, input);
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

pub(crate) fn add_always_allow_rules(
    engine: &mut PermissionEngine,
    tool_name: &str,
    profile: &[ToolPermissionFacet],
    input: &serde_json::Value,
) {
    for rule in always_allow_rules(tool_name, profile, input) {
        engine.add_runtime_allow_rule(rule);
    }
}

pub(crate) fn always_allow_rules(
    tool_name: &str,
    profile: &[ToolPermissionFacet],
    input: &serde_json::Value,
) -> Vec<PermissionRule> {
    permission_facets_or_default(profile, tool_name)
        .into_iter()
        .map(|facet| {
            let mut resource = facet
                .resource
                .clone()
                .or_else(|| ResourceExtractor::extract(facet.nature, input));
            let resource_kind = facet
                .resource_kind
                .map(ResourceKind::from)
                .or_else(|| Some(default_resource_kind(facet.nature)));

            if facet.nature == ToolNature::Write && resource_kind == Some(ResourceKind::Path) {
                resource = resource.map(|path| write_always_scope(&path));
            }

            PermissionRule::new_nature(
                facet.nature,
                resource,
                resource_kind,
                PermissionDecision::Allow,
            )
        })
        .collect()
}

pub(crate) fn always_allow_rule_descriptions(
    tool_name: &str,
    profile: &[ToolPermissionFacet],
    input: &serde_json::Value,
) -> Vec<String> {
    always_allow_scope_entries(tool_name, profile, input)
        .into_iter()
        .map(|entry| {
            format!(
                "session allow: {} {} `{}`; configured deny rules still win",
                entry.nature, entry.resource_kind, entry.resource
            )
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AlwaysAllowScopeEntry {
    pub(crate) nature: String,
    pub(crate) resource_kind: String,
    pub(crate) resource: String,
}

pub(crate) fn always_allow_scope_entries(
    tool_name: &str,
    profile: &[ToolPermissionFacet],
    input: &serde_json::Value,
) -> Vec<AlwaysAllowScopeEntry> {
    always_allow_rules(tool_name, profile, input)
        .into_iter()
        .map(|rule| {
            let nature = rule
                .nature
                .map(|nature| format!("{nature:?}").to_ascii_lowercase())
                .unwrap_or_else(|| tool_name.to_string());
            let resource = rule.resource.unwrap_or_else(|| "*".to_string());
            let resource_kind = rule
                .resource_kind
                .map(|kind| format!("{kind:?}").to_ascii_lowercase())
                .unwrap_or_else(|| "any".to_string());
            AlwaysAllowScopeEntry {
                nature,
                resource_kind,
                resource,
            }
        })
        .collect()
}

fn write_always_scope(resource: &str) -> String {
    if resource.ends_with('/') {
        return directory_glob(resource);
    }

    let path = Path::new(resource);
    match path.parent().and_then(Path::to_str) {
        Some(parent) if !parent.is_empty() && parent != "." => directory_glob(parent),
        _ => resource.to_string(),
    }
}

fn directory_glob(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        return path.to_string();
    }
    format!("{trimmed}/**")
}

fn permission_facets_or_default(
    profile: &[ToolPermissionFacet],
    _tool_name: &str,
) -> Vec<ToolPermissionFacet> {
    if profile.is_empty() {
        vec![ToolPermissionFacet::new(ToolNature::Write)]
    } else {
        profile.to_vec()
    }
}

fn default_resource_kind(nature: ToolNature) -> ResourceKind {
    match nature {
        ToolNature::Network => ResourceKind::Domain,
        ToolNature::Execute => ResourceKind::Command,
        ToolNature::Read | ToolNature::Write => ResourceKind::Path,
        ToolNature::Internal => ResourceKind::Remote,
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

    #[test]
    fn test_always_allow_write_scopes_to_parent_directory() {
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "crates/talos-cli/src/main.rs",
            talos_core::tool::ToolResourceKind::Path,
        )];

        let rules = always_allow_rules("write", &profile, &serde_json::json!({}));

        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].resource.as_deref(),
            Some("crates/talos-cli/src/**")
        );
        assert_eq!(rules[0].resource_kind, Some(ResourceKind::Path));
    }

    #[test]
    fn test_always_allow_root_write_stays_file_scoped() {
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "Cargo.toml",
            talos_core::tool::ToolResourceKind::Path,
        )];

        let rules = always_allow_rules("write", &profile, &serde_json::json!({}));

        assert_eq!(rules[0].resource.as_deref(), Some("Cargo.toml"));
    }

    #[test]
    fn test_always_allow_rule_is_effective_against_default_ask() {
        let mut engine = PermissionEngine::new();
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "bash:read_only_inspection:abc",
            talos_core::tool::ToolResourceKind::Command,
        )];

        add_always_allow_rules(&mut engine, "bash", &profile, &serde_json::json!({}));

        assert_eq!(
            engine.evaluate_profile("bash", &profile, &serde_json::json!({})),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn test_always_allow_descriptions_show_reusable_scope() {
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "src/main.rs",
            talos_core::tool::ToolResourceKind::Path,
        )];

        let descriptions =
            always_allow_rule_descriptions("write", &profile, &serde_json::json!({}));

        assert_eq!(
            descriptions,
            vec!["session allow: write path `src/**`; configured deny rules still win"]
        );
    }

    #[test]
    fn test_configured_deny_precedes_runtime_always_allow() {
        let mut engine = PermissionEngine::new();
        engine
            .load_from_config(&serde_json::json!({
                "rules": [{
                    "nature": "Execute",
                    "resource": "bash:validation_build:*",
                    "resource_kind": "command",
                    "decision": {"Deny": "validation builds are blocked in this workspace"}
                }]
            }))
            .unwrap();
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "bash:validation_build:abc123",
            talos_core::tool::ToolResourceKind::Command,
        )];

        add_always_allow_rules(&mut engine, "bash", &profile, &serde_json::json!({}));

        assert_eq!(
            engine.evaluate_profile("bash", &profile, &serde_json::json!({})),
            PermissionDecision::Deny("validation builds are blocked in this workspace".to_string())
        );
    }

    #[test]
    fn test_repeated_always_approval_reduces_same_operation_to_zero_prompts() {
        let mut engine = PermissionEngine::new();
        let input = serde_json::json!({});
        let profile = vec![ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "bash:read_only_inspection:trace",
            talos_core::tool::ToolResourceKind::Command,
        )];

        assert_eq!(
            engine.evaluate_profile("bash", &profile, &input),
            PermissionDecision::Ask
        );
        add_always_allow_rules(&mut engine, "bash", &profile, &input);

        let repeated_asks = (0..5)
            .filter(|_| {
                engine.evaluate_profile("bash", &profile, &input) == PermissionDecision::Ask
            })
            .count();
        assert_eq!(repeated_asks, 0);
    }

    #[test]
    fn test_low_risk_bash_template_reduces_different_object_prompts() {
        use talos_core::tool::AgentTool;

        let tool = talos_tools::BashTool::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        let first_input = serde_json::json!({ "command": "cat src/lib.rs" });
        let second_input = serde_json::json!({ "command": "cat Cargo.toml" });
        let first_profile = tool.permission_profile(&first_input);
        let second_profile = tool.permission_profile(&second_input);
        let mut engine = PermissionEngine::new();

        assert_eq!(first_profile[0].resource, second_profile[0].resource);
        assert_eq!(
            engine.evaluate_profile("bash", &first_profile, &first_input),
            PermissionDecision::Ask
        );

        add_always_allow_rules(&mut engine, "bash", &first_profile, &first_input);

        assert_eq!(
            engine.evaluate_profile("bash", &second_profile, &second_input),
            PermissionDecision::Allow
        );
    }
}
