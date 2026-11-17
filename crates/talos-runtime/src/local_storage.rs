//! Explicit composition of existing local persistence implementations (STORE-002 I328 E).
//!
//! The profile is *selection*, not an eager database factory or a new provider
//! hierarchy. Its methods delegate to the same concrete domain stores and
//! constructors already used by Talos; no production host opts in implicitly.
//! In particular, default Session startup reconciliation and custom-directory
//! construction have intentionally different semantics.
//!
//! ARCH-035 owns future RuntimeBuilder/CLI/Desktop/SDK/Server consumption. This
//! module introduces no Host injection, cross-resource transaction, storage
//! migration, or executable vector/embedding/object/rerank backend.

use std::path::{Path, PathBuf};

use talos_evolution::{EvolutionResult, store::KnowledgeStore};
use talos_exploration::{ExplorationError, ExplorationStore};
use talos_memory::{MemoryStore, MemoryStoreError};
use talos_session::{
    PendingSubmissionStore, Session, SessionError, SessionManager, TodoError, TodoRepository,
};

/// Paths and bounded constructor choices for the existing local implementations.
///
/// Merely constructing this value creates **no** files or directories. Stores
/// are opened only by the explicitly named methods below; callers retain
/// control of when optional resources are selected or ignored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalStorageProfile {
    talos_root: PathBuf,
}

impl LocalStorageProfile {
    /// Select the existing local filenames underneath an explicit `.talos` root.
    ///
    /// This is path selection only: no Session index reconciliation or database
    /// creation occurs. It does not override the actual default home-directory
    /// semantics of `SessionManager::new()`.
    pub fn under_talos_root(root: impl Into<PathBuf>) -> Self {
        Self {
            talos_root: root.into(),
        }
    }

    /// Select current local filenames beneath a supplied home directory.
    pub fn under_home_dir(home: impl AsRef<Path>) -> Self {
        Self::under_talos_root(home.as_ref().join(".talos"))
    }

    /// The selected `.talos` root.
    pub fn talos_root(&self) -> &Path {
        &self.talos_root
    }

    /// Current session transcript and metadata directory.
    pub fn sessions_dir(&self) -> PathBuf {
        self.talos_root.join("sessions")
    }

    /// Current semantic-memory SQLite file.
    pub fn memory_db(&self) -> PathBuf {
        self.talos_root.join("memory.db")
    }

    /// Current exploration SQLite/FTS5 file.
    pub fn exploration_db(&self) -> PathBuf {
        self.talos_root.join("exploration.db")
    }

    /// Existing runtime Evolution hook store path.
    ///
    /// The hook's existing `open_default` must still perform its own directory
    /// creation and pattern-size cleanup; this selector does not replace it.
    pub fn evolution_hook_db(&self) -> PathBuf {
        self.talos_root.join("evolution").join("knowledge.db")
    }

    /// Historical CLI learned-mode path, distinct from the Evolution hook.
    ///
    /// The existing CLI uses `.talos/index.db` for this read path; E must not
    /// silently combine it with `.talos/evolution/knowledge.db`.
    pub fn learned_index_db(&self) -> PathBuf {
        self.talos_root.join("index.db")
    }

    /// Use the existing default Session startup path, including its bounded
    /// index/orphan-sidecar reconciliation and diagnostics.
    ///
    /// This is deliberately separate from the custom-directory selector.
    pub fn default_session_manager() -> Result<SessionManager, SessionError> {
        SessionManager::new()
    }

    /// Use the existing custom Session constructor **without** startup
    /// index/orphan-sidecar reconciliation or implicit directory creation.
    pub fn custom_session_manager(&self) -> SessionManager {
        SessionManager::with_dir(self.sessions_dir())
    }

    /// Retain the session-owned pending receipt and generation-fence storage.
    pub fn pending_for_session(session: &Session) -> PendingSubmissionStore {
        PendingSubmissionStore::for_session(session)
    }

    /// Open the existing session-scoped Todo SQLite repository on demand.
    ///
    /// This delegates to `SessionManager::todo_repository` so domain schema
    /// initialization and error semantics stay where they are today.
    pub fn todo_repository(&self) -> Result<TodoRepository, TodoError> {
        self.custom_session_manager().todo_repository()
    }

    /// Open the semantic-memory database only if it already exists.
    ///
    /// Read/prompt injection must not create a missing memory store. An
    /// existing but unreadable/corrupt database still returns its typed error.
    pub fn memory_if_exists(&self) -> Result<Option<MemoryStore>, MemoryStoreError> {
        let path = self.memory_db();
        if !path.exists() {
            return Ok(None);
        }
        MemoryStore::open(path).map(Some)
    }

    /// Explicit opt-in write/consolidation path; preserves the existing
    /// `MemoryStore::open` parent-directory and migration behavior.
    pub fn memory_for_write(&self) -> Result<MemoryStore, MemoryStoreError> {
        MemoryStore::open(self.memory_db())
    }

    /// Read/query exploration only if its database is already initialized.
    pub fn exploration_if_exists(&self) -> Result<Option<ExplorationStore>, ExplorationError> {
        let path = self.exploration_db();
        if !path.exists() {
            return Ok(None);
        }
        ExplorationStore::open(path).map(Some)
    }

    /// Explicit research-ingest path; preserves the existing SQLite/FTS5
    /// constructor and its create/migrate semantics.
    pub fn exploration_for_ingest(&self) -> Result<ExplorationStore, ExplorationError> {
        ExplorationStore::open(self.exploration_db())
    }

    /// Preserve the historical learned-mode read path and its absent-DB no-op.
    ///
    /// This deliberately does not change `EvolutionHookHandler::open_default`
    /// or its separate cleanup and default-directory behavior.
    pub fn learned_knowledge_if_exists(&self) -> EvolutionResult<Option<KnowledgeStore>> {
        let path = self.learned_index_db();
        if !path.exists() {
            return Ok(None);
        }
        KnowledgeStore::open(learned_index_path_utf8(&path)?).map(Some)
    }
}

// Keep the existing fallible conversion after the absent-file check. Separating
// it makes the typed error testable even when macOS rejects non-UTF-8 names
// before they can be created on disk (EILSEQ).
fn learned_index_path_utf8(path: &Path) -> EvolutionResult<&str> {
    path.to_str().ok_or_else(|| {
        talos_evolution::EvolutionError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "learned index path cannot be represented as UTF-8",
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_evolution::KnowledgePersistence;
    use talos_exploration::ExplorationPersistence;
    use talos_memory::MemoryPersistence;

    #[test]
    fn selecting_defaults_and_read_only_missing_stores_creates_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let profile = LocalStorageProfile::under_home_dir(dir.path());
        let root = dir.path().join(".talos");
        assert_eq!(profile.talos_root(), root);
        assert_eq!(profile.sessions_dir(), root.join("sessions"));
        assert_eq!(profile.memory_db(), root.join("memory.db"));
        assert_eq!(profile.exploration_db(), root.join("exploration.db"));
        assert_eq!(
            profile.evolution_hook_db(),
            root.join("evolution/knowledge.db")
        );
        assert_eq!(profile.learned_index_db(), root.join("index.db"));
        assert_ne!(profile.evolution_hook_db(), profile.learned_index_db());
        assert!(!root.exists());

        let sessions = profile.custom_session_manager();
        assert_eq!(sessions.sessions_dir(), profile.sessions_dir());
        assert!(profile.memory_if_exists().expect("memory gate").is_none());
        assert!(
            profile
                .exploration_if_exists()
                .expect("research gate")
                .is_none()
        );
        assert!(
            profile
                .learned_knowledge_if_exists()
                .expect("learned gate")
                .is_none()
        );
        assert!(
            !root.exists(),
            "selection must not create a local DB or directory"
        );
    }

    #[test]
    fn selected_todo_repository_reuses_session_directory_and_domain_contract() {
        let dir = tempfile::tempdir().expect("tempdir");
        let profile = LocalStorageProfile::under_home_dir(dir.path());
        let selected_db = profile.sessions_dir().join("todos.sqlite");
        assert!(!selected_db.exists());

        let store = profile.todo_repository().expect("selected todo repository");
        assert_eq!(store.db_path(), selected_db.as_path());
        let contract: &dyn talos_session::TodoPersistence = &store;
        assert!(
            contract
                .list(uuid::Uuid::new_v4(), talos_session::TodoQuery::default())
                .expect("session scoped empty todo list")
                .is_empty()
        );
        assert!(
            selected_db.is_file(),
            "explicit Todo open must use selected SQLite path"
        );
    }

    #[test]
    fn explicit_local_stores_delegate_to_existing_domain_traits() {
        let dir = tempfile::tempdir().expect("tempdir");
        let profile = LocalStorageProfile::under_home_dir(dir.path());
        let memory = profile.memory_for_write().expect("explicit memory open");
        let memory_ops: &dyn MemoryPersistence = &memory;
        assert_eq!(memory_ops.count().expect("memory count"), 0);
        assert!(
            profile
                .memory_if_exists()
                .expect("existing memory")
                .is_some()
        );

        let exploration = profile
            .exploration_for_ingest()
            .expect("explicit exploration open");
        let exploration_ops: &dyn ExplorationPersistence = &exploration;
        assert_eq!(exploration_ops.count_sources().expect("source count"), 0);
        assert!(
            profile
                .exploration_if_exists()
                .expect("existing exploration")
                .is_some()
        );

        let legacy_path = profile.learned_index_db();
        let learned = KnowledgeStore::open(legacy_path.to_str().expect("UTF-8 tempdir"))
            .expect("explicit learned creation");
        let knowledge_ops: &dyn KnowledgePersistence = &learned;
        assert!(
            knowledge_ops
                .get_all_patterns()
                .expect("patterns")
                .is_empty()
        );
        assert!(
            profile
                .learned_knowledge_if_exists()
                .expect("existing learned")
                .is_some()
        );
    }

    #[cfg(unix)]
    #[test]
    fn learned_query_rejects_non_utf8_paths_without_substituting_a_database() {
        use std::os::unix::ffi::OsStringExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let non_utf8 = std::ffi::OsString::from_vec(b"broken-\xff".to_vec());
        let profile = LocalStorageProfile::under_talos_root(dir.path().join(non_utf8));
        let selected_db = profile.learned_index_db();
        assert!(selected_db.to_str().is_none());

        // Exercise the same typed conversion independently of filesystem name
        // support, including macOS volumes that reject invalid UTF-8 at mkdir.
        match learned_index_path_utf8(&selected_db) {
            Err(talos_evolution::EvolutionError::Io(error)) => {
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
            }
            _ => panic!("non-UTF-8 path conversion must return InvalidInput"),
        }

        if let Err(error) = std::fs::create_dir_all(profile.talos_root()) {
            // macOS commonly rejects non-UTF-8 directory names with EILSEQ (92).
            // The unreadable absent path must remain a no-op, not be redirected
            // to another database or silently converted with to_string_lossy().
            if cfg!(target_os = "macos") && error.raw_os_error() == Some(92) {
                assert!(
                    profile
                        .learned_knowledge_if_exists()
                        .expect("missing learned index stays optional")
                        .is_none()
                );
                assert!(!selected_db.exists());
                assert!(!dir.path().join("index.db").exists());
                return;
            }
            panic!("unexpected failure creating selected dir: {error}");
        }
        std::fs::write(&selected_db, b"non-sqlite fixture").expect("write selected DB fixture");

        match profile.learned_knowledge_if_exists() {
            Err(talos_evolution::EvolutionError::Io(error)) => {
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
            }
            _ => panic!("existing non-UTF-8 path must fail with InvalidInput"),
        }
        assert!(selected_db.exists());
    }
}
