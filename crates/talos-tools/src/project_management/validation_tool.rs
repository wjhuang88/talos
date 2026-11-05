//! Fixed-profile adapter; process execution and cleanup remain host-owned.

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use talos_conversation::{
    EvidenceStatus, ValidationHostExecutor, ValidationProfile, collect_validation_plan,
    project_validation_available, render_json_evidence, render_json_plan,
    run_validation_plan_with_executor,
};
use talos_core::background_job::ToolExecutionAdmission;
use talos_core::tool::{
    AgentTool, PreparedNormalToolInvocation, PreparedPlanIdentity, ToolCleanupOwner,
    ToolContribution, ToolContributionSource, ToolExecutionOutput, ToolNature, ToolPermissionFacet,
    ToolProvenance, ToolResourceKind, ToolResult,
};
use talos_permission::{AdmittedInvocation, PermissionRequest};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Input {
    mode: Mode,
    profile: Profile,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Plan,
    Run,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Profile {
    Governance,
    I076,
    Workspace,
}

impl From<Profile> for ValidationProfile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Governance => Self::Governance,
            Profile::I076 => Self::I076,
            Profile::Workspace => Self::Workspace,
        }
    }
}

/// Creates independent invocation scopes under one retained shutdown owner.
/// Implementors must not share invocation cancellation state across calls.
pub trait ValidationHostFactory: ToolCleanupOwner {
    /// Create an executor whose operation cleanup excludes unrelated invocations.
    /// Construction must not start checks. Prepared execution requires a real
    /// operation-local `ValidationHostExecutor::cleanup_receipt`; None refuses
    /// execution and reports unconfirmed cleanup. Factory and receipt methods
    /// must be prompt and nonblocking; the host retains aggregate ownership.
    fn invocation(&self) -> Box<dyn ValidationHostExecutor>;
}

/// Construct validation for a recognized, host-selected workspace.
///
/// The same host adapter supplies execution and aggregate cleanup ownership. Apply normal
/// permission admission. Neither discovery nor a fixed profile grants authority.
/// This injection point does not install a platform process implementation.
/// Run uses the normal prepared-invocation protocol and consumes exact permission
/// admission before invoking the factory. Bare `AgentTool::execute` supports plan
/// only; embedders must not bypass the prepared dispatcher for run mode.
pub fn validation_tool_contributions<T: ValidationHostFactory + 'static>(
    workspace: PathBuf,
    executor: Arc<T>,
) -> Vec<ToolContribution> {
    if !project_validation_available(&workspace) {
        return Vec::new();
    }
    vec![ToolContribution::new(
        ToolContributionSource::new("builtin.project-management"),
        Arc::new(ValidationTool {
            workspace,
            cleanup: executor.clone(),
            executor,
        }),
    )]
}

#[derive(Clone)]
struct ValidationTool {
    workspace: PathBuf,
    executor: Arc<dyn ValidationHostFactory>,
    cleanup: Arc<dyn ToolCleanupOwner>,
}

fn parse(input: &Value) -> Result<Input, String> {
    serde_json::from_value(input.clone()).map_err(|_| {
        "validate_project requires mode plan/run and profile governance/i076/workspace; no other fields are accepted".into()
    })
}

#[async_trait]
impl AgentTool for ValidationTool {
    fn name(&self) -> &str {
        "validate_project"
    }

    fn description(&self) -> &str {
        "Plan or run supported fixed checks in the host-selected project. Use for 'plan validation', 'run workspace checks', or a required check of an accepted implementation task. Do not invoke for ordinary chat, isolated code explanations, or project progress alone (use agile_summary when available). Plan spawns no commands. Run requires Execute permission and may create build artifacts or run build scripts; it is not read-only. Governance checks are internal; Cargo profiles require Rust project prerequisites. Recognition of another language does not supply a host adapter. No arbitrary commands, directory/environment overrides, task edits, activation, Issue changes or release operations. Missing prerequisites and failed checks are not completion evidence."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(Input))
            .expect("serializable validation input schema")
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn execution_admission(&self, input: &Value) -> Result<ToolExecutionAdmission, String> {
        parse(input)?;
        Ok(ToolExecutionAdmission::Foreground)
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        match parse(input) {
            Ok(Input {
                mode: Mode::Plan, ..
            }) => vec![ToolPermissionFacet::new(ToolNature::Read)],
            Ok(Input {
                mode: Mode::Run,
                profile,
            }) => {
                let plan = collect_validation_plan(&self.workspace, profile.into());
                plan.checks.iter().map(|check| {
                    ToolPermissionFacet::with_resource(
                        ToolNature::Execute, check.command, ToolResourceKind::Command,
                    ).with_description("Fixed validation operation; internal: checks run in-process, Cargo checks may run build scripts and write artifacts")
                }).collect()
            }
            Err(_) => vec![ToolPermissionFacet::new(ToolNature::Execute)],
        }
    }

    fn execution_working_directory(&self) -> Option<PathBuf> {
        Some(self.workspace.clone())
    }

    fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupOwner>> {
        Some(self.cleanup.clone())
    }

    fn prepare_normalized_with_provenance(
        &self,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Option<Box<dyn PreparedNormalToolInvocation>>, String> {
        let parsed = parse(input)?;
        if matches!(parsed.mode, Mode::Plan) {
            return Ok(None);
        }
        Ok(Some(Box::new(PreparedValidation {
            tool: self.clone(),
            input: input.clone(),
            provenance,
            facets: self.permission_profile(input),
            identity: PreparedPlanIdentity::new(),
            receipt: Arc::new(InvocationReceipt::default()),
        })))
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match parse(&input) {
            Ok(Input {
                mode: Mode::Run, ..
            }) => ToolResult::error(
                "validate_project run requires exact prepared permission admission",
            ),
            Err(error) => ToolResult::error(error),
            Ok(_) => {
                self.execute_admitted(
                    input,
                    "read-only plan; no execution admission requested",
                    &InvocationReceipt::default(),
                )
                .await
            }
        }
    }
}

struct PreparedValidation {
    tool: ValidationTool,
    input: Value,
    provenance: ToolProvenance,
    facets: Vec<ToolPermissionFacet>,
    identity: PreparedPlanIdentity,
    receipt: Arc<InvocationReceipt>,
}

#[derive(Default)]
struct InvocationReceipt(std::sync::Mutex<ReceiptState>);

#[derive(Clone, Default)]
enum ReceiptState {
    #[default]
    NotStarted,
    Closed,
    Unconfirmed,
    Owned(Arc<dyn talos_core::tool::ToolCleanupReceipt>),
}

#[async_trait]
impl talos_core::tool::ToolCleanupReceipt for InvocationReceipt {
    async fn wait(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), talos_core::tool::ToolCleanupError> {
        let receipt = {
            let mut state = self
                .0
                .lock()
                .map_err(|_| talos_core::tool::ToolCleanupError::Unconfirmed)?;
            if matches!(*state, ReceiptState::NotStarted) {
                *state = ReceiptState::Closed;
            }
            state.clone()
        };
        match receipt {
            ReceiptState::Owned(receipt) => receipt.wait(timeout).await,
            ReceiptState::NotStarted | ReceiptState::Closed => Ok(()),
            ReceiptState::Unconfirmed => Err(talos_core::tool::ToolCleanupError::Unconfirmed),
        }
    }
}

#[async_trait]
impl PreparedNormalToolInvocation for PreparedValidation {
    fn cleanup_receipt(&self) -> Option<Arc<dyn talos_core::tool::ToolCleanupReceipt>> {
        Some(self.receipt.clone())
    }
    fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
        Some(&self.identity)
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }
    async fn execute(
        self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> ToolExecutionOutput {
        let proof = match admission.downcast::<AdmittedInvocation>() {
            Ok(proof) => proof,
            Err(_) => {
                return ToolExecutionOutput::from_result(ToolResult::error(
                    "missing exact validation admission",
                ));
            }
        };
        let request = PermissionRequest::new(
            "validate_project",
            self.provenance.clone(),
            &self.facets,
            &self.input,
        );
        if proof
            .consume_for_prepared(&request, &self.identity)
            .is_err()
        {
            return ToolExecutionOutput::from_result(ToolResult::error(
                "validation admission does not match prepared invocation",
            ));
        }
        if self.tool.permission_profile(&self.input) != self.facets {
            return ToolExecutionOutput::from_result(ToolResult::error(
                "validation checks changed after admission",
            ));
        }
        ToolExecutionOutput::from_result(
            self.tool
                .execute_admitted(
                    self.input,
                    "exact prepared invocation admitted and consumed",
                    &self.receipt,
                )
                .await,
        )
    }
}

impl ValidationTool {
    async fn execute_admitted(
        &self,
        input: Value,
        permission_decision: &str,
        receipt: &InvocationReceipt,
    ) -> ToolResult {
        let input = match parse(&input) {
            Ok(input) => input,
            Err(error) => return ToolResult::error(error),
        };
        if !project_validation_available(&self.workspace) {
            return ToolResult::error("validation unavailable: project markers missing");
        }
        let plan = collect_validation_plan(&self.workspace, input.profile.into());
        if matches!(input.mode, Mode::Plan) {
            return ToolResult::success(render_json_plan(&plan));
        }
        match receipt.0.lock() {
            Ok(mut slot) if matches!(*slot, ReceiptState::NotStarted) => {
                *slot = ReceiptState::Unconfirmed
            }
            Ok(_) => return ToolResult::error("validation operation already closed or started"),
            Err(_) => return ToolResult::error("validation cleanup receipt unavailable"),
        }
        let invocation = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.executor.invocation()
        })) {
            Ok(invocation) => invocation,
            Err(_) => {
                return ToolResult::error(
                    "validation host factory panicked; no checks started; cleanup not confirmed",
                );
            }
        };
        // Publish operation ownership before the first await that can execute a check.
        // Cancellation cannot interrupt this synchronous factory/publication segment.
        let operation_receipt = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            invocation.cleanup_receipt()
        })) {
            Ok(Some(receipt)) => receipt,
            _ => {
                return ToolResult::error(
                    "validation operation cleanup receipt missing or failed; no checks started; cleanup not confirmed",
                );
            }
        };
        match receipt.0.lock() {
            Ok(mut slot) => *slot = ReceiptState::Owned(operation_receipt),
            Err(_) => return ToolResult::error("validation cleanup receipt unavailable"),
        }
        let evidence = run_validation_plan_with_executor(
            &self.workspace,
            plan,
            invocation.as_ref(),
            permission_decision,
        )
        .await;
        let failed = evidence
            .findings
            .iter()
            .any(|finding| finding.severity == talos_conversation::FindingSeverity::Blocked)
            || evidence
                .records
                .iter()
                .any(|record| record.required && record.status != EvidenceStatus::Passed);
        let content = render_json_evidence(&evidence);
        if failed {
            ToolResult::error(content)
        } else {
            ToolResult::success(content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    use talos_conversation::{ValidationCheck, ValidationHostFailure, ValidationHostFuture};
    use talos_core::tool::{ToolCleanupError, ToolCleanupReceipt};

    fn admission_proof(
        tool: &dyn AgentTool,
        operation: &dyn PreparedNormalToolInvocation,
        input: &Value,
    ) -> AdmittedInvocation {
        use talos_permission::{
            InteractionCapability, PermissionContext, PermissionEngine, PermissionInvocation,
            PermissionMode, PermissionSessionState,
        };
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            tool.execution_working_directory().expect("bound workspace"),
        ));
        let context = PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        );
        let request =
            PermissionRequest::native("validate_project", operation.permission_profile(), input);
        let pending = match state
            .begin_invocation(&request, &context)
            .expect("evaluation")
        {
            PermissionInvocation::Allow(pending) => *pending,
            PermissionInvocation::Ask { once, .. } => state
                .approve_once(*once, &request, &context)
                .expect("approve once"),
            PermissionInvocation::Deny(_) => panic!("unexpected denial"),
        };
        state
            .try_admit_prepared_invocation(
                pending,
                &request,
                &context,
                operation.plan_identity().expect("identity"),
            )
            .expect("admission")
    }

    async fn admitted_run(tool: &dyn AgentTool, input: Value) -> ToolResult {
        let operation = tool
            .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
            .expect("prepare")
            .expect("run operation");
        let proof = admission_proof(tool, operation.as_ref(), &input);
        operation.execute(Box::new(proof)).await.result
    }

    #[tokio::test]
    async fn missing_or_panicked_operation_receipt_refuses_checks_and_stays_unconfirmed() {
        #[derive(Clone)]
        struct BrokenReceipt(bool);
        impl ValidationHostFactory for BrokenReceipt {
            fn invocation(&self) -> Box<dyn ValidationHostExecutor> {
                Box::new(self.clone())
            }
        }
        #[async_trait]
        impl ToolCleanupReceipt for BrokenReceipt {
            async fn wait(&self, _: Duration) -> Result<(), ToolCleanupError> {
                Ok(())
            }
        }
        impl ToolCleanupOwner for BrokenReceipt {
            fn seal(&self) {}
        }
        impl ValidationHostExecutor for BrokenReceipt {
            fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupReceipt>> {
                assert!(!self.0, "injected receipt panic");
                None
            }
            fn is_cancelled(&self) -> bool {
                false
            }
            fn execute<'a>(
                &'a self,
                _: &'a Path,
                _: &'a ValidationCheck,
                _: Duration,
                _: usize,
            ) -> ValidationHostFuture<'a> {
                panic!("missing receipt must prevent execution")
            }
        }
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let input = serde_json::json!({"mode":"run", "profile":"workspace"});
        for panic in [false, true] {
            let tools = validation_tool_contributions(
                root.path().to_owned(),
                Arc::new(BrokenReceipt(panic)),
            );
            let tool = tools[0].tool().as_ref();
            let operation = tool
                .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
                .expect("prepare")
                .expect("operation");
            let proof = admission_proof(tool, operation.as_ref(), &input);
            let receipt = operation.cleanup_receipt().expect("receipt");
            let result = operation.execute(Box::new(proof)).await.result;
            assert!(result.is_error);
            assert!(result.content.contains("cleanup receipt missing or failed"));
            assert_eq!(
                receipt.wait(Duration::from_secs(1)).await,
                Err(ToolCleanupError::Unconfirmed)
            );
        }
    }

    #[tokio::test]
    async fn waiting_before_dispatch_seals_factory_and_factory_panic_stays_unconfirmed() {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let tools =
            validation_tool_contributions(root.path().to_owned(), Arc::new(PanickingFactory));
        let tool = tools[0].tool().as_ref();
        let input = serde_json::json!({"mode":"run", "profile":"workspace"});
        for close_first in [true, false] {
            let operation = tool
                .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
                .expect("prepare")
                .expect("operation");
            let proof = admission_proof(tool, operation.as_ref(), &input);
            let receipt = operation.cleanup_receipt().expect("local receipt");
            if close_first {
                receipt
                    .wait(Duration::from_secs(1))
                    .await
                    .expect("closed unstarted");
            }
            let result = operation.execute(Box::new(proof)).await.result;
            assert!(result.is_error);
            if close_first {
                assert!(result.content.contains("already closed"));
                receipt
                    .wait(Duration::from_secs(1))
                    .await
                    .expect("no factory execution");
            } else {
                assert!(result.content.contains("factory panicked"));
                assert_eq!(
                    receipt.wait(Duration::from_secs(1)).await,
                    Err(ToolCleanupError::Unconfirmed)
                );
            }
        }
    }

    #[tokio::test]
    async fn changed_facets_invalidate_consumption_before_executor_creation() {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let factory = Arc::new(PanickingFactory);
        let tool = ValidationTool {
            workspace: root.path().to_owned(),
            executor: factory.clone(),
            cleanup: factory,
        };
        let input = serde_json::json!({"mode":"run", "profile":"workspace"});
        let mut operation = PreparedValidation {
            facets: tool.permission_profile(&input),
            tool: tool.clone(),
            input: input.clone(),
            provenance: ToolProvenance::Native,
            identity: PreparedPlanIdentity::new(),
            receipt: Arc::new(InvocationReceipt::default()),
        };
        let proof = admission_proof(&tool, &operation, &input);
        operation.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            "unapproved command",
            ToolResourceKind::Command,
        ));
        let result = Box::new(operation).execute(Box::new(proof)).await.result;
        assert!(result.is_error);
        assert!(
            result
                .content
                .contains("does not match prepared invocation")
        );
    }

    #[tokio::test]
    async fn missing_wrong_input_and_wrong_plan_proofs_never_construct_executor() {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        // Any factory entry panics, so a clean refusal proves it was not reached.
        let tools =
            validation_tool_contributions(root.path().to_owned(), Arc::new(PanickingFactory));
        let tool = tools[0].tool().as_ref();
        let input = serde_json::json!({"mode":"run", "profile":"workspace"});
        for kind in 0..3 {
            let operation = tool
                .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
                .expect("prepare")
                .expect("operation");
            let proof: Box<dyn std::any::Any + Send> = match kind {
                0 => Box::new(()),
                1 => Box::new(admission_proof(
                    tool,
                    operation.as_ref(),
                    &serde_json::json!({"mode":"run", "profile":"i076"}),
                )),
                _ => {
                    let other = tool
                        .prepare_normalized_with_provenance(&input, ToolProvenance::Native)
                        .expect("prepare")
                        .expect("other identity");
                    Box::new(admission_proof(tool, other.as_ref(), &input))
                }
            };
            let result = operation.execute(proof).await.result;
            assert!(result.is_error);
            assert!(result.content.contains("admission"));
            assert!(!result.content.contains("factory panicked"));
        }
    }

    #[derive(Clone, Default)]
    struct RefusingHost(Arc<AtomicUsize>);

    impl ValidationHostFactory for RefusingHost {
        fn invocation(&self) -> Box<dyn ValidationHostExecutor> {
            Box::new(self.clone())
        }
    }

    impl ValidationHostExecutor for RefusingHost {
        fn cleanup_receipt(&self) -> Option<Arc<dyn ToolCleanupReceipt>> {
            Some(Arc::new(self.clone()))
        }
        fn is_cancelled(&self) -> bool {
            false
        }

        fn execute<'a>(
            &'a self,
            _workspace: &'a Path,
            _check: &'a ValidationCheck,
            _timeout: Duration,
            _output_limit: usize,
        ) -> ValidationHostFuture<'a> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(ValidationHostFailure::NotStarted("fixture refusal".into())) })
        }
    }

    #[async_trait]
    impl ToolCleanupReceipt for RefusingHost {
        async fn wait(&self, _timeout: Duration) -> Result<(), ToolCleanupError> {
            Ok(())
        }
    }

    impl ToolCleanupOwner for RefusingHost {
        fn seal(&self) {}
    }

    struct PanickingFactory;

    impl ValidationHostFactory for PanickingFactory {
        fn invocation(&self) -> Box<dyn ValidationHostExecutor> {
            panic!("private fixture payload")
        }
    }

    #[async_trait]
    impl ToolCleanupReceipt for PanickingFactory {
        async fn wait(&self, _timeout: Duration) -> Result<(), ToolCleanupError> {
            Err(ToolCleanupError::Unconfirmed)
        }
    }

    impl ToolCleanupOwner for PanickingFactory {
        fn seal(&self) {}
    }

    #[tokio::test]
    async fn factory_panic_is_an_error_without_payload_or_cleanup_claim() {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("marker");
        let contributions =
            validation_tool_contributions(root.path().to_owned(), Arc::new(PanickingFactory));
        let result = admitted_run(
            contributions[0].tool().as_ref(),
            serde_json::json!({"mode":"run", "profile":"workspace"}),
        )
        .await;
        assert!(result.is_error);
        assert!(result.content.contains("cleanup not confirmed"));
        assert!(!result.content.contains("private fixture payload"));
    }

    #[tokio::test]
    async fn governance_only_runs_internally_and_other_languages_do_not_invent_cargo_support() {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::create_dir(root.path().join(".agent-governance")).expect("directory");
        std::fs::write(
            root.path().join(".agent-governance/manifest.yaml"),
            "profile: minimal\nstatus: adopting\n",
        )
        .expect("manifest");
        let host = Arc::new(RefusingHost::default());
        let tools = validation_tool_contributions(root.path().to_owned(), host.clone());
        let result = admitted_run(
            tools[0].tool().as_ref(),
            serde_json::json!({"mode":"run", "profile":"governance"}),
        )
        .await;
        let evidence: Value = serde_json::from_str(&result.content).expect("structured evidence");
        assert_eq!(evidence["records"].as_array().expect("records").len(), 1);
        assert_eq!(evidence["records"][0]["execution_mode"], "internal");
        assert_eq!(host.0.load(Ordering::SeqCst), 0);
        for marker in ["package.json", "pyproject.toml", "go.mod", "pom.xml"] {
            let project = tempfile::tempdir().expect("language workspace");
            std::fs::write(project.path().join(marker), "").expect("language marker");
            let tools = validation_tool_contributions(project.path().to_owned(), host.clone());
            assert_eq!(tools.len(), 1);
            let result = admitted_run(
                tools[0].tool().as_ref(),
                serde_json::json!({"mode":"run", "profile":"workspace"}),
            )
            .await;
            assert!(
                result.is_error,
                "unsupported Cargo execution must not look successful"
            );
            assert!(
                result
                    .content
                    .contains("Cargo host-tool adapter unavailable")
            );
            assert_eq!(host.0.load(Ordering::SeqCst), 0);
            assert!(!project.path().join("Cargo.toml").exists());
        }
    }

    #[tokio::test]
    async fn strict_input_and_plan_never_call_host_but_run_reports_failure() {
        let root = tempfile::tempdir().expect("workspace");
        let host = Arc::new(RefusingHost::default());
        assert!(validation_tool_contributions(root.path().to_owned(), host.clone()).is_empty());
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("Cargo marker");
        let contributions = validation_tool_contributions(root.path().to_owned(), host.clone());
        let tool = contributions[0].tool();
        for input in [
            serde_json::json!({}),
            serde_json::json!({"mode":"run", "profile":"arbitrary"}),
            serde_json::json!({"mode":"run", "profile":"workspace", "command":"touch marker"}),
            serde_json::json!({"mode":"run", "profile":"workspace", "workspace":"/"}),
            serde_json::json!({"mode":"run", "profile":"workspace", "env":{}}),
            serde_json::json!({"mode":true, "profile":"workspace"}),
        ] {
            assert!(tool.execution_admission(&input).is_err());
            assert!(tool.execute(input).await.is_error);
        }
        let plan = serde_json::json!({"mode":"plan", "profile":"workspace"});
        assert_eq!(tool.permission_profile(&plan)[0].nature, ToolNature::Read);
        assert!(!tool.execute(plan).await.is_error);
        assert_eq!(host.0.load(Ordering::SeqCst), 0);
        let run = serde_json::json!({"mode":"run", "profile":"workspace"});
        assert_eq!(tool.permission_profile(&run)[0].nature, ToolNature::Execute);
        assert!(
            tool.execute(run.clone())
                .await
                .content
                .contains("requires exact prepared")
        );
        assert_eq!(host.0.load(Ordering::SeqCst), 0);
        let result = admitted_run(tool.as_ref(), run.clone()).await;
        assert!(result.is_error);
        assert!(result.content.contains("fixture refusal"));
        assert!(
            result
                .content
                .contains("exact prepared invocation admitted and consumed")
        );
        assert_eq!(host.0.load(Ordering::SeqCst), 3);
        std::fs::remove_file(root.path().join("Cargo.toml")).expect("remove marker");
        assert!(tool.execute(run).await.is_error);
        assert_eq!(host.0.load(Ordering::SeqCst), 3);
        assert!(tool.cleanup_receipt().is_some());
    }
}
