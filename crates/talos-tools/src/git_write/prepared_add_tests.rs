// Shared fixtures for the prepared Git test fragments below.
use super::*;
use talos_core::tool::AgentTool;
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionSessionState,
};

fn proof(plan: &AddPlan) -> AdmittedInvocation {
    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
        plan.workspace.clone(),
    ));
    let context = PermissionContext::new(
        PermissionMode::Interactive,
        InteractionCapability::Available,
    );
    let request = plan.request();
    let proposal = state
        .propose(&request, &context, GrantScope::Once)
        .expect("proposal");
    let pending = state
        .approve_once(proposal, &request, &context)
        .expect("approval");
    state
        .try_admit_prepared_invocation(pending, &request, &context, &plan.identity)
        .expect("admission")
}

// Match native recursive fetch's cwd/GIT_DIR setup and the prepared command's
// explicit worktree override. Neither changes relative fetch resource lookup.

pub(crate) async fn execute_approved(plan: AddPlan) -> ToolExecutionOutput {
    let name = plan.operation.name();
    let root = plan.workspace.clone();
    let input = plan.input.clone();
    let provenance = plan.provenance.clone();
    let mut operation: Box<dyn PreparedNormalToolInvocation> = Box::new(plan);
    let mut history = talos_core::tool::PreparedStageHistory::default();
    loop {
        if !history.record_profile(
            operation.plan_identity().expect("identity"),
            operation.permission_profile(),
        ) {
            return ToolExecutionOutput::error("stage limit");
        }
        let state =
            PermissionSessionState::new(PermissionEngine::with_workspace_root(root.clone()));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request = PermissionRequest::new(
            name,
            provenance.clone(),
            operation.permission_profile(),
            &input,
        );
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("approval");
        let proof = state
            .try_admit_prepared_invocation(
                pending,
                &request,
                &context,
                operation.plan_identity().expect("identity"),
            )
            .expect("admit");
        match operation.execute_stage(Box::new(proof)).await {
            PreparedNormalStage::Complete(output) => return output,
            PreparedNormalStage::Continue(next) => operation = next,
        }
    }
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success());
    std::fs::write(root.path().join("file"), "before").expect("file");
    root
}

fn prepare(root: &Path) -> AddPlan {
    AddPlan::prepare(
        root,
        &serde_json::json!({"paths":["file"]}),
        ToolProvenance::Native,
    )
    .expect("plan")
}

include!("prepared_add_tests/authorization.rs");

include!("prepared_add_tests/checkout.rs");

include!("prepared_add_tests/commit.rs");

include!("prepared_add_tests/pull.rs");
include!("prepared_add_tests/pull_prefetch.rs");
include!("prepared_add_tests/pull_fetch_admission.rs");
include!("prepared_add_tests/pull_postfetch.rs");
include!("prepared_add_tests/pull_graph_admission.rs");
include!("prepared_add_tests/pull_rebase.rs");

include!("prepared_add_tests/pull_children.rs");

include!("prepared_add_tests/push.rs");

include!("prepared_add_tests/sender_program.rs");

include!("prepared_add_tests/sender_configuration.rs");

include!("prepared_add_tests/pull_integration.rs");
include!("prepared_add_tests/pull_reconcile.rs");
