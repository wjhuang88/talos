//! Validation plan and allowlisted execution evidence reporting.

use std::fmt;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use serde_json::json;

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

impl fmt::Display for ValidationProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Governance => "governance",
            Self::I076 => "i076",
            Self::Workspace => "workspace",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationPlan {
    profile: ValidationProfile,
    checks: Vec<ValidationCheck>,
    findings: Vec<ValidationFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationCheck {
    id: &'static str,
    command: &'static str,
    program: &'static str,
    args: &'static [&'static str],
    required: bool,
    source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationFinding {
    severity: FindingSeverity,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FindingSeverity {
    Ok,
    Blocked,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationEvidence {
    profile: ValidationProfile,
    authority: &'static str,
    findings: Vec<ValidationFinding>,
    records: Vec<ValidationEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationEvidenceRecord {
    id: &'static str,
    command: &'static str,
    required: bool,
    source: &'static str,
    permission_decision: String,
    status: EvidenceStatus,
    exit_status: Option<i32>,
    stdout_summary: String,
    stderr_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceStatus {
    Passed,
    Failed,
    NotStarted,
}

impl fmt::Display for EvidenceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotStarted => "not_started",
        })
    }
}

impl fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ok => "ok",
            Self::Blocked => "blocked",
            Self::Info => "info",
        })
    }
}

pub(crate) fn run_validate_command(command: ValidateCommand) -> Result<()> {
    match command {
        ValidateCommand::Plan { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = collect_validation_plan(&workspace, profile);
            if json {
                println!("{}", render_json_plan(&plan));
            } else {
                print!("{}", render_text_plan(&plan));
            }
        }
        ValidateCommand::Run { profile, json } => {
            let workspace = std::env::current_dir()?;
            let plan = collect_validation_plan(&workspace, profile);
            let evidence = run_validation_plan(&workspace, plan);
            if json {
                println!("{}", render_json_evidence(&evidence));
            } else {
                print!("{}", render_text_evidence(&evidence));
            }
        }
    }

    Ok(())
}

fn collect_validation_plan(workspace: &Path, profile: ValidationProfile) -> ValidationPlan {
    let mut plan = ValidationPlan {
        profile,
        checks: checks_for_profile(profile),
        findings: Vec::new(),
    };

    let cargo_manifest = workspace.join("Cargo.toml");
    if cargo_manifest.is_file() {
        plan.findings.push(ValidationFinding {
            severity: FindingSeverity::Ok,
            message: "Cargo workspace manifest found".to_string(),
        });
    } else {
        plan.findings.push(ValidationFinding {
            severity: FindingSeverity::Blocked,
            message: "Cargo workspace manifest missing: Cargo.toml".to_string(),
        });
    }

    if requires_governance_script(profile) {
        let script = workspace
            .join("scripts")
            .join("validate_project_governance.sh");
        if script.is_file() {
            plan.findings.push(ValidationFinding {
                severity: FindingSeverity::Ok,
                message: "Governance validator found".to_string(),
            });
        } else {
            plan.findings.push(ValidationFinding {
                severity: FindingSeverity::Blocked,
                message: "Governance validator missing: scripts/validate_project_governance.sh"
                    .to_string(),
            });
        }
    }

    plan.findings.push(ValidationFinding {
        severity: FindingSeverity::Info,
        message: "Plan mode is read-only: commands are listed but not executed".to_string(),
    });

    plan
}

fn checks_for_profile(profile: ValidationProfile) -> Vec<ValidationCheck> {
    match profile {
        ValidationProfile::Governance => vec![governance_check()],
        ValidationProfile::I076 => vec![
            ValidationCheck {
                id: "fmt",
                command: "cargo fmt --all -- --check",
                program: "cargo",
                args: &["fmt", "--all", "--", "--check"],
                required: true,
                source: "I076 planned validation",
            },
            ValidationCheck {
                id: "provider-usage",
                command: "cargo test -p talos-provider",
                program: "cargo",
                args: &["test", "-p", "talos-provider"],
                required: true,
                source: "T101",
            },
            ValidationCheck {
                id: "tui-status",
                command: "cargo test -p talos-tui status_bar",
                program: "cargo",
                args: &["test", "-p", "talos-tui", "status_bar"],
                required: true,
                source: "T102/T103",
            },
            ValidationCheck {
                id: "tool-results",
                command: "cargo test -p talos-tools file_tool_tests",
                program: "cargo",
                args: &["test", "-p", "talos-tools", "file_tool_tests"],
                required: true,
                source: "T104",
            },
            ValidationCheck {
                id: "model-switch",
                command: "cargo test -p talos-cli model_switch_marker",
                program: "cargo",
                args: &["test", "-p", "talos-cli", "model_switch_marker"],
                required: true,
                source: "T106",
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                program: "cargo",
                args: &["check", "--workspace"],
                required: true,
                source: "I076 planned validation",
            },
            governance_check(),
        ],
        ValidationProfile::Workspace => vec![
            ValidationCheck {
                id: "fmt",
                command: "cargo fmt --all -- --check",
                program: "cargo",
                args: &["fmt", "--all", "--", "--check"],
                required: true,
                source: "workspace validation",
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                program: "cargo",
                args: &["check", "--workspace"],
                required: true,
                source: "workspace validation",
            },
            ValidationCheck {
                id: "test",
                command: "cargo test --workspace",
                program: "cargo",
                args: &["test", "--workspace"],
                required: true,
                source: "workspace validation",
            },
            governance_check(),
        ],
    }
}

fn governance_check() -> ValidationCheck {
    ValidationCheck {
        id: "governance",
        command: "scripts/validate_project_governance.sh .",
        program: "scripts/validate_project_governance.sh",
        args: &["."],
        required: true,
        source: "governance validation",
    }
}

fn requires_governance_script(profile: ValidationProfile) -> bool {
    matches!(
        profile,
        ValidationProfile::Governance | ValidationProfile::I076 | ValidationProfile::Workspace
    )
}

fn render_text_plan(plan: &ValidationPlan) -> String {
    let mut out = String::new();
    out.push_str("Talos Validation Plan\n");
    out.push_str("=====================\n\n");
    out.push_str(&format!("Profile: {}\n", plan.profile));
    out.push_str("Authority: read-only plan; commands are not executed\n\n");
    out.push_str("Checks\n");
    out.push_str("------\n");
    for check in &plan.checks {
        let required = if check.required {
            "required"
        } else {
            "optional"
        };
        out.push_str(&format!(
            "- [{}] {} ({}) - {}\n",
            required, check.command, check.id, check.source
        ));
    }
    out.push('\n');
    out.push_str("Findings\n");
    out.push_str("--------\n");
    for finding in &plan.findings {
        out.push_str(&format!("- [{}] {}\n", finding.severity, finding.message));
    }
    out
}

fn render_json_plan(plan: &ValidationPlan) -> String {
    let checks: Vec<_> = plan
        .checks
        .iter()
        .map(|check| {
            json!({
                "id": check.id,
                "command": check.command,
                "required": check.required,
                "source": check.source,
            })
        })
        .collect();
    let findings: Vec<_> = plan
        .findings
        .iter()
        .map(|finding| {
            json!({
                "severity": finding.severity.to_string(),
                "message": finding.message,
            })
        })
        .collect();

    json!({
        "profile": plan.profile.to_string(),
        "authority": "read-only plan; commands are not executed",
        "checks": checks,
        "findings": findings,
    })
    .to_string()
}

fn run_validation_plan(workspace: &Path, plan: ValidationPlan) -> ValidationEvidence {
    let authority = "allowlisted validation execution; no arbitrary commands accepted";
    let mut findings = plan.findings;
    findings.push(ValidationFinding {
        severity: FindingSeverity::Info,
        message: "Run mode executes only the selected profile's allowlisted commands".to_string(),
    });
    let records = plan
        .checks
        .iter()
        .map(|check| run_validation_check(workspace, plan.profile, check))
        .collect();

    ValidationEvidence {
        profile: plan.profile,
        authority,
        findings: findings
            .into_iter()
            .filter(|finding| {
                finding.message != "Plan mode is read-only: commands are listed but not executed"
            })
            .collect(),
        records,
    }
}

fn run_validation_check(
    workspace: &Path,
    profile: ValidationProfile,
    check: &ValidationCheck,
) -> ValidationEvidenceRecord {
    let permission_decision = format!("allowlisted validation profile: {profile}");
    match Command::new(check.program)
        .args(check.args)
        .current_dir(workspace)
        .output()
    {
        Ok(output) => {
            let status = if output.status.success() {
                EvidenceStatus::Passed
            } else {
                EvidenceStatus::Failed
            };
            ValidationEvidenceRecord {
                id: check.id,
                command: check.command,
                required: check.required,
                source: check.source,
                permission_decision,
                status,
                exit_status: output.status.code(),
                stdout_summary: summarize_output(&output.stdout),
                stderr_summary: summarize_output(&output.stderr),
            }
        }
        Err(err) => ValidationEvidenceRecord {
            id: check.id,
            command: check.command,
            required: check.required,
            source: check.source,
            permission_decision,
            status: EvidenceStatus::NotStarted,
            exit_status: None,
            stdout_summary: "<empty>".to_string(),
            stderr_summary: err.to_string(),
        },
    }
}

fn summarize_output(output: &[u8]) -> String {
    let text = String::from_utf8_lossy(output);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<empty>".to_string();
    }

    const MAX_CHARS: usize = 4000;
    let mut summary: String = trimmed.chars().take(MAX_CHARS).collect();
    if trimmed.chars().count() > MAX_CHARS {
        summary.push_str("\n[truncated]");
    }
    summary
}

fn render_text_evidence(evidence: &ValidationEvidence) -> String {
    let mut out = String::new();
    out.push_str("Talos Validation Evidence\n");
    out.push_str("=========================\n\n");
    out.push_str(&format!("Profile: {}\n", evidence.profile));
    out.push_str(&format!("Authority: {}\n\n", evidence.authority));
    out.push_str("Findings\n");
    out.push_str("--------\n");
    for finding in &evidence.findings {
        out.push_str(&format!("- [{}] {}\n", finding.severity, finding.message));
    }
    out.push('\n');
    out.push_str("Records\n");
    out.push_str("-------\n");
    for record in &evidence.records {
        let exit_status = record
            .exit_status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "none".to_string());
        out.push_str(&format!(
            "- [{}] {} ({})\n",
            record.status, record.command, record.id
        ));
        out.push_str(&format!("  required: {}\n", record.required));
        out.push_str(&format!("  source: {}\n", record.source));
        out.push_str(&format!(
            "  permission_decision: {}\n",
            record.permission_decision
        ));
        out.push_str(&format!("  exit_status: {exit_status}\n"));
        out.push_str(&format!("  stdout_summary: {}\n", record.stdout_summary));
        out.push_str(&format!("  stderr_summary: {}\n", record.stderr_summary));
    }
    out
}

fn render_json_evidence(evidence: &ValidationEvidence) -> String {
    let findings: Vec<_> = evidence
        .findings
        .iter()
        .map(|finding| {
            json!({
                "severity": finding.severity.to_string(),
                "message": finding.message,
            })
        })
        .collect();
    let records: Vec<_> = evidence
        .records
        .iter()
        .map(|record| {
            json!({
                "id": record.id,
                "command": record.command,
                "required": record.required,
                "source": record.source,
                "permission_decision": record.permission_decision,
                "status": record.status.to_string(),
                "exit_status": record.exit_status,
                "stdout_summary": record.stdout_summary,
                "stderr_summary": record.stderr_summary,
            })
        })
        .collect();

    json!({
        "profile": evidence.profile.to_string(),
        "authority": evidence.authority,
        "findings": findings,
        "records": records,
    })
    .to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn governance_plan_lists_command_without_executing_script() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let script_dir = dir.path().join("scripts");
        fs::create_dir_all(&script_dir).unwrap();
        fs::write(
            script_dir.join("validate_project_governance.sh"),
            "#!/usr/bin/env bash\ntouch executed-marker\n",
        )
        .unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::Governance);
        let rendered = render_text_plan(&plan);

        assert!(rendered.contains("scripts/validate_project_governance.sh ."));
        assert!(rendered.contains("commands are not executed"));
        assert!(!dir.path().join("executed-marker").exists());
    }

    #[test]
    fn missing_governance_script_is_blocked_not_hidden() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::Workspace);

        assert!(plan.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Blocked
                && finding.message.contains("Governance validator missing")
        }));
    }

    #[test]
    fn i076_profile_includes_targeted_checks() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let script_dir = dir.path().join("scripts");
        fs::create_dir_all(&script_dir).unwrap();
        fs::write(script_dir.join("validate_project_governance.sh"), "").unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::I076);

        assert!(plan.checks.iter().any(|check| check.id == "provider-usage"));
        assert!(plan.checks.iter().any(|check| check.id == "model-switch"));
        assert_eq!(plan.checks.len(), 7);
    }

    #[test]
    fn json_plan_is_structured() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let plan = collect_validation_plan(dir.path(), ValidationProfile::Workspace);

        let value: serde_json::Value = serde_json::from_str(&render_json_plan(&plan)).unwrap();

        assert_eq!(value["profile"], "workspace");
        assert_eq!(
            value["authority"],
            "read-only plan; commands are not executed"
        );
        assert!(value["checks"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn run_records_missing_program_without_hiding_failure() {
        let dir = tempdir().unwrap();
        let check = ValidationCheck {
            id: "missing",
            command: "talos-validation-command-that-should-not-exist",
            program: "talos-validation-command-that-should-not-exist",
            args: &[],
            required: true,
            source: "test",
        };

        let record = run_validation_check(dir.path(), ValidationProfile::Workspace, &check);

        assert_eq!(record.status, EvidenceStatus::NotStarted);
        assert_eq!(record.exit_status, None);
        assert!(
            record
                .permission_decision
                .contains("allowlisted validation profile: workspace")
        );
        assert_ne!(record.stderr_summary, "<empty>");
    }

    #[test]
    fn json_evidence_is_structured() {
        let evidence = ValidationEvidence {
            profile: ValidationProfile::Governance,
            authority: "allowlisted validation execution; no arbitrary commands accepted",
            findings: vec![ValidationFinding {
                severity: FindingSeverity::Ok,
                message: "ready".to_string(),
            }],
            records: vec![ValidationEvidenceRecord {
                id: "governance",
                command: "scripts/validate_project_governance.sh .",
                required: true,
                source: "governance validation",
                permission_decision: "allowlisted validation profile: governance".to_string(),
                status: EvidenceStatus::Passed,
                exit_status: Some(0),
                stdout_summary: "Governance validation passed: 0 warning(s).".to_string(),
                stderr_summary: "<empty>".to_string(),
            }],
        };

        let value: serde_json::Value =
            serde_json::from_str(&render_json_evidence(&evidence)).unwrap();

        assert_eq!(value["profile"], "governance");
        assert_eq!(value["records"][0]["status"], "passed");
        assert_eq!(value["records"][0]["exit_status"], 0);
        assert_eq!(
            value["records"][0]["permission_decision"],
            "allowlisted validation profile: governance"
        );
    }

    #[test]
    fn output_summary_is_bounded() {
        let long = "x".repeat(5000);
        let summary = summarize_output(long.as_bytes());

        assert!(summary.ends_with("[truncated]"));
        assert!(summary.chars().count() < long.chars().count());
    }
}
