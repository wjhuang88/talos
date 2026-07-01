//! Read-only validation plan reporting.

use std::fmt;
use std::path::Path;

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
        message: "Read-only plan only: commands are listed but not executed".to_string(),
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
                required: true,
                source: "I076 planned validation",
            },
            ValidationCheck {
                id: "provider-usage",
                command: "cargo test -p talos-provider",
                required: true,
                source: "T101",
            },
            ValidationCheck {
                id: "tui-status",
                command: "cargo test -p talos-tui status_bar",
                required: true,
                source: "T102/T103",
            },
            ValidationCheck {
                id: "tool-results",
                command: "cargo test -p talos-tools file_tool_tests",
                required: true,
                source: "T104",
            },
            ValidationCheck {
                id: "model-switch",
                command: "cargo test -p talos-cli model_switch_marker",
                required: true,
                source: "T106",
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                required: true,
                source: "I076 planned validation",
            },
            governance_check(),
        ],
        ValidationProfile::Workspace => vec![
            ValidationCheck {
                id: "fmt",
                command: "cargo fmt --all -- --check",
                required: true,
                source: "workspace validation",
            },
            ValidationCheck {
                id: "check",
                command: "cargo check --workspace",
                required: true,
                source: "workspace validation",
            },
            ValidationCheck {
                id: "test",
                command: "cargo test --workspace",
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
}
