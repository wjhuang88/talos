use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;
use talos_core::tool::ToolNature;
use talos_permission::{PermissionDecision, PermissionEngine};
use talos_session::{SessionCleanupCandidate, SessionCleanupPolicy, SessionManager};

/// CLI subcommands for local storage visibility and maintenance.
#[derive(Subcommand, Clone)]
pub(crate) enum StorageCommand {
    /// Show local storage usage and status (read-only).
    Status,
    /// Clean up old sessions (defaults to dry-run).
    Cleanup {
        /// Actually delete matched sessions (default is dry-run preview).
        #[arg(long)]
        apply: bool,
        /// Keep at most N newest sessions per workspace after protecting active sessions.
        #[arg(long)]
        max_sessions: Option<usize>,
        /// Delete sessions older than N days.
        #[arg(long)]
        max_age_days: Option<i64>,
        /// Limit cleanup to one workspace root path.
        #[arg(long)]
        workspace: Option<String>,
        /// Session UUID to protect from cleanup (the active session).
        #[arg(long)]
        protect_session: Option<String>,
    },
    /// Run SQLite maintenance operations.
    Maintenance {
        /// Checkpoint and truncate WAL files.
        #[arg(long)]
        checkpoint: bool,
        /// Vacuum databases to reclaim free pages.
        #[arg(long)]
        vacuum: bool,
        /// Reconcile session index drift (reindex missing, remove orphan rows).
        #[arg(long)]
        reconcile: bool,
    },
}

/// Dispatch a storage command.
pub(crate) fn run_storage_command(cmd: StorageCommand) -> Result<()> {
    match cmd {
        StorageCommand::Status => run_storage_status(),
        StorageCommand::Cleanup {
            apply,
            max_sessions,
            max_age_days,
            workspace,
            protect_session,
        } => run_storage_cleanup(&CleanupArgs {
            apply,
            max_sessions,
            max_age_days,
            workspace,
            protect_session,
        }),
        StorageCommand::Maintenance {
            checkpoint,
            vacuum,
            reconcile,
        } => run_storage_maintenance(&MaintenanceArgs {
            checkpoint,
            vacuum,
            reconcile,
        }),
    }
}

/// Aggregated CLI args for cleanup.
pub(crate) struct CleanupArgs {
    pub(crate) apply: bool,
    pub(crate) max_sessions: Option<usize>,
    pub(crate) max_age_days: Option<i64>,
    pub(crate) workspace: Option<String>,
    pub(crate) protect_session: Option<String>,
}

/// Aggregated CLI args for maintenance.
pub(crate) struct MaintenanceArgs {
    pub(crate) checkpoint: bool,
    pub(crate) vacuum: bool,
    pub(crate) reconcile: bool,
}

fn run_storage_status() -> Result<()> {
    let talos_root = resolve_talos_root();
    let status = collect_storage_status(&talos_root);
    print_storage_status(&status);
    Ok(())
}

/// Evaluate storage cleanup through the permission engine.
///
/// Returns `Allow` when the engine permits the operation, `Deny(reason)` when
/// a rule blocks it, or `Ask` when user approval would be required (the caller
/// should resolve `Ask` based on whether `--apply` was passed).
pub(crate) fn authorize_cleanup(
    engine: &PermissionEngine,
    candidates: &[SessionCleanupCandidate],
) -> PermissionDecision {
    let session_ids: Vec<serde_json::Value> = candidates
        .iter()
        .map(|c| serde_json::Value::String(c.id.to_string()))
        .collect();

    let input = serde_json::json!({
        "operation": "storage_cleanup",
        "candidate_count": candidates.len(),
        "sessions": session_ids,
    });

    engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input)
}

fn run_storage_cleanup(args: &CleanupArgs) -> Result<()> {
    let manager = SessionManager::new().context("failed to create session manager")?;

    let mut protected_ids = Vec::new();
    if let Some(ref sid) = args.protect_session
        && let Ok(uuid) = uuid::Uuid::parse_str(sid)
    {
        protected_ids.push(uuid);
    }

    let policy = SessionCleanupPolicy {
        workspace_root: args.workspace.clone(),
        max_sessions_per_workspace: args.max_sessions,
        max_age_days: args.max_age_days,
        protected_session_ids: protected_ids,
    };

    if args.apply && args.max_sessions.is_none() && args.max_age_days.is_none() {
        println!(
            "Error: cleanup --apply requires at least one selection criterion (--max-sessions or --max-age-days)"
        );
        return Ok(());
    }

    // Collect candidates first (needed for both dry-run and permission check).
    let candidates = manager
        .cleanup_candidates(&policy)
        .context("failed to collect cleanup candidates")?;

    if !args.apply {
        // Dry-run: no writes occur, so no authorization needed.
        print_cleanup_dry_run(&candidates);
        return Ok(());
    }

    // --apply path: evaluate through the permission boundary.
    let engine = storage_permission_engine()?;
    match authorize_cleanup(&engine, &candidates) {
        PermissionDecision::Allow => {
            // Proceed with deletion.
        }
        PermissionDecision::Deny(reason) => {
            println!("Storage cleanup denied by permission rules: {reason}");
            return Ok(());
        }
        PermissionDecision::Ask => {
            // --apply serves as explicit user authorization.
            println!("Authorized by --apply (local maintenance boundary).");
        }
    }

    let report = manager
        .apply_cleanup(&policy)
        .context("failed to apply cleanup")?;
    print_cleanup_report(&report);
    Ok(())
}

fn storage_permission_engine() -> Result<PermissionEngine> {
    storage_permission_engine_from_paths(storage_permission_rule_paths())
}

fn storage_permission_engine_from_paths(paths: Vec<PathBuf>) -> Result<PermissionEngine> {
    let mut engine = PermissionEngine::new();
    for path in paths {
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let config = serde_json::from_str::<serde_json::Value>(&contents)
            .with_context(|| format!("invalid permission rules in {}", path.display()))?;
        engine
            .load_from_config(&config)
            .with_context(|| format!("invalid permission rules in {}", path.display()))?;
    }
    Ok(engine)
}

fn storage_permission_rule_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".talos").join("permissions.json"));
    }
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".talos").join("permissions.json"));
    }
    paths
}

fn run_storage_maintenance(args: &MaintenanceArgs) -> Result<()> {
    if !args.checkpoint && !args.vacuum && !args.reconcile {
        println!("Usage: talos storage maintenance [OPTIONS]");
        println!();
        println!("Options:");
        println!("  --checkpoint   Checkpoint and truncate WAL files");
        println!("  --vacuum       Vacuum databases to reclaim free pages");
        println!("  --reconcile    Reconcile session index drift");
        return Ok(());
    }

    let manager = SessionManager::new().context("failed to create session manager")?;

    if args.checkpoint {
        match manager.checkpoint_index() {
            Ok(()) => println!("Session index: checkpoint completed."),
            Err(e) => eprintln!("Session index checkpoint failed: {e}"),
        }

        let talos_root = resolve_talos_root();
        let mem_db = talos_root.join("memory.db");
        if mem_db.exists() {
            match talos_memory::MemoryStore::open(&mem_db) {
                Ok(store) => match store.checkpoint_truncate() {
                    Ok(()) => println!("Memory DB: checkpoint completed."),
                    Err(e) => eprintln!("Memory DB checkpoint failed: {e}"),
                },
                Err(e) => eprintln!("Memory DB open failed: {e}"),
            }
        }
    }

    if args.vacuum {
        match manager.vacuum_index() {
            Ok(()) => println!("Session index: vacuum completed."),
            Err(e) => eprintln!("Session index vacuum failed: {e}"),
        }

        let talos_root = resolve_talos_root();
        let mem_db = talos_root.join("memory.db");
        if mem_db.exists() {
            match talos_memory::MemoryStore::open(&mem_db) {
                Ok(store) => match store.vacuum() {
                    Ok(()) => println!("Memory DB: vacuum completed."),
                    Err(e) => eprintln!("Memory DB vacuum failed: {e}"),
                },
                Err(e) => eprintln!("Memory DB open failed: {e}"),
            }
        }
    }

    if args.reconcile {
        match manager.reconcile_index() {
            Ok(fixed) => println!(
                "Session index: reconciled {fixed} entr{}.",
                if fixed == 1 { "y" } else { "ies" }
            ),
            Err(e) => eprintln!("Session index reconcile failed: {e}"),
        }
    }

    Ok(())
}

/// Resolved `~/.talos` path.
pub(crate) fn resolve_talos_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".talos")
}

/// Collected status data for a talos root directory.
#[derive(Debug, Default)]
pub(crate) struct StorageStatus {
    pub(crate) talos_root_exists: bool,
    pub(crate) session_count: usize,
    pub(crate) session_total_bytes: u64,
    pub(crate) session_by_workspace: std::collections::HashMap<String, usize>,
    pub(crate) top_sessions: Vec<(String, u64)>,
    pub(crate) index_db_bytes: u64,
    pub(crate) index_wal_bytes: Option<u64>,
    pub(crate) index_shm_bytes: Option<u64>,
    pub(crate) total_forks: usize,
    pub(crate) logs_bytes: u64,
    pub(crate) logs_path: Option<String>,
    pub(crate) cache_bytes: u64,
    pub(crate) memory_db_exists: bool,
    pub(crate) memory_db_bytes: u64,
    pub(crate) memory_item_count: Option<usize>,
}

/// Collect storage status from a given talos root path.
pub(crate) fn collect_storage_status(talos_root: &Path) -> StorageStatus {
    let mut status = StorageStatus::default();

    if !talos_root.exists() {
        return status;
    }
    status.talos_root_exists = true;

    let sessions_dir = talos_root.join("sessions");
    if sessions_dir.exists() {
        let manager = SessionManager::with_dir(sessions_dir.clone());
        if let Ok(sessions) = manager.list_sessions() {
            status.session_count = sessions.len();
            for s in &sessions {
                let ws = if s.workspace_root.is_empty() {
                    s.project.clone()
                } else {
                    s.workspace_root.clone()
                };
                *status.session_by_workspace.entry(ws).or_insert(0) += 1;
            }

            let mut sizes: Vec<(String, u64)> = Vec::new();
            if let Ok(ws_entries) = std::fs::read_dir(&sessions_dir) {
                for ws_entry in ws_entries.flatten() {
                    if !ws_entry.file_type().is_ok_and(|t| t.is_dir()) {
                        continue;
                    }
                    if let Ok(file_entries) = std::fs::read_dir(ws_entry.path()) {
                        for file_entry in file_entries.flatten() {
                            let path = file_entry.path();
                            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                                continue;
                            }
                            if let Ok(meta) = std::fs::metadata(&path) {
                                let size = meta.len();
                                status.session_total_bytes += size;
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    sizes.push((stem.to_string(), size));
                                }
                            }
                        }
                    }
                }
            }
            sizes.sort_by_key(|b| std::cmp::Reverse(b.1));
            status.top_sessions = sizes.into_iter().take(5).collect();
        }

        let index_path = sessions_dir.join("index.db");
        if let Ok(meta) = std::fs::metadata(&index_path) {
            status.index_db_bytes = meta.len();
        }
        let wal_path = sessions_dir.join("index.db-wal");
        if let Ok(meta) = std::fs::metadata(&wal_path) {
            status.index_wal_bytes = Some(meta.len());
        }
        let shm_path = sessions_dir.join("index.db-shm");
        if let Ok(meta) = std::fs::metadata(&shm_path) {
            status.index_shm_bytes = Some(meta.len());
        }

        if let Ok(sessions) = manager.list_sessions() {
            let mut total = 0usize;
            for s in &sessions {
                if let Ok(forks) = manager.get_forks(&s.id.to_string()) {
                    total += forks.len();
                }
            }
            status.total_forks = total;
        }
    }

    let logs_dir = talos_root.join("logs");
    if logs_dir.exists() {
        status.logs_bytes = dir_size(&logs_dir);
        let log_file = logs_dir.join("talos.log");
        if log_file.exists() {
            status.logs_path = Some(log_file.display().to_string());
        }
    }

    let cache_dir = talos_root.join("cache").join("models");
    if cache_dir.exists() {
        status.cache_bytes = dir_size(&cache_dir);
    }

    let mem_db = talos_root.join("memory.db");
    if mem_db.exists() {
        status.memory_db_exists = true;
        if let Ok(meta) = std::fs::metadata(&mem_db) {
            status.memory_db_bytes = meta.len();
        }
        if let Ok(store) = talos_memory::MemoryStore::open(&mem_db)
            && let Ok(count) = store.count()
        {
            status.memory_item_count = Some(count);
        }
    }

    status
}

/// Print storage status to stdout.
pub(crate) fn print_storage_status(status: &StorageStatus) {
    if !status.talos_root_exists {
        println!("Talos root (~/.talos): not found");
        println!();
        println!("No local storage detected. Run talos to create sessions.");
        return;
    }

    println!("=== Sessions ===");
    println!("Total sessions: {}", status.session_count);
    println!(
        "Total JSONL size: {}",
        format_bytes(status.session_total_bytes)
    );
    if !status.session_by_workspace.is_empty() {
        println!("By workspace:");
        let mut ws_entries: Vec<_> = status.session_by_workspace.iter().collect();
        ws_entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        for (ws, count) in ws_entries {
            println!("  {ws}: {count} session(s)");
        }
    }
    if !status.top_sessions.is_empty() {
        println!("Largest sessions:");
        for (id, size) in &status.top_sessions {
            println!("  {id}: {}", format_bytes(*size));
        }
    }

    println!();
    println!("=== Session Index ===");
    if status.index_db_bytes > 0 {
        println!("index.db: {}", format_bytes(status.index_db_bytes));
    } else {
        println!("index.db: not initialized");
    }
    if let Some(wal) = status.index_wal_bytes {
        println!("index.db-wal: {}", format_bytes(wal));
    }
    if let Some(shm) = status.index_shm_bytes {
        println!("index.db-shm: {}", format_bytes(shm));
    }

    println!();
    println!("=== Forks ===");
    println!("Total forks: {}", status.total_forks);

    println!();
    println!("=== Logs ===");
    if status.logs_bytes > 0 {
        println!("Log directory size: {}", format_bytes(status.logs_bytes));
    } else {
        println!("Log directory: empty or not found");
    }
    if let Some(ref path) = status.logs_path {
        println!("Log file: {path}");
    }

    println!();
    println!("=== Model Cache ===");
    if status.cache_bytes > 0 {
        println!("Cache size: {}", format_bytes(status.cache_bytes));
    } else {
        println!("Cache: empty or not found");
    }

    println!();
    println!("=== Memory DB ===");
    if status.memory_db_exists {
        println!("memory.db: {}", format_bytes(status.memory_db_bytes));
        if let Some(count) = status.memory_item_count {
            println!("Memory items: {count}");
        }
    } else {
        println!("Memory DB: not initialized");
    }
}

/// Print dry-run cleanup preview.
pub(crate) fn print_cleanup_dry_run(candidates: &[talos_session::SessionCleanupCandidate]) {
    if candidates.is_empty() {
        println!("No sessions match the cleanup criteria.");
        println!("(dry-run — no files deleted. Use --apply to delete.)");
        return;
    }

    let total_bytes: u64 = candidates.iter().map(|c| c.size_bytes).sum();
    println!(
        "Cleanup preview ({}) candidate(s), {} total:",
        candidates.len(),
        format_bytes(total_bytes)
    );
    println!();
    for c in candidates {
        println!(
            "  {}  workspace={}  size={}  modified={}  reason={}",
            c.id,
            c.workspace_root,
            format_bytes(c.size_bytes),
            c.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            c.reason,
        );
    }
    println!();
    println!("(dry-run — no files deleted. Use --apply to delete.)");
}

/// Print apply cleanup report.
pub(crate) fn print_cleanup_report(report: &talos_session::SessionCleanupReport) {
    if report.removed == 0 {
        println!("No sessions were removed.");
        return;
    }
    println!(
        "Cleanup complete: {} session(s) removed, {} reclaimed.",
        report.removed,
        format_bytes(report.bytes_removed),
    );
}

/// Compute total size of a directory recursively.
fn dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(meta) = std::fs::metadata(&path) {
                total += meta.len();
            }
        }
    }
    total
}

/// Format bytes into a human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_permission::PermissionRule;

    #[test]
    fn storage_cleanup_denied_by_permission_rule() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({
            "rules": [{
                "tool_name": "storage_cleanup",
                "path_pattern": null,
                "decision": "Deny"
            }]
        });
        engine.load_from_config(&config).unwrap();

        let input = serde_json::json!({
            "operation": "storage_cleanup",
            "candidate_count": 1,
            "sessions": ["00000000-0000-0000-0000-000000000000"],
        });
        let decision = engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input);

        assert!(matches!(decision, PermissionDecision::Deny(_)));
    }

    #[test]
    fn storage_cleanup_default_engine_returns_ask() {
        let engine = PermissionEngine::new();
        let input = serde_json::json!({
            "operation": "storage_cleanup",
            "candidate_count": 1,
            "sessions": ["00000000-0000-0000-0000-000000000000"],
        });
        let decision = engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input);

        assert!(matches!(decision, PermissionDecision::Ask));
    }

    #[test]
    fn storage_cleanup_explicit_allow_rule() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({
            "rules": [{
                "tool_name": "storage_cleanup",
                "path_pattern": null,
                "decision": "Allow"
            }]
        });
        engine.load_from_config(&config).unwrap();

        let input = serde_json::json!({
            "operation": "storage_cleanup",
            "candidate_count": 1,
            "sessions": ["00000000-0000-0000-0000-000000000000"],
        });
        let decision = engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input);

        assert!(matches!(decision, PermissionDecision::Allow));
    }

    #[test]
    fn storage_cleanup_authorize_helper_denies_with_rule() {
        let mut engine = PermissionEngine::new();
        let config = serde_json::json!({
            "rules": [{
                "tool_name": "storage_cleanup",
                "path_pattern": null,
                "decision": "Deny"
            }]
        });
        engine.load_from_config(&config).unwrap();

        let empty: &[talos_session::SessionCleanupCandidate] = &[];
        let decision = authorize_cleanup(&engine, empty);

        assert!(matches!(decision, PermissionDecision::Deny(_)));
    }

    #[test]
    fn storage_cleanup_authorize_helper_ask_with_defaults() {
        let engine = PermissionEngine::new();
        let empty: &[talos_session::SessionCleanupCandidate] = &[];
        let decision = authorize_cleanup(&engine, empty);

        assert!(matches!(decision, PermissionDecision::Ask));
    }

    #[test]
    fn storage_cleanup_nature_deny_rule() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Deny("all writes blocked".to_owned()),
        ));

        let input = serde_json::json!({
            "operation": "storage_cleanup",
            "candidate_count": 1,
            "sessions": ["00000000-0000-0000-0000-000000000000"],
        });
        let decision = engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input);

        assert!(matches!(decision, PermissionDecision::Deny(_)));
    }

    #[test]
    fn storage_cleanup_nature_allow_rule_precedes_default_ask() {
        let mut engine = PermissionEngine {
            rules: Vec::new(),
            workspace_root: None,
        };
        engine.add_rule(PermissionRule::new_nature(
            ToolNature::Write,
            None,
            None,
            PermissionDecision::Allow,
        ));

        let input = serde_json::json!({
            "operation": "storage_cleanup",
            "candidate_count": 1,
            "sessions": ["00000000-0000-0000-0000-000000000000"],
        });
        let decision = engine.evaluate_with_nature("storage_cleanup", ToolNature::Write, &input);

        assert!(matches!(decision, PermissionDecision::Allow));
    }

    #[test]
    fn storage_permission_engine_loads_project_rules() {
        let dir = tempfile::tempdir().unwrap();
        let rule_dir = dir.path().join(".talos");
        std::fs::create_dir_all(&rule_dir).unwrap();
        let rule_path = rule_dir.join("permissions.json");
        std::fs::write(
            &rule_path,
            r#"{"rules":[{"tool_name":"storage_cleanup","path_pattern":null,"decision":"Deny"}]}"#,
        )
        .unwrap();

        let engine = storage_permission_engine_from_paths(vec![rule_path]).unwrap();

        let empty: &[talos_session::SessionCleanupCandidate] = &[];
        let decision = authorize_cleanup(&engine, empty);
        assert!(matches!(decision, PermissionDecision::Deny(_)));
    }

    #[test]
    fn storage_permission_engine_rejects_malformed_rules() {
        let dir = tempfile::tempdir().unwrap();
        let rule_path = dir.path().join("permissions.json");
        std::fs::write(&rule_path, r#"{"rules":"not-an-array"}"#).unwrap();

        let result = storage_permission_engine_from_paths(vec![rule_path]);

        assert!(result.is_err());
    }
}
