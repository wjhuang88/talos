from __future__ import annotations

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file_path = Path(path)
    text = file_path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match in {path}, found {count}: {old[:120]!r}")
    file_path.write_text(text.replace(old, new, 1))


def append_once(path: str, sentinel: str, addition: str) -> None:
    file_path = Path(path)
    text = file_path.read_text()
    if sentinel in text:
        raise SystemExit(f"sentinel already present in {path}: {sentinel}")
    file_path.write_text(text.rstrip() + "\n\n" + addition.strip() + "\n")


# H1: make every emitted cleanup instruction executable on the real command surface.
replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    "fn rollback_owned_session_message(\n",
    "pub(crate) fn rollback_owned_session_message(\n",
)
replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    "            \"[Error] {operation}: {primary_error}. Cleanup also failed for Session {session_id} at {transcript}: {cleanup_error}. Cleanup is retryable: close open SQLite handles, retry /delete {session_id} while the transcript remains discoverable, or run talos --storage-maintenance --reconcile for a transcript-less sidecar.\\n\"\n",
    "            \"[Error] {operation}: {primary_error}. Cleanup also failed for Session {session_id} at {transcript}: {cleanup_error}. Cleanup is retryable: close open SQLite handles, retry /delete {session_id} while the transcript remains discoverable, or run talos storage maintenance --reconcile for a transcript-less sidecar.\\n\"\n",
)
replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    "#[allow(clippy::too_many_arguments)]\npub(crate) async fn handle_session_delete(\n",
    "fn resolve_session_delete_target<'a>(\n    sessions: &'a [talos_session::SessionInfo],\n    argument: &str,\n) -> Option<&'a talos_session::SessionInfo> {\n    if let Ok(ordinal) = argument.parse::<usize>()\n        && ordinal >= 1\n        && ordinal <= sessions.len()\n    {\n        return sessions.get(ordinal - 1);\n    }\n\n    let session_id = uuid::Uuid::parse_str(argument).ok()?;\n    sessions.iter().find(|session| session.id == session_id)\n}\n\n#[allow(clippy::too_many_arguments)]\npub(crate) async fn handle_session_delete(\n",
)
replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    "            let target = match arg.parse::<usize>() {\n                Ok(n) if n >= 1 && n <= sessions.len() => &sessions[n - 1],\n                _ => {\n                    let text = format!(\n                        \"[Error] Invalid selection '{arg}'. Use /delete to pick a session.\\n\"\n                    );\n                    send_stream(ui_tx, MessageSource::Error, text);\n                    return;\n                }\n            };\n",
    "            let Some(target) = resolve_session_delete_target(&sessions, arg) else {\n                let text = format!(\n                    \"[Error] Invalid selection '{arg}'. Use /delete to pick a session or /delete <session-uuid>.\\n\"\n                );\n                send_stream(ui_tx, MessageSource::Error, text);\n                return;\n            };\n",
)
replace_once(
    "crates/talos-cli/src/session_setup.rs",
    "                \"failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; cleanup also failed: {cleanup_error}; cleanup is retryable after closing open SQLite handles via --delete or --storage-maintenance --reconcile\",\n",
    "                \"failed to fork Session {source_session_id} into child {new_id} at {}: {primary_error:#}; cleanup also failed: {cleanup_error}; cleanup is retryable after closing open SQLite handles via `/delete {new_id}` in the TUI while the transcript remains discoverable, or `talos storage maintenance --reconcile` for a transcript-less sidecar\",\n",
)

append_once(
    "crates/talos-cli/src/session_handlers.rs",
    "cleanup_failure_instructions_parse_and_execute",
    r'''
#[cfg(test)]
mod cleanup_recovery_command_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn storage_reconcile_instruction_parses_on_the_real_cli_surface() {
        let cli = crate::Cli::try_parse_from([
            "talos",
            "storage",
            "maintenance",
            "--reconcile",
        ])
        .expect("emitted storage maintenance instruction must parse");

        match cli.command {
            Some(crate::TalosCommand::Storage {
                command:
                    crate::storage::StorageCommand::Maintenance {
                        checkpoint,
                        vacuum,
                        reconcile,
                    },
            }) => {
                assert!(!checkpoint);
                assert!(!vacuum);
                assert!(reconcile);
            }
            _ => panic!("emitted instruction must select storage maintenance reconcile"),
        }
    }

    #[tokio::test]
    async fn cleanup_failure_instructions_parse_and_execute() {
        let temp = tempfile::tempdir().expect("create cleanup recovery fixture");
        let sessions_dir = temp.path().join("sessions");
        let workspace = temp.path().join("workspace");
        std::fs::create_dir_all(&workspace).expect("create workspace");
        let workspace_root = canonical_workspace_root(&workspace);
        let manager = talos_session::SessionManager::with_dir(sessions_dir);
        let active = manager
            .create_session("active", &workspace_root)
            .expect("create active Session");
        let target = manager
            .create_session("target", &workspace_root)
            .expect("create target Session");

        let sidecar = target
            .file_path
            .with_file_name(format!("{}.pending.sqlite", target.id));
        std::fs::create_dir(&sidecar).expect("create blocked sidecar path");
        std::fs::write(sidecar.join("held"), b"held").expect("hold sidecar path");

        let diagnostic = rollback_owned_session_message(
            &manager,
            &target,
            "forced runtime construction failure",
            "primary failure",
        );
        assert!(diagnostic.contains(&format!("/delete {}", target.id)));
        assert!(diagnostic.contains("talos storage maintenance --reconcile"));
        assert!(!diagnostic.contains("--storage-maintenance"));
        assert!(target.file_path.exists());

        std::fs::remove_file(sidecar.join("held")).expect("release sidecar path");
        std::fs::remove_dir(&sidecar).expect("remove blocked sidecar directory");

        let (ui_tx, _ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_active_tx, active_rx) = tokio::sync::watch::channel(active.clone());
        handle_session_delete(
            &ui_tx,
            &workspace,
            &manager,
            &active_rx,
            Some(target.id.to_string()),
        )
        .await;

        assert!(manager.get_session(&target.id).is_err());
        assert!(manager.get_session(&active.id).is_ok());
    }
}
''',
)

append_once(
    "crates/talos-cli/tests/i169_source_layout.rs",
    "cleanup_recovery_instructions_match_the_real_command_surface",
    r'''
#[test]
fn cleanup_recovery_instructions_match_the_real_command_surface() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let handlers = fs::read_to_string(crate_root.join("src/session_handlers.rs"))
        .expect("read Session handlers");
    let setup = fs::read_to_string(crate_root.join("src/session_setup.rs"))
        .expect("read Session setup");
    let combined = format!("{handlers}\n{setup}");

    assert!(combined.contains("talos storage maintenance --reconcile"));
    assert!(handlers.contains("/delete <session-uuid>"));
    assert!(!combined.contains("talos --storage-maintenance"));
    assert!(!combined.contains("--storage-maintenance --reconcile"));
    assert!(!combined.contains("via --delete"));
}
''',
)

# M1: preserve a strict per-pass bound while guaranteeing eventual coverage of finite roots.
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "use std::fs;\nuse std::path::{Path, PathBuf};\n",
    "use std::fs::{self, OpenOptions};\nuse std::io::Write;\nuse std::path::{Path, PathBuf};\n",
)
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "pub const DEFAULT_ORPHAN_SIDECAR_MINIMUM_AGE: Duration = Duration::from_secs(300);\n",
    "pub const DEFAULT_ORPHAN_SIDECAR_MINIMUM_AGE: Duration = Duration::from_secs(300);\nconst ORPHAN_SIDECAR_SCAN_STATE_FILE: &str = \".orphan-sidecar-scan-budget\";\n",
)
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "pub(crate) fn reconcile_orphan_sidecars_in_root(\n",
    r'''fn orphan_scan_state_path(canonical_root: &Path) -> PathBuf {
    canonical_root.join(ORPHAN_SIDECAR_SCAN_STATE_FILE)
}

fn read_orphan_scan_limit(canonical_root: &Path, base_limit: usize) -> Result<usize, SessionError> {
    let state_path = orphan_scan_state_path(canonical_root);
    match fs::symlink_metadata(&state_path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.file_type().is_file() => {
            return Err(SessionError::OrphanReconciliation {
                path: state_path,
                message: "scan continuation state must be a regular file".to_string(),
            });
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(base_limit),
        Err(error) => return Err(error.into()),
    }

    let persisted = fs::read_to_string(&state_path)?
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0);
    match persisted {
        Some(limit) => Ok(limit.max(base_limit)),
        None => {
            fs::remove_file(&state_path)?;
            Ok(base_limit)
        }
    }
}

fn persist_next_orphan_scan_limit(
    canonical_root: &Path,
    current_limit: usize,
) -> Result<(), SessionError> {
    let state_path = orphan_scan_state_path(canonical_root);
    if fs::symlink_metadata(&state_path).is_ok_and(|metadata| {
        metadata.file_type().is_symlink() || !metadata.file_type().is_file()
    }) {
        return Err(SessionError::OrphanReconciliation {
            path: state_path,
            message: "scan continuation state must be a regular file".to_string(),
        });
    }
    let next_limit = current_limit
        .saturating_mul(2)
        .max(current_limit.saturating_add(1));
    let mut state = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&state_path)?;
    writeln!(state, "{next_limit}")?;
    state.sync_all()?;
    Ok(())
}

fn clear_orphan_scan_limit(canonical_root: &Path) -> Result<(), SessionError> {
    let state_path = orphan_scan_state_path(canonical_root);
    match fs::remove_file(state_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn reconcile_orphan_sidecars_in_root(
''',
)
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "    let protected: HashSet<Uuid> = policy.protected_session_ids.iter().copied().collect();\n    let limit = policy.max_entries.max(1);\n",
    "    let protected: HashSet<Uuid> = policy.protected_session_ids.iter().copied().collect();\n    let base_limit = policy.max_entries.max(1);\n    let limit = read_orphan_scan_limit(&canonical_root, base_limit)?;\n",
)
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "    'workspaces: for workspace_entry in fs::read_dir(&canonical_root)? {\n        if report.scanned_entries >= limit {\n            report.bounded = true;\n            break;\n        }\n        report.scanned_entries = report.scanned_entries.saturating_add(1);\n        let workspace_entry = workspace_entry?;\n",
    "    'workspaces: for workspace_entry in fs::read_dir(&canonical_root)? {\n        let workspace_entry = workspace_entry?;\n        if workspace_entry.file_name().to_str() == Some(ORPHAN_SIDECAR_SCAN_STATE_FILE) {\n            continue;\n        }\n        if report.scanned_entries >= limit {\n            report.bounded = true;\n            break;\n        }\n        report.scanned_entries = report.scanned_entries.saturating_add(1);\n",
)
replace_once(
    "crates/talos-session/src/artifacts.rs",
    "            Err(error) => report.failures.push(OrphanSidecarFailure {\n                session_id: id,\n                path: set.sqlite,\n                error: error.to_string(),\n            }),\n        }\n    }\n\n    Ok(report)\n}\n",
    "            Err(error) => report.failures.push(OrphanSidecarFailure {\n                session_id: id,\n                path: set.sqlite,\n                error: error.to_string(),\n            }),\n        }\n    }\n\n    if report.bounded {\n        persist_next_orphan_scan_limit(&canonical_root, limit)?;\n    } else {\n        clear_orphan_scan_limit(&canonical_root)?;\n    }\n\n    Ok(report)\n}\n",
)

replace_once(
    "crates/talos-session/src/manager.rs",
    "            Ok(report) => {\n                for failure in report.failures {\n",
    "            Ok(report) => {\n                if report.bounded {\n                    eprintln!(\n                        \"Session orphan-sidecar reconciliation reached its safety bound after scanning {} entries; continuation state was saved. Run `talos storage maintenance --reconcile` to continue.\",\n                        report.scanned_entries,\n                    );\n                }\n                for failure in report.failures {\n",
)
replace_once(
    "crates/talos-cli/src/storage.rs",
    "                for failure in report.failures {\n",
    "                if report.bounded {\n                    println!(\n                        \"Session sidecar scan reached its safety bound; continuation state was saved. Run `talos storage maintenance --reconcile` again to continue.\"\n                    );\n                }\n                for failure in report.failures {\n",
)

append_once(
    "crates/talos-session/tests/i169_session_artifact_cleanup.rs",
    "bounded_orphan_scan_eventually_covers_a_finite_root",
    r'''
#[test]
fn bounded_orphan_scan_eventually_covers_a_finite_root() {
    let dir = tempdir().expect("create temporary directory");
    let sessions = dir.path().join("sessions");
    let workspace = sessions.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let manager = SessionManager::with_dir(sessions.clone());

    for index in 0..24 {
        fs::write(workspace.join(format!("unrelated-{index:02}.txt")), b"unrelated")
            .expect("write unrelated prefix entry");
    }
    let orphan_id = Uuid::new_v4();
    let orphan = workspace.join(format!("{orphan_id}.pending.sqlite-wal"));
    fs::write(&orphan, b"wal").expect("write eventual orphan");

    let policy = OrphanSidecarReconciliationPolicy {
        protected_session_ids: Vec::new(),
        max_entries: 2,
        minimum_age: Duration::ZERO,
    };
    let mut saw_bounded = false;
    let mut max_scanned = 0usize;
    let mut exhausted = false;
    for _ in 0..8 {
        let report = manager
            .reconcile_orphan_sidecars(&policy)
            .expect("bounded continuation pass");
        saw_bounded |= report.bounded;
        max_scanned = max_scanned.max(report.scanned_entries);
        if !report.bounded {
            exhausted = true;
            break;
        }
    }

    assert!(saw_bounded, "the small initial budget must remain bounded");
    assert!(
        max_scanned > policy.max_entries,
        "persisted continuation must expand later bounded passes"
    );
    assert!(exhausted, "a finite root must eventually be fully scanned");
    assert!(!orphan.exists(), "the valid orphan must eventually be reached");
    assert!(
        !sessions.join(".orphan-sidecar-scan-budget").exists(),
        "exhaustive completion resets continuation state"
    );
}
''',
)

print("I169 review follow-up patch applied")
