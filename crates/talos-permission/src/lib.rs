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

mod resource;
mod rule;
mod workspace_trust;

pub use workspace_trust::{WorkspaceTrustStore, is_git_workspace, is_within_repo};

pub use resource::{ResourceExtractor, ResourceKind};
pub use rule::{PermissionError, PermissionRule};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use talos_core::tool::{ToolNature, ToolPermissionFacet};

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

/// The permission rules engine.
///
/// Evaluates tool calls against a set of rules and returns a
/// [`PermissionDecision`]. Rules are evaluated in insertion order; the first
/// match wins. If no rule matches, a default decision is applied based on
/// the tool name.
pub struct PermissionEngine {
    pub rules: Vec<PermissionRule>,
    pub workspace_root: Option<PathBuf>,
    pub trusted_workspace: bool,
}

impl PermissionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            workspace_root: None,
            trusted_workspace: false,
        };
        engine.add_default_rules();
        engine
    }

    pub fn with_workspace_root(root: PathBuf) -> Self {
        let mut engine = Self::new();
        engine.workspace_root = Some(root);
        engine
    }

    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    pub fn set_trusted_workspace(&mut self, trusted: bool) {
        self.trusted_workspace = trusted;
    }

    /// Adds the default ruleset to the engine.
    ///
    /// Default rules use nature form (one rule per ToolNature variant):
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

    /// Adds a runtime "always allow" rule ahead of the default catch-all ask.
    ///
    /// This is used for user-approved session rules. It preserves existing
    /// deny rules and other custom policy rules that appear before the default
    /// catch-all ask rule for the same nature, while ensuring the newly approved
    /// resource is not shadowed by the default ask rule.
    pub fn add_runtime_allow_rule(&mut self, rule: PermissionRule) {
        let insert_at = rule
            .nature
            .and_then(|nature| {
                self.rules.iter().position(|existing| {
                    existing.nature == Some(nature)
                        && existing.resource.is_none()
                        && existing.decision == PermissionDecision::Ask
                })
            })
            .unwrap_or(self.rules.len());
        self.rules.insert(insert_at, rule);
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

        for rule in &self.rules {
            match rule.matches(tool_name, nature, input, facet.resource.as_deref()) {
                Ok(true) => return rule.decision.clone(),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }

        if let Some(ref root) = self.workspace_root
            && nature == talos_core::tool::ToolNature::Read
            && is_workspace_path_allowed_with_resource(input, root, facet.resource.as_deref())
        {
            return PermissionDecision::Allow;
        }

        if self.trusted_workspace
            && let Some(ref root) = self.workspace_root
            && nature == talos_core::tool::ToolNature::Write
            && is_workspace_path_allowed_with_resource(input, root, facet.resource.as_deref())
        {
            return PermissionDecision::Allow;
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
#[path = "permission_tests.rs"]
mod tests;
