//! Unix validation execution through the existing owned command supervisor.

use std::{path::Path, process::Command, time::Duration};

use async_trait::async_trait;
use talos_conversation::{
    ValidationCheck, ValidationHostExecutor, ValidationHostFailure, ValidationHostFuture,
    ValidationHostOutput,
};
use talos_core::tool::{ToolCleanupError, ToolCleanupOwner, ToolCleanupReceipt};
use talos_sandbox::{CommandStartState, OwnedCommandExecutor, OwnedCommandOperation};
use tokio_util::sync::CancellationToken;

use super::ValidationHostFactory;

const CLEANUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Aggregate Unix process owner for authorized validation invocations.
/// This owns process groups, not sandbox policy or permission admission.
#[derive(Default)]
pub struct UnixValidationHost {
    owner: OwnedCommandExecutor,
}

impl UnixValidationHost {
    /// Create an isolated invocation with caller-owned cancellation.
    /// The caller retains this host through cancellation and aggregate cleanup.
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

impl ValidationHostFactory for UnixValidationHost {
    fn invocation(&self) -> Box<dyn ValidationHostExecutor> {
        self.invocation_with_cancellation(CancellationToken::new())
    }
}

#[async_trait]
impl ToolCleanupReceipt for UnixValidationHost {
    async fn wait(&self, timeout: Duration) -> Result<(), ToolCleanupError> {
        ToolCleanupReceipt::wait(&self.owner, timeout).await
    }
}

impl ToolCleanupOwner for UnixValidationHost {
    fn seal(&self) {
        self.owner.seal();
    }
}

struct Invocation {
    operation: OwnedCommandOperation,
    cancellation: CancellationToken,
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
            if self.is_cancelled() {
                return Err(ValidationHostFailure::NotStarted(
                    "validation cancelled".into(),
                ));
            }
            let program = check.program.ok_or_else(|| {
                ValidationHostFailure::NotStarted("host check has no program".into())
            })?;
            let mut command = Command::new(program);
            command.current_dir(workspace).args(check.args);
            for variable in talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names() {
                command.env_remove(variable);
            }
            self.run(command, timeout, output_limit).await
        })
    }
}

impl Invocation {
    async fn run(
        &self,
        command: Command,
        timeout: Duration,
        output_limit: usize,
    ) -> Result<ValidationHostOutput, ValidationHostFailure> {
        let receipt = self.operation.cleanup_receipt();
        // This scope drops execution BEFORE awaiting its cleanup receipt. Dropping
        // the entire caller future also requests cleanup through the supervisor;
        // the registry retains the aggregate host until shutdown joins it.
        let outcome = {
            let execution =
                self.operation
                    .execute_with_start_evidence(command, timeout, output_limit);
            tokio::pin!(execution);
            tokio::select! {
                biased;
                _ = self.cancellation.cancelled() => None,
                outcome = &mut execution => Some(outcome),
            }
        };
        if receipt.wait_confirmed(CLEANUP_TIMEOUT).await.is_err() {
            return Err(ValidationHostFailure::CleanupUnconfirmed(
                "validation process cleanup could not be confirmed".into(),
            ));
        }
        match outcome {
            Some(Ok(output)) => Ok(ValidationHostOutput {
                success: output.status.success(),
                exit_status: output.status.code(),
                stdout: output.stdout,
                stderr: output.stderr,
            }),
            Some(Err(failure)) => {
                let message = failure.source.to_string();
                match failure.start_state {
                    CommandStartState::NotStarted => {
                        Err(ValidationHostFailure::NotStarted(message))
                    }
                    CommandStartState::Started | CommandStartState::Unknown => {
                        Err(ValidationHostFailure::Failed(message))
                    }
                }
            }
            // Cancellation may race a successful spawn. Never claim zero startup.
            None => Err(ValidationHostFailure::Failed(
                "validation cancelled; cleanup confirmed".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    #[tokio::test]
    async fn cancellation_during_both_output_streams_returns_started_failure() {
        let root = tempfile::tempdir().expect("workspace");
        let host = UnixValidationHost::default();
        let token = CancellationToken::new();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: token.clone(),
        };
        let mut command = shell(
            "printf initial; printf initial >&2; touch ready; while :; do printf chunk; printf chunk >&2; sleep 0.01; done",
        );
        command.current_dir(root.path());
        let cancel = async {
            tokio::time::timeout(Duration::from_secs(3), async {
                while !root.path().join("ready").exists() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("streams started");
            token.cancel();
        };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(
                invocation.run(command, Duration::from_secs(10), 16384),
                cancel
            )
        })
        .await
        .expect("bounded cancellation and cleanup");
        match result {
            Err(ValidationHostFailure::Failed(message)) => {
                assert!(message.contains("cancelled; cleanup confirmed"))
            }
            _ => panic!(
                "cancellation during output must be a started failure with confirmed cleanup"
            ),
        }
        host.seal();
        host.wait(Duration::from_secs(3))
            .await
            .expect("aggregate cleanup");
    }

    #[tokio::test]
    async fn concurrent_drains_accept_exact_limit_beyond_pipe_capacity() {
        let host = UnixValidationHost::default();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: CancellationToken::new(),
        };
        // Each writer exceeds a pipe buffer. Waiting before draining either stream
        // would deadlock; each stream is exactly at the retained-output bound.
        let output = tokio::time::timeout(
            Duration::from_secs(8),
            invocation.run(
                shell("(head -c 262144 /dev/zero) & (head -c 262144 /dev/zero >&2) & wait"),
                Duration::from_secs(5),
                262144,
            ),
        )
        .await
        .expect("bounded drains");
        let Ok(output) = output else {
            panic!("exact-limit output must succeed")
        };
        assert!(output.success);
        assert_eq!(output.stdout, vec![0; 262144]);
        assert_eq!(output.stderr, vec![0; 262144]);
        host.seal();
        host.wait(Duration::from_secs(3))
            .await
            .expect("writer cleanup");
    }

    #[tokio::test]
    async fn deadline_cleans_descendant_holding_pipes_after_parent_exit() {
        let root = tempfile::tempdir().expect("workspace");
        let host = UnixValidationHost::default();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: CancellationToken::new(),
        };
        let mut command = shell("(sleep 2; touch escaped) & printf started; exit 0");
        command.current_dir(root.path());
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            invocation.run(command, Duration::from_millis(200), 128),
        )
        .await
        .expect("deadline includes held pipes and bounded cleanup");
        assert!(
            matches!(result, Err(ValidationHostFailure::Failed(_))),
            "deadline must report a started failure"
        );
        host.seal();
        host.wait(Duration::from_secs(3))
            .await
            .expect("descendant cleanup");
        tokio::time::sleep(Duration::from_millis(2100)).await;
        assert!(!root.path().join("escaped").exists());
    }

    #[tokio::test]
    async fn owned_execution_reports_output_exit_overflow_and_spawn_failure() {
        let host = UnixValidationHost::default();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: CancellationToken::new(),
        };
        let output = invocation
            .run(
                shell("printf out; printf err >&2; exit 7"),
                Duration::from_secs(2),
                16,
            )
            .await;
        let Ok(output) = output else {
            panic!("normal completion expected")
        };
        assert!(!output.success);
        assert_eq!(output.exit_status, Some(7));
        assert_eq!(output.stdout, b"out");
        assert_eq!(output.stderr, b"err");
        let overflow = invocation
            .run(shell("printf 12345"), Duration::from_secs(2), 4)
            .await
            .err();
        assert!(
            matches!(overflow, Some(ValidationHostFailure::Failed(_))),
            "{overflow:?}"
        );
        let directory = tempfile::tempdir().expect("workspace");
        assert!(matches!(
            invocation
                .run(
                    Command::new(directory.path().join("missing")),
                    Duration::from_secs(2),
                    16
                )
                .await,
            Err(ValidationHostFailure::NotStarted(_))
        ));
        host.wait(Duration::from_secs(3))
            .await
            .expect("aggregate cleanup");
    }

    #[tokio::test]
    async fn dropping_call_future_leaves_cleanup_owned_until_host_shutdown() {
        let directory = tempfile::tempdir().expect("workspace");
        let host = UnixValidationHost::default();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: CancellationToken::new(),
        };
        let mut command = shell("touch ready; (sleep 1; touch escaped) & wait");
        command.current_dir(directory.path());
        let running =
            tokio::spawn(
                async move { invocation.run(command, Duration::from_secs(10), 128).await },
            );
        tokio::time::timeout(Duration::from_secs(3), async {
            while !directory.path().join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("process started");
        host.seal();
        running.abort();
        assert!(running.await.is_err());
        host.wait(Duration::from_secs(3))
            .await
            .expect("shutdown confirms cleanup");
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(!directory.path().join("escaped").exists());
    }

    #[tokio::test]
    async fn cancelled_invocation_does_not_wait_for_or_cancel_concurrent_work() {
        let directory = tempfile::tempdir().expect("workspace");
        let host = UnixValidationHost::default();
        let first_cancel = CancellationToken::new();
        let second_cancel = CancellationToken::new();
        let first = Invocation {
            operation: host.owner.operation(),
            cancellation: first_cancel.clone(),
        };
        let second = Invocation {
            operation: host.owner.operation(),
            cancellation: second_cancel.clone(),
        };
        let mut first_command = shell("touch first-ready; exec sleep 30");
        first_command.current_dir(directory.path());
        let mut second_command = shell(
            "touch second-ready; while [ ! -f release ]; do sleep 0.02; done; printf survived",
        );
        second_command.current_dir(directory.path());
        let first_task =
            tokio::spawn(
                async move { first.run(first_command, Duration::from_secs(10), 128).await },
            );
        let second_task = tokio::spawn(async move {
            second
                .run(second_command, Duration::from_secs(10), 128)
                .await
        });
        tokio::time::timeout(Duration::from_secs(3), async {
            while !directory.path().join("first-ready").exists()
                || !directory.path().join("second-ready").exists()
            {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("both processes started");
        first_cancel.cancel();
        let first_result = tokio::time::timeout(Duration::from_secs(4), first_task)
            .await
            .expect("first cleanup excludes second")
            .expect("first task");
        assert!(matches!(
            first_result,
            Err(ValidationHostFailure::Failed(_))
        ));
        assert!(!second_cancel.is_cancelled());
        assert!(!second_task.is_finished());
        assert_eq!(
            host.wait(Duration::from_millis(20)).await,
            Err(ToolCleanupError::Timeout)
        );
        std::fs::write(directory.path().join("release"), "").expect("release second");
        let second_result = second_task.await.expect("second task");
        let Ok(output) = second_result else {
            panic!("second must finish normally")
        };
        assert!(output.success);
        assert_eq!(output.stdout, b"survived");
        host.wait(Duration::from_secs(3))
            .await
            .expect("both cleaned");
    }

    #[tokio::test]
    async fn cancellation_is_local_and_cleans_descendants_before_returning() {
        let directory = tempfile::tempdir().expect("workspace");
        let host = UnixValidationHost::default();
        let cancellation = CancellationToken::new();
        let invocation = Invocation {
            operation: host.owner.operation(),
            cancellation: cancellation.clone(),
        };
        let mut command = shell("touch ready; (sleep 1; touch escaped) & wait");
        command.current_dir(directory.path());
        let ready = directory.path().join("ready");
        let cancel = async {
            tokio::time::timeout(Duration::from_secs(3), async {
                while !ready.exists() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("child started");
            cancellation.cancel();
        };
        let (result, ()) =
            tokio::join!(invocation.run(command, Duration::from_secs(5), 128), cancel);
        assert!(matches!(result, Err(ValidationHostFailure::Failed(_))));
        assert!(invocation.is_cancelled());
        assert!(!host.invocation().is_cancelled());
        host.wait(Duration::from_secs(3))
            .await
            .expect("aggregate cleanup");
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(!directory.path().join("escaped").exists());
    }
}
