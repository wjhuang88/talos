//! CLI frontend for the shared validation service.

use anyhow::Result;
use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Clone)]
pub(crate) enum ValidateCommand {
    /// Print a validation plan without executing commands.
    Plan {
        /// Validation profile to plan.
        #[arg(long, value_enum, default_value_t = ValidationProfile::Workspace)]
        profile: ValidationProfile,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Execute an allowlisted validation profile and print durable evidence.
    Run {
        /// Validation profile to execute.
        #[arg(long, value_enum, default_value_t = ValidationProfile::Workspace)]
        profile: ValidationProfile,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum ValidationProfile {
    Governance,
    I076,
    Workspace,
}

impl From<ValidationProfile> for talos_conversation::ValidationProfile {
    fn from(value: ValidationProfile) -> Self {
        match value {
            ValidationProfile::Governance => Self::Governance,
            ValidationProfile::I076 => Self::I076,
            ValidationProfile::Workspace => Self::Workspace,
        }
    }
}

pub(crate) async fn run_validate_command(command: ValidateCommand) -> Result<()> {
    match command {
        ValidateCommand::Plan { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = talos_conversation::collect_validation_plan(&workspace, profile.into());
            if json {
                println!("{}", talos_conversation::render_json_plan(&plan));
            } else {
                print!("{}", talos_conversation::render_text_plan(&plan));
            }
        }
        ValidateCommand::Run { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = talos_conversation::collect_validation_plan(&workspace, profile.into());
            #[cfg(any(unix, windows))]
            let evidence = run_owned_validation(&workspace, plan).await?;
            #[cfg(not(any(unix, windows)))]
            let evidence = {
                struct UnsupportedHost;
                impl talos_conversation::ValidationHostExecutor for UnsupportedHost {
                    fn is_cancelled(&self) -> bool {
                        false
                    }
                    fn execute<'a>(
                        &'a self,
                        _: &'a std::path::Path,
                        _: &'a talos_conversation::ValidationCheck,
                        _: std::time::Duration,
                        _: usize,
                    ) -> talos_conversation::ValidationHostFuture<'a> {
                        Box::pin(async {
                            Err(talos_conversation::ValidationHostFailure::NotStarted(
                                "host validation execution is unsupported on this platform".into(),
                            ))
                        })
                    }
                }
                talos_conversation::run_validation_plan_with_executor(
                    &workspace,
                    plan,
                    &UnsupportedHost,
                    "CLI request; no supported host executor",
                )
                .await
            };
            if json {
                println!("{}", talos_conversation::render_json_evidence(&evidence));
            } else {
                print!("{}", talos_conversation::render_text_evidence(&evidence));
            }
            if evidence
                .findings
                .iter()
                .any(|finding| finding.severity == talos_conversation::FindingSeverity::Blocked)
                || evidence.records.iter().any(|record| {
                    record.required && record.status != talos_conversation::EvidenceStatus::Passed
                })
            {
                anyhow::bail!("validation failed or required checks did not run");
            }
        }
    }

    Ok(())
}

#[cfg(any(unix, windows))]
async fn run_owned_validation(
    workspace: &std::path::Path,
    plan: talos_conversation::ValidationPlan,
) -> Result<talos_conversation::ValidationEvidence> {
    use std::time::Duration;
    use talos_core::tool::{ToolCleanupOwner, ToolCleanupReceipt};
    #[cfg(unix)]
    use talos_tools::project_management::UnixValidationHost as PlatformValidationHost;
    #[cfg(windows)]
    use talos_tools::project_management::WindowsValidationHost as PlatformValidationHost;
    use tokio_util::sync::CancellationToken;

    let host = PlatformValidationHost::default();
    let cancellation = CancellationToken::new();
    // Register before polling any execution future. Registration failure must
    // leave every check unstarted while retaining structured failure evidence.
    #[cfg(unix)]
    let mut interrupts = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt());
    #[cfg(windows)]
    let mut interrupts = tokio::signal::windows::ctrl_c();
    let mut signal_failure = interrupts.as_ref().err().map(ToString::to_string);
    if signal_failure.is_some() {
        cancellation.cancel();
    }
    let invocation = host.invocation_with_cancellation(cancellation.clone());
    let mut evidence = {
        let run = talos_conversation::run_validation_plan_with_executor(
            workspace,
            plan,
            invocation.as_ref(),
            "explicit local CLI validate run request; fixed-profile execution",
        );
        tokio::pin!(run);
        tokio::select! {
            result = &mut run => result,
            signal = async {
                match &mut interrupts {
                    Ok(stream) => stream.recv().await,
                    Err(_) => std::future::pending().await,
                }
            } => {
                cancellation.cancel();
                // Complete operation-local cleanup and mark later checks not
                // started; do not destroy the runtime with live owned work.
                let evidence = run.await;
                if signal.is_none() {
                    signal_failure = Some("interrupt stream closed".to_owned());
                }
                evidence
            }
        }
    };
    host.seal();
    if let Err(error) = host.wait(Duration::from_secs(3)).await {
        evidence
            .findings
            .push(talos_conversation::ValidationFinding {
                severity: talos_conversation::FindingSeverity::Blocked,
                message: format!("aggregate validation cleanup failed: {error}"),
            });
    }
    if let Some(error) = signal_failure {
        evidence
            .findings
            .push(talos_conversation::ValidationFinding {
                severity: talos_conversation::FindingSeverity::Blocked,
                message: format!("validation interrupt listener failed: {error}"),
            });
    }
    Ok(evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[tokio::test]
    async fn owned_cli_validation_reports_missing_cargo_without_host_execution() {
        let directory = tempfile::tempdir().expect("workspace");
        let plan = talos_conversation::collect_validation_plan(
            directory.path(),
            talos_conversation::ValidationProfile::Workspace,
        );
        let evidence = run_owned_validation(directory.path(), plan)
            .await
            .expect("bounded execution");
        let host_checks: Vec<_> = evidence
            .records
            .iter()
            .filter(|record| {
                record.execution_mode == talos_conversation::ValidationExecutionMode::HostTool
            })
            .collect();
        assert_eq!(host_checks.len(), 3);
        for record in host_checks {
            assert_eq!(
                record.status,
                talos_conversation::EvidenceStatus::NotStarted
            );
            assert!(record.stderr_summary.contains("Cargo.toml missing"));
            assert!(record.permission_decision.contains("explicit local CLI"));
        }
    }

    #[test]
    fn cli_profile_maps_to_shared_service_profile() {
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::Governance),
            talos_conversation::ValidationProfile::Governance
        );
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::I076),
            talos_conversation::ValidationProfile::I076
        );
        assert_eq!(
            talos_conversation::ValidationProfile::from(ValidationProfile::Workspace),
            talos_conversation::ValidationProfile::Workspace
        );
    }
}
