//! Dashboard snapshot and governance summary helpers.

use std::path::Path;
use talos_config::Config;

pub(crate) fn build_dashboard_snapshot(
    config: &Config,
    session_manager: &talos_session::SessionManager,
    workspace_root: &str,
) -> talos_dashboard::DashboardSnapshot {
    let config_toml = toml::to_string_pretty(config).unwrap_or_default();
    let config_masked = crate::mask_secrets(&config_toml, config);

    let status = serde_json::json!({
        "model": config.model,
        "provider": config.provider,
        "workspace": workspace_root,
    });

    let history = session_manager
        .list_recent(10)
        .map(|sessions| {
            serde_json::Value::Array(
                sessions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id.to_string(),
                            "workspace": s.workspace_root,
                            "messages": s.message_count,
                            "preview": s.last_message_preview,
                        })
                    })
                    .collect(),
            )
        })
        .unwrap_or(serde_json::json!([]));

    let governance = build_dashboard_governance_summary(Path::new(workspace_root));

    talos_dashboard::DashboardSnapshot {
        config_masked,
        status,
        history,
        governance,
    }
}

fn build_dashboard_governance_summary(workspace_root: &Path) -> String {
    let mut lines = vec!["Talos Governance".to_string()];

    let manifest_path = workspace_root
        .join(".agent-governance")
        .join("manifest.yaml");
    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
        for line in content.lines() {
            let t = line.trim();
            for key in ["profile", "status", "last_audited_at"] {
                if let Some(val) = t.strip_prefix(&format!("{key}:")) {
                    lines.push(format!("Manifest {key}: {}", val.trim().trim_matches('"')));
                }
            }
        }
    }

    let board_path = workspace_root.join("docs").join("BOARD.md");
    if let Ok(content) = std::fs::read_to_string(board_path) {
        for heading in ["Now", "Blocked / Paused", "Next"] {
            let items = parse_dashboard_board_section(&content, heading);
            lines.push(format!("{heading}: {} item(s)", items.len()));
            for (item, state) in items {
                lines.push(format!("- {item} [{state}]"));
            }
        }
    }

    let iter_path = workspace_root
        .join("docs")
        .join("iterations")
        .join("README.md");
    if let Ok(content) = std::fs::read_to_string(iter_path) {
        let open = parse_dashboard_open_iterations(&content);
        if open.is_empty() {
            lines.push("Iterations: all complete".to_string());
        } else {
            let ids: Vec<&str> = open.iter().map(|(id, _, _)| id.as_str()).collect();
            lines.push(format!("Open iterations: {}", ids.join(", ")));
        }
    }

    lines.join("\n")
}

pub(crate) fn parse_dashboard_board_section(content: &str, heading: &str) -> Vec<(String, String)> {
    let target = format!("## {heading}");
    let mut in_section = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == target;
            continue;
        }
        if !in_section || !line.starts_with("| ") || line.starts_with("|---") {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 4 || cols[1].trim() == "Item" {
            continue;
        }
        let item = clean_dashboard_cell(cols[1]);
        let state = clean_dashboard_cell(cols[2]);
        if !item.is_empty() && !item.starts_with("_(no ") {
            items.push((item, state));
        }
    }

    items
}

fn clean_dashboard_cell(cell: &str) -> String {
    cell.trim()
        .trim_matches('`')
        .replace('*', "")
        .trim()
        .to_string()
}

fn parse_dashboard_open_iterations(content: &str) -> Vec<(String, String, String)> {
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
        let id = clean_dashboard_cell(cols[1]);
        let codename = clean_dashboard_cell(cols[2]);
        let state = clean_dashboard_cell(cols[3]).to_lowercase();
        if !state.contains("complete")
            && !state.contains("superseded")
            && [
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
        {
            items.push((id, codename, state));
        }
    }

    items
}
