use std::path::Path;

pub(crate) fn format_governance_summary(workspace: &Path) -> String {
    let mut out = String::new();
    out.push_str("[System] Talos Governance Status\n");
    out.push_str("[System] ========================\n\n");

    format_manifest(workspace, &mut out);
    format_board(workspace, &mut out);
    format_iterations(workspace, &mut out);
    format_backlog(workspace, &mut out);
    format_validation(workspace, &mut out);

    out
}

fn format_manifest(workspace: &Path, out: &mut String) {
    let path = workspace.join(".agent-governance").join("manifest.yaml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Manifest: not found\n\n");
        return;
    };

    out.push_str("[System] Manifest\n");
    for line in content.lines() {
        let t = line.trim();
        for key in ["profile", "status", "last_audited_at"] {
            if let Some(val) = t.strip_prefix(&format!("{key}:")) {
                out.push_str(&format!(
                    "[System]   {key}: {}\n",
                    val.trim().trim_matches('"')
                ));
            }
        }
    }
    out.push('\n');
}

fn format_board(workspace: &Path, out: &mut String) {
    let path = workspace.join("docs").join("BOARD.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Board: not found\n\n");
        return;
    };

    out.push_str("[System] Board Disposition\n");
    for heading in ["Now", "Blocked / Paused", "Next"] {
        let items = parse_board_section(&content, heading);
        out.push_str(&format!("[System]   {heading}: {} item(s)\n", items.len()));
        for item in items {
            out.push_str(&format!("[System]     - {} [{}]\n", item.0, item.1));
        }
    }
    out.push('\n');
}

fn parse_board_section(content: &str, heading: &str) -> Vec<(String, String)> {
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
        let item = clean_cell(cols[1]);
        let state = clean_cell(cols[2]);
        if !item.is_empty() && !item.starts_with("_(no active") {
            items.push((item, state));
        }
    }

    items
}

fn clean_cell(cell: &str) -> String {
    cell.trim()
        .trim_matches('`')
        .replace('*', "")
        .trim()
        .to_string()
}

fn format_iterations(workspace: &Path, out: &mut String) {
    let path = workspace.join("docs").join("iterations").join("README.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Iterations: not found\n\n");
        return;
    };

    let open = parse_open_iterations(&content);
    out.push_str("[System] Open Iterations\n");
    if open.is_empty() {
        out.push_str("[System]   (all iterations complete)\n");
    }
    for iter in &open {
        out.push_str(&format!("[System]   {} {} — {}\n", iter.0, iter.1, iter.2));
    }
    out.push('\n');
}

fn parse_open_iterations(content: &str) -> Vec<(String, String, String)> {
    let mut in_current = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_current = line.starts_with("## Current Iterations");
            continue;
        }
        if !in_current
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
        let id = clean_cell(cols[1]);
        let codename = clean_cell(cols[2]);
        let state = clean_cell(cols[3]).to_lowercase();
        if is_open_state(&state) {
            items.push((id, codename, state));
        }
    }

    items
}

fn is_open_state(state: &str) -> bool {
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

fn format_backlog(workspace: &Path, out: &mut String) {
    let path = workspace
        .join("docs")
        .join("backlog")
        .join("PRODUCT-BACKLOG.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Backlog: not found\n\n");
        return;
    };

    let active = count_backlog_active_items(&content);
    out.push_str("[System] Backlog\n");
    out.push_str(&format!("[System]   Active items: {}\n", active.len()));
    let in_progress: Vec<&str> = active
        .iter()
        .filter(|(_, s)| {
            let l = s.to_lowercase();
            l.contains("progress") || l.contains("active") || l.contains("review")
        })
        .map(|(id, _)| id.as_str())
        .collect();
    if !in_progress.is_empty() {
        out.push_str(&format!(
            "[System]   In progress: {}\n",
            in_progress.join(", ")
        ));
    }
    out.push('\n');
}

fn count_backlog_active_items(content: &str) -> Vec<(String, String)> {
    let mut in_active = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_active = line.starts_with("## Active Items");
            continue;
        }
        if !in_active
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
        let id = clean_cell(cols[1]);
        let status = clean_cell(cols[3]);
        if !id.is_empty() && !id.starts_with("_(no") {
            items.push((id, status));
        }
    }

    items
}

fn format_validation(workspace: &Path, out: &mut String) {
    let script = workspace
        .join("scripts")
        .join("validate_project_governance.sh");
    out.push_str("[System] Validation\n");
    if !script.exists() {
        out.push_str("[System]   Script not found\n\n");
        return;
    }

    let result = std::process::Command::new("bash")
        .arg(&script)
        .arg(".")
        .current_dir(workspace)
        .output();

    match result {
        Ok(output) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output.status.success() {
                out.push_str("[System]   Status: PASS\n");
            } else {
                out.push_str("[System]   Status: FAIL\n");
            }
            for line in combined.lines() {
                if line.contains("warning") || line.contains("passed") || line.contains("error") {
                    out.push_str(&format!("[System]   {line}\n"));
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("[System]   Unable to run: {e}\n"));
        }
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_workspace_does_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let out = format_governance_summary(dir.path());
        assert!(out.contains("not found"));
    }

    #[test]
    fn board_section_parsing() {
        let board = "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T27 Governance | Active | [x](x.md) | Gate |\n\n## Next\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T38 Rehearsal | Planned | [x](x.md) | Evidence |\n";
        assert_eq!(parse_board_section(board, "Now").len(), 1);
        assert_eq!(parse_board_section(board, "Next").len(), 1);
        assert_eq!(parse_board_section(board, "Blocked / Paused").len(), 0);
    }

    #[test]
    fn open_iterations_filter() {
        let content = "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I018 | Obs | Planned | no |\n| I009 | Ext | **Complete** | yes |\n";
        let open = parse_open_iterations(content);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].0, "I018");
    }

    #[test]
    fn full_summary_with_files() {
        let dir = tempfile::tempdir().unwrap();
        let gov_dir = dir.path().join(".agent-governance");
        std::fs::create_dir_all(&gov_dir).unwrap();
        std::fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"high-risk\"\nstatus: \"conformant\"\n",
        )
        .unwrap();

        let docs_dir = dir.path().join("docs");
        std::fs::create_dir_all(docs_dir.join("iterations")).unwrap();
        std::fs::write(
            docs_dir.join("BOARD.md"),
            "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| I047 Test | Active | [x](x.md) | Gate |\n",
        )
        .unwrap();
        std::fs::write(
            docs_dir.join("iterations").join("README.md"),
            "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I080 | Frontline | Planned | no |\n",
        )
        .unwrap();

        let backlog_dir = docs_dir.join("backlog");
        std::fs::create_dir_all(&backlog_dir).unwrap();
        std::fs::write(
            backlog_dir.join("PRODUCT-BACKLOG.md"),
            "## Active Items\n\n| ID | Title | Status | Priority | Decision Context | Required Reads |\n|---|---|---|---|---|---|\n| TUI-021 | Composer Nav | In Progress | P3 | ctx | reads |\n| CONF-001 | Config | Planned | P2 | ctx | reads |\n",
        )
        .unwrap();

        let out = format_governance_summary(dir.path());
        assert!(out.contains("conformant"));
        assert!(out.contains("I047 Test"));
        assert!(out.contains("I080"));
        assert!(out.contains("Backlog"));
        assert!(out.contains("Active items: 2"));
        assert!(out.contains("TUI-021"));
    }

    #[test]
    fn backlog_parsing_counts_active_items() {
        let content = "## Active Items\n\n| ID | Title | Status | Priority | Decision Context | Required Reads |\n|---|---|---|---|---|---|\n| REL-002 | Release | Planned | P1 | ctx | reads |\n| GOV-003 | Governance | In Progress | P2 | ctx | reads |\n";
        let items = count_backlog_active_items(content);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, "REL-002");
        assert_eq!(items[1].0, "GOV-003");
    }

    #[test]
    fn validation_section_reports_missing_script() {
        let dir = tempfile::tempdir().unwrap();
        let mut out = String::new();
        format_validation(dir.path(), &mut out);
        assert!(out.contains("not found") || out.contains("Unable"));
    }
}
