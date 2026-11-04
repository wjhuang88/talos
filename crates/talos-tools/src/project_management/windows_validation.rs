//! Windows validation uses the existing assigned-before-resume Job executor.

use super::ValidationHostFactory;
use crate::process_boundary::windows_foreground::{
    CommandStartState, OwnedCommandExecutor, OwnedCommandOperation,
};
use async_trait::async_trait;
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use talos_conversation::{
    ValidationCheck, ValidationHostExecutor, ValidationHostFailure, ValidationHostFuture,
    ValidationHostOutput,
};
use talos_core::tool::{ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt};
use tokio_util::sync::CancellationToken;

/// Aggregate Windows Job owner for already-authorized validation checks.
#[derive(Default)]
pub struct WindowsValidationHost {
    owner: OwnedCommandExecutor,
}

impl WindowsValidationHost {
    /// Create an independent operation and cancellation scope under this owner.
    pub fn invocation_with_cancellation(
        &self,
        cancellation: CancellationToken,
    ) -> Box<dyn ValidationHostExecutor> {
        Box::new(Invocation {
            operation: self.owner.operation(),
            cancellation,
        })
    }
}

impl ValidationHostFactory for WindowsValidationHost {
    fn invocation(&self) -> Box<dyn ValidationHostExecutor> {
        self.invocation_with_cancellation(CancellationToken::new())
    }
}

#[async_trait]
impl ToolCleanupReceipt for WindowsValidationHost {
    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError> {
        self.owner.wait(timeout).await
    }
}
impl ToolCleanupOwner for WindowsValidationHost {
    fn seal(&self) {
        self.owner.seal();
    }
}

struct Invocation {
    operation: OwnedCommandOperation,
    cancellation: CancellationToken,
}

// Only fixed Cargo profiles currently have a Windows host adapter. Do not run
// command shells or infer support for arbitrary executable suffixes.
fn preparation_checkpoint(
    deadline: std::time::Instant,
    cancellation: &CancellationToken,
) -> Result<(), ValidationHostFailure> {
    if cancellation.is_cancelled() || std::time::Instant::now() >= deadline {
        return Err(ValidationHostFailure::NotStarted(
            "validation cancelled or preparation deadline exceeded".into(),
        ));
    }
    Ok(())
}

fn cargo_executable(
    deadline: std::time::Instant,
    cancellation: &CancellationToken,
) -> Result<PathBuf, ValidationHostFailure> {
    preparation_checkpoint(deadline, cancellation)?;
    let path = std::env::var_os("PATH").ok_or_else(|| {
        ValidationHostFailure::NotStarted("PATH unavailable for Cargo discovery".into())
    })?;
    if path.len() > 64 * 1024 {
        return Err(ValidationHostFailure::NotStarted(
            "Cargo discovery PATH exceeds 64 KiB".into(),
        ));
    }
    for (index, directory) in std::env::split_paths(&path).enumerate() {
        if index >= 256 {
            return Err(ValidationHostFailure::NotStarted(
                "Cargo discovery exceeds 256 PATH entries".into(),
            ));
        }
        preparation_checkpoint(deadline, cancellation)?;
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join("cargo.exe");
        if candidate.is_file() {
            preparation_checkpoint(deadline, cancellation)?;
            let resolved = candidate.canonicalize().map_err(|_| {
                ValidationHostFailure::NotStarted("Cargo executable could not be bound".into())
            })?;
            preparation_checkpoint(deadline, cancellation)?;
            return Ok(resolved);
        }
    }
    Err(ValidationHostFailure::NotStarted(
        "cargo.exe unavailable on absolute PATH entries".into(),
    ))
}

impl ValidationHostExecutor for Invocation {
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn ToolCleanupReceipt>> {
        Some(std::sync::Arc::new(self.operation.cleanup_receipt()))
    }
    fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
    fn execute<'a>(
        &'a self,
        workspace: &'a Path,
        check: &'a ValidationCheck,
        timeout: Duration,
        output_limit: usize,
    ) -> ValidationHostFuture<'a> {
        Box::pin(async move {
            let deadline = std::time::Instant::now() + timeout;
            if self.is_cancelled() {
                return Err(ValidationHostFailure::NotStarted(
                    "validation cancelled".into(),
                ));
            }
            if check.program != Some("cargo") {
                return Err(ValidationHostFailure::NotStarted(
                    "Windows validation supports fixed Cargo checks only".into(),
                ));
            }
            let workspace = workspace.to_owned();
            let args = check.args;
            let receipt = self.operation.cleanup_receipt();
            let prepared = {
                let preparation = self.operation.run_preparation(
                    self.cancellation.child_token(),
                    move |cancellation| {
                        let executable = cargo_executable(deadline, &cancellation)?;
                        preparation_checkpoint(deadline, &cancellation)?;
                        let workspace = workspace.canonicalize().map_err(|_| {
                            ValidationHostFailure::NotStarted("workspace unavailable".into())
                        })?;
                        preparation_checkpoint(deadline, &cancellation)?;
                        let dangerous =
                            talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
                        let mut environment = Vec::new();
                        let mut environment_bytes = 0usize;
                        for (index, (name, value)) in std::env::vars_os().enumerate() {
                            preparation_checkpoint(deadline, &cancellation)?;
                            environment_bytes = environment_bytes
                                .saturating_add(name.len())
                                .saturating_add(value.len());
                            if index >= 4096 || environment_bytes > 1024 * 1024 {
                                return Err(ValidationHostFailure::NotStarted(
                                    "validation environment exceeds 4096 entries or 1 MiB".into(),
                                ));
                            }
                            if !dangerous
                                .iter()
                                .any(|blocked| name.to_string_lossy().eq_ignore_ascii_case(blocked))
                            {
                                environment.push((name, value));
                            }
                        }
                        let mut command = Command::new(executable);
                        command
                            .current_dir(workspace)
                            .args(args)
                            .env_clear()
                            .envs(environment);
                        preparation_checkpoint(deadline, &cancellation)?;
                        Ok::<_, ValidationHostFailure>(command)
                    },
                );
                tokio::pin!(preparation);
                tokio::select! {
                    biased;
                    _ = self.cancellation.cancelled() => None,
                    _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => None,
                    result = &mut preparation => Some(result),
                }
            };
            if receipt.wait(Duration::from_secs(3)).await.is_err() {
                return Err(ValidationHostFailure::CleanupUnconfirmed(
                    "Windows preparation worker cleanup could not be confirmed".into(),
                ));
            }
            let command =
                match prepared {
                    Some(Ok(result)) => result?,
                    Some(Err(message)) => return Err(ValidationHostFailure::NotStarted(message)),
                    None => return Err(ValidationHostFailure::NotStarted(
                        "validation preparation cancelled or timed out; worker cleanup confirmed"
                            .into(),
                    )),
                };
            preparation_checkpoint(deadline, &self.cancellation)?;
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let receipt = self.operation.cleanup_receipt();
            let outcome = {
                let execution =
                    self.operation
                        .execute_with_start_evidence(command, remaining, output_limit);
                tokio::pin!(execution);
                tokio::select! {
                    biased;
                    _ = self.cancellation.cancelled() => None,
                    result = &mut execution => Some(result),
                }
            };
            if receipt.wait(Duration::from_secs(3)).await.is_err() {
                return Err(ValidationHostFailure::CleanupUnconfirmed(
                    "Windows validation Job cleanup could not be confirmed".into(),
                ));
            }
            match outcome {
                Some(Ok(output)) => Ok(ValidationHostOutput {
                    success: output.status.success(),
                    exit_status: output.status.code(),
                    stdout: output.stdout,
                    stderr: output.stderr,
                }),
                Some(Err(failure)) => match failure.start_state {
                    CommandStartState::NotStarted => {
                        Err(ValidationHostFailure::NotStarted(failure.message))
                    }
                    CommandStartState::Started | CommandStartState::Unknown => {
                        Err(ValidationHostFailure::Failed(failure.message))
                    }
                },
                None => Err(ValidationHostFailure::Failed(
                    "validation cancelled; Job cleanup confirmed".into(),
                )),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn native_cargo_execution_returns_output_and_operation_cleanup() {
        let root = tempfile::tempdir().expect("workspace");
        let host = WindowsValidationHost::default();
        let invocation = host.invocation();
        let check = ValidationCheck {
            id: "native-adapter-fixture",
            command: "cargo --version",
            execution_mode: talos_conversation::ValidationExecutionMode::HostTool,
            program: Some("cargo"),
            args: &["--version"],
            required: true,
            source: "native Windows adapter regression",
            ecosystem: Some("rust"),
        };
        let receipt = invocation.cleanup_receipt().expect("operation receipt");
        let result = tokio::time::timeout(
            Duration::from_secs(20),
            invocation.execute(root.path(), &check, Duration::from_secs(15), 16384),
        )
        .await
        .expect("bounded native adapter");
        let output = result.expect("Cargo installed by native CI toolchain");
        assert!(output.success);
        assert_eq!(output.exit_status, Some(0));
        assert!(String::from_utf8_lossy(&output.stdout).starts_with("cargo "));
        receipt
            .wait(Duration::from_secs(3))
            .await
            .expect("operation cleaned");
        let overflow = tokio::time::timeout(
            Duration::from_secs(20),
            invocation.execute(root.path(), &check, Duration::from_secs(15), 1),
        )
        .await
        .expect("bounded overflow");
        assert!(
            matches!(overflow, Err(ValidationHostFailure::Failed(_))),
            "output overflow must be a started failure"
        );
        receipt
            .wait(Duration::from_secs(3))
            .await
            .expect("overflow operation cleaned");
        host.seal();
        host.wait(Duration::from_secs(3))
            .await
            .expect("aggregate cleaned");
    }

    #[test]
    fn preparation_budget_and_cancellation_refuse_before_path_discovery() {
        let token = CancellationToken::new();
        assert!(matches!(
            cargo_executable(std::time::Instant::now(), &token),
            Err(ValidationHostFailure::NotStarted(_))
        ));
        token.cancel();
        assert!(matches!(
            cargo_executable(std::time::Instant::now() + Duration::from_secs(30), &token),
            Err(ValidationHostFailure::NotStarted(_))
        ));
    }

    #[tokio::test]
    async fn cancellation_before_preparation_never_resolves_or_launches_cargo() {
        let host = WindowsValidationHost::default();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let invocation = host.invocation_with_cancellation(cancellation);
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        let plan = talos_conversation::collect_validation_plan(
            root.path(),
            talos_conversation::ValidationProfile::Workspace,
        );
        let result = invocation
            .execute(root.path(), &plan.checks[0], Duration::from_secs(1), 128)
            .await;
        assert!(matches!(result, Err(ValidationHostFailure::NotStarted(_))));
        assert!(!host.invocation().is_cancelled());
        host.seal();
        host.wait(Duration::from_secs(1))
            .await
            .expect("no owned process");
    }
}
