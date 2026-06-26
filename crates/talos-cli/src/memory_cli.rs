use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use talos_memory::{
    ConsolidationConfig, MemoryStatus, MemoryStore, RetentionCandidate, RetentionPolicy,
    RuleBasedExtractor, SessionEpisode, consolidate_episodes,
};
use talos_session::{SessionEntry, SessionManager};

/// CLI subcommands for memory operations.
#[derive(Subcommand, Clone)]
pub(crate) enum MemoryCommand {
    /// Consolidate session episodes into semantic memory.
    Consolidate {
        /// Specific session UUID to consolidate. If omitted, uses the latest workspace session.
        #[arg(long)]
        session: Option<String>,
    },
    /// Show memory store status (counts and sizes, no content).
    Status,
    /// Report memory retention candidates (dry-run, no deletion).
    Retention {
        #[arg(long)]
        min_confidence: Option<f64>,
        #[arg(long)]
        max_age_days: Option<i64>,
        #[arg(long)]
        unreinforced_only: bool,
    },
}

/// Dispatch a memory command.
pub(crate) fn run_memory_command(cmd: MemoryCommand) -> Result<()> {
    match cmd {
        MemoryCommand::Consolidate { session } => run_consolidate(session.as_deref()),
        MemoryCommand::Status => run_status(),
        MemoryCommand::Retention {
            min_confidence,
            max_age_days,
            unreinforced_only,
        } => run_retention(&RetentionArgs {
            min_confidence,
            max_age_days,
            unreinforced_only,
        }),
    }
}

fn run_consolidate(session_arg: Option<&str>) -> Result<()> {
    let manager = SessionManager::new().context("failed to create session manager")?;

    let session = if let Some(sid) = session_arg {
        manager
            .resume_session(sid)
            .with_context(|| format!("session not found: {sid}"))?
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let workspace = cwd.to_string_lossy().to_string();
        let latest = manager
            .latest_workspace_session(&workspace)
            .context("failed to list workspace sessions")?;
        match latest {
            Some(info) => manager
                .resume_session(&info.id.to_string())
                .with_context(|| format!("failed to resume latest session: {}", info.id))?,
            None => {
                println!("No sessions found in the current workspace.");
                return Ok(());
            }
        }
    };

    let entries = session
        .read_entries()
        .context("failed to read session entries")?;

    if entries.is_empty() {
        println!("Session {} has no entries.", session.id);
        return Ok(());
    }

    let session_id = session.id.to_string();
    let episodes: Vec<SessionEpisode> = entries
        .iter()
        .enumerate()
        .map(|(turn_index, entry)| session_entry_to_episode(&session_id, turn_index, entry))
        .collect();

    let talos_root = resolve_talos_root();
    let mem_db = talos_root.join("memory.db");
    let mut store = MemoryStore::open(&mem_db).context("failed to open memory store")?;

    let config = ConsolidationConfig {
        enabled: true,
        max_candidates_per_session: 20,
    };
    let extractor = RuleBasedExtractor::new();

    let report = consolidate_episodes(&mut store, &episodes, &extractor, &config)
        .context("consolidation failed")?;

    println!("Consolidation report:");
    println!("  Candidates extracted: {}", report.candidates_extracted);
    println!("  Inserted: {}", report.inserted);
    println!("  Duplicates skipped: {}", report.duplicates_skipped);
    println!(
        "  Evidence links created: {}",
        report.evidence_links_created
    );
    if !report.errors.is_empty() {
        println!("  Errors:");
        for err in &report.errors {
            println!("    - {err}");
        }
    }

    Ok(())
}

fn session_entry_to_episode(
    session_id: &str,
    turn_index: usize,
    entry: &SessionEntry,
) -> SessionEpisode {
    SessionEpisode {
        session_id: session_id.to_string(),
        entry_id: entry.id.clone(),
        turn_index,
        role: entry.role.clone(),
        content: entry.content.clone(),
        timestamp: entry.timestamp,
    }
}

fn resolve_talos_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".talos")
}

struct RetentionArgs {
    min_confidence: Option<f64>,
    max_age_days: Option<i64>,
    unreinforced_only: bool,
}

fn run_status() -> Result<()> {
    let talos_root = resolve_talos_root();
    let mem_db = talos_root.join("memory.db");

    if !mem_db.exists() {
        println!("Memory DB: not initialized");
        return Ok(());
    }

    let store = match MemoryStore::open(&mem_db) {
        Ok(s) => s,
        Err(e) => {
            println!("Memory DB: corrupt or unreadable — memory features disabled ({e})");
            return Ok(());
        }
    };

    match store.memory_status() {
        Ok(status) => print_memory_status(&status),
        Err(e) => {
            println!("Memory DB: corrupt or unreadable — memory features disabled ({e})");
        }
    }

    Ok(())
}

fn run_retention(args: &RetentionArgs) -> Result<()> {
    let talos_root = resolve_talos_root();
    let mem_db = talos_root.join("memory.db");

    if !mem_db.exists() {
        println!("Memory DB: not initialized");
        return Ok(());
    }

    let store = match MemoryStore::open(&mem_db) {
        Ok(s) => s,
        Err(e) => {
            println!("Memory DB: corrupt or unreadable — memory features disabled ({e})");
            return Ok(());
        }
    };

    let policy = RetentionPolicy {
        min_confidence: args.min_confidence,
        max_age_days: args.max_age_days,
        unreinforced_only: args.unreinforced_only,
    };

    match store.retention_candidates(&policy) {
        Ok(candidates) => print_retention_candidates(&candidates),
        Err(e) => {
            println!("Memory DB: corrupt or unreadable — memory features disabled ({e})");
        }
    }

    Ok(())
}

fn print_memory_status(status: &MemoryStatus) {
    println!("=== Memory Store Status ===");
    println!("Total items: {}", status.total_items);
    println!("Semantic: {}", status.semantic_count);
    println!("Procedural: {}", status.procedural_count);
    println!("Evidence links: {}", status.evidence_count);
    println!("Entities: {}", status.entity_count);
    if let Some(ref path) = status.db_path {
        println!("DB path: {path}");
        println!("DB size: {}", format_bytes(status.db_size_bytes));
    } else {
        println!("DB: in-memory (no file)");
    }
}

fn print_retention_candidates(candidates: &[RetentionCandidate]) {
    if candidates.is_empty() {
        println!("No retention candidates found.");
        println!("(dry-run — no items deleted. Retention is advisory only.)");
        return;
    }

    println!("Retention candidates ({} found):", candidates.len());
    println!();

    for c in candidates {
        println!(
            "  {}  kind={}  key=\"{}\"  confidence={:.2}  age={}d  evidence={}  reason={}",
            c.id, c.kind, c.key_preview, c.confidence, c.age_days, c.evidence_count, c.reason,
        );
    }

    println!();
    println!("(dry-run — no items deleted. Retention is advisory only.)");
}

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
