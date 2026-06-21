//! Permission rules engine for gating tool execution.
//!
//! This crate provides a [`PermissionEngine`] that evaluates tool calls against
//! a set of configurable rules. Rules can be loaded from configuration or added
//! programmatically. Each rule specifies a tool name, an optional path pattern,
//! and a decision (allow, deny, or ask).
//!
//! # Default Behavior
//!
//! The engine ships with a default ruleset:
//! - Read tools (name contains "read" or "list") → [`PermissionDecision::Allow`]
//! - Write tools (name contains "write" or "edit") → [`PermissionDecision::Ask`]
//! - Bash tool → [`PermissionDecision::Ask`]
//!
//! # Rule Precedence
//!
//! Rules are evaluated in order. The first matching rule wins. If no rule matches,
//! the default decision is applied based on the tool name.
//!
//! # Path Patterns
//!
//! Path patterns use glob syntax. For example, `src/**/*.rs` matches any `.rs`
//! file under the `src/` directory. Patterns are matched against the `path` field
//! in the tool input JSON.
//!
//! # Example
//!
//! ```
//! use talos_permission::{PermissionEngine, PermissionDecision};
//!
//! let mut engine = PermissionEngine::new();
//!
//! // Read tools are allowed by default
//! let decision = engine.evaluate("read", &serde_json::json!({"path": "Cargo.toml"}));
//! assert!(matches!(decision, PermissionDecision::Allow));
//!
//! // Write tools require approval by default
//! let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
//! assert!(matches!(decision, PermissionDecision::Ask));
//!
//! // Load custom rules from config (prepended, higher precedence)
//! let config = serde_json::json!({
//!     "rules": [{
//!         "tool_name": "write",
//!         "path_pattern": "tmp/**",
//!         "decision": "Allow"
//!     }]
//! });
//! engine.load_from_config(&config).unwrap();
//!
//! let decision = engine.evaluate("write", &serde_json::json!({"path": "tmp/output.txt"}));
//! assert!(matches!(decision, PermissionDecision::Allow));
//! ```

use glob::Pattern;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during permission evaluation.
#[derive(Debug, Error)]
pub enum PermissionError {
    /// The rule configuration is invalid or malformed.
    #[error("invalid permission rule: {0}")]
    InvalidRule(String),

    /// The glob pattern in a rule is malformed.
    #[error("invalid glob pattern: {0}")]
    InvalidGlobPattern(String),
}

/// The decision produced by the permission engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PermissionDecision {
    /// The tool call is permitted without user intervention.
    Allow,
    /// The tool call is blocked with the given reason.
    Deny(String),
    /// The tool call requires user approval before proceeding.
    Ask,
}

impl<'de> Deserialize<'de> for PermissionDecision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(deserializer)?;

        match value {
            Value::String(s) => match s.as_str() {
                "Allow" => Ok(PermissionDecision::Allow),
                "Ask" => Ok(PermissionDecision::Ask),
                "Deny" => Ok(PermissionDecision::Deny(String::new())),
                other => Err(Error::unknown_variant(other, &["Allow", "Deny", "Ask"])),
            },
            Value::Object(mut map) => {
                if let Some(reason) = map.remove("Deny") {
                    let reason = reason
                        .as_str()
                        .map(String::from)
                        .ok_or_else(|| Error::custom("Deny reason must be a string"))?;
                    Ok(PermissionDecision::Deny(reason))
                } else {
                    Err(Error::custom("expected Deny variant"))
                }
            }
            _ => Err(Error::custom(
                "expected string or object for PermissionDecision",
            )),
        }
    }
}

/// A single permission rule that matches tool calls and produces a decision.
///
/// Rules are evaluated in order. The first rule whose `tool_name` matches the
/// invoked tool and whose `path_pattern` (if present) matches the path in the
/// tool input determines the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// The tool name to match. Case-sensitive exact match.
    pub tool_name: String,
    /// Optional glob pattern to match against the `path` field in tool input.
    /// If `None`, the rule matches all invocations of the tool regardless of path.
    pub path_pattern: Option<String>,
    /// The decision to apply when this rule matches.
    pub decision: PermissionDecision,
}

impl PermissionRule {
    /// Creates a new permission rule.
    pub fn new(
        tool_name: impl Into<String>,
        path_pattern: Option<String>,
        decision: PermissionDecision,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            path_pattern,
            decision,
        }
    }

    /// Checks if this rule matches the given tool name and optional path.
    fn matches(&self, tool_name: &str, path: Option<&str>) -> Result<bool, PermissionError> {
        if self.tool_name != tool_name {
            return Ok(false);
        }

        if let Some(ref pattern) = self.path_pattern {
            let path = path.ok_or_else(|| {
                PermissionError::InvalidRule(
                    "rule has path_pattern but tool input has no path field".to_owned(),
                )
            })?;

            let glob = Pattern::new(pattern)
                .map_err(|e| PermissionError::InvalidGlobPattern(format!("{pattern}: {e}")))?;

            return Ok(glob.matches(path));
        }

        Ok(true)
    }
}

/// The permission rules engine.
///
/// Evaluates tool calls against a set of rules and returns a
/// [`PermissionDecision`]. Rules are evaluated in insertion order; the first
/// match wins. If no rule matches, a default decision is applied based on
/// the tool name.
pub struct PermissionEngine {
    /// Ordered list of permission rules.
    pub rules: Vec<PermissionRule>,
    /// Optional workspace root. When set, file operations (read/write/edit)
    /// targeting paths within this directory are auto-allowed.
    pub workspace_root: Option<PathBuf>,
}

impl PermissionEngine {
    /// Creates a new permission engine with the default ruleset.
    ///
    /// Default rules:
    /// - Read tools (name contains "read" or "list") → [`PermissionDecision::Allow`]
    /// - Write tools (name contains "write" or "edit") → [`PermissionDecision::Ask`]
    /// - Bash tool → [`PermissionDecision::Ask`]
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_default_rules();
        engine
    }

    /// Creates a new permission engine that auto-allows file operations
    /// within the given workspace root directory.
    pub fn with_workspace_root(root: PathBuf) -> Self {
        let mut engine = Self::new();
        engine.workspace_root = Some(root);
        engine
    }

    /// Sets the workspace root for auto-approval of workspace-relative paths.
    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    /// Adds the default ruleset to the engine.
    fn add_default_rules(&mut self) {
        self.rules.push(PermissionRule {
            tool_name: "read".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "list".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "grep".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "glob".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "ls".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "diff".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "stat".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "find_symbol".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "find_references".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "list_symbols".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        self.rules.push(PermissionRule {
            tool_name: "list_imports".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });

        self.rules.push(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });
        self.rules.push(PermissionRule {
            tool_name: "edit".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });
        self.rules.push(PermissionRule {
            tool_name: "delete".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });

        self.rules.push(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });

        self.rules.push(PermissionRule {
            tool_name: "http_request".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });

        self.rules.push(PermissionRule {
            tool_name: "web_search".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });

        self.rules.push(PermissionRule {
            tool_name: "fetch_url".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Ask,
        });
    }

    /// Adds a custom rule to the engine.
    ///
    /// Rules are appended to the end of the list, so they have lower precedence
    /// than existing rules. To override a default rule, add the custom rule
    /// before calling [`Self::new`] or use [`Self::load_from_config`] which
    /// prepends custom rules.
    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.rules.push(rule);
    }

    /// Evaluates a tool call against the ruleset and returns a decision.
    ///
    /// Rules are checked in order. The first rule whose `tool_name` matches and
    /// whose `path_pattern` (if present) matches the `path` field in `input`
    /// determines the result.
    ///
    /// If no rule matches, the default decision is applied:
    /// - Tools with names containing "read" or "list" → [`PermissionDecision::Allow`]
    /// - Tools with names containing "write" or "edit" → [`PermissionDecision::Ask`]
    /// - All other tools → [`PermissionDecision::Ask`]
    ///
    /// When a [`workspace_root`](Self::workspace_root) is set, file operations
    /// (read/write/edit/list) targeting paths within that directory are
    /// auto-allowed before rule evaluation.
    pub fn evaluate(&self, tool_name: &str, input: &Value) -> PermissionDecision {
        let nature = infer_nature(tool_name);
        self.evaluate_with_nature(tool_name, nature, input)
    }

    pub fn evaluate_with_nature(
        &self,
        tool_name: &str,
        nature: talos_core::tool::ToolNature,
        input: &Value,
    ) -> PermissionDecision {
        if let Some(ref root) = self.workspace_root
            && nature == talos_core::tool::ToolNature::Read
            && is_workspace_path_allowed(input, root)
        {
            return PermissionDecision::Allow;
        }

        let path = input.get("path").and_then(Value::as_str);

        for rule in &self.rules {
            match rule.matches(tool_name, path) {
                Ok(true) => return rule.decision.clone(),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }

        match nature {
            talos_core::tool::ToolNature::Read => PermissionDecision::Allow,
            talos_core::tool::ToolNature::Write
            | talos_core::tool::ToolNature::Execute
            | talos_core::tool::ToolNature::Network => PermissionDecision::Ask,
        }
    }

    /// Loads rules from a JSON configuration value.
    ///
    /// The config should be an object with a `rules` array, where each element
    /// has `tool_name`, optional `path_pattern`, and `decision` fields.
    ///
    /// Custom rules are prepended to the existing ruleset, giving them higher
    /// precedence than defaults.
    ///
    /// # Example config
    ///
    /// ```json
    /// {
    ///   "rules": [
    ///     {
    ///       "tool_name": "bash",
    ///       "path_pattern": null,
    ///       "decision": "Allow"
    ///     },
    ///     {
    ///       "tool_name": "write",
    ///       "path_pattern": "src/**/*.rs",
    ///       "decision": "Deny"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn load_from_config(&mut self, config: &Value) -> Result<(), PermissionError> {
        let rules_array = config
            .get("rules")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                PermissionError::InvalidRule("config must contain a 'rules' array".to_owned())
            })?;

        let mut custom_rules = Vec::new();
        for (i, rule_value) in rules_array.iter().enumerate() {
            let rule: PermissionRule = serde_json::from_value(rule_value.clone())
                .map_err(|e| PermissionError::InvalidRule(format!("rule at index {i}: {e}")))?;
            custom_rules.push(rule);
        }

        // Prepend custom rules so they take precedence over defaults
        let mut all_rules = custom_rules;
        all_rules.append(&mut self.rules);
        self.rules = all_rules;

        Ok(())
    }
}

/// Returns true if the tool is a file operation that should benefit from
/// workspace-relative auto-approval.
fn infer_nature(tool_name: &str) -> talos_core::tool::ToolNature {
    let name_lower = tool_name.to_lowercase();
    if name_lower.contains("read")
        || name_lower.contains("list")
        || name_lower == "grep"
        || name_lower == "glob"
        || name_lower == "ls"
        || name_lower == "diff"
        || name_lower == "stat"
        || name_lower.starts_with("find")
    {
        talos_core::tool::ToolNature::Read
    } else if name_lower == "bash" || name_lower == "sh" {
        talos_core::tool::ToolNature::Execute
    } else if name_lower == "http_request" || name_lower == "web_search" || name_lower == "fetch_url" {
        talos_core::tool::ToolNature::Network
    } else {
        talos_core::tool::ToolNature::Write
    }
}

fn is_workspace_path_allowed(input: &Value, root: &Path) -> bool {
    for key in &["path", "file"] {
        if let Some(path_str) = input.get(*key).and_then(Value::as_str)
            && is_path_in_workspace(path_str, root)
        {
            return true;
        }
    }
    false
}

/// Checks whether `path` is within (or relative to) the workspace `root`.
///
/// Relative paths are assumed to be workspace-relative. Absolute paths are
/// checked with `starts_with`.
fn is_path_in_workspace(path_str: &str, root: &Path) -> bool {
    let path = Path::new(path_str);
    if path.is_relative() {
        return true;
    }
    path.starts_with(root)
}

impl Default for PermissionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Default ruleset tests ---

    #[test]
    fn test_default_read_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("read", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_list_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("list", &serde_json::json!({"path": "src"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_write_tool_ask() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_default_edit_tool_ask() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_default_bash_tool_ask() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_default_grep_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("grep", &serde_json::json!({"pattern": "fn"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_glob_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("glob", &serde_json::json!({"pattern": "*.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_ls_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("ls", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_delete_tool_ask() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("delete", &serde_json::json!({"path": "temp.txt"}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_default_find_symbol_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("find_symbol", &serde_json::json!({"name": "Tool"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_find_references_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate(
            "find_references",
            &serde_json::json!({"name": "Tool", "file": "src/main.rs"}),
        );
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_list_symbols_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("list_symbols", &serde_json::json!({"path": "src/lib.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_list_imports_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("list_imports", &serde_json::json!({"file": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_workspace_auto_allow_file_param() {
        let engine = PermissionEngine::with_workspace_root(PathBuf::from("/tmp"));
        let decision = engine.evaluate(
            "find_references",
            &serde_json::json!({"name": "Tool", "file": "src/main.rs"}),
        );
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_workspace_auto_allow_path_param() {
        let engine = PermissionEngine::with_workspace_root(PathBuf::from("/tmp"));
        let decision = engine.evaluate(
            "find_symbol",
            &serde_json::json!({"name": "Tool", "path": "src/main.rs"}),
        );
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_default_unknown_tool_ask() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate("custom_tool", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    // --- Custom rule tests ---

    #[test]
    fn test_custom_rule_allow_bash() {
        let mut engine = PermissionEngine::new();
        engine.add_rule(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });

        // Custom rule is appended, so default bash rule still matches first
        // We need to test with a new engine where we control rule order
        let mut engine2 = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine2.add_rule(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });

        let decision = engine2.evaluate("bash", &serde_json::json!({"command": "ls"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_custom_rule_deny_write_to_sensitive_path() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: Some(".env".to_owned()),
            decision: PermissionDecision::Deny("sensitive file".to_owned()),
        });

        let decision = engine.evaluate("write", &serde_json::json!({"path": ".env"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("sensitive file".to_owned())
        );
    }

    // --- Path pattern matching tests ---

    #[test]
    fn test_path_pattern_src_glob_matches() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "read".to_owned(),
            path_pattern: Some("src/**/*.rs".to_owned()),
            decision: PermissionDecision::Allow,
        });

        let decision = engine.evaluate("read", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_path_pattern_src_glob_nested() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "read".to_owned(),
            path_pattern: Some("src/**/*.rs".to_owned()),
            decision: PermissionDecision::Allow,
        });

        let decision =
            engine.evaluate("read", &serde_json::json!({"path": "src/utils/helpers.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_path_pattern_src_glob_no_match() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "read".to_owned(),
            path_pattern: Some("src/**/*.rs".to_owned()),
            decision: PermissionDecision::Allow,
        });

        let decision = engine.evaluate("read", &serde_json::json!({"path": "tests/main.rs"}));
        // No rule matches, default for "read" is Allow
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_path_pattern_deny_outside_src() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: Some("src/**/*.rs".to_owned()),
            decision: PermissionDecision::Allow,
        });
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("only src allowed".to_owned()),
        });

        let decision = engine.evaluate("write", &serde_json::json!({"path": "tests/main.rs"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("only src allowed".to_owned())
        );

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/lib.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    // --- Rule precedence tests ---

    #[test]
    fn test_first_match_wins() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });
        engine.add_rule(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("blocked".to_owned()),
        });

        let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_specific_rule_before_general() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: Some("tmp/**".to_owned()),
            decision: PermissionDecision::Allow,
        });
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("write not allowed".to_owned()),
        });

        let decision = engine.evaluate("write", &serde_json::json!({"path": "tmp/out.txt"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write not allowed".to_owned())
        );
    }

    // --- Load from config tests ---

    #[test]
    fn test_load_from_config() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({
            "rules": [
                {
                    "tool_name": "bash",
                    "path_pattern": null,
                    "decision": "Allow"
                },
                {
                    "tool_name": "write",
                    "path_pattern": "src/**/*.rs",
                    "decision": "Deny"
                }
            ]
        });

        engine
            .load_from_config(&config)
            .expect("config should load");

        // Custom bash rule is prepended, so it matches first
        let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
        assert_eq!(decision, PermissionDecision::Allow);

        // Write to src/ is denied by custom rule
        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Deny("".to_owned()));
    }

    #[test]
    fn test_load_from_config_invalid() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({"not_rules": []});

        let result = engine.load_from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_config_malformed_rule() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({
            "rules": [
                {"tool_name": 123}
            ]
        });

        let result = engine.load_from_config(&config);
        assert!(result.is_err());
    }

    // --- Edge cases ---

    #[test]
    fn test_tool_name_case_insensitive_default() {
        let engine = PermissionEngine::new();
        // Default decision uses lowercase comparison
        let decision = engine.evaluate("READ", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("Write", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_tool_name_exact_match_in_rules() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule {
            tool_name: "read".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Allow,
        });

        // Rule matching is case-sensitive
        let decision = engine.evaluate("READ", &serde_json::json!({}));
        // No rule matches, falls through to default which is case-insensitive
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_empty_rules_falls_to_default() {
        let engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        let decision = engine.evaluate("read", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_permission_rule_new() {
        let rule = PermissionRule::new(
            "bash",
            Some("safe/**".to_owned()),
            PermissionDecision::Allow,
        );
        assert_eq!(rule.tool_name, "bash");
        assert_eq!(rule.path_pattern, Some("safe/**".to_owned()));
        assert_eq!(rule.decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_permission_decision_serialization() {
        let allow = PermissionDecision::Allow;
        let json = serde_json::to_string(&allow).expect("serialize");
        assert_eq!(json, "\"Allow\"");

        let deny = PermissionDecision::Deny("nope".to_owned());
        let json = serde_json::to_string(&deny).expect("serialize");
        assert_eq!(json, "{\"Deny\":\"nope\"}");

        let ask = PermissionDecision::Ask;
        let json = serde_json::to_string(&ask).expect("serialize");
        assert_eq!(json, "\"Ask\"");
    }

    #[test]
    fn test_permission_rule_serialization() {
        let rule = PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: Some("src/**".to_owned()),
            decision: PermissionDecision::Ask,
        };
        let json = serde_json::to_string(&rule).expect("serialize");
        let parsed: PermissionRule = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.tool_name, "write");
        assert_eq!(parsed.path_pattern, Some("src/**".to_owned()));
        assert_eq!(parsed.decision, PermissionDecision::Ask);
    }
}
