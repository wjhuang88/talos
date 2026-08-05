#!/usr/bin/env python3
from pathlib import Path

MODE_RUNNERS = Path("crates/talos-cli/src/mode_runners.rs")
SESSION_SETUP = Path("crates/talos-cli/src/session_setup.rs")
SOURCE_LAYOUT = Path("crates/talos-cli/tests/i169_source_layout.rs")

mode = MODE_RUNNERS.read_text()
old_route = '''    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Latest,
        false,
    )?;
'''
new_route = '''    let session = resolve_session_for_workspace(
        &session_manager,
        &workspace_root_str,
        &display_name,
        &cli,
        ResumeSelection::Latest,
        true,
    )?;
'''
if mode.count(old_route) != 1:
    raise SystemExit("expected exactly one disabled TUI fork route")
MODE_RUNNERS.write_text(mode.replace(old_route, new_route, 1))

setup = SESSION_SETUP.read_text()
marker = "fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<Session> {"
start = setup.find(marker)
if start < 0:
    raise SystemExit("fork_session implementation not found")
if setup[start:].count(marker) != 1:
    raise SystemExit("ambiguous fork_session implementation")

replacement = r'''fn fork_session(manager: &SessionManager, source_session_id: &str) -> Result<Session> {
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
    let source_bytes = source
        .snapshot_bytes()
        .context("failed to snapshot source session")?;

    let new_id = Uuid::new_v4();
    let project_path = source
        .file_path
        .parent()
        .context("source session file has no parent directory")?
        .to_path_buf();
    std::fs::create_dir_all(&project_path).context("failed to create project directory")?;
    let new_file_path = project_path.join(format!("{new_id}.{}", source.file_extension()));

    let fork_result = (|| -> Result<Session> {
        std::fs::write(&new_file_path, &source_bytes)
            .context("failed to clone source session bytes")?;

        let mut child = Session::new(
            new_id,
            source.project.clone(),
            source.workspace_root.clone(),
            new_file_path.clone(),
        );
        child
            .fork(&fork_entry_id)
            .context("failed to create fork branch")?;

        match talos_session::PendingSubmissionStore::for_session(&source)
            .runtime_state()
            .context("failed to read source Session runtime identity")?
        {
            Some(state)
                if state.status
                    == talos_session::SessionRuntimeActivationStatus::Committed =>
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

        let index_path = manager.sessions_dir().join("index.db");
        let mut index = talos_session::SessionIndex::new(&index_path)
            .context("failed to open session index")?;
        index.init_schema().context("failed to initialize session index")?;
        index
            .index_session(&source)
            .context("failed to index source session")?;
        index
            .index_session(&child)
            .context("failed to index forked session")?;
        index
            .record_fork(source_session_id, &new_id.to_string(), &fork_entry_id)
            .context("failed to record fork relationship")?;

        Ok(child)
    })();

    match fork_result {
        Ok(child) => {
            eprintln!(
                "Forked session {source_session_id} -> {new_id} (from entry {fork_entry_id})"
            );
            Ok(child)
        }
        Err(error) => {
            let _ = talos_session::remove_session_artifacts_for_transcript(&new_file_path);
            if let Ok(mut index) =
                talos_session::SessionIndex::new(&manager.sessions_dir().join("index.db"))
            {
                let _ = index.init_schema();
                let _ = index.delete_session(&new_id.to_string());
            }
            Err(error)
        }
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

        let identity = talos_session::SessionRuntimeIdentity::new(
            "openai",
            "o3",
            Some("high-reasoning"),
        );
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
'''
SESSION_SETUP.write_text(setup[:start] + replacement)

layout = SOURCE_LAYOUT.read_text()
layout_test = r'''

#[test]
fn tui_startup_honors_cli_fork_selection() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(crate_root.join("src/mode_runners.rs"))
        .expect("read TUI composition root")
        .replace("\r\n", "\n");
    let call_start = source
        .find("let session = resolve_session_for_workspace(")
        .expect("TUI Session selection call");
    let call_end = call_start
        + source[call_start..]
            .find(")?;")
            .expect("TUI Session selection call end");
    let call = &source[call_start..call_end];
    assert!(
        call.contains("ResumeSelection::Latest,\n        true,"),
        "TUI startup must route --fork through the durable Session clone path"
    );
}
'''
if "fn tui_startup_honors_cli_fork_selection()" in layout:
    raise SystemExit("TUI fork layout guard already exists")
SOURCE_LAYOUT.write_text(layout.rstrip() + layout_test + "\n")
