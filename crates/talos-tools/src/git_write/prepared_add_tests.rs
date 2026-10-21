// Shared fixtures for the prepared Git test fragments below.
use super::*;
use talos_core::tool::AgentTool;
use talos_permission::{
    GrantScope, InteractionCapability, PermissionContext, PermissionEngine, PermissionMode,
    PermissionSessionState,
};

pub(super) fn proof(plan: &AddPlan) -> AdmittedInvocation {
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
    execute_approved_with_history(plan, talos_core::tool::PreparedStageHistory::default()).await
}

async fn execute_approved_with_history(
    plan: AddPlan,
    mut history: talos_core::tool::PreparedStageHistory,
) -> ToolExecutionOutput {
    let name = plan.operation.name();
    let root = plan.workspace.clone();
    let input = plan.input.clone();
    let provenance = plan.provenance.clone();
    let mut operation: Box<dyn PreparedNormalToolInvocation> = Box::new(plan);
    let mut profiles = Vec::new();
    loop {
        profiles.push(format!("{:?}", operation.permission_profile()));
        if !history.record_profile(
            operation.plan_identity().expect("identity"),
            operation.permission_profile(),
        ) {
            return ToolExecutionOutput::error(format!("stage limit: {profiles:#?}"));
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

// Complete sender discovery through its admitted executable frontier, as the
// production stage dispatcher does. Windows Git installations can use distinct
// entry and internal binaries; calling discover() directly skips that binding.
async fn discover_sender_approved(mut plan: Box<AddPlan>) -> Result<Box<AddPlan>, GitToolError> {
    loop {
        let tokens = proof(&plan)
            .consume_for_prepared(&plan.request(), &plan.identity)
            .expect("sender discovery admission");
        plan = plan.discover_sender(&tokens).await?;
        if plan.sender_program_candidate.is_none() {
            return Ok(plan);
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

// Git daemon concatenates base-path with the URL's slash-prefixed path and
// compares the result byte-for-byte with its export list, even on Windows.
fn daemon_fixture_path(path: &Path) -> String {
    let text = path.to_str().expect("fixture path");
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.to_owned()
    }
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
