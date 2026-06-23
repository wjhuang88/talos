use crate::SessionError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Metadata associated with a session entry.
///
/// Captures optional context about the model, token usage, and working directory
/// at the time the entry was created.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// The model name used to generate this entry (e.g., `claude-sonnet-4-20250514`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Approximate token count for this entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u32>,

    /// Working directory at the time of this entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

impl SessionMetadata {
    /// Returns `true` if all fields are `None`.
    pub(crate) fn is_empty(&self) -> bool {
        self.model.is_none() && self.token_count.is_none() && self.working_directory.is_none()
    }
}

/// A single entry in a session branch.
///
/// Each entry has a unique ID and an optional parent ID that links it to a previous
/// entry, enabling tree-structured branching conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Unique identifier for this entry.
    pub id: String,

    /// ID of the parent entry. `None` for root entries (first entry in a branch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// When this entry was created.
    pub timestamp: DateTime<Utc>,

    /// The role of this entry: `"user"`, `"assistant"`, or `"system"`.
    pub role: String,

    /// The content of this entry.
    pub content: String,

    /// Optional metadata about this entry.
    #[serde(default, skip_serializing_if = "SessionMetadata::is_empty")]
    pub metadata: SessionMetadata,
}

/// A linear branch within a session.
///
/// A branch is a sequence of entries sharing a common root. Branches are created
/// via [`Session::fork`] and are identified by a unique branch ID.
#[derive(Debug, Clone)]
pub struct SessionBranch {
    /// ID of the root entry this branch originates from.
    pub root_id: String,

    /// Ordered entries in this branch.
    pub entries: Vec<SessionEntry>,
}

/// Information about a session, returned when listing sessions.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub id: Uuid,

    /// Human-readable project display name (basename).
    pub project: String,

    /// Stable workspace identity (canonical absolute path).
    pub workspace_root: String,

    /// Preview of the last message in the session.
    pub last_message_preview: String,

    /// When the session file was last modified.
    pub timestamp: DateTime<Utc>,

    /// Total number of entries across all branches.
    pub message_count: usize,
}

/// A single session, backed by a JSONL file, with support for tree-branching.
///
/// Sessions maintain a collection of branches, each identified by a unique ID.
/// The `current_branch` field tracks which branch is active for appending new entries.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub project: String,
    pub workspace_root: String,
    pub created_at: DateTime<Utc>,
    pub file_path: PathBuf,
    pub current_branch: String,
    pub branches: HashMap<String, SessionBranch>,
    pub persisted: bool,
    pub(crate) last_entry_id: Arc<Mutex<Option<String>>>,
    pub(crate) write_lock: Arc<Mutex<()>>,
}

impl Session {
    /// Create a new session with a single empty root branch.
    /// The file path MUST already exist on disk.
    pub fn new(id: Uuid, project: String, workspace_root: String, file_path: PathBuf) -> Self {
        let root_id = Uuid::new_v4().to_string();
        let mut branches = HashMap::new();
        branches.insert(
            root_id.clone(),
            SessionBranch {
                root_id: root_id.clone(),
                entries: Vec::new(),
            },
        );

        Self {
            id,
            project,
            workspace_root,
            created_at: Utc::now(),
            file_path,
            current_branch: root_id,
            branches,
            persisted: true,
            last_entry_id: Arc::new(Mutex::new(None)),
            write_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Create a deferred session — metadata only, no file on disk.
    /// The file is created lazily on the first `ensure_persisted()` call.
    pub fn new_deferred(id: Uuid, project: String, workspace_root: String, file_path: PathBuf) -> Self {
        let mut session = Self::new(id, project, workspace_root, file_path);
        session.persisted = false;
        session
    }

    /// Create the parent directory and empty JSONL file if not yet persisted.
    /// No-op if already persisted (e.g., resumed or forked sessions).
    pub fn ensure_persisted(&mut self) -> Result<(), SessionError> {
        if self.persisted {
            return Ok(());
        }
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(&self.file_path)?;
        self.persisted = true;
        Ok(())
    }

    /// Fork the current session from a specific entry, creating a new branch.
    ///
    /// Returns the ID of the newly created branch.
    ///
    /// # Arguments
    ///
    /// * `from_entry_id` - The ID of the entry to fork from. This entry must exist
    ///   in one of the existing branches.
    ///
    /// # Errors
    ///
    /// Returns [`SessionError::EntryNotFound`] if the entry ID doesn't exist.
    pub fn fork(&mut self, from_entry_id: &str) -> Result<String, SessionError> {
        let all_entries = self.read_entries()?;

        let pos = all_entries
            .iter()
            .position(|e| e.id == from_entry_id)
            .ok_or_else(|| SessionError::EntryNotFound(from_entry_id.to_string()))?;

        let entries_up_to_fork: Vec<SessionEntry> = all_entries[..=pos].to_vec();

        let new_branch_id = Uuid::new_v4().to_string();

        let new_branch = SessionBranch {
            root_id: from_entry_id.to_string(),
            entries: entries_up_to_fork,
        };

        self.branches.insert(new_branch_id.clone(), new_branch);
        self.current_branch = new_branch_id.clone();

        Ok(new_branch_id)
    }

    /// Atomically re-stamp a forked session with a new identity.
    ///
    /// After [`fork`](Self::fork) creates a new branch in memory, callers typically write
    /// the branched entries to a fresh JSONL file under a new [`Uuid`]. This method
    /// updates the in-memory [`Session`] so that subsequent [`append`](Self::append)
    /// and [`append_event`](Self::append_event) calls write to the new file and so the
    /// SQLite index sees a coherent `(id, file_path, branch_id)` triple.
    ///
    /// Without this, the original session's `id`/`file_path` would survive a fork in
    /// memory while the on-disk file moved to a new UUID — the SQLite index would then
    /// either point at the wrong file or fail to locate the fork.
    ///
    /// # Arguments
    ///
    /// * `new_id` - The new session [`Uuid`] (must match the JSONL filename).
    /// * `new_file_path` - The path to the new JSONL file the fork was written to.
    /// * `branch_id` - The branch ID to mark as currently active on the fork.
    pub fn with_fork_identity(&mut self, new_id: Uuid, new_file_path: PathBuf, branch_id: String) {
        self.id = new_id;
        self.file_path = new_file_path;
        self.current_branch = branch_id;
    }

    /// Get a reference to a branch by its ID.
    ///
    /// Returns `None` if the branch ID doesn't exist.
    pub fn get_branch(&self, branch_id: &str) -> Option<&SessionBranch> {
        self.branches.get(branch_id)
    }

    /// Read the session JSONL file as raw bytes, holding the per-session write
    /// lock so concurrent `append` calls cannot produce a torn read.
    ///
    /// Used by fork-style operations that need to copy the source file
    /// byte-for-byte without racing with in-flight event persistence.
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, SessionError> {
        let _guard = self
            .write_lock
            .lock()
            .map_err(|_| SessionError::LockPoisoned)?;
        std::fs::read(&self.file_path).map_err(SessionError::IoError)
    }

    /// List all branch IDs in this session.
    pub fn list_branches(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.branches.keys().cloned().collect();
        ids.sort();
        ids
    }
}
