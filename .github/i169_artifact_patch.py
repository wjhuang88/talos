from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE_WORKFLOW = ROOT / ".github/workflows/i169-artifact-core-remediation.yml"


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    if text.count(old) != 1:
        raise RuntimeError(
            f"expected exactly one source block in {path}: count={text.count(old)} marker={old[:120]!r}"
        )
    path.write_text(text.replace(old, new, 1))


def extract_heredoc(target: str) -> str:
    text = CORE_WORKFLOW.read_text()
    marker = f"          cat > {target} <<'EOF'\n"
    start = text.find(marker)
    if start < 0:
        raise RuntimeError(f"missing embedded source block for {target}")
    start += len(marker)
    end_marker = "          EOF\n"
    end = text.find(end_marker, start)
    if end < 0:
        raise RuntimeError(f"unterminated embedded source block for {target}")
    lines = text[start:end].splitlines()
    normalized: list[str] = []
    for line in lines:
        if line and not line.startswith("          "):
            raise RuntimeError(f"unexpected indentation in embedded source for {target}: {line!r}")
        normalized.append(line[10:] if line else "")
    return "\n".join(normalized) + "\n"


# Reuse the reviewed Rust/test payload already embedded in the first temporary workflow.
(ROOT / "crates/talos-session/src/artifacts.rs").write_text(
    extract_heredoc("crates/talos-session/src/artifacts.rs")
)
(ROOT / "crates/talos-session/tests/i169_session_artifact_cleanup.rs").write_text(
    extract_heredoc("crates/talos-session/tests/i169_session_artifact_cleanup.rs")
)

lib = ROOT / "crates/talos-session/src/lib.rs"
replace_once(lib, "mod compact_text;\n", "mod artifacts;\nmod compact_text;\n")
replace_once(
    lib,
    "pub use diagnostic::{ProviderTerminalDiagnostic, ProviderTerminalOutcome, ProviderTerminalSource};\n",
    "pub use artifacts::{\n"
    "    DEFAULT_MAX_ORPHAN_SIDECAR_ENTRIES, DEFAULT_ORPHAN_SIDECAR_MINIMUM_AGE,\n"
    "    OrphanSidecarFailure, OrphanSidecarReconciliationPolicy,\n"
    "    OrphanSidecarReconciliationReport, SessionArtifactCleanupReport,\n"
    "    remove_session_artifacts_for_transcript, remove_session_sidecars_for_transcript,\n"
    "    remove_session_transcript,\n"
    "};\n"
    "pub use diagnostic::{ProviderTerminalDiagnostic, ProviderTerminalOutcome, ProviderTerminalSource};\n",
)
replace_once(
    lib,
    "pub use manager::{\n"
    "    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,\n"
    "    remove_session_artifacts_for_transcript,\n"
    "};\n",
    "pub use manager::{\n"
    "    SessionCleanupCandidate, SessionCleanupPolicy, SessionCleanupReport, SessionManager,\n"
    "};\n",
)

manager = ROOT / "crates/talos-session/src/manager.rs"
text = manager.read_text()
helper_start = text.index("/// Remove the complete artifact set owned by one Session transcript path.\n")
helper_end = text.index("/// Policy for selecting session cleanup candidates.\n", helper_start)
text = text[:helper_start] + text[helper_end:]
old_import = "use crate::{DurableSession, Session, SessionError, SessionInfo};\n"
new_import = (
    "use crate::{\n"
    "    DurableSession, OrphanSidecarReconciliationPolicy, OrphanSidecarReconciliationReport,\n"
    "    Session, SessionArtifactCleanupReport, SessionError, SessionInfo,\n"
    "    remove_session_sidecars_for_transcript, remove_session_transcript,\n"
    "};\n"
)
if text.count(old_import) != 1:
    raise RuntimeError("manager import boundary changed")
text = text.replace(old_import, new_import, 1)

old_new = '''    pub fn new() -> Result<Self, SessionError> {
        let dir = Self::default_sessions_dir()?;
        let manager = Self {
            sessions_dir: dir,
            index: Arc::new(Mutex::new(None)),
            store: Arc::new(CompactTextSessionStore),
            jsonl_store: Arc::new(JsonlSessionStore),
        };
        let _ = manager.reconcile_index();
        Ok(manager)
    }
'''
new_new = '''    pub fn new() -> Result<Self, SessionError> {
        let dir = Self::default_sessions_dir()?;
        let manager = Self {
            sessions_dir: dir,
            index: Arc::new(Mutex::new(None)),
            store: Arc::new(CompactTextSessionStore),
            jsonl_store: Arc::new(JsonlSessionStore),
        };
        if let Err(error) = manager.reconcile_index() {
            eprintln!("Session index reconciliation failed during startup: {error}");
        }
        match manager.reconcile_orphan_sidecars(&OrphanSidecarReconciliationPolicy::default()) {
            Ok(report) => {
                for failure in report.failures {
                    eprintln!(
                        "Session orphan-sidecar reconciliation failed for {} at {}: {}",
                        failure.session_id,
                        failure.path.display(),
                        failure.error,
                    );
                }
            }
            Err(error) => {
                eprintln!("Session orphan-sidecar reconciliation failed during startup: {error}");
            }
        }
        Ok(manager)
    }
'''
if text.count(old_new) != 1:
    raise RuntimeError("SessionManager::new boundary changed")
text = text.replace(old_new, new_new, 1)

delete_start = text.index(
    "    #[allow(clippy::collapsible_if)]\n    pub fn delete_session(&self, id: &Uuid) -> Result<(), SessionError> {\n"
)
delete_end = text.index(
    "    /// Return sessions that would be removed by `policy` without deleting files.\n",
    delete_start,
)
new_delete = '''    /// Delete one discoverable Session through the retryable ownership boundary.
    pub fn delete_session(&self, id: &Uuid) -> Result<(), SessionError> {
        let file_path = self.find_session_file(id)?;
        self.remove_owned_session_artifacts(id, &file_path).map(|_| ())
    }

    /// Roll back all artifacts owned by a prepared or partially created Session.
    ///
    /// Unlike `delete_session`, this does not require the transcript to exist, so
    /// post-identity initialization failures remain recoverable.
    pub fn rollback_session_artifacts(
        &self,
        session: &Session,
    ) -> Result<SessionArtifactCleanupReport, SessionError> {
        self.remove_owned_session_artifacts(&session.id, &session.file_path)
    }

    fn remove_owned_session_artifacts(
        &self,
        id: &Uuid,
        transcript_path: &Path,
    ) -> Result<SessionArtifactCleanupReport, SessionError> {
        let mut report = remove_session_sidecars_for_transcript(transcript_path)?;
        crate::durable::remove_binding_for_session(&self.sessions_dir, id)?;
        let mut guard = self.get_or_create_index().map_err(|error| {
            SessionError::IndexCleanup {
                session_id: *id,
                message: error.to_string(),
            }
        })?;
        if let Some(index) = guard.as_mut() {
            index.delete_session(&id.to_string()).map_err(|error| {
                SessionError::IndexCleanup {
                    session_id: *id,
                    message: error.to_string(),
                }
            })?;
        }
        report.merge(remove_session_transcript(transcript_path)?);
        Ok(report)
    }

    /// Discover and remove safe transcript-less pending SQLite artifacts.
    pub fn reconcile_orphan_sidecars(
        &self,
        policy: &OrphanSidecarReconciliationPolicy,
    ) -> Result<OrphanSidecarReconciliationReport, SessionError> {
        crate::artifacts::reconcile_orphan_sidecars_in_root(&self.sessions_dir, policy)
    }

'''
text = text[:delete_start] + new_delete + text[delete_end:]

apply_owner = text.index("    pub fn apply_cleanup(")
apply_start = text.index("        for candidate in &report.candidates {\n", apply_owner)
apply_end = text.index("\n        Ok(report)", apply_start)
new_apply = '''        for candidate in &report.candidates {
            let cleanup =
                self.remove_owned_session_artifacts(&candidate.id, &candidate.file_path)?;
            report.removed = report.removed.saturating_add(1);
            report.bytes_removed = report.bytes_removed.saturating_add(cleanup.bytes_removed);
        }
'''
text = text[:apply_start] + new_apply + text[apply_end:]
text = text.replace(
    "    /// Total bytes removed from JSONL files.\n",
    "    /// Total bytes removed from complete Session-owned artifact sets.\n",
    1,
)
manager.write_text(text)

error = ROOT / "crates/talos-session/src/error.rs"
replace_once(
    error,
    '''    /// Removing one artifact from a Session-owned artifact set failed.
    #[error("failed to remove session artifact {path}: {source}")]
    ArtifactCleanup {
        /// Exact artifact path whose removal failed.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: std::io::Error,
    },
''',
    '''    /// Removing one artifact from a Session-owned artifact set failed.
    #[error(
        "failed to remove session artifact {path}: {source}; removed={removed:?}; remaining={remaining:?}; retryable=true"
    )]
    ArtifactCleanup {
        /// Exact artifact path whose removal failed.
        path: PathBuf,
        /// Underlying filesystem failure.
        #[source]
        source: std::io::Error,
        /// Paths already removed before the failure.
        removed: Vec<PathBuf>,
        /// Paths still present and safe to retry.
        remaining: Vec<PathBuf>,
    },

    /// Session index cleanup failed while the transcript remained discoverable.
    #[error("failed to remove Session {session_id} from the index: {message}")]
    IndexCleanup {
        /// Session whose supplementary index/fork rows could not be removed.
        session_id: Uuid,
        /// Content-free index diagnostic.
        message: String,
    },

    /// Orphan-sidecar validation or SQLite ownership probing failed.
    #[error("failed to reconcile orphan Session sidecar {path}: {message}")]
    OrphanReconciliation {
        /// Exact validated sidecar path.
        path: PathBuf,
        /// Content-free validation or SQLite diagnostic.
        message: String,
    },
''',
)

storage = ROOT / "crates/talos-cli/src/storage.rs"
replace_once(
    storage,
    "use talos_session::{SessionCleanupCandidate, SessionCleanupPolicy, SessionManager};\n",
    "use talos_session::{\n"
    "    OrphanSidecarReconciliationPolicy, SessionCleanupCandidate, SessionCleanupPolicy,\n"
    "    SessionManager,\n"
    "};\n",
)
text = storage.read_text()
reconcile_start = text.index("    if args.reconcile {\n")
reconcile_end = text.index("\n    Ok(())\n}", reconcile_start)
new_reconcile = '''    if args.reconcile {
        match manager.reconcile_index() {
            Ok(fixed) => println!(
                "Session index: reconciled {fixed} entr{}.",
                if fixed == 1 { "y" } else { "ies" }
            ),
            Err(e) => eprintln!("Session index reconcile failed: {e}"),
        }
        match manager.reconcile_orphan_sidecars(
            &OrphanSidecarReconciliationPolicy::default(),
        ) {
            Ok(report) => {
                println!(
                    "Session sidecars: scanned {}, removed {} set(s) / {} artifact(s) / {} byte(s), skipped {}, failures {}, bounded={}.",
                    report.scanned_entries,
                    report.removed_sets,
                    report.removed_artifacts,
                    report.bytes_removed,
                    report.skipped_sets,
                    report.failures.len(),
                    report.bounded,
                );
                for failure in report.failures {
                    eprintln!(
                        "Session sidecar reconcile failed for {} at {}: {}",
                        failure.session_id,
                        failure.path.display(),
                        failure.error,
                    );
                }
            }
            Err(e) => eprintln!("Session sidecar reconcile failed: {e}"),
        }
    }
'''
storage.write_text(text[:reconcile_start] + new_reconcile + text[reconcile_end:])
