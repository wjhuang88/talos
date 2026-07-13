//! Diagnostics status command — read-only summary of release, toolchain, session,
//! trust, and residual-gate state without exposing secrets.
//!
//! Origin: I116 LT012 / N103.

use std::path::Path;

use anyhow::Result;
use clap::Subcommand;

/// Subcommands for `talos diagnostics`.
#[derive(Subcommand, Clone)]
pub(crate) enum DiagnosticsCommand {
    /// Print a read-only status summary (release, toolchain, session format,
    /// workspace trust, residual gates). Secrets are masked.
    Status {
        /// Output JSON instead of human-readable text.
        #[arg(long)]
        json: bool,
    },
}

/// Entry point for `talos diagnostics` subcommands.
pub(crate) fn run_diagnostics_command(command: DiagnosticsCommand) -> Result<()> {
    match command {
        DiagnosticsCommand::Status { json } => run_diagnostics_status(json),
    }
}

fn run_diagnostics_status(json: bool) -> Result<()> {
    let summary = collect_diagnostics_summary();
    if json {
        let json_str = serde_json::to_string_pretty(&summary)?;
        println!("{json_str}");
    } else {
        print_text(&summary);
    }
    Ok(())
}

/// Read-only diagnostics summary. No credential values are included.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DiagnosticsSummary {
    talos_version: String,
    rust_toolchain: String,
    session_formats: Vec<String>,
    workspace_root: String,
    is_git_workspace: bool,
    workspace_trusted: bool,
    trusted_workspace_count: usize,
    config_exists: bool,
    active_iterations: Vec<String>,
    residual_gates: Vec<String>,
}

fn collect_diagnostics_summary() -> DiagnosticsSummary {
    let workspace_root = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "<unknown>".to_string());
    let ws_path = workspace_root.clone();
    collect_diagnostics_summary_at(Path::new(&ws_path), workspace_root)
}

fn collect_diagnostics_summary_at(workspace: &Path, workspace_root: String) -> DiagnosticsSummary {
    let talos_root = crate::storage::resolve_talos_root();
    let trust_store = talos_permission::WorkspaceTrustStore::new(&talos_root);
    let is_git = talos_permission::is_git_workspace(workspace);
    let trusted = trust_store.is_trusted(workspace);

    let config_path = talos_root.join("config.toml");
    let active_iterations = collect_active_iterations_at(workspace);
    let residual_gates = collect_residual_gates();

    DiagnosticsSummary {
        talos_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_toolchain: rustc_version(),
        session_formats: session_formats(),
        workspace_root,
        is_git_workspace: is_git,
        workspace_trusted: trusted,
        trusted_workspace_count: trust_store.trusted_count(),
        config_exists: config_path.exists(),
        active_iterations,
        residual_gates,
    }
}

fn rustc_version() -> String {
    // Static toolchain version from rust-toolchain.toml is not directly accessible.
    // Report the compile-time Rust version instead.
    option_env!("RUSTC_VERSION")
        .unwrap_or("see rust-toolchain.toml")
        .to_string()
}

fn session_formats() -> Vec<String> {
    // ADR-037: compact text (.tlog) is the new-session default; JSONL is legacy read-only.
    vec![
        "compact-text (.tlog) — new sessions (ADR-037)".to_string(),
        "jsonl (.jsonl) — legacy read-only compatibility".to_string(),
    ]
}

fn collect_active_iterations_at(workspace: &Path) -> Vec<String> {
    let iter_path = workspace.join("docs").join("iterations").join("README.md");
    let Ok(content) = std::fs::read_to_string(&iter_path) else {
        return vec!["unavailable: iteration index not found".to_string()];
    };
    if !has_valid_iteration_table(&content) {
        return vec!["unavailable: iteration index malformed".to_string()];
    }
    let iterations = crate::governance::parse_open_iterations(&content);
    if iterations.is_empty() {
        return vec!["(no open iterations)".to_string()];
    }
    iterations
        .iter()
        .map(|it| format!("{} {} — {}", it.id, it.codename, it.state))
        .collect()
}

fn has_valid_iteration_table(content: &str) -> bool {
    let mut in_section = false;
    let mut found_table_sep = false;
    for line in content.lines() {
        if line.starts_with("## ") {
            if found_table_sep {
                break;
            }
            in_section = line.starts_with("## Current Iterations");
            continue;
        }
        if !in_section {
            continue;
        }
        if line.starts_with("|---") {
            found_table_sep = true;
        }
    }
    found_table_sep
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ResidualGate {
    id: &'static str,
    summary: &'static str,
}

fn residual_gate_registry() -> Vec<ResidualGate> {
    vec![
        ResidualGate {
            id: "REL-002",
            summary: "v1.0 Self-Bootstrap — NO-GO (zero qualifying Talos-primary sessions)",
        },
        ResidualGate {
            id: "PERM-005",
            summary: "bash/exec remains per-command Ask/Deny (evidence is diagnostic-only)",
        },
        ResidualGate {
            id: "PERM-004",
            summary: "file-write trust within Git repo; command trust not broadened",
        },
    ]
}

fn collect_residual_gates() -> Vec<String> {
    residual_gate_registry()
        .iter()
        .map(|g| format!("{} — {}", g.id, g.summary))
        .collect()
}

fn print_text(s: &DiagnosticsSummary) {
    println("=== Talos Diagnostics Status ===");
    println("");
    println("=== Release And Toolchain ===");
    println(&format!("  Talos version:  {}", s.talos_version));
    println(&format!("  Rust toolchain: {}", s.rust_toolchain));
    println("");
    println("=== Session Format ===");
    for fmt in &s.session_formats {
        println(&format!("  {fmt}"));
    }
    println("");
    println("=== Workspace Trust ===");
    println(&format!("  Workspace:      {}", s.workspace_root));
    println(&format!("  Git repository: {}", yes_no(s.is_git_workspace)));
    println(&format!(
        "  Trusted:        {}",
        yes_no(s.workspace_trusted)
    ));
    println(&format!(
        "  Trusted total:  {} workspace(s)",
        s.trusted_workspace_count
    ));
    println(&format!("  Config exists:  {}", yes_no(s.config_exists)));
    println("");
    println("=== Active Iterations ===");
    for it in &s.active_iterations {
        println(&format!("  {it}"));
    }
    println("");
    println("=== Residual Gates ===");
    for g in &s.residual_gates {
        println(&format!("  {g}"));
    }
    println("");
    println("All values are read-only. Credential values are not displayed.");
}

fn println(s: &str) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = writeln!(lock, "{s}");
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_diagnostics_summary_no_secrets() {
        let summary = collect_diagnostics_summary();
        let combined = format!(
            "{} {} {:?} {:?} {:?}",
            summary.talos_version,
            summary.rust_toolchain,
            summary.session_formats,
            summary.active_iterations,
            summary.residual_gates
        );
        assert!(
            !combined.contains("api_key"),
            "diagnostics must not contain api_key"
        );
        assert!(
            !combined.contains("sk-ant"),
            "diagnostics must not contain API key prefixes"
        );
        assert!(
            !combined.to_lowercase().contains("secret"),
            "diagnostics must not contain 'secret'"
        );
    }

    #[test]
    fn test_session_formats_lists_both() {
        let summary = collect_diagnostics_summary();
        assert!(
            summary
                .session_formats
                .iter()
                .any(|f| f.contains("compact-text")),
            "should list compact-text format"
        );
        assert!(
            summary.session_formats.iter().any(|f| f.contains("jsonl")),
            "should list jsonl legacy format"
        );
    }

    #[test]
    fn test_residual_gates_include_rel002_and_perm005() {
        let summary = collect_diagnostics_summary();
        assert!(
            summary.residual_gates.iter().any(|g| g.contains("REL-002")),
            "should report REL-002 residual"
        );
        assert!(
            summary
                .residual_gates
                .iter()
                .any(|g| g.contains("PERM-005")),
            "should report PERM-005 residual"
        );
    }

    #[test]
    fn test_no_stale_i085_paused_claim() {
        let gates = collect_residual_gates();
        assert!(
            !gates
                .iter()
                .any(|g| g.contains("I085") && g.contains("Paused")),
            "stale I085 Paused claim must not remain in residual gates"
        );
    }

    #[test]
    fn test_json_output_is_valid_structure() {
        let summary = collect_diagnostics_summary();
        assert!(!summary.talos_version.is_empty());
        assert!(!summary.workspace_root.is_empty());
        assert!(!summary.session_formats.is_empty());
        assert!(!summary.active_iterations.is_empty());
        assert!(!summary.residual_gates.is_empty());
    }

    #[test]
    fn test_diagnostics_summary_serializes_to_valid_json() {
        let summary = collect_diagnostics_summary();
        let json_str = serde_json::to_string(&summary).expect("serialize must succeed");
        let value: serde_json::Value =
            serde_json::from_str(&json_str).expect("output must parse as serde_json::Value");
        assert!(value.get("talos_version").is_some());
        assert!(value.get("active_iterations").is_some());
        assert!(value.get("residual_gates").is_some());
        assert!(value.get("workspace_trusted").is_some());
    }

    #[test]
    fn test_diagnostics_summary_round_trips_through_serde() {
        let summary = collect_diagnostics_summary();
        let json_str = serde_json::to_string(&summary).expect("serialize");
        let restored: DiagnosticsSummary = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(summary.talos_version, restored.talos_version);
        assert_eq!(summary.active_iterations, restored.active_iterations);
        assert_eq!(summary.residual_gates, restored.residual_gates);
    }

    #[test]
    fn test_active_iterations_from_clean_source() {
        let dir = tempdir().expect("tempdir");
        let iter_dir = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "# Iterations\n\n## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I120 | Dynamic Diagnostics | **Active** (2026-07-13) | no |\n| I001 | Project Scaffold | Complete | yes |\n",
        )
        .unwrap();

        let iterations = collect_active_iterations_at(dir.path());
        assert!(
            iterations
                .iter()
                .any(|i| i.contains("I120") && i.contains("active")),
            "should find I120 as active: {iterations:?}"
        );
        assert!(
            !iterations.iter().any(|i| i.contains("I001")),
            "should not list completed I001"
        );
    }

    #[test]
    fn test_active_iterations_when_index_missing() {
        let dir = tempdir().expect("tempdir");
        let iterations = collect_active_iterations_at(dir.path());
        assert_eq!(
            iterations,
            vec!["unavailable: iteration index not found".to_string()],
            "missing index should produce bounded unavailable diagnostic"
        );
    }

    #[test]
    fn test_active_iterations_when_index_malformed() {
        let dir = tempdir().expect("tempdir");
        let iter_dir = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "# Iterations\n\nNo table here, just prose.\n",
        )
        .unwrap();

        let iterations = collect_active_iterations_at(dir.path());
        assert!(
            iterations.iter().any(|i| i.contains("unavailable")),
            "malformed index should produce unavailable, not '(no open iterations)': {iterations:?}"
        );
    }

    #[test]
    fn test_active_iterations_with_empty_table() {
        let dir = tempdir().expect("tempdir");
        let iter_dir = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n",
        )
        .unwrap();

        let iterations = collect_active_iterations_at(dir.path());
        assert_eq!(
            iterations,
            vec!["(no open iterations)".to_string()],
            "empty table should produce bounded empty diagnostic"
        );
    }

    #[test]
    fn test_json_string_escaping_via_serde() {
        let mut summary = collect_diagnostics_summary();
        summary.talos_version = "test\"with\\backslash\nand\ttab".to_string();
        let json_str = serde_json::to_string(&summary).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json_str).expect("must parse");
        assert_eq!(value["talos_version"], "test\"with\\backslash\nand\ttab");
    }

    #[test]
    fn test_full_summary_from_clean_workspace() {
        let dir = tempdir().expect("tempdir");
        let iter_dir = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "# Iterations\n\n## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I120 | Dynamic Diagnostics | **Active** (2026-07-13) | no |\n",
        )
        .unwrap();

        let summary =
            collect_diagnostics_summary_at(dir.path(), dir.path().to_string_lossy().into());
        assert!(!summary.talos_version.is_empty());
        assert!(
            summary.active_iterations.iter().any(|i| i.contains("I120")),
            "clean workspace should find I120: {:?}",
            summary.active_iterations
        );
        assert!(
            !summary.residual_gates.is_empty(),
            "residual gates should always be present from typed registry"
        );
    }

    #[test]
    fn test_full_summary_from_empty_workspace() {
        let dir = tempdir().expect("tempdir");
        let summary =
            collect_diagnostics_summary_at(dir.path(), dir.path().to_string_lossy().into());

        assert!(
            summary
                .active_iterations
                .iter()
                .any(|i| i.contains("unavailable")),
            "empty workspace should report unavailable iterations: {:?}",
            summary.active_iterations
        );
        assert!(
            !summary.residual_gates.is_empty(),
            "residual gates must always be bounded even with no workspace"
        );
        assert!(
            summary.residual_gates.iter().any(|g| g.contains("REL-002")),
            "typed registry fallback should include REL-002"
        );
    }

    #[test]
    fn test_full_summary_from_malformed_workspace() {
        let dir = tempdir().expect("tempdir");
        let iter_dir = dir.path().join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "{{{{not valid markdown{{{{\n\n```\nbinary\x00null\n```\n",
        )
        .unwrap_or(());

        let summary =
            collect_diagnostics_summary_at(dir.path(), dir.path().to_string_lossy().into());
        assert!(
            !summary.active_iterations.is_empty(),
            "malformed workspace should not produce empty iterations"
        );
        assert!(
            summary
                .active_iterations
                .iter()
                .any(|i| i.contains("unavailable")),
            "malformed workspace should report unavailable, not false empty: {:?}",
            summary.active_iterations
        );
        assert!(
            !summary.residual_gates.is_empty(),
            "malformed workspace should not affect residual gates"
        );
    }

    #[test]
    fn test_text_and_json_views_share_same_summary() {
        let dir = tempdir().expect("tempdir");
        let summary =
            collect_diagnostics_summary_at(dir.path(), dir.path().to_string_lossy().into());

        let json_str = serde_json::to_string(&summary).expect("serialize");
        let json_value: serde_json::Value = serde_json::from_str(&json_str).expect("parse");

        assert_eq!(
            json_value["active_iterations"]
                .as_array()
                .expect("array")
                .len(),
            summary.active_iterations.len(),
            "JSON and struct should have same iteration count"
        );
        assert_eq!(
            json_value["residual_gates"]
                .as_array()
                .expect("array")
                .len(),
            summary.residual_gates.len(),
            "JSON and struct should have same gate count"
        );
    }

    #[test]
    fn test_residual_gates_always_bounded() {
        let dir = tempdir().expect("tempdir");
        let gates = collect_residual_gates();
        assert!(!gates.is_empty(), "residual gates must never be empty");
        assert!(
            gates.iter().all(|g| !g.is_empty()),
            "each gate string must be non-empty"
        );
    }

    #[test]
    fn test_summary_with_unicode_workspace_path() {
        let dir = tempdir().expect("tempdir");
        let unicode_subdir = dir.path().join("テスト");
        fs::create_dir_all(&unicode_subdir).unwrap();
        let iter_dir = unicode_subdir.join("docs").join("iterations");
        fs::create_dir_all(&iter_dir).unwrap();
        fs::write(
            iter_dir.join("README.md"),
            "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I120 | Test | Active | no |\n",
        )
        .unwrap();

        let summary = collect_diagnostics_summary_at(
            &unicode_subdir,
            unicode_subdir.to_string_lossy().into(),
        );
        let json_str = serde_json::to_string(&summary).expect("serialize");
        let value: serde_json::Value = serde_json::from_str(&json_str).expect("parse");
        assert!(
            value["workspace_root"].as_str().unwrap().contains("テスト"),
            "unicode path should be properly serialized"
        );
    }
}
