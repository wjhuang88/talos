//! Permission rules engine for gating tool execution.
//!
//! Rules match on [`talos_core::tool::ToolNature`] (Read/Write/Execute/Network)
//! plus an optional resource pattern (glob path or domain). Legacy rules that
//! only specify `tool_name` are supported for backward compatibility — their
//! nature is inferred from the tool name at config load time.
//!
//! # Default Behavior
//!
//! The engine ships with nature-based defaults:
//! - Read → [`PermissionDecision::Allow`]
//! - Write → [`PermissionDecision::Ask`]
//! - Execute → [`PermissionDecision::Ask`]
//! - Network → [`PermissionDecision::Ask`]
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
use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
use thiserror::Error;
use url::Url;

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

/// How to interpret the `resource` field in a [`PermissionRule`].
///
/// Determines whether the resource string is treated as a file path glob
/// or a URL host pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    /// Glob matched against a file path (Read, Write, Execute tools).
    Path,
    /// Glob or exact match against a URL host (Network tools).
    Domain,
    /// Glob matched against an executable or command token.
    Command,
    /// Glob matched against a named remote resource.
    Remote,
}

impl From<ToolResourceKind> for ResourceKind {
    fn from(value: ToolResourceKind) -> Self {
        match value {
            ToolResourceKind::Path => Self::Path,
            ToolResourceKind::Domain => Self::Domain,
            ToolResourceKind::Command => Self::Command,
            ToolResourceKind::Remote => Self::Remote,
        }
    }
}

/// Extracts resource strings from tool input based on [`ToolNature`].
///
/// Each nature maps to specific input fields:
/// - Read/Write → `input["path"]`, `input["file"]`, or `input["destination"]`
/// - Execute → first whitespace-delimited token of `input["command"]`
///   (e.g., `scripts/deploy.sh --arg` → `scripts/deploy.sh`)
/// - Network → host from `input["url"]` (lowercase, no port)
pub struct ResourceExtractor;

impl ResourceExtractor {
    /// Extracts the resource string from tool input based on the tool's nature.
    ///
    /// Returns `None` when the expected field is missing or (for Network)
    /// when the URL cannot be parsed.
    pub fn extract(nature: ToolNature, input: &Value) -> Option<String> {
        match nature {
            ToolNature::Read | ToolNature::Write => input
                .get("path")
                .or_else(|| input.get("file"))
                .or_else(|| input.get("destination"))
                .and_then(Value::as_str)
                .map(String::from),
            ToolNature::Execute => input
                .get("command")
                .and_then(Value::as_str)
                .and_then(|cmd| cmd.split_whitespace().next().map(String::from)),
            ToolNature::Network => input
                .get("url")
                .and_then(Value::as_str)
                .and_then(|url_str| {
                    Url::parse(url_str)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
                }),
            ToolNature::Internal => None,
        }
    }
}

/// A single permission rule that matches tool calls and produces a decision.
///
/// Rules are evaluated in order. The first rule whose nature (or tool_name for
/// legacy rules) matches the invoked tool and whose resource (or path_pattern
/// for legacy rules) matches the resource in the tool input determines the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// The tool name to match. Case-sensitive exact match.
    /// Used for legacy tool_name-only rules when `nature` is not set.
    #[serde(default)]
    pub tool_name: String,
    /// Optional glob pattern to match against the `path` field in tool input.
    /// Used for legacy rules when `nature` is not set.
    pub path_pattern: Option<String>,
    /// The decision to apply when this rule matches.
    pub decision: PermissionDecision,
    /// The ToolNature this rule applies to. When set, matching is by nature
    /// + resource instead of tool_name + path_pattern.
    #[serde(default)]
    pub nature: Option<ToolNature>,
    /// The resource pattern to match (glob for Path, host pattern for Domain).
    #[serde(default)]
    pub resource: Option<String>,
    /// How to interpret the `resource` field. Inferred from nature if absent.
    #[serde(default)]
    pub resource_kind: Option<ResourceKind>,
}

impl PermissionRule {
    /// Creates a new legacy permission rule (tool_name + path_pattern matching).
    pub fn new(
        tool_name: impl Into<String>,
        path_pattern: Option<String>,
        decision: PermissionDecision,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            path_pattern,
            decision,
            nature: None,
            resource: None,
            resource_kind: None,
        }
    }

    /// Creates a new nature-based permission rule.
    ///
    /// Matching is by `nature` + `resource` instead of `tool_name` + `path_pattern`.
    pub fn new_nature(
        nature: ToolNature,
        resource: Option<String>,
        resource_kind: Option<ResourceKind>,
        decision: PermissionDecision,
    ) -> Self {
        Self {
            tool_name: String::new(),
            path_pattern: None,
            decision,
            nature: Some(nature),
            resource,
            resource_kind,
        }
    }

    /// Checks if this rule matches the given tool invocation.
    ///
    /// If `nature` is set on the rule, matching is by nature + resource.
    /// Otherwise falls back to legacy tool_name + path_pattern matching.
    fn matches(
        &self,
        tool_name: &str,
        nature: ToolNature,
        input: &Value,
        explicit_resource: Option<&str>,
    ) -> Result<bool, PermissionError> {
        // Nature-based matching (new form)
        if let Some(rule_nature) = self.nature {
            if rule_nature != nature {
                return Ok(false);
            }

            // If no resource is set, match all invocations of this nature
            let Some(ref resource_pattern) = self.resource else {
                return Ok(true);
            };

            // Extract the resource from input based on nature
            let extracted = explicit_resource
                .map(str::to_owned)
                .or_else(|| Self::extract_resource(nature, input));

            let Some(extracted) = extracted else {
                return Ok(false);
            };

            // Match the resource against the pattern using glob
            let pattern = Pattern::new(resource_pattern).map_err(|e| {
                PermissionError::InvalidGlobPattern(format!("{resource_pattern}: {e}"))
            })?;

            return Ok(pattern.matches(&extracted));
        }

        // Legacy tool_name-based matching
        if self.tool_name != tool_name {
            return Ok(false);
        }

        if let Some(ref pattern) = self.path_pattern {
            let path = explicit_resource
                .map(str::to_owned)
                .or_else(|| {
                    input
                        .get("path")
                        .or_else(|| input.get("file"))
                        .or_else(|| input.get("destination"))
                        .and_then(Value::as_str)
                        .map(String::from)
                })
                .ok_or_else(|| {
                    PermissionError::InvalidRule(
                        "rule has path_pattern but tool input has no path field".to_owned(),
                    )
                })?;

            let glob = Pattern::new(pattern)
                .map_err(|e| PermissionError::InvalidGlobPattern(format!("{pattern}: {e}")))?;

            return Ok(glob.matches(&path));
        }

        Ok(true)
    }

    /// Extracts the resource string from tool input based on the tool's nature.
    fn extract_resource(nature: ToolNature, input: &Value) -> Option<String> {
        ResourceExtractor::extract(nature, input)
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
    ///
    /// Default rules use nature form (one rule per ToolNature):
    /// - Read → Allow
    /// - Write → Ask
    /// - Execute → Ask
    /// - Network → Ask
    /// - Internal → Allow (session plumbing, not user-visible)
    fn add_default_rules(&mut self) {
        self.rules.push(PermissionRule::new_nature(
            ToolNature::Read,
            None,
            None,
            PermissionDecision::Allow,
        ));
        self.rules.push(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Ask,
        ));
        self.rules.push(PermissionRule::new_nature(
            ToolNature::Execute,
            None,
            None,
            PermissionDecision::Ask,
        ));
        self.rules.push(PermissionRule::new_nature(
            ToolNature::Network,
            None,
            None,
            PermissionDecision::Ask,
        ));
        self.rules.push(PermissionRule::new_nature(
            ToolNature::Internal,
            None,
            None,
            PermissionDecision::Allow,
        ));
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
        self.evaluate_profile(tool_name, &[ToolPermissionFacet::new(nature)], input)
    }

    /// Evaluates a tool invocation with all risk facets considered.
    ///
    /// Aggregation is conservative: any denied facet denies the whole call,
    /// otherwise any ask facet asks for approval, otherwise the call is
    /// allowed.
    pub fn evaluate_profile(
        &self,
        tool_name: &str,
        profile: &[ToolPermissionFacet],
        input: &Value,
    ) -> PermissionDecision {
        let facets = if profile.is_empty() {
            vec![ToolPermissionFacet::new(infer_nature(tool_name))]
        } else {
            profile.to_vec()
        };

        let mut saw_ask = false;
        for facet in facets {
            match self.evaluate_facet(tool_name, &facet, input) {
                PermissionDecision::Allow => {}
                PermissionDecision::Ask => saw_ask = true,
                PermissionDecision::Deny(reason) => return PermissionDecision::Deny(reason),
            }
        }

        if saw_ask {
            PermissionDecision::Ask
        } else {
            PermissionDecision::Allow
        }
    }

    /// Evaluates one explicit permission facet.
    pub fn evaluate_facet(
        &self,
        tool_name: &str,
        facet: &ToolPermissionFacet,
        input: &Value,
    ) -> PermissionDecision {
        let nature = facet.nature;
        if let Some(ref root) = self.workspace_root
            && nature == talos_core::tool::ToolNature::Read
            && is_workspace_path_allowed_with_resource(input, root, facet.resource.as_deref())
        {
            return PermissionDecision::Allow;
        }

        for rule in &self.rules {
            match rule.matches(tool_name, nature, input, facet.resource.as_deref()) {
                Ok(true) => return rule.decision.clone(),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }

        match nature {
            talos_core::tool::ToolNature::Read | talos_core::tool::ToolNature::Internal => {
                PermissionDecision::Allow
            }
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
            let mut rule: PermissionRule = serde_json::from_value(rule_value.clone())
                .map_err(|e| PermissionError::InvalidRule(format!("rule at index {i}: {e}")))?;

            // For legacy rules (no nature set), infer nature from tool_name
            // and migrate path_pattern → resource for nature-based matching
            if rule.nature.is_none() && !rule.tool_name.is_empty() {
                rule.nature = Some(infer_nature(&rule.tool_name));
                if let Some(ref pattern) = rule.path_pattern {
                    rule.resource = Some(pattern.clone());
                    rule.resource_kind = Some(ResourceKind::Path);
                }
            }

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
    if name_lower.starts_with("todo_") {
        // Safety net: todo tools always set Internal via permission_profile,
        // but `evaluate` (which uses this heuristic) must agree.
        return talos_core::tool::ToolNature::Internal;
    }
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
    } else if name_lower == "http_request" || name_lower == "web_search" {
        talos_core::tool::ToolNature::Network
    } else {
        talos_core::tool::ToolNature::Write
    }
}

fn is_workspace_path_allowed_with_resource(
    input: &Value,
    root: &Path,
    explicit_resource: Option<&str>,
) -> bool {
    if let Some(resource) = explicit_resource {
        return is_path_in_workspace(resource, root);
    }

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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
        });
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("only src allowed".to_owned()),
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
        });
        engine.add_rule(PermissionRule {
            tool_name: "bash".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("blocked".to_owned()),
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
        });
        engine.add_rule(PermissionRule {
            tool_name: "write".to_owned(),
            path_pattern: None,
            decision: PermissionDecision::Deny("write not allowed".to_owned()),
            nature: None,
            resource: None,
            resource_kind: None,
        });

        let decision = engine.evaluate("write", &serde_json::json!({"path": "tmp/out.txt"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write not allowed".to_owned())
        );
    }

    // --- Nature-based rule tests (T1) ---

    #[test]
    fn test_nature_match_without_resource_matches_all() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ));

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("delete", &serde_json::json!({"path": "tmp.txt"}));
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_nature_path_resource_match() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            Some("src/**".to_owned()),
            Some(ResourceKind::Path),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Deny("write not allowed".to_owned()),
        ));

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/lib.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write not allowed".to_owned())
        );
    }

    #[test]
    fn test_nature_domain_resource_match() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Network,
            Some("api.github.com".to_owned()),
            Some(ResourceKind::Domain),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Network,
            None,
            None,
            PermissionDecision::Ask,
        ));

        let decision = engine.evaluate(
            "http_request",
            &serde_json::json!({"url": "https://api.github.com/repos"}),
        );
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate(
            "http_request",
            &serde_json::json!({"url": "https://example.com/api"}),
        );
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_profile_denies_when_any_facet_is_denied() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Network,
            Some("example.com".to_owned()),
            Some(ResourceKind::Domain),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            Some("blocked/**".to_owned()),
            Some(ResourceKind::Path),
            PermissionDecision::Deny("write blocked".to_owned()),
        ));

        let profile = vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "example.com",
                ToolResourceKind::Domain,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Write,
                "blocked/file.txt",
                ToolResourceKind::Path,
            ),
        ];

        let decision = engine.evaluate_profile("save_url", &profile, &serde_json::json!({}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write blocked".to_owned())
        );
    }

    #[test]
    fn test_profile_asks_when_any_facet_requires_approval() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Network,
            Some("example.com".to_owned()),
            Some(ResourceKind::Domain),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Ask,
        ));

        let profile = vec![
            ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "example.com",
                ToolResourceKind::Domain,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Write,
                "out/file.txt",
                ToolResourceKind::Path,
            ),
        ];

        let decision = engine.evaluate_profile("save_url", &profile, &serde_json::json!({}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_extractor_write_from_destination() {
        let input = serde_json::json!({"destination": "downloads/file.txt"});
        let result = ResourceExtractor::extract(ToolNature::Write, &input);
        assert_eq!(result, Some("downloads/file.txt".to_owned()));
    }

    #[test]
    fn test_legacy_tool_name_rule_still_works() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new(
            "write",
            Some("src/**".to_owned()),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new(
            "write",
            None,
            PermissionDecision::Deny("write not allowed".to_owned()),
        ));

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write not allowed".to_owned())
        );
    }

    #[test]
    fn test_first_match_wins_nature_rules() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            Some("src/**".to_owned()),
            Some(ResourceKind::Path),
            PermissionDecision::Allow,
        ));
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Deny("write not allowed".to_owned()),
        ));

        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(
            decision,
            PermissionDecision::Deny("write not allowed".to_owned())
        );
    }

    // --- ResourceExtractor tests (T2) ---

    #[test]
    fn test_extractor_read_from_path() {
        let input = serde_json::json!({"path": "src/main.rs"});
        let result = ResourceExtractor::extract(ToolNature::Read, &input);
        assert_eq!(result, Some("src/main.rs".to_owned()));
    }

    #[test]
    fn test_extractor_read_from_file_fallback() {
        let input = serde_json::json!({"name": "Tool", "file": "src/lib.rs"});
        let result = ResourceExtractor::extract(ToolNature::Read, &input);
        assert_eq!(result, Some("src/lib.rs".to_owned()));
    }

    #[test]
    fn test_extractor_write_from_path() {
        let input = serde_json::json!({"path": "src/main.rs", "content": "hello"});
        let result = ResourceExtractor::extract(ToolNature::Write, &input);
        assert_eq!(result, Some("src/main.rs".to_owned()));
    }

    #[test]
    fn test_extractor_execute_first_token() {
        let input = serde_json::json!({"command": "scripts/deploy.sh --arg1 --arg2"});
        let result = ResourceExtractor::extract(ToolNature::Execute, &input);
        assert_eq!(result, Some("scripts/deploy.sh".to_owned()));
    }

    #[test]
    fn test_extractor_execute_single_word() {
        let input = serde_json::json!({"command": "cargo"});
        let result = ResourceExtractor::extract(ToolNature::Execute, &input);
        assert_eq!(result, Some("cargo".to_owned()));
    }

    #[test]
    fn test_extractor_network_host_extraction() {
        let input = serde_json::json!({"url": "https://api.github.com/repos"});
        let result = ResourceExtractor::extract(ToolNature::Network, &input);
        assert_eq!(result, Some("api.github.com".to_owned()));
    }

    #[test]
    fn test_extractor_network_host_lowercase() {
        let input = serde_json::json!({"url": "https://API.GITHUB.COM/repos"});
        let result = ResourceExtractor::extract(ToolNature::Network, &input);
        assert_eq!(result, Some("api.github.com".to_owned()));
    }

    #[test]
    fn test_extractor_network_host_no_port() {
        let input = serde_json::json!({"url": "https://api.github.com:443/repos"});
        let result = ResourceExtractor::extract(ToolNature::Network, &input);
        assert_eq!(result, Some("api.github.com".to_owned()));
    }

    #[test]
    fn test_extractor_network_invalid_url() {
        let input = serde_json::json!({"url": "not-a-url"});
        let result = ResourceExtractor::extract(ToolNature::Network, &input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extractor_missing_field_returns_none() {
        let input = serde_json::json!({});
        assert_eq!(ResourceExtractor::extract(ToolNature::Read, &input), None);
        assert_eq!(ResourceExtractor::extract(ToolNature::Write, &input), None);
        assert_eq!(
            ResourceExtractor::extract(ToolNature::Execute, &input),
            None
        );
        assert_eq!(
            ResourceExtractor::extract(ToolNature::Network, &input),
            None
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

    #[test]
    fn test_load_old_config_format_tool_name_only() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        let config = serde_json::json!({
            "rules": [
                {
                    "tool_name": "write",
                    "path_pattern": "src/**",
                    "decision": "Allow"
                },
                {
                    "tool_name": "write",
                    "decision": "Ask"
                }
            ]
        });

        engine
            .load_from_config(&config)
            .expect("config should load");

        // Old format: tool_name-based matching with inferred nature
        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(decision, PermissionDecision::Ask);
    }

    #[test]
    fn test_load_new_config_format_nature_form() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        let config = serde_json::json!({
            "rules": [
                {
                    "nature": "Write",
                    "resource": "src/**",
                    "resource_kind": "path",
                    "decision": "Allow"
                },
                {
                    "nature": "Write",
                    "decision": "Deny"
                }
            ]
        });

        engine
            .load_from_config(&config)
            .expect("config should load");

        // New format: nature-based matching
        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/lib.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
        assert_eq!(decision, PermissionDecision::Deny("".to_owned()));
    }

    #[test]
    fn test_default_ruleset_is_nature_form() {
        let engine = PermissionEngine::new();
        // Default ruleset should have exactly 5 rules (one per ToolNature variant)
        assert_eq!(engine.rules.len(), 5);
        for rule in &engine.rules {
            assert!(
                rule.nature.is_some(),
                "default rules should use nature form"
            );
        }
    }

    #[test]
    fn test_default_internal_tool_allowed() {
        let engine = PermissionEngine::new();
        let decision = engine.evaluate_with_nature(
            "todo_create",
            ToolNature::Internal,
            &serde_json::json!({}),
        );
        assert_eq!(decision, PermissionDecision::Allow);
    }

    #[test]
    fn test_config_with_both_tool_name_and_nature_prefers_nature() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        // Rule has both tool_name AND nature set — nature should take precedence
        let config = serde_json::json!({
            "rules": [
                {
                    "tool_name": "read",
                    "nature": "Write",
                    "resource": "src/**",
                    "resource_kind": "path",
                    "decision": "Allow"
                }
            ]
        });

        engine
            .load_from_config(&config)
            .expect("config should load");

        // Nature is Write, so it matches write tools, not read tools
        let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow);

        // Read tool doesn't match the Write nature rule
        let decision = engine.evaluate("read", &serde_json::json!({"path": "src/main.rs"}));
        assert_eq!(decision, PermissionDecision::Allow); // falls through to default Read Allow
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
            nature: None,
            resource: None,
            resource_kind: None,
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
            nature: None,
            resource: None,
            resource_kind: None,
        };
        let json = serde_json::to_string(&rule).expect("serialize");
        let parsed: PermissionRule = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.tool_name, "write");
        assert_eq!(parsed.path_pattern, Some("src/**".to_owned()));
        assert_eq!(parsed.decision, PermissionDecision::Ask);
    }
}
