//! Read-only governance status reporting.
//!
//! Parses standard governance documents (manifest, board, iteration index) and
//! prints a structured summary. This command is strictly read-only.

use std::path::Path;

use anyhow::Result;

pub(crate) fn run_governance_status() -> Result<()> {
    let workspace = std::env::current_dir()?;

    println!("Talos Governance Status");
    println!("========================\n");

    print_manifest(&workspace);
    print_board_disposition(&workspace);
    print_iteration_summary(&workspace);
    print_validation_status(&workspace);
    print_git_status(&workspace);

    Ok(())
}

fn print_manifest(workspace: &Path) {
    let path = workspace.join(".agent-governance").join("manifest.yaml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        println!("Manifest: not found\n");
        return;
    };

    println!("Manifest");
    println!("--------");
    for line in content.lines() {
        let t = line.trim();
        for key in ["profile", "status", "last_audited_at"] {
            if let Some(val) = t.strip_prefix(&format!("{key}:")) {
                println!("  {key}: {}", val.trim().trim_matches('"'));
            }
        }
    }
    println!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoardItem {
    item: String,
    state: String,
}

fn print_board_disposition(workspace: &Path) {
    let path = workspace.join("docs").join("BOARD.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        println!("Board: not found\n");
        return;
    };

    println!("Board Disposition");
    println!("-----------------");
    for heading in ["Now", "Blocked / Paused", "Next"] {
        let items = parse_board_section(&content, heading);
        println!("  {heading}: {} item(s)", items.len());
        for item in items {
            println!("    - {} [{}]", item.item, item.state);
        }
    }
    println!();
}

fn parse_board_section(content: &str, heading: &str) -> Vec<BoardItem> {
    let target = format!("## {heading}");
    let mut in_section = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == target;
            continue;
        }
        if !in_section
            || !line.starts_with("| ")
            || line.starts_with("|---")
            || line.starts_with("| Item")
        {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let item = clean_table_cell(cols[1]);
        let state = clean_table_cell(cols[2]);
        if !item.is_empty() && !item.starts_with("_(no active") {
            items.push(BoardItem { item, state });
        }
    }

    items
}

fn clean_table_cell(cell: &str) -> String {
    cell.trim()
        .trim_matches('`')
        .replace('*', "")
        .trim()
        .to_string()
}

fn print_iteration_summary(workspace: &Path) {
    let path = workspace.join("docs").join("iterations").join("README.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        println!("Iterations: not found\n");
        return;
    };

    println!("Open Iterations");
    println!("---------------");
    let open = parse_open_iterations(&content);
    for iteration in &open {
        println!(
            "  {} {} — {}",
            iteration.id, iteration.codename, iteration.state
        );
    }
    if open.is_empty() {
        println!("  (all iterations complete)");
    }
    println!();
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IterationItem {
    id: String,
    codename: String,
    state: String,
}

fn parse_open_iterations(content: &str) -> Vec<IterationItem> {
    let mut in_current_iterations = false;
    let mut found = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_current_iterations = line.starts_with("## Current Iterations");
            continue;
        }
        if !in_current_iterations
            || !line.starts_with("| ")
            || line.starts_with("|---")
            || line.starts_with("| ID")
        {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let id = clean_table_cell(cols[1]);
        let codename = clean_table_cell(cols[2]);
        let state = clean_table_cell(cols[3]).to_lowercase();
        if is_open_iteration_state(&state) {
            items.push(IterationItem {
                id,
                codename,
                state,
            });
            found = true;
        }
    }

    if found { items } else { Vec::new() }
}

fn is_open_iteration_state(state: &str) -> bool {
    if state.contains("complete") || state.contains("superseded") {
        return false;
    }
    [
        "planned",
        "active",
        "paused",
        "review",
        "blocked",
        "refinement",
        "tracking",
    ]
    .iter()
    .any(|s| state.contains(s))
}

fn print_validation_status(workspace: &Path) {
    let report = talos_conversation::collect_governance_validation(workspace);
    println!("Validation");
    println!("-----------");
    if report.errors == 0 {
        println!("  Status: PASS ({} warning(s))", report.warnings);
    } else {
        println!(
            "  Status: FAIL ({} error(s), {} warning(s))",
            report.errors, report.warnings
        );
    }
    for finding in report.findings.iter().take(12) {
        println!("  {finding}");
    }
    if report.findings.len() > 12 {
        println!("  ... {} more finding(s)", report.findings.len() - 12);
    }
    println!();
}

fn print_git_status(workspace: &Path) {
    if let Ok(out) = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(workspace)
        .output()
    {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let dirty = stdout.lines().filter(|l| !l.is_empty()).count();
        if dirty == 0 {
            println!("Git: clean working tree");
        } else {
            println!("Git: {dirty} uncommitted change(s)");
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn manifest_missing_does_not_panic() {
        let dir = tempdir().unwrap();
        print_manifest(dir.path());
    }

    #[test]
    fn board_missing_does_not_panic() {
        let dir = tempdir().unwrap();
        print_board_disposition(dir.path());
    }

    #[test]
    fn iterations_missing_does_not_panic() {
        let dir = tempdir().unwrap();
        print_iteration_summary(dir.path());
    }

    #[test]
    fn full_workspace_parses_without_panic() {
        let dir = tempdir().unwrap();

        let gov_dir = dir.path().join(".agent-governance");
        fs::create_dir_all(&gov_dir).unwrap();
        fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"high-risk\"\nstatus: \"conformant\"\nlast_audited_at: \"2026-06-25\"\n",
        )
        .unwrap();

        let docs_dir = dir.path().join("docs");
        fs::create_dir_all(&docs_dir).unwrap();
        fs::write(
            docs_dir.join("BOARD.md"),
            "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| I047 Test | Active | [I047](x.md) | Gate |\n\n## Next\n",
        )
        .unwrap();

        let iter_dir = docs_dir.join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "| ID | Codename | State | Verified |\n|---|---|---|---|\n| I047 | Test | Active | yes |\n| I001 | Done | Complete | yes |\n",
        )
        .unwrap();

        print_manifest(dir.path());
        print_board_disposition(dir.path());
        print_iteration_summary(dir.path());
    }

    #[test]
    fn validation_status_does_not_execute_workspace_script() {
        let dir = tempdir().unwrap();
        let gov_dir = dir.path().join(".agent-governance");
        fs::create_dir_all(&gov_dir).unwrap();
        fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"small\"\nstatus: \"conformant\"\n",
        )
        .unwrap();
        let script_dir = dir.path().join("scripts");
        fs::create_dir_all(&script_dir).unwrap();
        fs::write(
            script_dir.join("validate_project_governance.sh"),
            "#!/usr/bin/env bash\ntouch executed-marker\n",
        )
        .unwrap();

        print_validation_status(dir.path());

        assert!(!dir.path().join("executed-marker").exists());
    }

    #[test]
    fn parses_board_disposition_sections() {
        let content = "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T27 Governance | Active | [x](x.md) | Gate |\n\n## Blocked / Paused\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T28 Dashboard | Blocked | [x](x.md) | ADR gate |\n\n## Next\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T38 Rehearsal | Planned | [x](x.md) | Evidence |\n";

        assert_eq!(
            parse_board_section(content, "Now"),
            vec![BoardItem {
                item: "T27 Governance".to_string(),
                state: "Active".to_string()
            }]
        );
        assert_eq!(parse_board_section(content, "Blocked / Paused").len(), 1);
        assert_eq!(parse_board_section(content, "Next").len(), 1);
    }

    #[test]
    fn open_iterations_ignore_completed_rows_and_execution_rounds() {
        let content = "# Iterations\n\n## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I018 | Observability | Planned | no |\n| I009 | Extensible | **Complete** (2026-06-03) | yes |\n| I012 | Portable | **Superseded** | yes |\n| I075 | Month 1 | **Planned (2026-06-30)** | no |\n\n## Next Execution Rounds\n\n| Round | When | Work Items | Promotion Rule |\n|---|---|---|---|\n| R16: Two-Week Handoff | ✅ Done (2026-06-18) | `x.md` | Done |\n| R27: High-Risk Governance Gate | In Progress (2026-06-27) | `x.md` | Gate |\n";

        let open = parse_open_iterations(content);

        assert_eq!(
            open,
            vec![
                IterationItem {
                    id: "I018".to_string(),
                    codename: "Observability".to_string(),
                    state: "planned".to_string()
                },
                IterationItem {
                    id: "I075".to_string(),
                    codename: "Month 1".to_string(),
                    state: "planned (2026-06-30)".to_string()
                }
            ]
        );
    }

    use std::fs;
}
