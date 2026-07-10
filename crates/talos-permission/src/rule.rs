//! Permission rule definition and matching logic.

use glob::Pattern;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::ToolNature;
use thiserror::Error;

use crate::PermissionDecision;
use crate::resource::{ResourceExtractor, ResourceKind};

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
    pub(crate) fn matches(
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
