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

use serde_json::json;
use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind};
use talos_permission::{
    PermissionContext, PermissionDecision, PermissionRequest, PermissionSessionState,
};

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
    pub(crate) fn evaluate(
        path: &std::path::Path,
        state: &PermissionSessionState,
        context: &PermissionContext,
    ) -> Result<Self, talos_permission::GrantError> {
        let path = path
            .to_str()
            .ok_or(talos_permission::GrantError::InvalidPath)?;
        let input = json!({ "path": path });
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Read,
            path,
            ToolResourceKind::Path,
        )];
        let request = PermissionRequest::new(
            ATTACH_IMAGE_TOOL_NAME,
            ToolProvenance::Native,
            &facets,
            &input,
        );
        Ok(match state.evaluate(&request, context)?.decision() {
            PermissionDecision::Allow => Self::Allow,
            PermissionDecision::Ask => Self::Ask,
            PermissionDecision::Deny(reason) => Self::Deny(reason),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_permission::{
        GrantScope, GrantSource, InteractionCapability, PermissionContext, PermissionEngine,
        PermissionMode, PermissionRequest, PermissionSessionState,
    };

    fn state_with_root(root: &std::path::Path) -> PermissionSessionState {
        PermissionSessionState::new(PermissionEngine::with_workspace_root(root.to_path_buf()))
    }

    fn context() -> PermissionContext {
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        )
    }

    /// P1-A: a path inside the workspace resolves to Allow without
    /// any explicit rule, because `Read` defaults to Allow and the
    /// SEC-001 path check auto-allows workspace-internal reads.
    #[test]
    fn workspace_internal_path_is_allowed() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let state = state_with_root(dir.path());
        let inside = dir.path().join("image.png");
        std::fs::write(&inside, b"data").expect("operation should succeed");
        let decision = ImageAuthorization::evaluate(&inside, &state, &context());
        assert!(
            matches!(decision, Ok(ImageAuthorization::Allow)),
            "workspace-internal path must be Allow, got {decision:?}"
        );
    }

    /// P1-A: an external path with no explicit rule resolves to Ask,
    /// NOT Allow. This is the SEC-001 fail-closed guarantee: external
    /// paths require explicit approval.
    #[test]
    fn external_path_without_rule_is_ask() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let state = state_with_root(dir.path());
        let outside = std::path::Path::new("/tmp/p1-a-external-path-not-allowed.png");
        // Do not create the file — evaluate must NOT touch the fs.
        let decision = ImageAuthorization::evaluate(outside, &state, &context());
        match decision {
            Ok(ImageAuthorization::Ask) => {}
            other => panic!("external path must be Ask, got {other:?}"),
        }
    }

    /// P1-A: the Session grant is scoped — a different external path
    /// still evaluates to Ask after approving its sibling.
    #[test]
    fn session_grant_is_scoped_to_exact_path() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let state = state_with_root(dir.path());
        let approved = std::path::Path::new("/tmp/p1-a-approved.png");
        let other = std::path::Path::new("/tmp/p1-a-other.png");
        let input = json!({"path": approved.display().to_string()});
        let facets = [ToolPermissionFacet::with_resource(
            ToolNature::Read,
            approved.display().to_string(),
            ToolResourceKind::Path,
        )];
        let context = context();
        let request = PermissionRequest::new(
            ATTACH_IMAGE_TOOL_NAME,
            ToolProvenance::Native,
            &facets,
            &input,
        );
        let proposal = state
            .propose(&request, &context, GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
            .expect("approval");

        let approved_decision = ImageAuthorization::evaluate(approved, &state, &context);
        assert!(matches!(approved_decision, Ok(ImageAuthorization::Allow)));

        let other_decision = ImageAuthorization::evaluate(other, &state, &context);
        assert!(
            matches!(other_decision, Ok(ImageAuthorization::Ask)),
            "non-approved sibling must remain Ask, got {other_decision:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_path_fails_closed_before_scope_projection() {
        use std::os::unix::ffi::OsStringExt;

        let dir = tempfile::tempdir().expect("operation should succeed");
        let invalid = std::ffi::OsString::from_vec(vec![b'i', 0x80]);
        let path = dir.path().join(invalid);
        let state = state_with_root(dir.path());

        assert!(matches!(
            ImageAuthorization::evaluate(&path, &state, &context()),
            Err(talos_permission::GrantError::InvalidPath)
        ));
    }
}
