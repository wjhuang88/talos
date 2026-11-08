//! Presentation-only projection; the original tool evidence remains untouched.

use serde_json::Value;
use talos_conversation::ToolResultDisplay;

struct ResultBody {
    profile: String,
    authority: String,
    findings: Vec<Finding>,
    checks: Option<Vec<Check>>,
    records: Option<Vec<Record>>,
}

struct Finding {
    severity: String,
    message: String,
}

struct Check {
    id: String,
    command: String,
    required: bool,
}

struct Record {
    id: String,
    command: String,
    required: bool,
    status: String,
    permission_decision: String,
    exit_status: Option<i32>,
    stdout_summary: String,
    stderr_summary: String,
}

fn string(value: &Value, key: &str) -> Option<String> {
    Some(value.get(key)?.as_str()?.to_owned())
}

fn parse(value: &Value) -> Option<ResultBody> {
    let findings = value
        .get("findings")?
        .as_array()?
        .iter()
        .map(|finding| {
            Some(Finding {
                severity: string(finding, "severity")?,
                message: string(finding, "message")?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let checks = if let Some(checks) = value.get("checks") {
        Some(
            checks
                .as_array()?
                .iter()
                .map(|check| {
                    Some(Check {
                        id: string(check, "id")?,
                        command: string(check, "command")?,
                        required: check.get("required")?.as_bool()?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        )
    } else {
        None
    };
    let records = if let Some(records) = value.get("records") {
        Some(
            records
                .as_array()?
                .iter()
                .map(|record| {
                    Some(Record {
                        id: string(record, "id")?,
                        command: string(record, "command")?,
                        required: record.get("required")?.as_bool()?,
                        status: string(record, "status")?,
                        permission_decision: string(record, "permission_decision")?,
                        exit_status: match record.get("exit_status")? {
                            Value::Null => None,
                            value => Some(i32::try_from(value.as_i64()?).ok()?),
                        },
                        stdout_summary: string(record, "stdout_summary")?,
                        stderr_summary: string(record, "stderr_summary")?,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        )
    } else {
        None
    };
    Some(ResultBody {
        profile: string(value, "profile")?,
        authority: string(value, "authority")?,
        findings,
        checks,
        records,
    })
}

fn requirement(required: bool) -> &'static str {
    if required { "required" } else { "optional" }
}

pub(crate) fn project(display: &ToolResultDisplay) -> Option<String> {
    if display.tool_name.as_deref() != Some("validate_project") {
        return None;
    }
    let value: Value = serde_json::from_str(&display.content).ok()?;
    let body = parse(&value)?;
    let mut lines = Vec::new();
    match (&body.checks, &body.records) {
        (Some(checks), None) => {
            lines.push(format!(
                "Validation plan · {} · {} checks · NOT EXECUTED",
                body.profile,
                checks.len()
            ));
            lines.push(format!("Authority: {}", body.authority));
            append_findings(&mut lines, &body.findings);
            for check in checks {
                lines.push(format!(
                    "[planned] {} ({}) · {}",
                    check.id,
                    requirement(check.required),
                    check.command
                ));
            }
        }
        (None, Some(records)) => {
            let passed = records
                .iter()
                .filter(|record| record.status == "passed")
                .count();
            lines.push(format!(
                "Validation results · {} · {passed}/{} passed{}",
                body.profile,
                records.len(),
                if display.is_error { " · ERROR" } else { "" }
            ));
            let attention: Vec<_> = records
                .iter()
                .filter(|record| record.status != "passed")
                .map(|record| {
                    format!(
                        "{}: {} ({})",
                        record.id,
                        record.status,
                        requirement(record.required)
                    )
                })
                .collect();
            if !attention.is_empty() {
                lines.push(format!("Attention: {}", attention.join(", ")));
            }
            lines.push(format!("Authority: {}", body.authority));
            append_findings(&mut lines, &body.findings);
            for record in records {
                lines.push(format!(
                    "[{}] {} ({}) · {}",
                    record.status,
                    record.id,
                    requirement(record.required),
                    record.command
                ));
                let exit = record
                    .exit_status
                    .map_or_else(|| "none".into(), |exit| exit.to_string());
                lines.push(format!(
                    "  permission: {} · exit: {exit}",
                    record.permission_decision
                ));
                for (label, output) in [
                    ("stdout", &record.stdout_summary),
                    ("stderr", &record.stderr_summary),
                ] {
                    if !output.trim().is_empty() && output.trim() != "<empty>" {
                        lines.push(format!("  {label}:"));
                        lines.extend(output.lines().map(|line| format!("    {line}")));
                    }
                }
            }
        }
        _ => return None,
    }
    Some(lines.join("\n"))
}

fn append_findings(lines: &mut Vec<String>, findings: &[Finding]) {
    for finding in findings {
        lines.push(format!("[{}] {}", finding.severity, finding.message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn display(content: serde_json::Value, is_error: bool) -> ToolResultDisplay {
        ToolResultDisplay {
            tool_name: Some("validate_project".into()),
            content: content.to_string(),
            is_error,
            todo_presentation: None,
        }
    }

    #[test]
    fn plan_is_readable_and_never_claims_execution() {
        let result = display(
            json!({
                "profile":"workspace", "authority":"read-only plan; commands are not executed",
                "findings":[{"severity":"warning", "message":"需要原生 Windows 验证"}],
                "checks":[{"id":"fmt", "command":"cargo fmt --all -- --check", "required":true},
                    {"id":"optional", "command":"internal:governance.validation", "required":false}]
            }),
            false,
        );
        let raw = result.content.clone();
        let text = project(&result).expect("plan");
        assert!(text.contains("NOT EXECUTED"));
        assert!(text.contains("[planned] fmt (required) · cargo fmt --all -- --check"));
        assert!(text.contains("[warning] 需要原生 Windows 验证"));
        assert!(text.contains("optional (optional)"));
        let live = crate::tool_display::build_tool_result_scrollback_lines(&result, "✓", None, 100);
        let replay = crate::tool_display::build_tool_result_scrollback_lines(
            &result.clone(),
            "✓",
            None,
            100,
        );
        assert_eq!(format!("{live:?}"), format!("{replay:?}"));
        assert!(format!("{live:?}").contains("NOT EXECUTED"));
        assert_eq!(result.content, raw, "export/model evidence stays raw");
    }

    #[test]
    fn failed_denied_and_skipped_checks_remain_distinct() {
        let records: Vec<_> = ["passed", "failed", "denied", "skipped"].into_iter().map(|status| json!({
            "id":status, "command":"cargo test --locked", "required":true,
            "status":status, "permission_decision": if status == "denied" { "denied" } else { "allowed" },
            "exit_status": if status == "failed" { Some(1) } else { None },
            "stdout_summary":"", "stderr_summary":if status == "failed" { "first error\nsecond error" } else { "" }
        })).collect();
        let result = display(
            json!({"profile":"workspace", "authority":"permission gated run", "findings":[], "records":records}),
            true,
        );
        let text = project(&result).expect("evidence");
        assert!(text.starts_with("Validation results · workspace · 1/4 passed · ERROR"));
        assert!(text.contains(
            "failed: failed (required), denied: denied (required), skipped: skipped (required)"
        ));
        assert!(text.contains("permission: denied · exit: none"));
        assert!(text.contains("    first error\n    second error"));
    }

    #[test]
    fn malformed_ambiguous_and_unrelated_results_use_raw_fallback() {
        for value in [
            json!({}),
            json!({"profile":"workspace", "authority":"plan", "findings":[], "checks":[], "records":[]}),
            json!({"profile":"workspace", "authority":"plan", "findings":[], "checks":[{}]}),
        ] {
            assert!(project(&display(value, false)).is_none());
        }
        let mut result = display(
            json!({"profile":"workspace", "authority":"read-only", "findings":[], "checks":[]}),
            false,
        );
        result.tool_name = Some("other_tool".into());
        assert!(project(&result).is_none());
        result.tool_name = Some("validate_project".into());
        result.content = "permission denied".into();
        assert!(project(&result).is_none());
    }
}
