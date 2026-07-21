//! Image attachment path authorization (P1-A Owner rework, 2026-07-21).
//!
//! Reuses the SEC-001 / ADR-047 permission pipeline to authorize image
//! attachment paths before any filesystem probe. Both TUI `/attach` and
//! CLI `--attach` go through this module so the authorization surface
//! is identical.
//!
//! The decision maps the path against `PermissionEngine` with a
//! synthetic `attach_image` tool name and `ToolNature::Read`. External
//! paths produce `Ask`, which the TUI resolves through an interactive
//! `UiOutput::ToolApprovalRequest` and print mode treats as fail-closed
//! (headless unresolved Ask cannot authorize).

use std::path::PathBuf;

use serde_json::json;
use talos_core::tool::ToolNature;
use talos_permission::{PermissionDecision, PermissionEngine};

/// Synthetic tool name used to identify image-attachment permission
/// facets in rules and approval diagnostics. Not a real `AgentTool`;
/// it exists only so the permission engine can route image attachments
/// through the same pipeline as `read`.
pub const ATTACH_IMAGE_TOOL_NAME: &str = "attach_image";

/// Outcome of evaluating an image attachment path against the
/// permission engine. The caller MUST consult this before invoking
/// `create_image_content_part`.
#[derive(Debug)]
pub(crate) enum ImageAuthorization {
    /// Workspace-internal or explicitly allowed by a rule. The path
    /// is safe to read.
    Allow,
    /// External path with no explicit rule. Requires interactive
    /// approval (TUI) or is rejected (headless).
    Ask,
    /// Explicitly denied by a rule. Must not be read.
    Deny(String),
}

impl ImageAuthorization {
    /// Returns the decision for the given path under the engine's
    /// current rule set. Does NOT mutate the engine and does NOT
    /// prompt the user.
    pub(crate) fn evaluate(path: &std::path::Path, engine: &PermissionEngine) -> Self {
        let input = json!({ "path": path.display().to_string() });
        match engine.evaluate_with_nature(ATTACH_IMAGE_TOOL_NAME, ToolNature::Read, &input) {
            PermissionDecision::Allow => Self::Allow,
            PermissionDecision::Ask => Self::Ask,
            PermissionDecision::Deny(reason) => Self::Deny(reason),
        }
    }
}

/// Adds a runtime "always allow" rule for an approved external image
/// path, mirroring the pattern used by `add_always_allow_rules` for
/// tool calls. The rule is scoped to the exact resource and the
/// `attach_image` tool name so it cannot broaden other tools' access.
pub(crate) fn add_attach_image_allow_rule(engine: &mut PermissionEngine, path: PathBuf) {
    use talos_permission::{PermissionDecision, PermissionRule, ResourceKind};

    let mut rule =
        PermissionRule::new_nature(ToolNature::Read, None, None, PermissionDecision::Allow);
    rule.tool_name = ATTACH_IMAGE_TOOL_NAME.to_string();
    rule.resource = Some(path.display().to_string());
    rule.resource_kind = Some(ResourceKind::Path);
    engine.add_runtime_allow_rule(rule);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_root(root: &std::path::Path) -> PermissionEngine {
        PermissionEngine::with_workspace_root(root.to_path_buf())
    }

    /// P1-A: a path inside the workspace resolves to Allow without
    /// any explicit rule, because `Read` defaults to Allow and the
    /// SEC-001 path check auto-allows workspace-internal reads.
    #[test]
    fn workspace_internal_path_is_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_root(dir.path());
        let inside = dir.path().join("image.png");
        std::fs::write(&inside, b"data").unwrap();
        let decision = ImageAuthorization::evaluate(&inside, &engine);
        assert!(
            matches!(decision, ImageAuthorization::Allow),
            "workspace-internal path must be Allow, got {decision:?}"
        );
    }

    /// P1-A: an external path with no explicit rule resolves to Ask,
    /// NOT Allow. This is the SEC-001 fail-closed guarantee: external
    /// paths require explicit approval.
    #[test]
    fn external_path_without_rule_is_ask() {
        let dir = tempfile::tempdir().unwrap();
        let engine = engine_with_root(dir.path());
        let outside = std::path::Path::new("/tmp/p1-a-external-path-not-allowed.png");
        // Do not create the file — evaluate must NOT touch the fs.
        let decision = ImageAuthorization::evaluate(outside, &engine);
        match decision {
            ImageAuthorization::Ask => {}
            other => panic!("external path must be Ask, got {other:?}"),
        }
    }

    /// P1-A: adding a runtime allow rule for a specific external path
    /// turns subsequent evaluations from Ask into Allow.
    #[test]
    fn runtime_allow_rule_promotes_ask_to_allow() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = engine_with_root(dir.path());
        let outside = std::path::Path::new("/tmp/p1-a-external-ruled.png");

        let before = ImageAuthorization::evaluate(outside, &engine);
        assert!(matches!(before, ImageAuthorization::Ask));

        add_attach_image_allow_rule(&mut engine, outside.to_path_buf());

        let after = ImageAuthorization::evaluate(outside, &engine);
        match after {
            ImageAuthorization::Allow => {}
            other => panic!("external path with rule must be Allow, got {other:?}"),
        }
    }

    /// P1-A: the allow rule is scoped — a different external path
    /// still evaluates to Ask after approving its sibling.
    #[test]
    fn allow_rule_is_scoped_to_exact_path() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = engine_with_root(dir.path());
        let approved = std::path::Path::new("/tmp/p1-a-approved.png");
        let other = std::path::Path::new("/tmp/p1-a-other.png");

        add_attach_image_allow_rule(&mut engine, approved.to_path_buf());

        let approved_decision = ImageAuthorization::evaluate(approved, &engine);
        assert!(matches!(approved_decision, ImageAuthorization::Allow));

        let other_decision = ImageAuthorization::evaluate(other, &engine);
        assert!(
            matches!(other_decision, ImageAuthorization::Ask),
            "non-approved sibling must remain Ask, got {other_decision:?}"
        );
    }
}
