//! Permission-bounded governance mutation commands.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};

#[derive(Subcommand, Clone)]
pub(crate) enum GovernanceCommand {
    /// Preview or write a governance iteration execution record.
    IterationRecord {
        #[command(subcommand)]
        command: IterationRecordCommand,
    },
}

#[derive(Subcommand, Clone)]
pub(crate) enum IterationRecordCommand {
    /// Preview the owner-doc change without writing it.
    Preview(IterationRecordArgs),
    /// Write the owner-doc change after explicit preview confirmation.
    Write(IterationRecordWriteArgs),
}

#[derive(Args, Clone)]
pub(crate) struct IterationRecordArgs {
    /// Iteration ID, for example I096.
    #[arg(long)]
    iteration: String,
    /// Record date in YYYY-MM-DD form.
    #[arg(long)]
    date: String,
    /// Record type for the execution table.
    #[arg(long, value_enum)]
    record_type: GovernanceRecordType,
    /// Record text to append to the iteration execution table.
    #[arg(long)]
    record: String,
}

#[derive(Args, Clone)]
pub(crate) struct IterationRecordWriteArgs {
    #[command(flatten)]
    args: IterationRecordArgs,
    /// Confirm that the preview has been reviewed before writing.
    #[arg(long)]
    confirm_preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum GovernanceRecordType {
    Activation,
    Execution,
    Validation,
    Closeout,
    Note,
}

impl GovernanceRecordType {
    fn as_table_value(self) -> &'static str {
        match self {
            Self::Activation => "Activation",
            Self::Execution => "Execution",
            Self::Validation => "Validation",
            Self::Closeout => "Closeout",
            Self::Note => "Note",
        }
    }
}

struct IterationRecordPlan {
    owner_doc: PathBuf,
    row: String,
}

pub(crate) fn run_governance_command(command: GovernanceCommand) -> Result<()> {
    let workspace = std::env::current_dir()?;
    match command {
        GovernanceCommand::IterationRecord { command } => match command {
            IterationRecordCommand::Preview(args) => {
                let plan = build_iteration_record_plan(&workspace, &args)?;
                print_iteration_record_preview(&workspace, &plan);
            }
            IterationRecordCommand::Write(write_args) => {
                if !write_args.confirm_preview {
                    bail!("refusing governance write without --confirm-preview; run preview first");
                }
                let plan = build_iteration_record_plan(&workspace, &write_args.args)?;
                print_iteration_record_preview(&workspace, &plan);
                apply_iteration_record_plan(&workspace, &plan)?;
                println!("Write: applied");
                println!("Validation: passed");
            }
        },
    }
    Ok(())
}

fn build_iteration_record_plan(
    workspace: &Path,
    args: &IterationRecordArgs,
) -> Result<IterationRecordPlan> {
    validate_iteration_id(&args.iteration)?;
    validate_record_date(&args.date)?;
    let owner_doc = resolve_iteration_doc(workspace, &args.iteration)?;
    let row = format!(
        "| {} | {} | {} |",
        args.date,
        args.record_type.as_table_value(),
        escape_table_cell(&args.record)
    );
    let content = fs::read_to_string(&owner_doc)
        .with_context(|| format!("failed to read {}", owner_doc.display()))?;
    insert_iteration_record(&content, &row)?;

    Ok(IterationRecordPlan { owner_doc, row })
}

fn print_iteration_record_preview(workspace: &Path, plan: &IterationRecordPlan) {
    let owner_doc = plan
        .owner_doc
        .strip_prefix(workspace)
        .unwrap_or(&plan.owner_doc);
    println!("Talos Governance Mutation Preview");
    println!("================================");
    println!("Action: append iteration execution record");
    println!("Owner doc: {}", owner_doc.display());
    println!("Validation after write: internal governance validation");
    println!();
    println!("Row:");
    println!("{}", plan.row);
}

fn apply_iteration_record_plan(workspace: &Path, plan: &IterationRecordPlan) -> Result<()> {
    let original = fs::read_to_string(&plan.owner_doc)
        .with_context(|| format!("failed to read {}", plan.owner_doc.display()))?;
    let updated = insert_iteration_record(&original, &plan.row)?;
    fs::write(&plan.owner_doc, updated)
        .with_context(|| format!("failed to write {}", plan.owner_doc.display()))?;

    if let Err(err) = run_governance_validation(workspace) {
        fs::write(&plan.owner_doc, original).with_context(|| {
            format!(
                "governance validation failed and rollback failed for {}",
                plan.owner_doc.display()
            )
        })?;
        bail!("governance validation failed after write; rolled back: {err}");
    }

    Ok(())
}

fn resolve_iteration_doc(workspace: &Path, iteration: &str) -> Result<PathBuf> {
    let dir = workspace.join("docs").join("iterations");
    let mut matches = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name.starts_with(iteration) && file_name.ends_with(".md") {
            matches.push(path);
        }
    }

    match matches.len() {
        0 => bail!("iteration owner doc not found for {iteration}"),
        1 => Ok(matches.remove(0)),
        _ => bail!("multiple iteration owner docs found for {iteration}"),
    }
}

fn validate_iteration_id(iteration: &str) -> Result<()> {
    let valid = iteration.len() == 4
        && iteration.starts_with('I')
        && iteration[1..].chars().all(|ch| ch.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        bail!("iteration must use I### form, for example I096")
    }
}

fn validate_record_date(date: &str) -> Result<()> {
    let bytes = date.as_bytes();
    let valid = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || idx == 7 || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        bail!("date must use YYYY-MM-DD form")
    }
}

fn escape_table_cell(value: &str) -> String {
    value
        .replace(['\n', '\r'], " ")
        .replace('|', "\\|")
        .trim()
        .to_string()
}

fn insert_iteration_record(content: &str, row: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let header_idx = lines
        .iter()
        .position(|line| line.trim() == "| Date | Type | Record |")
        .context("iteration execution table header not found")?;
    let separator_idx = header_idx + 1;
    if lines
        .get(separator_idx)
        .map(|line| line.trim() == "|---|---|---|")
        != Some(true)
    {
        bail!("iteration execution table separator not found");
    }

    let mut output = String::new();
    for (idx, line) in lines.iter().enumerate() {
        output.push_str(line);
        output.push('\n');
        if idx == separator_idx {
            output.push_str(row);
            output.push('\n');
        }
    }
    Ok(output)
}

fn run_governance_validation(workspace: &Path) -> Result<()> {
    let report = talos_conversation::collect_governance_validation(workspace);
    if report.errors == 0 {
        Ok(())
    } else {
        bail!(
            "internal governance validation failed: {} error(s), {} warning(s): {}",
            report.errors,
            report.warnings,
            report.findings.join("; ")
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn insert_iteration_record_adds_row_after_table_separator() {
        let content = "# Iteration\n\n| Date | Type | Record |\n|---|---|---|\n";
        let updated =
            insert_iteration_record(content, "| 2026-07-04 | Execution | Done |").unwrap();

        assert!(updated.contains(
            "| Date | Type | Record |\n|---|---|---|\n| 2026-07-04 | Execution | Done |"
        ));
    }

    #[test]
    fn insert_iteration_record_rejects_missing_table() {
        let err = insert_iteration_record("# Iteration\n", "| 2026-07-04 | Note | x |")
            .expect_err("missing table should fail");

        assert!(err.to_string().contains("execution table header not found"));
    }

    #[test]
    fn resolve_iteration_doc_rejects_ambiguous_matches() {
        let dir = tempdir().unwrap();
        let iterations = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iterations).unwrap();
        fs::write(iterations.join("I123-one.md"), "").unwrap();
        fs::write(iterations.join("I123-two.md"), "").unwrap();

        let err = resolve_iteration_doc(dir.path(), "I123").expect_err("ambiguous docs fail");

        assert!(
            err.to_string()
                .contains("multiple iteration owner docs found")
        );
    }

    #[test]
    fn validate_iteration_id_requires_canonical_form() {
        assert!(validate_iteration_id("I096").is_ok());
        assert!(validate_iteration_id("96").is_err());
        assert!(validate_iteration_id("I96").is_err());
    }

    #[test]
    fn escape_table_cell_removes_newlines_and_escapes_pipes() {
        assert_eq!(escape_table_cell("a|b\nc"), "a\\|b c");
    }

    #[test]
    fn apply_iteration_record_uses_internal_validation_not_host_script() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agent-governance")).unwrap();
        fs::write(
            dir.path().join(".agent-governance").join("manifest.yaml"),
            "profile: personal\nstatus: adopting\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("docs").join("iterations")).unwrap();
        fs::write(
            dir.path()
                .join("docs")
                .join("iterations")
                .join("I123-internal-validation.md"),
            "# Iteration I123\n\n| Date | Type | Record |\n|---|---|---|\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("scripts")).unwrap();
        fs::write(
            dir.path()
                .join("scripts")
                .join("validate_project_governance.sh"),
            "#!/usr/bin/env bash\ntouch executed-marker\nexit 42\n",
        )
        .unwrap();
        let plan = build_iteration_record_plan(
            dir.path(),
            &IterationRecordArgs {
                iteration: "I123".to_string(),
                date: "2026-07-06".to_string(),
                record_type: GovernanceRecordType::Execution,
                record: "Internal validation path".to_string(),
            },
        )
        .unwrap();

        apply_iteration_record_plan(dir.path(), &plan).unwrap();

        assert!(!dir.path().join("executed-marker").exists());
        let updated = fs::read_to_string(plan.owner_doc).unwrap();
        assert!(updated.contains("Internal validation path"));
    }
}
