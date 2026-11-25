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
    let child = fork_captured_session(manager, &source, None)?;
    eprintln!("Forked Session {source_session_id} -> {}", child.id);
    Ok(child)
}

pub(crate) fn fork_captured_session(
    manager: &SessionManager,
    source: &Session,
    through_entry: Option<&str>,
) -> Result<Session> {
    let source_lease = talos_session::SessionRuntimeLease::acquire(&source.file_path)
        .context("fork requires idle source runtime ownership")?;
    fork_captured_session_using(manager, source, through_entry, |child| {
        let tail = source.publish_history_fork(
            child,
            &source_lease,
            through_entry,
            std::time::Instant::now() + std::time::Duration::from_secs(30),
            || false,
        )?;
        Ok(tail.id)
    })
}

/// Share metadata registration while letting an active Actor retain source ownership.
pub(crate) fn fork_captured_session_using(
    manager: &SessionManager,
    source: &Session,
    through_entry: Option<&str>,
    publish: impl FnOnce(&Session) -> Result<String>,
) -> Result<Session> {
    fork_session_from_snapshot(manager, source, through_entry, None, publish)
}

pub(crate) fn fork_session_from_snapshot(
    manager: &SessionManager,
    source: &Session,
    through_entry: Option<&str>,
    snapshot: Option<talos_core::session::OrdinaryForkCapture>,
    publish: impl FnOnce(&Session) -> Result<String>,
) -> Result<Session> {
    let source_session_id = source.id;
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

    let receipt_required =
        snapshot.is_none() && crate::history_restore::requires_fork_receipt(source)?;
    let _child_runtime_lease = if receipt_required {
        Some(
            talos_session::SessionRuntimeLease::acquire(&child.file_path)
                .context("cannot acquire fork child runtime ownership")?,
        )
    } else {
        None
    };
    let mut staged = None;
    let fork_entry_id = if receipt_required {
        publish(&child).with_context(|| format!(
            "fork publication did not complete for child {new_id} at {}; preserve any artifacts for reconciliation",
            new_file_path.display(),
        ))?
    } else if let Some(snapshot) = &snapshot {
        staged = Some(crate::history_restore::stage_captured_fork_history(
            &child, snapshot,
        )?);
        snapshot.tail_id.clone()
    } else {
        let (mut guard, _, tail) = crate::history_restore::stage_fork_history_blocking(
            source,
            new_file_path.with_extension("fork-staging"),
            through_entry,
        )
        .context("failed to stage source Session history")?;
        guard
            .acquire_child_ownership(&child)
            .context("cannot acquire legacy fork child runtime ownership")?;
        guard
            .install(&new_file_path)
            .context("failed to install fork history")?;
        staged = Some(guard);
        tail.id
    };
    let fork_result = (|| -> Result<()> {
        if !receipt_required {
            child
                .fork(&fork_entry_id)
                .context("failed to create fork branch")?;
        }

        if receipt_required {
            child
                .restore_history_fork_runtime(
                    _child_runtime_lease
                        .as_ref()
                        .context("missing fork child ownership")?,
                    std::time::Instant::now() + std::time::Duration::from_secs(30),
                )
                .context("failed to restore captured fork runtime identity")?;
        } else if let Some(snapshot) = &snapshot {
            if let Some((provider, model, variant)) = &snapshot.runtime_identity {
                talos_session::PendingSubmissionStore::for_session(&child)
                    .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                        provider,
                        model,
                        variant.as_deref(),
                    ))
                    .context("failed to initialize captured fork runtime identity")?;
            }
        } else {
            match talos_session::PendingSubmissionStore::for_session(source)
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
        }

        manager
            .update_index(source)
            .context("failed to index source Session")?;
        manager
            .update_index(&child)
            .context("failed to index forked Session")?;
        manager
            .record_fork(&source.id, &child.id, &fork_entry_id)
            .context("failed to record fork relationship")?;
        Ok(())
    })();

    // Publication succeeded; the manager now owns complete sidecar/index rollback
    // and its error reporting. Do not let the file guard retry a partial cleanup.
    if let Some(staged) = &mut staged {
        staged.retain();
    }
    if receipt_required && fork_result.is_err() {
        return Err(fork_result.expect_err("checked error").context(format!(
            "fork history was published for child {new_id} at {}; retain artifacts for metadata reconciliation",
            new_file_path.display(),
        )));
    }
    if fork_result.is_err()
        && let Some(staged) = &mut staged
    {
        staged.release_child_ownership();
    }
    match fork_result {
        Ok(()) => Ok(child),
        Err(primary_error) => match manager.rollback_session_artifacts(&child) {
            Ok(report) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; rollback removed {} filesystem artifact(s) / {} byte(s), plus binding and index/fork ownership",
                new_file_path.display(),
                report.removed_artifacts,
                report.bytes_removed,
            )),
            Err(cleanup_error) => Err(anyhow!(
                "failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; cleanup also failed: {cleanup_error}; cleanup is retryable after closing open SQLite handles via `/delete {new_id}` in the TUI while the transcript remains discoverable, or `talos storage maintenance --reconcile` for a transcript-less sidecar",
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
    fn ordinary_host_publishes_captured_bytes_and_absence_after_parent_changes() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "captured".into(),
            })
            .expect("append");
        let lease =
            talos_session::SessionRuntimeLease::acquire(&source.file_path).expect("ownership");
        let capture = source
            .capture_ordinary_fork(
                &lease,
                None,
                std::time::Instant::now() + std::time::Duration::from_secs(5),
                || false,
            )
            .expect("capture");
        source
            .append(&Message::User {
                content: "later".into(),
            })
            .expect("later append");
        talos_session::PendingSubmissionStore::for_session(&source)
            .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                "later", "model", None,
            ))
            .expect("later identity");
        let child = fork_session_from_snapshot(
            &manager,
            &source,
            None,
            Some(talos_core::session::OrdinaryForkCapture {
                bytes: capture.bytes,
                tail_id: capture.tail.id,
                runtime_identity: None,
            }),
            |_| anyhow::bail!("receipt publisher must not run"),
        )
        .expect("publish captured snapshot");
        assert_eq!(child.read_entries().expect("child history").len(), 1);
        assert_eq!(
            child.read_entries().expect("child history")[0].content,
            "captured"
        );
        assert!(
            talos_session::PendingSubmissionStore::for_session(&child)
                .runtime_state()
                .expect("identity")
                .is_none()
        );
        assert_eq!(source.read_entries().expect("parent history").len(), 2);
    }

    #[test]
    fn receipt_fork_metadata_preparation_holds_child_ownership() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "preserved original".into(),
            })
            .expect("append");
        source.compact_archived(0).expect("archive");
        let source_lease =
            talos_session::SessionRuntimeLease::acquire(&source.file_path).expect("source lease");
        let child = fork_captured_session_using(&manager, &source, None, |child| {
            assert!(
                talos_session::SessionRuntimeLease::acquire(&child.file_path).is_err(),
                "host excludes competing child runtime before publication"
            );
            let tail = source.publish_history_fork(
                child,
                &source_lease,
                None,
                std::time::Instant::now() + std::time::Duration::from_secs(30),
                || false,
            )?;
            assert!(
                talos_session::SessionRuntimeLease::acquire(&child.file_path).is_err(),
                "ownership remains after publication"
            );
            Ok(tail.id)
        })
        .expect("publish and register metadata");
        talos_session::SessionRuntimeLease::acquire(&child.file_path)
            .expect("metadata completion releases ownership");
        assert_eq!(
            manager
                .get_forks(&source.id.to_string())
                .expect("lineage")
                .len(),
            1
        );
        assert!(child.file_path.exists());
    }

    #[test]
    fn receipt_host_keeps_captured_absence_when_parent_identity_appears_after_publication() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "captured history".into(),
            })
            .expect("append");
        source.compact_archived(0).expect("archive");
        let lease = talos_session::SessionRuntimeLease::acquire(&source.file_path)
            .expect("source ownership");
        let child = fork_captured_session_using(&manager, &source, None, |child| {
            let tail = source.publish_history_fork(
                child,
                &lease,
                None,
                std::time::Instant::now() + std::time::Duration::from_secs(30),
                || false,
            )?;
            talos_session::PendingSubmissionStore::for_session(&source)
                .initialize_runtime_identity(talos_session::SessionRuntimeIdentity::new(
                    "later-provider",
                    "later-model",
                    None,
                ))?;
            Ok(tail.id)
        })
        .expect("host registers captured metadata");
        assert!(
            talos_session::PendingSubmissionStore::for_session(&source)
                .runtime_state()
                .expect("parent state")
                .is_some()
        );
        assert!(
            talos_session::PendingSubmissionStore::for_session(&child)
                .runtime_state()
                .expect("child state")
                .is_none(),
            "publication-time absence must not become the parent's later identity"
        );
        assert_eq!(child.read_entries().expect("captured history").len(), 1);
    }

    #[test]
    fn startup_fork_restores_archive_and_publishes_independent_child() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("archive", "").expect("source");
        for index in 0..205 {
            source
                .append(&Message::User {
                    content: format!("fact {index}"),
                })
                .expect("append");
        }
        source.compact_archived(50).expect("archive");
        let before = source.snapshot_bytes().expect("before");
        let child = fork_session(&manager, &source.id.to_string()).expect("archived startup fork");
        assert_eq!(child.read_entries().expect("child originals").len(), 205);
        assert_eq!(
            manager
                .get_forks(&source.id.to_string())
                .expect("forks")
                .len(),
            1
        );
        child
            .append(&Message::User {
                content: "child continuation".into(),
            })
            .expect("append child");
        let reopened = manager.get_session(&child.id).expect("reopen child");
        let mut scan = reopened.begin_message_scan().expect("restore child");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !scan.step(deadline, || false).expect("page") {}
        assert_eq!(scan.finish().expect("child history").len(), 206);
        assert_eq!(source.snapshot_bytes().expect("preserved"), before);
        let evidence = source
            .inspect_archive_recovery(
                std::time::Instant::now() + std::time::Duration::from_secs(5),
                || false,
            )
            .expect("source remains recoverable without overlapping child");
        assert_eq!(evidence.entries().len(), 205);
    }

    #[test]
    fn startup_nested_archived_fork_survives_parent_removal() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager
            .create_session("nested-archive", "")
            .expect("source");
        for i in 0..12 {
            source
                .append(&Message::User {
                    content: format!("original {i}"),
                })
                .expect("append");
        }
        source.compact_archived(0).expect("keep0");
        let child = fork_session(&manager, &source.id.to_string()).expect("first fork");
        child
            .append(&Message::User {
                content: "child addition".into(),
            })
            .expect("child append");
        child.compact_archived(1).expect("child keep1");
        let grandchild = fork_session(&manager, &child.id.to_string()).expect("nested fork");
        manager
            .rollback_session_artifacts(&child)
            .expect("remove disposable child");
        manager
            .rollback_session_artifacts(&source)
            .expect("remove disposable parent");
        let reopened = manager
            .get_session(&grandchild.id)
            .expect("reopen grandchild");
        let mut scan = reopened.begin_message_scan().expect("capture");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !scan.step(deadline, || false).expect("page") {}
        assert_eq!(scan.finish().expect("inherited history").len(), 13);
        reopened
            .append(&Message::User {
                content: "grandchild addition".into(),
            })
            .expect("continue");
        assert_eq!(reopened.read_entries().expect("entries").len(), 14);
    }

    #[test]
    fn ordinary_startup_fork_requires_source_ownership_and_retries_after_release() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("source", "").expect("source");
        source
            .append(&Message::User {
                content: "preserve me".into(),
            })
            .expect("append");
        let before = source.snapshot_bytes().expect("source bytes");
        let lease =
            talos_session::SessionRuntimeLease::acquire(&source.file_path).expect("active owner");
        let error = fork_session(&manager, &source.id.to_string()).expect_err("busy source");
        assert!(error.to_string().contains("source runtime ownership"));
        assert_eq!(manager.list_sessions().expect("sessions").len(), 1);
        assert!(
            manager
                .get_forks(&source.id.to_string())
                .expect("lineage")
                .is_empty()
        );
        assert_eq!(source.snapshot_bytes().expect("source preserved"), before);
        drop(lease);
        let child = fork_session(&manager, &source.id.to_string()).expect("retry fork");
        assert_eq!(child.snapshot_bytes().expect("inherited bytes"), before);
    }

    #[test]
    fn default_startup_and_repl_forks_exclude_incomplete_tail_and_can_append_after_reopen() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("partial-tail", "").expect("source");
        source
            .append(&Message::User {
                content: "original".into(),
            })
            .expect("append");
        let target = source.read_entries().expect("entries")[0].id.clone();
        let prefix = source.snapshot_bytes().expect("prefix");
        let mut before = prefix.clone();
        before.extend_from_slice(b"E\t0\t1000\tincomplete\t-\t2097153:x");
        std::fs::write(&source.file_path, &before).expect("incomplete tail");
        for startup in [false, true] {
            let child = if startup {
                fork_session(&manager, &source.id.to_string()).expect("startup fork")
            } else {
                fork_captured_session(&manager, &source, None).expect("default REPL fork")
            };
            assert_eq!(child.snapshot_bytes().expect("child prefix"), prefix);
            let reopened = manager.get_session(&child.id).expect("cold reopen");
            reopened
                .append(&Message::User {
                    content: "child continuation".into(),
                })
                .expect("cold append");
            let entries = manager
                .get_session(&child.id)
                .expect("reopen again")
                .read_entries()
                .expect("entries");
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[1].parent_id.as_deref(), Some(target.as_str()));
            assert_eq!(entries[1].content, "child continuation");
            assert_eq!(source.snapshot_bytes().expect("source unchanged"), before);
        }
    }

    #[test]
    fn selected_fork_jsonl_without_final_newline_can_append_and_reopen() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let id = Uuid::new_v4();
        let source = Session::new(
            id,
            "legacy".into(),
            String::new(),
            manager
                .sessions_dir()
                .join("legacy")
                .join(format!("{id}.jsonl")),
        );
        source
            .append(&Message::User {
                content: "original legacy message".into(),
            })
            .expect("first");
        let target = source.read_entries().expect("source entries")[0].id.clone();
        let mut prefix = source.snapshot_bytes().expect("bytes");
        assert_eq!(prefix.pop(), Some(b'\n'));
        std::fs::write(&source.file_path, &prefix).expect("legacy no final newline");
        let child = fork_captured_session(&manager, &source, Some(&target)).expect("fork");
        assert_eq!(child.snapshot_bytes().expect("unchanged prefix"), prefix);
        child
            .append(&Message::User {
                content: "new child message".into(),
            })
            .expect("append");
        let reopened = manager.get_session(&child.id).expect("reopen");
        let entries = reopened.read_entries().expect("entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, target);
        assert_eq!(entries[1].parent_id.as_deref(), Some(target.as_str()));
        assert_eq!(entries[1].content, "new child message");
        assert!(child.snapshot_bytes().expect("child").starts_with(&prefix));
        assert_eq!(source.snapshot_bytes().expect("source untouched"), prefix);
    }

    #[test]
    fn selected_fork_preserves_original_prefix_and_excludes_later_parent_messages() {
        let root = tempfile::tempdir().expect("root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let source = manager.create_session("selected", "").expect("source");
        source
            .append(&Message::User {
                content: "first".into(),
            })
            .expect("first");
        let id = source.read_entries().expect("entries")[0].id.clone();
        let prefix = String::from_utf8(source.snapshot_bytes().expect("bytes"))
            .expect("utf8")
            .replace("\t5:first", "\t05:first")
            .into_bytes();
        std::fs::write(&source.file_path, &prefix).expect("noncanonical original length");
        source
            .append(&Message::User {
                content: "later parent only".into(),
            })
            .expect("later");
        let before = source.snapshot_bytes().expect("parent");
        let child =
            fork_captured_session(&manager, &source, Some(&id)).expect("fork selected entry");
        assert_eq!(child.snapshot_bytes().expect("exact prefix"), prefix);
        assert_eq!(source.snapshot_bytes().expect("parent unchanged"), before);
        assert_eq!(child.read_entries().expect("child").len(), 1);
        child
            .append(&Message::User {
                content: "child only".into(),
            })
            .expect("continue child");
        let entries = child.read_entries().expect("continued");
        assert_eq!(entries[1].parent_id.as_deref(), Some(id.as_str()));
        assert_eq!(
            source.snapshot_bytes().expect("parent still unchanged"),
            before
        );
        assert!(fork_captured_session(&manager, &source, Some("missing-id")).is_err());
        assert_eq!(
            manager
                .get_forks(&source.id.to_string())
                .expect("fork index")
                .len(),
            1
        );
    }

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

#[cfg(test)]
mod i334_selection_tests {
    use super::*;
    use clap::Parser;
    use std::time::{Duration, UNIX_EPOCH};
    use talos_core::message::Message;

    #[test]
    fn i334_cli_selection_preserves_session_identity_and_workspace_scope() {
        let root = tempfile::tempdir().expect("isolated root");
        let manager = SessionManager::with_dir(root.path().join("sessions"));
        let workspace = root.path().join("workspace");
        let other_workspace = root.path().join("other");
        std::fs::create_dir(&workspace).expect("workspace");
        std::fs::create_dir(&other_workspace).expect("other workspace");
        let workspace = canonical_workspace_root(&workspace);
        let other_workspace = canonical_workspace_root(&other_workspace);

        let seed = |workspace: &str, content: &str, seconds: u64| {
            let session = manager
                .create_session("i334", workspace)
                .expect("create session");
            session
                .append(&Message::User {
                    content: content.into(),
                })
                .expect("append message");
            manager.update_index(&session).expect("index session");
            // Set distinct modification times instead of sleeping or relying
            // on filesystem timestamp resolution or directory iteration order.
            std::fs::OpenOptions::new()
                .write(true)
                .open(&session.file_path)
                .expect("open transcript for timestamp")
                .set_modified(UNIX_EPOCH + Duration::from_secs(seconds))
                .expect("set fixture timestamp");
            session
        };
        let older = seed(&workspace, "older local fact", 1_700_000_000);
        let latest = seed(&workspace, "latest local fact", 1_700_000_010);
        let foreign = seed(&other_workspace, "other workspace fact", 1_700_000_020);
        let original_bytes = [&older, &latest, &foreign]
            .map(|session| session.snapshot_bytes().expect("original transcript"));
        let original_ids = [&older, &latest, &foreign].map(|session| {
            session.read_entries().expect("original entries")[0]
                .id
                .clone()
        });

        let explicit_id = older.id.to_string();
        for (cli, selection, expected) in [
            (
                Cli::parse_from(["talos", "--session", explicit_id.as_str(), "--continue"]),
                ResumeSelection::Latest,
                &older,
            ),
            (
                Cli::parse_from(["talos", "--continue"]),
                ResumeSelection::Disabled,
                &latest,
            ),
            (
                Cli::parse_from(["talos", "--resume"]),
                ResumeSelection::Latest,
                &latest,
            ),
        ] {
            let selected =
                resolve_session_for_workspace(&manager, &workspace, "i334", &cli, selection, true)
                    .expect("resolve actual CLI selection");
            let direct = manager.get_session(&expected.id).expect("direct session");
            assert_eq!(selected.id, expected.id);
            assert_eq!(selected.file_path, expected.file_path);
            assert_eq!(selected.file_path, direct.file_path);
            assert_eq!(selected.file_extension(), "tlog");
            assert_eq!(
                selected.snapshot_bytes().expect("selected bytes"),
                direct.snapshot_bytes().expect("direct bytes")
            );
            assert_eq!(
                selected.read_entries().expect("selected entries")[0].id,
                expected.read_entries().expect("original entries")[0].id
            );
        }

        let indexed = manager.list_recent(10).expect("original index");
        assert_eq!(indexed.len(), 3);
        assert_eq!(manager.list_sessions().expect("all sessions").len(), 3);
        for (index, session) in [&older, &latest, &foreign].into_iter().enumerate() {
            assert_eq!(
                session.snapshot_bytes().expect("unchanged transcript"),
                original_bytes[index]
            );
            assert_eq!(
                session.read_entries().expect("unchanged entries")[0].id,
                original_ids[index]
            );
            assert!(
                indexed
                    .iter()
                    .any(|row| row.id == session.id && row.message_count == 1)
            );
        }
    }

    #[test]
    fn i334_cli_selection_errors_and_empty_resume_do_not_create_session() {
        let root = tempfile::tempdir().expect("isolated root");
        let sessions = root.path().join("not-created");
        let manager = SessionManager::with_dir(sessions.clone());
        let workspace = canonical_workspace_root(root.path());
        let missing = Uuid::new_v4().to_string();
        for id in ["not-a-uuid", missing.as_str()] {
            let cli = Cli::parse_from(["talos", "--session", id, "--continue"]);
            let error = resolve_session_for_workspace(
                &manager,
                &workspace,
                "i334",
                &cli,
                ResumeSelection::Latest,
                true,
            )
            .expect_err("explicit invalid or missing ID must not fall back");
            assert!(matches!(
                error.downcast_ref::<talos_session::SessionError>(),
                Some(talos_session::SessionError::SessionNotFound(_))
            ));
            assert!(!sessions.exists(), "explicit failure creates no store");
        }

        for (cli, selection) in [
            (Cli::parse_from(["talos"]), ResumeSelection::Latest),
            (
                Cli::parse_from(["talos", "--continue"]),
                ResumeSelection::Latest,
            ),
            (
                Cli::parse_from(["talos", "--resume"]),
                ResumeSelection::Latest,
            ),
            (
                Cli::parse_from(["talos", "--resume"]),
                ResumeSelection::Prompt,
            ),
        ] {
            let deferred =
                resolve_session_for_workspace(&manager, &workspace, "i334", &cli, selection, true)
                    .expect("empty workspace selects deferred session");
            assert!(!deferred.persisted);
            assert!(deferred.file_path.starts_with(&sessions));
            assert_eq!(deferred.file_extension(), "tlog");
            assert!(!deferred.file_path.exists());
            assert!(!sessions.exists(), "selection alone creates no store");
        }
    }
}
