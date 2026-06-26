use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use talos_memory::{
    ConsolidationConfig, MemoryStore, RuleBasedExtractor, SessionEpisode, consolidate_episodes,
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
}

/// Dispatch a memory command.
pub(crate) fn run_memory_command(cmd: MemoryCommand) -> Result<()> {
    match cmd {
        MemoryCommand::Consolidate { session } => run_consolidate(session.as_deref()),
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
