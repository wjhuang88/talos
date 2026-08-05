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

pub(crate) fn workspace_path_display(workspace_root: &Path) -> String {
    let full = workspace_root.to_string_lossy().to_string();
    if let Ok(home) = std::env::var("HOME")
        && !home.is_empty()
        && full.starts_with(&home)
    {
        return format!("~{}", &full[home.len()..]);
    }
    full
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
        .defer_create_session(display_name, workspace_root)
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
            .defer_create_session(display_name, workspace_root)
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
            .defer_create_session(display_name, workspace_root)
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
    let source = manager
        .resume_session(source_session_id)
        .with_context(|| format!("failed to load source Session {source_session_id}"))?;

    let entries = source
        .read_entries()
        .context("failed to read source entries")?;
    if entries.is_empty() {
        bail!("cannot fork an empty Session");
    }
    let fork_entry_id = entries
        .last()
        .expect("entries checked non-empty above")
        .id
        .clone();
    let source_bytes = source
        .snapshot_bytes()
        .context("failed to snapshot source Session")?;

    let new_id = Uuid::new_v4();
    let project_path = source
        .file_path
        .parent()
        .context("source Session file has no parent directory")?
        .to_path_buf();
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;
    let new_file_path = project_path.join(format!("{new_id}.{}", source.file_extension()));
    let mut child = Session::new(
        new_id,
        source.project.clone(),
        source.workspace_root.clone(),
        new_file_path.clone(),
    );

    let fork_result = (|| -> Result<()> {
        std::fs::write(&new_file_path, &source_bytes)
            .context("failed to clone source Session bytes")?;
        child
            .fork(&fork_entry_id)
            .context("failed to create fork branch")?;

        match talos_session::PendingSubmissionStore::for_session(&source)
            .runtime_state()
            .context("failed to read source Session runtime identity")?
        {
            Some(state)
                if state.status == talos_session::SessionRuntimeActivationStatus::Committed =>
            {
                talos_session::PendingSubmissionStore::for_session(&child)
                    .initialize_runtime_identity(state.activation.target)
                    .context("failed to initialize fork runtime identity")?;
            }
            Some(state) => {
                bail!(
                    "source Session activation {} is not committed",
                    state.activation.activation_id
                );
            }
            None => {}
        }

        manager
            .update_index(&source)
            .context("failed to index source Session")?;
        manager
            .update_index(&child)
            .context("failed to index forked Session")?;
        manager
            .record_fork(&source.id, &child.id, &fork_entry_id)
            .context("failed to record fork relationship")?;
        Ok(())
    })();

    match fork_result {
        Ok(()) => {
            eprintln!(
                "Forked Session {source_session_id} -> {new_id} (from entry {fork_entry_id})"
            );
            Ok(child)
        }
        Err(primary_error) => match manager.rollback_session_artifacts(&child) {
            Ok(report) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; rollback removed {} filesystem artifact(s) / {} byte(s), plus binding and index/fork ownership",
                new_file_path.display(),
                report.removed_artifacts,
                report.bytes_removed,
            )),
            Err(cleanup_error) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; cleanup also failed: {cleanup_error}; cleanup is retryable after closing open SQLite handles via --delete or --storage-maintenance --reconcile",
                new_file_path.display(),
            )),
        },
    }
}

#[cfg(test)]
mod fork_tests {
    use super::*;
    use clap::Parser;
    use talos_core::message::Message;

    #[test]
    fn cli_fork_copies_tlog_history_runtime_identity_and_index() {
        let temp = tempfile::tempdir().expect("temporary fork test directory");
        let sessions_dir = temp.path().join("sessions");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("create fork test workspace");
        let workspace_root = canonical_workspace_root(&workspace);
        let manager = SessionManager::with_dir(sessions_dir);

        let source = manager
            .create_session("talos", &workspace_root)
            .expect("create source Session");
        source
            .append(&Message::User {
                content: "fork-source-A".to_string(),
            })
            .expect("append first source entry");
        source
            .append(&Message::User {
                content: "fork-source-B".to_string(),
            })
            .expect("append second source entry");

        let identity =
            talos_session::SessionRuntimeIdentity::new("openai", "o3", Some("high-reasoning"));
        talos_session::PendingSubmissionStore::for_session(&source)
            .initialize_runtime_identity(identity.clone())
            .expect("initialize source runtime identity");

        let source_id = source.id.to_string();
        let source_bytes = source.snapshot_bytes().expect("snapshot source bytes");
        let source_entries = source.read_entries().expect("read source entries");
        let source_tip = source_entries
            .last()
            .expect("source entries are non-empty")
            .id
            .clone();
        assert_eq!(source.file_extension(), "tlog");

        let cli = Cli::parse_from(["talos", "--fork", source_id.as_str()]);
        let child = resolve_session_for_workspace(
            &manager,
            &workspace_root,
            "talos",
            &cli,
            ResumeSelection::Latest,
            true,
        )
        .expect("resolve CLI fork");

        assert_ne!(child.id, source.id);
        assert_eq!(child.file_extension(), source.file_extension());
        assert_eq!(
            source.snapshot_bytes().expect("resnapshot source bytes"),
            source_bytes,
            "fork must not mutate the source transcript"
        );
        assert_eq!(
            child.snapshot_bytes().expect("snapshot child bytes"),
            source_bytes,
            "fork must preserve the source store format byte-for-byte"
        );

        let child_entries = child.read_entries().expect("read child entries");
        assert_eq!(child_entries.len(), 2);
        assert_eq!(child_entries[0].content, "fork-source-A");
        assert_eq!(child_entries[1].content, "fork-source-B");

        let child_runtime = talos_session::PendingSubmissionStore::for_session(&child)
            .runtime_state()
            .expect("read child runtime state")
            .expect("child runtime identity");
        assert_eq!(
            child_runtime.status,
            talos_session::SessionRuntimeActivationStatus::Committed
        );
        assert_eq!(child_runtime.activation.target, identity);

        let forks = manager.get_forks(&source_id).expect("read fork index");
        assert_eq!(forks.len(), 1);
        assert_eq!(forks[0].forked_session_id, child.id.to_string());
        assert_eq!(forks[0].fork_entry_id, source_tip);

        let indexed = manager.list_recent(20).expect("list indexed Sessions");
        assert!(indexed.iter().any(|session| {
            session.id == child.id && session.message_count == child_entries.len()
        }));
    }
}
