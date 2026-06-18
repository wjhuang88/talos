//! Session management, workspace resolution, and session-related mode handlers.

use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use talos_session::{IndexError, Session, SessionInfo, SessionManager};
use uuid::Uuid;

use crate::Cli;
use crate::colors;

pub(crate) fn resolve_workspace_root(cli: &Cli) -> Result<PathBuf> {
    match &cli.workspace {
        Some(path) => {
            let abs = if PathBuf::from(path).is_absolute() {
                PathBuf::from(path)
            } else {
                std::env::current_dir()
                    .context("failed to determine working directory")?
                    .join(path)
            };
            if !abs.is_dir() {
                bail!(
                    "workspace path does not exist or is not a directory: {}",
                    abs.display()
                );
            }
            Ok(abs)
        }
        None => std::env::current_dir().context("failed to determine working directory"),
    }
}

pub(crate) fn workspace_display_name(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("default")
        .to_string()
}

pub(crate) fn canonical_workspace_root(workspace_root: &Path) -> String {
    workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf())
        .to_string_lossy()
        .to_string()
}

#[derive(Clone, Copy)]
pub(crate) enum ResumeSelection {
    Disabled,
    Latest,
    Prompt,
}

pub(crate) fn resolve_session_for_workspace(
    manager: &SessionManager,
    workspace_root: &str,
    display_name: &str,
    cli: &Cli,
    resume_selection: ResumeSelection,
    allow_fork: bool,
) -> Result<Session> {
    if allow_fork && let Some(ref source_session_id) = cli.fork {
        return fork_session(manager, source_session_id);
    }

    if let Some(ref session_id) = cli.session {
        return manager
            .resume_session(session_id)
            .with_context(|| format!("failed to resume session {session_id}"));
    }

    if cli.r#continue {
        return resume_latest_workspace_session_or_create(manager, workspace_root, display_name);
    }

    match resume_selection {
        ResumeSelection::Disabled => {}
        ResumeSelection::Latest if cli.resume => {
            return resume_latest_workspace_session_or_create(
                manager,
                workspace_root,
                display_name,
            );
        }
        ResumeSelection::Prompt if cli.resume => {
            return prompt_for_workspace_session_or_create(manager, workspace_root, display_name);
        }
        ResumeSelection::Latest | ResumeSelection::Prompt => {}
    }

    manager
        .create_session(display_name, workspace_root)
        .context("failed to create session")
}

fn resume_latest_workspace_session_or_create(
    manager: &SessionManager,
    workspace_root: &str,
    display_name: &str,
) -> Result<Session> {
    let Some(most_recent) = manager
        .latest_workspace_session(workspace_root)
        .context("failed to list sessions")?
    else {
        return manager
            .create_session(display_name, workspace_root)
            .context("failed to create session");
    };

    manager
        .get_session(&most_recent.id)
        .with_context(|| format!("failed to resume session {}", most_recent.id))
}

fn prompt_for_workspace_session_or_create(
    manager: &SessionManager,
    workspace_root: &str,
    display_name: &str,
) -> Result<Session> {
    let sessions = manager
        .list_workspace_sessions(workspace_root)
        .context("failed to list sessions")?;
    if sessions.is_empty() {
        println!("No existing sessions for this workspace. Creating a new one.");
        return manager
            .create_session(display_name, workspace_root)
            .context("failed to create session");
    }

    println!(
        "{}Available workspace sessions:{}\n",
        colors::BOLD,
        colors::RESET
    );
    for (idx, session) in sessions.iter().enumerate() {
        print_session_selection_row(idx, session);
    }
    print!("\nSelect a session (1-{}): ", sessions.len());
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read input")?;
    let choice: usize = input.trim().parse().context("invalid selection")?;
    if choice < 1 || choice > sessions.len() {
        bail!("selection out of range");
    }
    let selected = &sessions[choice - 1];
    manager
        .get_session(&selected.id)
        .with_context(|| format!("failed to resume session {}", selected.id))
}

fn print_session_selection_row(idx: usize, session: &SessionInfo) {
    let ts = session.timestamp.format("%Y-%m-%d %H:%M");
    println!(
        "  {}. {}{}{} ({}{}{}) {}{} messages | {}",
        idx + 1,
        colors::NORD8,
        session.id,
        colors::RESET,
        colors::NORD14,
        session.project,
        colors::RESET,
        colors::NORD3,
        session.message_count,
        ts,
    );
}

pub(crate) fn run_learned_mode(_cli: Cli) -> Result<()> {
    let db_path = dirs::home_dir()
        .context("failed to find home directory")?
        .join(".talos")
        .join("index.db");

    if !db_path.exists() {
        println!("No evolution data found. Run talos with an agent to start learning.");
        return Ok(());
    }

    let store = talos_evolution::store::KnowledgeStore::open(db_path.to_str().unwrap_or_default())
        .context("failed to open knowledge store")?;

    let patterns = store.get_all_patterns().context("failed to get patterns")?;

    if patterns.is_empty() {
        println!("No patterns learned yet. Use the agent and provide feedback to start learning.");
        return Ok(());
    }

    println!("=== Learned Patterns ===\n");

    for (i, pattern) in patterns.iter().enumerate() {
        let status = if pattern.active { "active" } else { "inactive" };
        println!(
            "{}. [{}] {} (confidence: {:.0}%, evidence: {}, status: {})",
            i + 1,
            pattern.category,
            pattern.description,
            pattern.confidence * 100.0,
            pattern.evidence_count,
            status
        );
        println!("   Instruction: {}", pattern.instruction);
        println!();
    }

    Ok(())
}

pub(crate) fn resolve_prompt(cli_prompt: Option<String>) -> Result<String> {
    if let Some(prompt) = cli_prompt {
        return Ok(prompt);
    }

    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read from stdin")?;
        let trimmed = buffer.trim().to_string();
        if trimmed.is_empty() {
            return Err(anyhow!("stdin is empty"));
        }
        return Ok(trimmed);
    }

    Err(anyhow!(
        "no prompt provided. Usage: talos \"your prompt\" -p, or echo \"prompt\" | talos -p"
    ))
}

pub(crate) fn run_search_mode(cli: Cli) -> Result<()> {
    let query = cli.search.as_ref().expect("search query required");
    let manager = SessionManager::new().context("failed to initialize session manager")?;

    let results = manager.search(query, cli.limit).map_err(|e| match e {
        IndexError::Store(e) => {
            anyhow!("search error: {e}\nHint: run a session first to build the index.")
        }
        IndexError::IoError(e) => anyhow!("I/O error: {e}"),
        IndexError::InvalidUuid(e) => anyhow!("invalid UUID: {e}"),
    })?;

    if results.is_empty() {
        println!("No results found for '{query}'.");
        return Ok(());
    }

    println!(
        "{}Found {} result(s) for '{}':{}\n",
        colors::BOLD,
        results.len(),
        query,
        colors::RESET
    );

    for (i, result) in results.iter().enumerate() {
        let ts = result.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let snippet = crate::registry::highlight_snippet(&result.snippet);
        println!(
            "{:>3}. {}{}{} {}{}{} {}{}{}\n     {}\n",
            i + 1,
            colors::NORD3,
            ts,
            colors::RESET,
            colors::NORD8,
            result.session_id,
            colors::RESET,
            colors::NORD14,
            result.project,
            colors::RESET,
            snippet,
        );
    }

    Ok(())
}

pub(crate) fn run_list_mode(cli: Cli) -> Result<()> {
    let manager = SessionManager::new().context("failed to initialize session manager")?;

    let sessions = manager.list_recent(cli.limit).map_err(|e| match e {
        IndexError::Store(e) => {
            anyhow!("list error: {e}\nHint: run `talos --search <term>` first to build the index.")
        }
        IndexError::IoError(e) => anyhow!("I/O error: {e}"),
        IndexError::InvalidUuid(e) => anyhow!("invalid UUID: {e}"),
    })?;

    if sessions.is_empty() {
        println!("No indexed sessions found. Run `talos --search <term>` to build the index.");
        return Ok(());
    }

    println!(
        "{}Recent sessions ({}):{}\n",
        colors::BOLD,
        sessions.len(),
        colors::RESET
    );

    for (i, session) in sessions.iter().enumerate() {
        let ts = session.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        println!(
            "{:>3}. {}{}{} | {}{}{} | {} messages | {}{}{}",
            i + 1,
            colors::NORD8,
            session.id,
            colors::RESET,
            colors::NORD14,
            session.project,
            colors::RESET,
            session.message_count,
            colors::NORD3,
            ts,
            colors::RESET,
        );
    }

    Ok(())
}

fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<Session> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let source = manager
        .resume_session(source_session_id)
        .with_context(|| format!("failed to load source session {source_session_id}"))?;

    let entries = source
        .read_entries()
        .context("failed to read source entries")?;
    if entries.is_empty() {
        bail!("cannot fork an empty session");
    }

    let fork_entry_id = entries
        .last()
        .expect("entries checked non-empty above")
        .id
        .clone();

    let new_id = Uuid::new_v4();
    let project_dir = manager
        .list_sessions()
        .context("failed to list sessions")?
        .iter()
        .find(|s| s.id.to_string() == source_session_id)
        .map(|s| s.project.clone())
        .unwrap_or_else(|| "default".to_string());

    let sessions_dir = manager.sessions_dir();
    let project_path = sessions_dir.join(&project_dir);
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;

    let new_file_path = project_path.join(format!("{new_id}.jsonl"));

    let mut new_session = Session::new(
        new_id,
        project_dir.clone(),
        source.workspace_root.clone(),
        new_file_path.clone(),
    );

    for entry in &entries {
        let line = serde_json::to_string(entry).map_err(|e| anyhow!("serialize error: {e}"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_file_path)
            .context("failed to create fork file")?;
        writeln!(file, "{line}").context("failed to write fork entry")?;
    }

    new_session
        .fork(&fork_entry_id)
        .context("failed to create fork branch")?;

    if let Ok(mut index) = talos_session::SessionIndex::new(&sessions_dir.join("index.db")) {
        let _ = index.init_schema();
        let _ = index.record_fork(source_session_id, &new_id.to_string(), &fork_entry_id);
        let _ = index.index_session(&new_session);
    }

    eprintln!("Forked session {source_session_id} -> {new_id} (from entry {fork_entry_id})");

    Ok(new_session)
}
