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
    print_board_summary(&workspace);
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

fn print_board_summary(workspace: &Path) {
    let path = workspace.join("docs").join("BOARD.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        println!("Board: not found\n");
        return;
    };

    println!("Board — Active");
    println!("---------------");
    let mut in_now = false;
    let mut found = false;
    for line in content.lines() {
        if line.starts_with("## ") {
            in_now = line.starts_with("## Now");
            continue;
        }
        if !in_now
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
        let item = cols[1].trim();
        let state = cols[2].trim();
        if !item.is_empty() && !item.starts_with("_(no active") {
            println!("  {item} [{state}]");
            found = true;
        }
    }
    if !found {
        println!("  (no active iteration)");
    }
    println!();
}

fn print_iteration_summary(workspace: &Path) {
    let path = workspace.join("docs").join("iterations").join("README.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        println!("Iterations: not found\n");
        return;
    };

    println!("Open Iterations");
    println!("---------------");
    let mut count = 0;
    for line in content.lines() {
        let t = line.trim();
        if !t.starts_with("| I") && !t.starts_with("| R") {
            continue;
        }
        let cols: Vec<&str> = t.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let id = cols[1].trim();
        let codename = cols[2].trim();
        let state = cols[3].trim().to_lowercase();
        if !["planned", "active", "paused", "refinement", "tracking"]
            .iter()
            .any(|s| state.contains(s))
        {
            continue;
        }
        println!("  {id} {codename} — {state}");
        count += 1;
    }
    if count == 0 {
        println!("  (all iterations complete)");
    }
    println!();
}

fn print_validation_status(workspace: &Path) {
    let script = workspace
        .join("scripts")
        .join("validate_project_governance.sh");
    if !script.exists() {
        return;
    }

    println!("Validation");
    println!("-----------");
    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg(".")
        .current_dir(workspace)
        .output();

    match output {
        Ok(out) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() {
                println!("  Status: PASS");
            } else {
                println!("  Status: FAIL");
            }
            for line in combined.lines() {
                if line.contains("warning") || line.contains("passed") || line.contains("error") {
                    println!("  {line}");
                }
            }
        }
        Err(e) => println!("  Unable to run validation: {e}"),
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
        print_board_summary(dir.path());
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
        print_board_summary(dir.path());
        print_iteration_summary(dir.path());
    }

    use std::fs;
}
