//! Host execution injection for shared validation plans.

use std::{future::Future, path::Path, pin::Pin, time::Duration};

use futures::FutureExt;

use super::*;

/// Output retained by a host adapter. The adapter must enforce bounds while reading.
pub struct ValidationHostOutput {
    /// Process exit status; absent when execution did not complete normally.
    pub exit_status: Option<i32>,
    /// True only for normal successful completion with confirmed cleanup.
    pub success: bool,
    /// Bounded standard output.
    pub stdout: Vec<u8>,
    /// Bounded standard error.
    pub stderr: Vec<u8>,
}

/// Host failure with explicit execution lifecycle evidence.
#[derive(Debug)]
pub enum ValidationHostFailure {
    /// No child was started, for example admission refusal or spawn failure.
    NotStarted(String),
    /// A child started, or execution cannot safely be certified as not started.
    Failed(String),
    /// Cleanup could not be confirmed; no subsequent check may start.
    CleanupUnconfirmed(String),
}

/// Async result of one host-owned validation check.
pub type ValidationHostFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ValidationHostOutput, ValidationHostFailure>> + Send + 'a>>;

/// Already-authorized host execution, injected without platform dependencies.
///
/// Implementors own timeout, concurrent bounded drains, cancellation and confirmed
/// cleanup. Dropping the future must request cleanup; the host must retain its
/// aggregate cleanup owner until shutdown completes. This trait grants no permission.
pub trait ValidationHostExecutor: Send + Sync {
    /// Operation-local completion receipt for cancellation-aware dispatch.
    /// None makes no claim of prepared-operation cleanup support.
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupReceipt>> {
        None
    }
    /// Whether the caller has cancelled this invocation.
    fn is_cancelled(&self) -> bool;

    /// Run one fixed-profile check in the host-bound workspace.
    fn execute<'a>(
        &'a self,
        workspace: &'a Path,
        check: &'a ValidationCheck,
        timeout: Duration,
        output_limit: usize,
    ) -> ValidationHostFuture<'a>;
}

/// Run a plan using shared internal checks and an authorized host adapter.
///
/// The caller supplies the actual permission decision. A profile allowlist alone
/// is not authorization. Cancellation prevents all later checks from starting.
/// Checks and project detection are rebuilt from the selected profile; caller
/// modifications to a previously collected plan cannot inject host commands.
pub async fn run_validation_plan_with_executor(
    workspace: &Path,
    plan: ValidationPlan,
    executor: &dyn ValidationHostExecutor,
    permission_decision: &str,
) -> ValidationEvidence {
    let plan = collect_validation_plan(workspace, plan.profile);
    let mut records = Vec::with_capacity(plan.checks.len());
    let mut adapter_failed = false;
    let mut cancelled_seen = false;
    for check in &plan.checks {
        let cancelled = if adapter_failed || cancelled_seen {
            Ok(true)
        } else {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| executor.is_cancelled()))
        };
        let result = if adapter_failed {
            Err(ValidationHostFailure::NotStarted(
                "validation stopped after host adapter contract failure".into(),
            ))
        } else if cancelled.is_err() {
            adapter_failed = true;
            Err(ValidationHostFailure::Failed(
                "host adapter cancellation query panicked; cleanup confirmation required".into(),
            ))
        } else if matches!(cancelled, Ok(true)) {
            cancelled_seen = true;
            Err(ValidationHostFailure::NotStarted(
                "validation cancelled before check started".into(),
            ))
        } else if check.execution_mode == ValidationExecutionMode::Internal {
            records.push(run_internal_validation_check(
                workspace,
                check,
                permission_decision.to_owned(),
            ));
            continue;
        } else if check.program.is_none() {
            Err(ValidationHostFailure::NotStarted(
                "host-tool validation check missing program".into(),
            ))
        } else if check.ecosystem == Some("rust")
            && !plan.project_types.contains(&ProjectType::Rust)
        {
            Err(ValidationHostFailure::NotStarted(
                "Cargo host-tool adapter unavailable: Cargo.toml missing".into(),
            ))
        } else {
            // Include construction and polling of the injected future in the
            // unwind boundary. Do not expose panic payloads (possibly secrets).
            match std::panic::AssertUnwindSafe(async {
                executor
                    .execute(workspace, check, HOST_CHECK_TIMEOUT, HOST_OUTPUT_LIMIT)
                    .await
            })
            .catch_unwind()
            .await
            {
                Ok(Ok(output))
                    if output.stdout.len() > HOST_OUTPUT_LIMIT
                        || output.stderr.len() > HOST_OUTPUT_LIMIT
                        || (output.success && output.exit_status != Some(0)) =>
                {
                    adapter_failed = true;
                    Err(ValidationHostFailure::Failed(
                        "host adapter returned invalid or oversized execution evidence".into(),
                    ))
                }
                Ok(Err(ValidationHostFailure::CleanupUnconfirmed(message))) => {
                    adapter_failed = true;
                    Err(ValidationHostFailure::CleanupUnconfirmed(message))
                }
                Ok(result) => result,
                Err(_) => {
                    adapter_failed = true;
                    Err(ValidationHostFailure::Failed(
                        "host adapter panicked; cleanup confirmation required".into(),
                    ))
                }
            }
        };
        let (status, exit_status, stdout_summary, stderr_summary) = match result {
            Ok(output) => (
                if output.success {
                    EvidenceStatus::Passed
                } else {
                    EvidenceStatus::Failed
                },
                output.exit_status,
                summarize_output(&output.stdout),
                summarize_output(&output.stderr),
            ),
            Err(error) => {
                let (status, message) = match error {
                    ValidationHostFailure::NotStarted(message) => {
                        (EvidenceStatus::NotStarted, message)
                    }
                    ValidationHostFailure::Failed(message)
                    | ValidationHostFailure::CleanupUnconfirmed(message) => {
                        (EvidenceStatus::Failed, message)
                    }
                };
                (status, None, "<empty>".into(), bounded_host_error(message))
            }
        };
        records.push(ValidationEvidenceRecord {
            id: check.id,
            command: check.command,
            execution_mode: check.execution_mode,
            required: check.required,
            source: check.source,
            ecosystem: check.ecosystem,
            permission_decision: permission_decision.to_owned(),
            status,
            exit_status,
            stdout_summary,
            stderr_summary,
        });
    }
    ValidationEvidence {
        profile: plan.profile,
        authority: "host-authorized fixed-profile validation; no arbitrary commands accepted",
        findings: plan
            .findings
            .into_iter()
            .filter(|finding| {
                finding.message != "Plan mode is read-only: commands are listed but not executed"
            })
            .collect(),
        records,
    }
}

fn bounded_host_error(mut message: String) -> String {
    const MARKER: &str = "\n[error truncated]";
    if message.len() > HOST_OUTPUT_LIMIT {
        let mut end = HOST_OUTPUT_LIMIT - MARKER.len();
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push_str(MARKER);
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Default)]
    struct CancelAfterFirst {
        cancelled: AtomicBool,
        calls: AtomicUsize,
    }

    impl ValidationHostExecutor for CancelAfterFirst {
        fn is_cancelled(&self) -> bool {
            self.cancelled.load(Ordering::SeqCst)
        }

        fn execute<'a>(
            &'a self,
            _: &'a Path,
            _: &'a ValidationCheck,
            timeout: Duration,
            output_limit: usize,
        ) -> ValidationHostFuture<'a> {
            Box::pin(async move {
                assert_eq!(timeout, Duration::from_secs(30));
                assert_eq!(output_limit, 16 * 1024);
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.cancelled.store(true, Ordering::SeqCst);
                Err(ValidationHostFailure::Failed(
                    "cancelled after spawn; cleanup confirmed".into(),
                ))
            })
        }
    }

    #[tokio::test]
    async fn cancellation_stops_later_checks_and_preserves_started_failure() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
        assert!(plan.checks.len() > 1);
        let host = CancelAfterFirst::default();
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "Execute approved")
                .await;
        assert_eq!(host.calls.load(Ordering::SeqCst), 1);
        assert_eq!(evidence.records[0].status, EvidenceStatus::Failed);
        assert!(
            evidence
                .records
                .iter()
                .skip(1)
                .all(|record| record.status == EvidenceStatus::NotStarted)
        );
        assert!(
            evidence
                .records
                .iter()
                .all(|record| record.permission_decision == "Execute approved")
        );
    }

    #[tokio::test]
    async fn missing_cargo_and_internal_governance_never_call_host() {
        let workspace = tempfile::tempdir().expect("workspace");
        let host = CancelAfterFirst::default();
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "Execute approved")
                .await;
        assert!(
            evidence
                .records
                .iter()
                .filter(|record| record.execution_mode == ValidationExecutionMode::HostTool)
                .all(|record| record.status == EvidenceStatus::NotStarted)
        );
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::Governance);
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "internal check")
                .await;
        assert!(
            evidence
                .records
                .iter()
                .all(|record| record.execution_mode == ValidationExecutionMode::Internal)
        );
        assert_eq!(host.calls.load(Ordering::SeqCst), 0);
    }

    struct BrokenAdapter {
        during_poll: bool,
        calls: AtomicUsize,
    }

    struct CancellationPanic;

    struct TransientCancellation(AtomicUsize);

    impl ValidationHostExecutor for TransientCancellation {
        fn is_cancelled(&self) -> bool {
            self.0.fetch_add(1, Ordering::SeqCst) == 0
        }

        fn execute<'a>(
            &'a self,
            _: &'a Path,
            _: &'a ValidationCheck,
            _: Duration,
            _: usize,
        ) -> ValidationHostFuture<'a> {
            panic!("cancelled plan must never execute")
        }
    }

    #[tokio::test]
    async fn first_cancellation_is_latched_for_the_entire_plan() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
        let host = TransientCancellation(AtomicUsize::new(0));
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "approved").await;
        assert_eq!(host.0.load(Ordering::SeqCst), 1);
        assert!(
            evidence
                .records
                .iter()
                .all(|record| record.status == EvidenceStatus::NotStarted)
        );
    }

    struct UnconfirmedCleanup(AtomicUsize);

    impl ValidationHostExecutor for UnconfirmedCleanup {
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
            Box::pin(async move {
                self.0.fetch_add(1, Ordering::SeqCst);
                Err(ValidationHostFailure::CleanupUnconfirmed(
                    "清理未确认".repeat(HOST_OUTPUT_LIMIT),
                ))
            })
        }
    }

    #[tokio::test]
    async fn unconfirmed_cleanup_stops_later_checks_and_bounds_error_evidence() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let host = UnconfirmedCleanup(AtomicUsize::new(0));
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "Execute approved")
                .await;
        assert_eq!(host.0.load(Ordering::SeqCst), 1);
        assert_eq!(evidence.records[0].status, EvidenceStatus::Failed);
        assert!(evidence.records[0].stderr_summary.len() <= HOST_OUTPUT_LIMIT);
        assert!(
            evidence.records[0]
                .stderr_summary
                .ends_with("[error truncated]")
        );
        assert!(
            evidence
                .records
                .iter()
                .skip(1)
                .all(|record| record.status == EvidenceStatus::NotStarted)
        );
    }

    impl ValidationHostExecutor for CancellationPanic {
        fn is_cancelled(&self) -> bool {
            panic!("private cancellation panic payload")
        }

        fn execute<'a>(
            &'a self,
            _: &'a Path,
            _: &'a ValidationCheck,
            _: Duration,
            _: usize,
        ) -> ValidationHostFuture<'a> {
            panic!("execution must not be reached")
        }
    }

    #[tokio::test]
    async fn cancellation_query_panic_is_failed_and_stops_the_plan() {
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
        let evidence = run_validation_plan_with_executor(
            workspace.path(),
            plan,
            &CancellationPanic,
            "approved",
        )
        .await;
        assert_eq!(evidence.records[0].status, EvidenceStatus::Failed);
        assert!(
            evidence
                .records
                .iter()
                .skip(1)
                .all(|record| record.status == EvidenceStatus::NotStarted)
        );
        assert!(!super::super::render_json_evidence(&evidence).contains("private"));
    }

    #[test]
    fn error_bound_preserves_utf8_and_includes_marker_within_cap() {
        let original = "界".repeat(HOST_OUTPUT_LIMIT);
        let bounded = bounded_host_error(original);
        assert!(bounded.len() <= HOST_OUTPUT_LIMIT);
        assert!(bounded.ends_with("\n[error truncated]"));
        assert_eq!(bounded_host_error("small error".into()), "small error");
    }

    #[tokio::test]
    async fn caller_modified_plan_cannot_inject_an_executable() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let mut plan = collect_validation_plan(workspace.path(), ValidationProfile::Governance);
        plan.checks = checks_for_profile(ValidationProfile::I076);
        plan.checks[0].program = Some("injected-executable");
        let host = CancelAfterFirst::default();
        let evidence =
            run_validation_plan_with_executor(workspace.path(), plan, &host, "approved").await;
        assert_eq!(host.calls.load(Ordering::SeqCst), 0);
        assert!(
            evidence
                .records
                .iter()
                .all(|record| record.execution_mode == ValidationExecutionMode::Internal)
        );
    }

    impl ValidationHostExecutor for BrokenAdapter {
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
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert!(self.during_poll, "private construction panic payload");
            Box::pin(async { panic!("private polling panic payload") })
        }
    }

    #[tokio::test]
    async fn adapter_panics_stop_remaining_checks_without_disclosing_payload() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(workspace.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        for during_poll in [false, true] {
            let host = BrokenAdapter {
                during_poll,
                calls: AtomicUsize::new(0),
            };
            let plan = collect_validation_plan(workspace.path(), ValidationProfile::I076);
            let evidence = run_validation_plan_with_executor(
                workspace.path(),
                plan,
                &host,
                "Execute approved",
            )
            .await;
            assert_eq!(host.calls.load(Ordering::SeqCst), 1);
            assert_eq!(evidence.records[0].status, EvidenceStatus::Failed);
            assert!(
                evidence
                    .records
                    .iter()
                    .skip(1)
                    .all(|record| record.status == EvidenceStatus::NotStarted)
            );
            assert!(!super::super::render_json_evidence(&evidence).contains("private"));
        }
    }
}
