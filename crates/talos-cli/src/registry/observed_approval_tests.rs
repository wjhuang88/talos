use super::*;
use std::time::Duration;
use talos_agent::permission_pipeline::{ApprovalAttribution, PermissionBinding};
use talos_core::tool::{ToolNature, ToolProvenance, ToolResourceKind};
use talos_permission::{PermissionContext, PermissionRequest};

fn request(handler: &TuiApprovalHandler, root: &Path) -> PermissionApprovalRequest {
    let input = serde_json::json!({"path": root.join("note.txt")});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        root.join("note.txt").to_string_lossy(),
        ToolResourceKind::Path,
    )];
    let raw = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let proposal = handler
        .state
        .propose(&raw, &context, GrantScope::Session)
        .expect("proposal");
    let snapshot = handler.state.state_snapshot().expect("snapshot");
    PermissionApprovalRequest {
        tool_name: "write".into(),
        provenance: ToolProvenance::Native,
        arguments: input,
        summary_fields: vec!["path".into()],
        preview: proposal.preview().clone(),
        binding: PermissionBinding {
            session_id: snapshot.session_id.stable_id(),
            revisions: snapshot.revisions.as_array(),
            mode: context.mode(),
            interaction: context.interaction(),
        },
    }
}

#[tokio::test]
async fn typed_tui_disconnection_and_publication_invalidation_are_unavailable() {
    for invalidate in [false, true] {
        let root = tempfile::tempdir().expect("root");
        let (legacy_tx, mut legacy_rx) = mpsc::unbounded_channel();
        let (typed_tx, mut typed_rx) = mpsc::unbounded_channel();
        let handler = TuiApprovalHandler::new(legacy_tx, root.path().into())
            .with_observed_transport(typed_tx);
        let completed = tokio::sync::Notify::new();
        let pending = handler.resolve_observed(
            request(&handler, root.path()),
            Duration::from_secs(10),
            Some("public explanation"),
        );
        let driver = async {
            let displayed = typed_rx.recv().await.expect("one typed request");
            assert!(
                legacy_rx.try_recv().is_err(),
                "no placeholder legacy request"
            );
            if invalidate {
                handler.approval_binding.invalidate();
                // Keep the responder alive: invalidation itself must wake the waiter.
                completed.notified().await;
            }
            drop(displayed);
        };
        let consumer = async {
            let result = pending.await;
            completed.notify_one();
            result
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async {
            tokio::join!(consumer, driver)
        })
        .await
        .expect("invalidation wakes before timeout or responder drop");
        let result = result.expect("observation");
        assert_eq!(result.attribution(), ApprovalAttribution::Unavailable);
        assert_eq!(result.choice(), &ApprovalChoice::Deny);
    }
}

#[tokio::test]
async fn stale_permission_session_is_refused_before_entering_tui_queue() {
    let root = tempfile::tempdir().expect("root");
    let (legacy_tx, _legacy_rx) = mpsc::unbounded_channel();
    let (typed_tx, mut typed_rx) = mpsc::unbounded_channel();
    let handler =
        TuiApprovalHandler::new(legacy_tx, root.path().into()).with_observed_transport(typed_tx);
    let stale = request(&handler, root.path());
    handler.state.rebind_session().expect("rebind");
    let result = handler
        .resolve_observed(stale, Duration::from_secs(1), None)
        .await
        .expect("stale observation");
    assert_eq!(result.attribution(), ApprovalAttribution::Unavailable);
    assert!(typed_rx.try_recv().is_err());
}

#[tokio::test]
async fn legacy_tui_response_keeps_unknown_attribution() {
    let root = tempfile::tempdir().expect("root");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let handler = TuiApprovalHandler::new(tx, root.path().into());
    let pending =
        handler.resolve_observed(request(&handler, root.path()), Duration::from_secs(1), None);
    let driver = async {
        let UiOutput::ToolApprovalRequest { response, .. } =
            rx.recv().await.expect("legacy request")
        else {
            panic!("legacy");
        };
        response
            .send(ApprovalChoice::ApproveOnce)
            .expect("response");
    };
    let (result, ()) = tokio::join!(pending, driver);
    let result = result.expect("observation");
    assert_eq!(result.choice(), &ApprovalChoice::ApproveOnce);
    assert_eq!(result.attribution(), ApprovalAttribution::LegacyHostUnknown);
}
