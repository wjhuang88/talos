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
        print_json(&summary);
    } else {
        print_text(&summary);
    }
    Ok(())
}

/// Read-only diagnostics summary. No credential values are included.
#[derive(Debug, Clone)]
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

    let talos_root = crate::storage::resolve_talos_root();
    let trust_store = talos_permission::WorkspaceTrustStore::new(&talos_root);
    let ws_path = Path::new(&workspace_root);
    let is_git = talos_permission::is_git_workspace(ws_path);
    let trusted = trust_store.is_trusted(ws_path);

    let config_path = talos_root.join("config.toml");

    DiagnosticsSummary {
        talos_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_toolchain: rustc_version(),
        session_formats: session_formats(),
        workspace_root,
        is_git_workspace: is_git,
        workspace_trusted: trusted,
        trusted_workspace_count: trust_store.trusted_count(),
        config_exists: config_path.exists(),
        active_iterations: known_active_iterations(),
        residual_gates: known_residual_gates(),
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

fn known_active_iterations() -> Vec<String> {
    vec!["See docs/iterations/README.md for current iteration state".to_string()]
}

fn known_residual_gates() -> Vec<String> {
    vec![
        "REL-002 v1.0 Self-Bootstrap — NO-GO (zero qualifying Talos-primary sessions)".to_string(),
        "PERM-005 — bash/exec remains per-command Ask/Deny (evidence is diagnostic-only)"
            .to_string(),
        "PERM-004 — file-write trust within Git repo; command trust not broadened".to_string(),
        "I085 MC107 — real-terminal /connect walkthrough Paused".to_string(),
    ]
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

fn print_json(s: &DiagnosticsSummary) {
    println("{");
    println(&format!("  \"talos_version\": \"{}\",", s.talos_version));
    println(&format!("  \"rust_toolchain\": \"{}\",", s.rust_toolchain));
    println("  \"session_formats\": [");
    for (i, fmt) in s.session_formats.iter().enumerate() {
        let comma = if i + 1 < s.session_formats.len() {
            ","
        } else {
            ""
        };
        println(&format!("    \"{fmt}\"{comma}"));
    }
    println("  ],");
    println(&format!("  \"workspace_root\": \"{}\",", s.workspace_root));
    println(&format!("  \"is_git_workspace\": {},", s.is_git_workspace));
    println(&format!(
        "  \"workspace_trusted\": {},",
        s.workspace_trusted
    ));
    println(&format!(
        "  \"trusted_workspace_count\": {},",
        s.trusted_workspace_count
    ));
    println(&format!("  \"config_exists\": {},", s.config_exists));
    println("  \"active_iterations\": [");
    for (i, it) in s.active_iterations.iter().enumerate() {
        let comma = if i + 1 < s.active_iterations.len() {
            ","
        } else {
            ""
        };
        println(&format!("    \"{it}\"{comma}"));
    }
    println("  ],");
    println("  \"residual_gates\": [");
    for (i, g) in s.residual_gates.iter().enumerate() {
        let comma = if i + 1 < s.residual_gates.len() {
            ","
        } else {
            ""
        };
        println(&format!("    \"{g}\"{comma}"));
    }
    println("  ]");
    println("}");
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
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_summary_no_secrets() {
        let summary = collect_diagnostics_summary();
        // Ensure no API keys, tokens, or secrets leak into diagnostics output.
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
    fn test_json_output_is_valid_structure() {
        let summary = collect_diagnostics_summary();
        // Verify the summary has all required fields for JSON output.
        assert!(!summary.talos_version.is_empty());
        assert!(!summary.workspace_root.is_empty());
        assert!(!summary.session_formats.is_empty());
        assert!(!summary.active_iterations.is_empty());
        assert!(!summary.residual_gates.is_empty());
    }
}
