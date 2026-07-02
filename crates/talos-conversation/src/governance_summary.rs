use std::path::Path;

pub(crate) fn format_governance_summary(workspace: &Path) -> String {
    let mut out = String::new();
    out.push_str("[System] Talos Governance Status\n");
    out.push_str("[System] ========================\n\n");

    format_manifest(workspace, &mut out);
    format_board(workspace, &mut out);
    format_iterations(workspace, &mut out);

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

        let out = format_governance_summary(dir.path());
        assert!(out.contains("conformant"));
        assert!(out.contains("I047 Test"));
        assert!(out.contains("I080"));
    }
}
