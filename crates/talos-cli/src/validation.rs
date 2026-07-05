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
    project_types: Vec<ProjectType>,
    checks: Vec<ValidationCheck>,
    findings: Vec<ValidationFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidationCheck {
    id: &'static str,
    command: &'static str,
    execution_mode: ValidationExecutionMode,
    program: Option<&'static str>,
    args: &'static [&'static str],
    required: bool,
    source: &'static str,
    ecosystem: Option<&'static str>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProjectType {
    TalosGovernance,
    Rust,
    Node,
    Python,
    Go,
    Java,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationExecutionMode {
    Internal,
    HostTool,
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
    execution_mode: ValidationExecutionMode,
    required: bool,
    source: &'static str,
    ecosystem: Option<&'static str>,
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

impl fmt::Display for ValidationExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Internal => "internal",
            Self::HostTool => "host_tool",
        })
    }
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TalosGovernance => "talos_governance",
            Self::Rust => "rust",
            Self::Node => "node",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
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
    let project_types = detect_project_types(workspace);
    let mut plan = ValidationPlan {
        profile,
        project_types,
        checks: checks_for_profile(profile),
        findings: Vec::new(),
    };

    if plan.project_types.is_empty() {
        plan.findings.push(ValidationFinding {
            severity: FindingSeverity::Info,
            message: "No common project type detected".to_string(),
        });
    } else {
        let detected = plan
            .project_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        plan.findings.push(ValidationFinding {
            severity: FindingSeverity::Ok,
            message: format!("Project type(s) detected: {detected}"),
        });
    }

    if requires_cargo_manifest(&plan.checks) {
        if plan.project_types.contains(&ProjectType::Rust) {
            plan.findings.push(ValidationFinding {
                severity: FindingSeverity::Ok,
                message: "Rust workspace manifest found for Cargo host-tool adapter".to_string(),
            });
        } else {
            plan.findings.push(ValidationFinding {
                severity: FindingSeverity::Blocked,
                message: "Cargo host-tool adapter unavailable: Cargo.toml missing".to_string(),
            });
        }
    }

    plan.findings.push(ValidationFinding {
        severity: FindingSeverity::Ok,
        message: if plan.project_types.contains(&ProjectType::TalosGovernance) {
            "Talos governance project detected; internal governance validator available".to_string()
        } else {
            "Internal governance validator available; Talos governance project not detected"
                .to_string()
        },
    });
    plan.findings.push(ValidationFinding {
        severity: FindingSeverity::Info,
        message: "Plan mode is read-only: commands are listed but not executed".to_string(),
    });

    plan
}

fn detect_project_types(workspace: &Path) -> Vec<ProjectType> {
    project_type_detectors()
        .iter()
        .filter_map(|detector| detector.detect(workspace))
        .collect()
}

trait ProjectTypeDetector {
    fn detect(&self, workspace: &Path) -> Option<ProjectType>;
}

struct MarkerDetector {
    project_type: ProjectType,
    markers: &'static [&'static str],
}

impl ProjectTypeDetector for MarkerDetector {
    fn detect(&self, workspace: &Path) -> Option<ProjectType> {
        self.markers
            .iter()
            .any(|marker| workspace.join(marker).exists())
            .then_some(self.project_type)
    }
}

const TALOS_GOVERNANCE_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::TalosGovernance,
    markers: &[
        ".agent-governance/manifest.yaml",
        "docs/sop",
        "docs/BOARD.md",
    ],
};
const RUST_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::Rust,
    markers: &["Cargo.toml"],
};
const NODE_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::Node,
    markers: &["package.json"],
};
const PYTHON_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::Python,
    markers: &["pyproject.toml", "requirements.txt", "setup.py"],
};
const GO_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::Go,
    markers: &["go.mod"],
};
const JAVA_DETECTOR: MarkerDetector = MarkerDetector {
    project_type: ProjectType::Java,
    markers: &["pom.xml", "build.gradle", "settings.gradle"],
};

fn project_type_detectors() -> &'static [&'static dyn ProjectTypeDetector] {
    &[
        &TALOS_GOVERNANCE_DETECTOR,
        &RUST_DETECTOR,
        &NODE_DETECTOR,
        &PYTHON_DETECTOR,
        &GO_DETECTOR,
        &JAVA_DETECTOR,
    ]
}

fn checks_for_profile(profile: ValidationProfile) -> Vec<ValidationCheck> {
    match profile {
        ValidationProfile::Governance => vec![governance_check()],
        ValidationProfile::I076 => vec![
            ValidationCheck {
                id: "fmt",
                command: "cargo fmt --all -- --check",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["fmt", "--all", "--", "--check"],
                required: true,
                source: "I076 planned validation",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "provider-usage",
                command: "cargo test -p talos-provider",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["test", "-p", "talos-provider"],
                required: true,
                source: "T101",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "tui-status",
                command: "cargo test -p talos-tui status_bar",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["test", "-p", "talos-tui", "status_bar"],
                required: true,
                source: "T102/T103",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "tool-results",
                command: "cargo test -p talos-tools file_tool_tests",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["test", "-p", "talos-tools", "file_tool_tests"],
                required: true,
                source: "T104",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "model-switch",
                command: "cargo test -p talos-cli model_switch_marker",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["test", "-p", "talos-cli", "model_switch_marker"],
                required: true,
                source: "T106",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["check", "--workspace"],
                required: true,
                source: "I076 planned validation",
                ecosystem: Some("rust"),
            },
            governance_check(),
        ],
        ValidationProfile::Workspace => vec![
            ValidationCheck {
                id: "fmt",
                command: "cargo fmt --all -- --check",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["fmt", "--all", "--", "--check"],
                required: true,
                source: "workspace validation",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["check", "--workspace"],
                required: true,
                source: "workspace validation",
                ecosystem: Some("rust"),
            },
            ValidationCheck {
                id: "test",
                command: "cargo test --workspace",
                execution_mode: ValidationExecutionMode::HostTool,
                program: Some("cargo"),
                args: &["test", "--workspace"],
                required: true,
                source: "workspace validation",
                ecosystem: Some("rust"),
            },
            governance_check(),
        ],
    }
}

fn governance_check() -> ValidationCheck {
    ValidationCheck {
        id: "governance",
        command: "internal:governance_validation",
        execution_mode: ValidationExecutionMode::Internal,
        program: None,
        args: &[],
        required: true,
        source: "governance validation",
        ecosystem: None,
    }
}

fn requires_cargo_manifest(checks: &[ValidationCheck]) -> bool {
    checks.iter().any(|check| {
        check.ecosystem == Some("rust") && check.execution_mode == ValidationExecutionMode::HostTool
    })
}

fn render_text_plan(plan: &ValidationPlan) -> String {
    let mut out = String::new();
    out.push_str("Talos Validation Plan\n");
    out.push_str("=====================\n\n");
    out.push_str(&format!("Profile: {}\n", plan.profile));
    let project_types = if plan.project_types.is_empty() {
        "none".to_string()
    } else {
        plan.project_types
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!("Project types: {project_types}\n"));
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
                "execution_mode": check.execution_mode.to_string(),
                "required": check.required,
                "source": check.source,
                "ecosystem": check.ecosystem,
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
        "project_types": plan.project_types.iter().map(ToString::to_string).collect::<Vec<_>>(),
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
    if check.execution_mode == ValidationExecutionMode::Internal {
        return run_internal_validation_check(workspace, check, permission_decision);
    }

    let Some(program) = check.program else {
        return ValidationEvidenceRecord {
            id: check.id,
            command: check.command,
            execution_mode: check.execution_mode,
            required: check.required,
            source: check.source,
            ecosystem: check.ecosystem,
            permission_decision,
            status: EvidenceStatus::NotStarted,
            exit_status: None,
            stdout_summary: "<empty>".to_string(),
            stderr_summary: "host-tool validation check missing program".to_string(),
        };
    };

    match Command::new(program)
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
                execution_mode: check.execution_mode,
                required: check.required,
                source: check.source,
                ecosystem: check.ecosystem,
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
            execution_mode: check.execution_mode,
            required: check.required,
            source: check.source,
            ecosystem: check.ecosystem,
            permission_decision,
            status: EvidenceStatus::NotStarted,
            exit_status: None,
            stdout_summary: "<empty>".to_string(),
            stderr_summary: err.to_string(),
        },
    }
}

fn run_internal_validation_check(
    workspace: &Path,
    check: &ValidationCheck,
    permission_decision: String,
) -> ValidationEvidenceRecord {
    match check.id {
        "governance" => {
            let report = talos_conversation::collect_governance_validation(workspace);
            let status = if report.errors == 0 {
                EvidenceStatus::Passed
            } else {
                EvidenceStatus::Failed
            };
            let stdout_summary = if report.errors == 0 {
                format!(
                    "Governance validation passed: {} warning(s).",
                    report.warnings
                )
            } else {
                format!(
                    "Governance validation failed: {} error(s), {} warning(s).\n{}",
                    report.errors,
                    report.warnings,
                    report.findings.join("\n")
                )
            };
            ValidationEvidenceRecord {
                id: check.id,
                command: check.command,
                execution_mode: check.execution_mode,
                required: check.required,
                source: check.source,
                ecosystem: check.ecosystem,
                permission_decision,
                status,
                exit_status: Some(if report.errors == 0 { 0 } else { 1 }),
                stdout_summary,
                stderr_summary: "<empty>".to_string(),
            }
        }
        _ => ValidationEvidenceRecord {
            id: check.id,
            command: check.command,
            execution_mode: check.execution_mode,
            required: check.required,
            source: check.source,
            ecosystem: check.ecosystem,
            permission_decision,
            status: EvidenceStatus::NotStarted,
            exit_status: None,
            stdout_summary: "<empty>".to_string(),
            stderr_summary: format!("unknown internal validation check: {}", check.id),
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
                "execution_mode": record.execution_mode.to_string(),
                "required": record.required,
                "source": record.source,
                "ecosystem": record.ecosystem,
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
    fn governance_plan_lists_internal_check_without_executing_script() {
        let dir = tempdir().unwrap();
        let script_dir = dir.path().join("scripts");
        fs::create_dir_all(&script_dir).unwrap();
        fs::write(
            script_dir.join("validate_project_governance.sh"),
            "#!/usr/bin/env bash\ntouch executed-marker\n",
        )
        .unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::Governance);
        let rendered = render_text_plan(&plan);

        assert!(rendered.contains("internal:governance_validation"));
        assert!(rendered.contains("internal"));
        assert!(rendered.contains("commands are not executed"));
        assert!(!dir.path().join("executed-marker").exists());
    }

    #[test]
    fn governance_profile_does_not_require_host_script() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agent-governance")).unwrap();
        fs::write(
            dir.path().join(".agent-governance").join("manifest.yaml"),
            "profile: product\n",
        )
        .unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::Governance);

        assert!(plan.project_types.contains(&ProjectType::TalosGovernance));
        assert!(!plan.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Blocked
                && finding.message.contains("Governance validator")
        }));
        assert!(plan.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Ok
                && finding
                    .message
                    .contains("internal governance validator available")
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
        assert_eq!(value["project_types"][0], "rust");
        assert_eq!(
            value["authority"],
            "read-only plan; commands are not executed"
        );
        assert_eq!(value["checks"][0]["execution_mode"], "host_tool");
        assert!(value["checks"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn project_type_detection_covers_common_manifests() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(dir.path().join("package.json"), "{}\n").unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
        fs::write(dir.path().join("go.mod"), "module example.com/test\n").unwrap();
        fs::write(dir.path().join("pom.xml"), "<project />\n").unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs").join("BOARD.md"), "# Board\n").unwrap();

        let detected = detect_project_types(dir.path());

        assert!(detected.contains(&ProjectType::TalosGovernance));
        assert!(detected.contains(&ProjectType::Rust));
        assert!(detected.contains(&ProjectType::Node));
        assert!(detected.contains(&ProjectType::Python));
        assert!(detected.contains(&ProjectType::Go));
        assert!(detected.contains(&ProjectType::Java));
    }

    #[test]
    fn cargo_adapter_requires_rust_project_detection() {
        let dir = tempdir().unwrap();

        let plan = collect_validation_plan(dir.path(), ValidationProfile::Workspace);

        assert!(!plan.project_types.contains(&ProjectType::Rust));
        assert!(plan.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Blocked
                && finding
                    .message
                    .contains("Cargo host-tool adapter unavailable")
        }));
    }

    #[test]
    fn run_records_missing_program_without_hiding_failure() {
        let dir = tempdir().unwrap();
        let check = ValidationCheck {
            id: "missing",
            command: "talos-validation-command-that-should-not-exist",
            execution_mode: ValidationExecutionMode::HostTool,
            program: Some("talos-validation-command-that-should-not-exist"),
            args: &[],
            required: true,
            source: "test",
            ecosystem: None,
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
                command: "internal:governance_validation",
                execution_mode: ValidationExecutionMode::Internal,
                required: true,
                source: "governance validation",
                ecosystem: None,
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
        assert_eq!(value["records"][0]["execution_mode"], "internal");
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
