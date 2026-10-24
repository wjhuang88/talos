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

// Preserve explicit fixture settings while masking unrelated host filter programs.
fn prepare_pull_fixture(
    root: &Path,
    input: &Value,
    provenance: ToolProvenance,
) -> Result<AddPlan, GitToolError> {
    isolate_integration_fixture_filters(root);
    AddPlan::prepare_pull(root, input, provenance)
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture");
    let output = std::process::Command::new("git")
        .arg("init")
        .arg(root.path())
        .output()
        .expect("Git fixture");
    assert!(output.status.success());
    isolate_integration_fixture_filters(root.path());
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

include!("prepared_add_tests/staging.rs");

include!("prepared_add_tests/checkout.rs");

include!("prepared_add_tests/commit.rs");

#[path = "prepared_add_tests/native_pull.rs"]
mod native_pull;

#[path = "prepared_add_tests/pull_command.rs"]
mod pull_command;

#[path = "prepared_add_tests/native_process.rs"]
mod native_process;

include!("prepared_add_tests/push.rs");

include!("prepared_add_tests/sender_program.rs");

include!("prepared_add_tests/sender_configuration.rs");

fn isolate_integration_fixture_filters(root: &Path) {
    // These integration fixtures exercise repository-local configuration,
    // not optional host filter installations (e.g. a system/global Git LFS
    // driver). Query the effective chain: Git for Windows installs LFS in the
    // system config, which a --global-only query misses. Preserve local values.
    let output = std::process::Command::new("git")
        .current_dir(root)
        .args([
            "config",
            "--includes",
            "--name-only",
            "--get-regexp",
            "^filter\\.",
        ])
        .output()
        .expect("effective filter names");
    assert!(output.status.success() || output.status.code() == Some(1));
    for key in String::from_utf8(output.stdout)
        .expect("filter names")
        .lines()
    {
        if [".clean", ".smudge", ".process"]
            .iter()
            .any(|suffix| key.ends_with(suffix))
            && !std::process::Command::new("git")
                .current_dir(root)
                .args(["config", "--local", "--get", key])
                .output()
                .expect("local filter")
                .status
                .success()
        {
            native_pull::git(root, &["config", key, ""]);
        }
    }
}

fn integration_git(root: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(root)
        // Fixture commits must finish maintenance before a prepared plan
        // snapshots the object layout; keep maintenance enabled and foreground.
        .args([
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
        ])
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
        ])
        .args(args)
        .output()
        .expect("Git fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git output")
        .trim()
        .to_owned()
}
