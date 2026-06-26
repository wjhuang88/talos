use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use talos_exploration::ExplorationStore;
use talos_exploration::ingestion::{ChunkingConfig, ingest_text};

/// CLI subcommands for exploration operations.
#[derive(Subcommand, Clone)]
pub(crate) enum ExploreCommand {
    /// Ingest a local file into an exploration run.
    Ingest {
        #[arg(long)]
        file: String,
        #[arg(long)]
        run: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    /// Search exploration sources.
    Search {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

/// Dispatch an exploration command.
pub(crate) fn run_explore_command(cmd: ExploreCommand) -> Result<()> {
    match cmd {
        ExploreCommand::Ingest { file, run, title } => {
            run_ingest(&file, run.as_deref(), title.as_deref())
        }
        ExploreCommand::Search { query, limit } => run_search(&query, limit),
    }
}

fn resolve_talos_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".talos")
}

fn run_ingest(file_path: &str, run_id: Option<&str>, title: Option<&str>) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("failed to read file: {file_path}"))?;

    let db_path = resolve_talos_root().join("exploration.db");
    let mut store = ExplorationStore::open(&db_path).context("failed to open exploration store")?;

    let run = match run_id {
        Some(id) => {
            // Use existing run — verify it exists by creating a dummy check.
            // For now, we just use the ID directly.
            id.to_string()
        }
        None => {
            // Create a new run using the filename as query.
            let filename = PathBuf::from(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let new_run = store
                .create_run(&filename, None)
                .context("failed to create exploration run")?;
            new_run.id
        }
    };

    let doc_title: String = title.map(String::from).unwrap_or_else(|| {
        PathBuf::from(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    });

    let config = ChunkingConfig::default();
    let report = ingest_text(&mut store, &run, &doc_title, &content, &config)
        .context("failed to ingest text")?;

    println!("Ingestion complete:");
    println!("  Run ID:      {run}");
    println!("  Source ID:   {}", report.source_id);
    println!("  Chunks:      {}", report.chunks_created);

    Ok(())
}

fn run_search(query: &str, limit: usize) -> Result<()> {
    let db_path = resolve_talos_root().join("exploration.db");

    if !db_path.exists() {
        println!("Exploration DB: not initialized. Ingest some sources first.");
        return Ok(());
    }

    let store = ExplorationStore::open(&db_path).context("failed to open exploration store")?;

    let results = store.search_chunks(query, limit).context("search failed")?;

    if results.is_empty() {
        println!("No results found for: {query}");
        return Ok(());
    }

    println!("Search results for \"{query}\" ({} found):", results.len());
    println!();

    for (i, r) in results.iter().enumerate() {
        println!(
            "  {}. [{}] {}\n     {}",
            i + 1,
            &r.chunk_id[..8],
            r.source_title,
            r.snippet,
        );
    }

    Ok(())
}
