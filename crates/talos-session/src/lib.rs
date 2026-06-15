//! Talos session management — JSONL-based session logging with tree-branching support.
//!
//! Sessions are stored as append-only JSONL files, organized by working directory.
//! Each line in a JSONL file is a JSON object representing a [`SessionEntry`] with
//! fields for `id`, `parent_id`, `timestamp`, `role`, `content`, and optional `metadata`.
//!
//! # Directory Layout
//!
//! ```text
//! ~/.talos/sessions/
//!   <project>/
//!     <uuid>.jsonl
//! ```
//!
//! # Branching Model
//!
//! Each session supports multiple branches. A branch is a linear sequence of entries
//! rooted at a specific entry. The `fork` method creates a new branch from any existing
//! entry, enabling tree-structured conversation histories.
//!
//! # Crash Safety
//!
//! JSONL is append-only. If a crash occurs, only the last line may be corrupted,
//! which can be detected and skipped during reads.
//!
//! # Backward Compatibility
//!
//! Entries without `id` or `parent_id` fields (from older JSONL files) are treated
//! as part of a single linear branch. They are assigned synthetic IDs on load.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

use sha2::{Digest, Sha256};
use talos_core::message::{AgentEvent, Message};

pub mod sqlite;
pub use sqlite::{ForkInfo, IndexError, SearchResult, SessionIndex};

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An I/O error occurred (file read/write, directory creation, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// A line in the JSONL file is not valid JSON.
    #[error("invalid JSON in session file: {0}")]
    InvalidJson(String),

    /// The requested session was not found.
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),

    /// The requested entry ID was not found in the session.
    #[error("entry not found: {0}")]
    EntryNotFound(String),

    /// The requested branch ID was not found.
    #[error("branch not found: {0}")]
    BranchNotFound(String),

    /// Failed to parse a session file.
    #[error("failed to parse session file: {0}")]
    ParseError(String),
}

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

impl SessionMetadata {
    /// Returns `true` if all fields are `None`.
    fn is_empty(&self) -> bool {
        self.model.is_none() && self.token_count.is_none() && self.working_directory.is_none()
    }
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
    /// Unique session identifier.
    pub id: Uuid,

    /// Human-readable project display name (basename).
    pub project: String,

    /// Stable workspace identity (canonical absolute path).
    pub workspace_root: String,

    /// When the session was created.
    pub created_at: DateTime<Utc>,

    /// Path to the JSONL file backing this session.
    pub file_path: PathBuf,

    /// ID of the currently active branch.
    pub current_branch: String,

    /// All branches in this session.
    pub branches: HashMap<String, SessionBranch>,
}

impl Session {
    /// Create a new session with a single empty root branch.
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
        }
    }

    /// Append a message to the current branch and persist it to the JSONL file.
    pub fn append(&self, message: &Message) -> Result<(), SessionError> {
        let (role, content) = match message {
            Message::User { content } => ("user".to_string(), content.clone()),
            Message::Assistant { content, .. } => ("assistant".to_string(), content.clone()),
            Message::Tool { result } => ("system".to_string(), result.content.clone()),
        };

        let entries = self.read_entries()?;
        let parent_id = entries.last().map(|e| e.id.clone());

        let entry = SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id,
            timestamp: Utc::now(),
            role,
            content,
            metadata: SessionMetadata::default(),
        };

        self.append_entry(&entry)
    }

    /// Append an agent event to the current branch and persist it to the JSONL file.
    pub fn append_event(&self, event: &AgentEvent) -> Result<(), SessionError> {
        let content =
            serde_json::to_string(event).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        let role = "system".to_string();

        let entries = self.read_entries()?;
        let parent_id = entries.last().map(|e| e.id.clone());

        let entry = SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id,
            timestamp: Utc::now(),
            role,
            content,
            metadata: SessionMetadata::default(),
        };

        self.append_entry(&entry)
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

    /// List all branch IDs in this session.
    pub fn list_branches(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.branches.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Read all entries from the session's JSONL file.
    ///
    /// Entries are reconstructed from the JSONL format. Entries without `id` or
    /// `parent_id` (backward compatibility) are assigned synthetic IDs and treated
    /// as a single linear branch.
    pub fn read_entries(&self) -> Result<Vec<SessionEntry>, SessionError> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut synthetic_counter: u64 = 0;

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            // Try to parse as a SessionEntry first (new format)
            if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
                entries.push(entry);
                continue;
            }

            // Try old format: {"type": "message", "data": <Message>}
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
                && value.get("type").and_then(|t| t.as_str()) == Some("message")
                && let Some(data) = value.get("data")
                && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
            {
                let (role, content) = match &msg {
                    Message::User { content } => ("user".to_string(), content.clone()),
                    Message::Assistant { content, .. } => {
                        ("assistant".to_string(), content.clone())
                    }
                    Message::Tool { result } => ("system".to_string(), result.content.clone()),
                };

                let id = format!("synthetic-{synthetic_counter}");
                let parent_id = if synthetic_counter > 0 {
                    Some(format!("synthetic-{}", synthetic_counter - 1))
                } else {
                    None
                };

                entries.push(SessionEntry {
                    id,
                    parent_id,
                    timestamp: Utc::now(),
                    role,
                    content,
                    metadata: SessionMetadata::default(),
                });
                synthetic_counter += 1;
            }
            // Invalid lines are silently skipped (crash-safety guarantee)
        }

        Ok(entries)
    }

    /// Read all messages from the session's JSONL file for the current branch.
    ///
    /// Only entries with role `"user"`, `"assistant"`, or `"system"` that contain
    /// valid message data are returned.
    pub fn read_messages(&self) -> Result<Vec<Message>, SessionError> {
        let entries = self.read_entries()?;
        let mut messages = Vec::new();

        for entry in entries {
            let msg = match entry.role.as_str() {
                "user" => Some(Message::User {
                    content: entry.content,
                }),
                "assistant" => Message::Assistant {
                    content: entry.content,
                    tool_calls: vec![],
                }
                .into(),
                "system" => {
                    // Try to parse as AgentEvent first
                    if serde_json::from_str::<AgentEvent>(&entry.content).is_ok() {
                        // System events are not messages, skip
                        None
                    } else {
                        // Treat as tool result
                        Some(Message::Tool {
                            result: talos_core::message::ToolResult {
                                tool_use_id: "unknown".to_string(),
                                content: entry.content,
                                is_error: false,
                            },
                        })
                    }
                }
                _ => None,
            };

            if let Some(msg) = msg {
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Read all events from the session's JSONL file.
    pub fn read_events(&self) -> Result<Vec<AgentEvent>, SessionError> {
        let entries = self.read_entries()?;
        let mut events = Vec::new();

        for entry in entries {
            if entry.role == "system"
                && let Ok(event) = serde_json::from_str::<AgentEvent>(&entry.content)
            {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Append a raw [`SessionEntry`] to the JSONL file.
    fn append_entry(&self, entry: &SessionEntry) -> Result<(), SessionError> {
        let line =
            serde_json::to_string(entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

/// Compute a filesystem-safe directory name from a workspace root path using SHA-256.
fn workspace_dir_name(workspace_root: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(workspace_root.as_bytes());
    hex::encode(&hasher.finalize()[..8])
}

/// Manages sessions on disk.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions_dir: PathBuf,
    index: Arc<Mutex<Option<SessionIndex>>>,
}

impl SessionManager {
    /// Create a new `SessionManager` with the default sessions directory (`~/.talos/sessions/`).
    pub fn new() -> Result<Self, SessionError> {
        let home =
            std::env::var("HOME").map_err(|e| SessionError::IoError(std::io::Error::other(e)))?;
        let dir = PathBuf::from(home).join(".talos").join("sessions");
        Ok(Self {
            sessions_dir: dir,
            index: Arc::new(Mutex::new(None)),
        })
    }

    /// Create a new `SessionManager` with a custom sessions directory.
    pub fn with_dir(sessions_dir: PathBuf) -> Self {
        Self {
            sessions_dir,
            index: Arc::new(Mutex::new(None)),
        }
    }

    /// Return the root directory used for session JSONL files and the colocated index.
    #[must_use]
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Create a new session for the given project and workspace.
    ///
    /// The session file is created at `~/.talos/sessions/<workspace_dir>/<uuid>.jsonl`,
    /// where `workspace_dir` is a hash of the workspace root path.
    pub fn create_session(
        &self,
        project: &str,
        workspace_root: &str,
    ) -> Result<Session, SessionError> {
        let id = Uuid::new_v4();
        let project_dir = self.sessions_dir.join(workspace_dir_name(workspace_root));
        fs::create_dir_all(&project_dir)?;

        let file_path = project_dir.join(format!("{id}.jsonl"));
        fs::File::create(&file_path)?;

        Ok(Session::new(
            id,
            project.to_string(),
            workspace_root.to_string(),
            file_path,
        ))
    }

    /// Load an existing session by ID.
    ///
    /// Scans all workspace directories (both old basename and new hashed layouts)
    /// for a file matching `<id>.jsonl`.
    pub fn get_session(&self, id: &Uuid) -> Result<Session, SessionError> {
        if !self.sessions_dir.exists() {
            return Err(SessionError::SessionNotFound(*id));
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_dir = entry.path();
            let dir_name = project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let file_path = project_dir.join(format!("{id}.jsonl"));
            if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                let created_at = metadata
                    .modified()
                    .ok()
                    .map(DateTime::<Utc>::from)
                    .unwrap_or_else(Utc::now);

                let workspace_root =
                    if dir_name.len() == 16 && dir_name.chars().all(|c| c.is_ascii_hexdigit()) {
                        dir_name.clone()
                    } else {
                        String::new()
                    };

                let mut session = Session::new(*id, dir_name.clone(), workspace_root, file_path);
                session.created_at = created_at;

                // Load entries from file
                let entries = session.read_entries()?;
                if !entries.is_empty()
                    && let Some(branch) = session.branches.get_mut(&session.current_branch)
                {
                    branch.entries = entries;
                }

                return Ok(session);
            }
        }

        Err(SessionError::SessionNotFound(*id))
    }

    /// List all sessions across all workspace directories.
    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>, SessionError> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_dir = entry.path();
            let dir_name = project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            for file_entry in fs::read_dir(&project_dir)? {
                let file_entry = file_entry?;
                let path = file_entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                    continue;
                }

                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| Uuid::parse_str(s).ok());

                if let Some(id) = file_stem {
                    let metadata = fs::metadata(&path)?;
                    let timestamp = metadata
                        .modified()
                        .ok()
                        .map(DateTime::<Utc>::from)
                        .unwrap_or_else(Utc::now);

                    let (message_count, last_preview) = Self::scan_file(&path)?;

                    sessions.push(SessionInfo {
                        id,
                        project: dir_name.clone(),
                        workspace_root: String::new(),
                        last_message_preview: last_preview,
                        timestamp,
                        message_count,
                    });
                }
            }
        }

        Ok(sessions)
    }

    /// List sessions for one workspace directory.
    pub fn list_workspace_sessions(
        &self,
        workspace_root: &str,
    ) -> Result<Vec<SessionInfo>, SessionError> {
        let workspace_dir = self.sessions_dir.join(workspace_dir_name(workspace_root));
        if !workspace_dir.exists() {
            return Ok(Vec::new());
        }

        Self::scan_workspace_sessions(workspace_root, &workspace_dir)
    }

    /// Return the most recently modified session for one workspace directory.
    pub fn latest_workspace_session(
        &self,
        workspace_root: &str,
    ) -> Result<Option<SessionInfo>, SessionError> {
        let sessions = self.list_workspace_sessions(workspace_root)?;
        Ok(sessions.into_iter().max_by_key(|s| s.timestamp))
    }

    fn scan_workspace_sessions(
        workspace_root: &str,
        workspace_dir: &Path,
    ) -> Result<Vec<SessionInfo>, SessionError> {
        let mut sessions = Vec::new();
        for file_entry in fs::read_dir(workspace_dir)? {
            let file_entry = file_entry?;
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            if let Some(id) = file_stem {
                let metadata = fs::metadata(&path)?;
                let timestamp = metadata
                    .modified()
                    .ok()
                    .map(DateTime::<Utc>::from)
                    .unwrap_or_else(Utc::now);

                let (message_count, last_preview) = Self::scan_file(&path)?;

                sessions.push(SessionInfo {
                    id,
                    project: String::new(),
                    workspace_root: workspace_root.to_string(),
                    last_message_preview: last_preview,
                    timestamp,
                    message_count,
                });
            }
        }

        Ok(sessions)
    }

    /// Resume a session by ID, loading all entries from the JSONL file.
    ///
    /// This is equivalent to [`SessionManager::get_session`] but with a clearer
    /// name for the "resume" use case.
    pub fn resume_session(&self, session_id: &str) -> Result<Session, SessionError> {
        let id = Uuid::parse_str(session_id)
            .map_err(|_| SessionError::SessionNotFound(Uuid::new_v4()))?;
        self.get_session(&id)
    }

    /// Scan a JSONL file and return the entry count and last message preview.
    fn scan_file(path: &Path) -> Result<(usize, String), SessionError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;
        let mut last_preview = String::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            // Try new format: SessionEntry
            if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
                count += 1;
                last_preview = Self::preview_text(&entry.content);
                continue;
            }

            // Try old format: {"type": "message", "data": <Message>}
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
                && value.get("type").and_then(|t| t.as_str()) == Some("message")
            {
                count += 1;
                if let Some(data) = value.get("data")
                    && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
                {
                    let content = match &msg {
                        Message::User { content } => content.clone(),
                        Message::Assistant { content, .. } => content.clone(),
                        Message::Tool { result } => result.content.clone(),
                    };
                    last_preview = Self::preview_text(&content);
                }
            }
        }

        Ok((count, last_preview))
    }

    fn preview_text(content: &str) -> String {
        const MAX_PREVIEW_CHARS: usize = 100;
        let mut chars = content.chars();
        let preview: String = chars.by_ref().take(MAX_PREVIEW_CHARS).collect();
        if chars.next().is_some() {
            format!("{preview}...")
        } else {
            preview
        }
    }

    fn get_or_create_index(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Option<SessionIndex>>, IndexError> {
        let mut guard = self.index.lock().expect("index lock poisoned");
        if guard.is_none() {
            let db_path = self.sessions_dir.join("index.db");

            let index = SessionIndex::new(&db_path)?;
            index.init_schema()?;
            *guard = Some(index);
        }
        Ok(guard)
    }

    /// Perform a full-text search across all indexed session messages.
    ///
    /// Returns results ranked by relevance. The index is created lazily if it
    /// does not exist.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.search(query, limit)
    }

    /// List the most recently updated sessions from the index.
    ///
    /// Returns sessions ordered by last update time, descending.
    pub fn list_recent(&self, limit: usize) -> Result<Vec<SessionInfo>, IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.list_recent(limit)
    }

    /// Update the search index for a session.
    ///
    /// If the index has not been initialized, this is a no-op.
    pub fn update_index(&self, session: &Session) -> Result<(), IndexError> {
        let mut guard = self.get_or_create_index()?;
        let index = guard.as_mut().expect("index just created");
        index.index_session(session)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Self {
            sessions_dir: PathBuf::from(home).join(".talos").join("sessions"),
            index: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{StopReason, ToolCall, ToolResult, Usage};

    fn test_manager() -> SessionManager {
        let dir = tempfile::tempdir().unwrap();
        SessionManager::with_dir(dir.path().to_path_buf())
    }

    #[test]
    fn create_session_creates_file() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        assert!(session.file_path.exists());
        assert_eq!(session.project, "test-project");
        assert!(session.file_path.to_string_lossy().ends_with(".jsonl"));
    }

    #[test]
    fn create_session_uses_correct_directory() {
        let manager = test_manager();
        let session = manager.create_session("my-project", "my-project").unwrap();

        let expected_dir = manager.sessions_dir.join(workspace_dir_name("my-project"));
        assert!(session.file_path.starts_with(expected_dir));
    }

    #[test]
    fn append_and_read_messages() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let msg1 = Message::User {
            content: "Hello!".into(),
        };
        let msg2 = Message::Assistant {
            content: "Hi there!".into(),
            tool_calls: vec![],
        };

        session.append(&msg1).unwrap();
        session.append(&msg2).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], msg1);
        assert_eq!(
            messages[1],
            Message::Assistant {
                content: "Hi there!".into(),
                tool_calls: vec![],
            }
        );
    }

    #[test]
    fn append_and_read_events() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let event1 = AgentEvent::TurnStart;
        let event2 = AgentEvent::TextDelta {
            delta: "Hello".into(),
        };
        let event3 = AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        };

        session.append_event(&event1).unwrap();
        session.append_event(&event2).unwrap();
        session.append_event(&event3).unwrap();

        let events = session.read_events().unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
        assert_eq!(events[2], event3);
    }

    #[test]
    fn read_messages_skips_events() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let msg = Message::User {
            content: "test".into(),
        };
        let event = AgentEvent::TurnStart;

        session.append(&msg).unwrap();
        session.append_event(&event).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], msg);
    }

    #[test]
    fn list_sessions() {
        let manager = test_manager();

        let s1 = manager.create_session("project-a", "").unwrap();
        let s2 = manager.create_session("project-b", "").unwrap();

        // Append a message to s1 so it has a count
        s1.append(&Message::User {
            content: "msg".into(),
        })
        .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);

        let ids: Vec<Uuid> = sessions.iter().map(|s| s.id).collect();
        assert!(ids.contains(&s1.id));
        assert!(ids.contains(&s2.id));

        let s1_info = sessions.iter().find(|s| s.id == s1.id).unwrap();
        assert_eq!(s1_info.message_count, 1);
        assert!(!s1_info.last_message_preview.is_empty());

        let s2_info = sessions.iter().find(|s| s.id == s2.id).unwrap();
        assert_eq!(s2_info.message_count, 0);
    }

    #[test]
    fn list_workspace_sessions_filters_by_workspace() {
        let manager = test_manager();
        let playit = manager.create_session("playit", "playit").unwrap();
        let talos = manager.create_session("talos", "").unwrap();

        playit
            .append(&Message::User {
                content: "playit message".into(),
            })
            .unwrap();
        talos
            .append(&Message::User {
                content: "talos message".into(),
            })
            .unwrap();

        let sessions = manager.list_workspace_sessions("playit").unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, playit.id);
        assert_eq!(sessions[0].last_message_preview, "playit message");
    }

    #[test]
    fn latest_workspace_session_returns_most_recent_session() {
        let manager = test_manager();
        let older = manager.create_session("playit", "playit").unwrap();
        let newer = manager.create_session("playit", "playit").unwrap();

        older
            .append(&Message::User {
                content: "older".into(),
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        newer
            .append(&Message::User {
                content: "newer".into(),
            })
            .unwrap();

        let latest = manager
            .latest_workspace_session("playit")
            .unwrap()
            .expect("expected latest session");

        assert_eq!(latest.id, newer.id);
        assert_eq!(latest.last_message_preview, "newer");
    }

    #[test]
    fn latest_workspace_session_returns_none_for_empty_workspace() {
        let manager = test_manager();

        let latest = manager.latest_workspace_session("missing").unwrap();

        assert!(latest.is_none());
    }

    #[test]
    fn get_session_existing() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();
        let id = session.id;

        let loaded = manager.get_session(&id).unwrap();
        assert_eq!(loaded.id, id);
        // project name may differ from display_name on disk readback (MEM-004 hash dirs)
    }

    #[test]
    fn get_session_not_found() {
        let manager = test_manager();
        let fake_id = Uuid::new_v4();

        let result = manager.get_session(&fake_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SessionError::SessionNotFound(id) => assert_eq!(id, fake_id),
            other => panic!("expected SessionNotFound, got {other:?}"),
        }
    }

    #[test]
    fn invalid_json_lines_are_skipped() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        // Write a valid message
        session
            .append(&Message::User {
                content: "valid".into(),
            })
            .unwrap();

        // Manually append an invalid JSON line
        let mut file = OpenOptions::new()
            .append(true)
            .open(&session.file_path)
            .unwrap();
        writeln!(file, "this is not json").unwrap();

        // Append another valid message
        session
            .append(&Message::User {
                content: "also valid".into(),
            })
            .unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0].clone(),
            Message::User {
                content: "valid".into(),
            }
        );
        assert_eq!(
            messages[1].clone(),
            Message::User {
                content: "also valid".into(),
            }
        );
    }

    #[test]
    fn list_sessions_empty_directory() {
        let manager = test_manager();
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn session_with_tool_calls() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let msg = Message::Assistant {
            content: "Let me check that file.".into(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "read_file".into(),
                input: serde_json::json!({"path": "src/main.rs"}),
            }],
        };

        session.append(&msg).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        // Content is preserved, tool_calls are not in the new format
        assert_eq!(
            messages[0],
            Message::Assistant {
                content: "Let me check that file.".into(),
                tool_calls: vec![],
            }
        );
    }

    #[test]
    fn session_with_tool_result() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let msg = Message::Tool {
            result: ToolResult {
                tool_use_id: "call_1".into(),
                content: "fn main() {}".into(),
                is_error: false,
            },
        };

        session.append(&msg).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        // Tool messages are stored as system role with content
        assert_eq!(
            messages[0],
            Message::Tool {
                result: ToolResult {
                    tool_use_id: "unknown".into(),
                    content: "fn main() {}".into(),
                    is_error: false,
                },
            }
        );
    }

    // === Branching Tests ===

    #[test]
    fn session_entry_with_parent_child_relationship() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        let msg1 = Message::User {
            content: "Hello".into(),
        };
        let msg2 = Message::Assistant {
            content: "Hi".into(),
            tool_calls: vec![],
        };

        session.append(&msg1).unwrap();
        session.append(&msg2).unwrap();

        let entries = session.read_entries().unwrap();
        assert_eq!(entries.len(), 2);

        // First entry has no parent
        assert!(entries[0].parent_id.is_none());
        // Second entry has parent_id pointing to first
        assert_eq!(entries[1].parent_id, Some(entries[0].id.clone()));
    }

    #[test]
    fn fork_creates_new_branch_with_correct_parent_id() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        // Add some messages
        session
            .append(&Message::User {
                content: "msg1".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "reply1".into(),
                tool_calls: vec![],
            })
            .unwrap();
        session
            .append(&Message::User {
                content: "msg2".into(),
            })
            .unwrap();

        let entries = session.read_entries().unwrap();
        let fork_from_id = entries[1].id.clone(); // Fork from the assistant's reply

        let original_branch = session.current_branch.clone();
        let new_branch_id = session.fork(&fork_from_id).unwrap();

        // New branch should be different from original
        assert_ne!(new_branch_id, original_branch);
        assert_eq!(session.current_branch, new_branch_id);

        // New branch should have entries up to and including the fork point
        let new_branch = session.get_branch(&new_branch_id).unwrap();
        assert_eq!(new_branch.entries.len(), 2);
        assert_eq!(new_branch.root_id, fork_from_id);

        let all_entries = session.read_entries().unwrap();
        assert_eq!(all_entries.len(), 3);
    }

    #[test]
    fn list_branches_returns_all_branch_ids() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        // Add a message and fork
        session
            .append(&Message::User {
                content: "msg".into(),
            })
            .unwrap();

        let entries = session.read_entries().unwrap();
        session.fork(&entries[0].id).unwrap();

        let branches = session.list_branches();
        assert_eq!(branches.len(), 2);
    }

    #[test]
    fn resume_session_loads_existing_jsonl_file() {
        let manager = test_manager();

        // Create and populate a session
        let session = manager.create_session("test-project", "").unwrap();
        let session_id = session.id.to_string();

        session
            .append(&Message::User {
                content: "Hello".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "Hi there".into(),
                tool_calls: vec![],
            })
            .unwrap();

        // Resume the session
        let resumed = manager.resume_session(&session_id).unwrap();
        assert_eq!(resumed.id.to_string(), session_id);

        // Entries should be loaded
        let entries = resumed.read_entries().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, "Hello");
        assert_eq!(entries[1].content, "Hi there");
    }

    #[test]
    fn list_sessions_preview_handles_utf8_char_boundary() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();
        let content = "你好！我是 Talos，一个 AI 编程助手。".repeat(8);

        session.append(&Message::User { content }).unwrap();

        let sessions = manager.list_sessions().unwrap();
        let info = sessions
            .iter()
            .find(|info| info.id == session.id)
            .expect("session should be listed");
        assert!(info.last_message_preview.ends_with("..."));
        assert!(
            info.last_message_preview
                .is_char_boundary(info.last_message_preview.len())
        );
    }

    #[test]
    fn list_sessions_old_format_preview_handles_utf8_char_boundary() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();
        let content = "你好！我是 Talos，一个 AI 编程助手。".repeat(8);

        let mut file = OpenOptions::new()
            .append(true)
            .open(&session.file_path)
            .unwrap();
        let old_entry = serde_json::json!({
            "type": "message",
            "data": {
                "role": "user",
                "content": content
            }
        });
        writeln!(file, "{old_entry}").unwrap();

        let sessions = manager.list_sessions().unwrap();
        let info = sessions
            .iter()
            .find(|info| info.id == session.id)
            .expect("session should be listed");
        assert!(info.last_message_preview.ends_with("..."));
        assert!(
            info.last_message_preview
                .is_char_boundary(info.last_message_preview.len())
        );
    }

    #[test]
    fn backward_compatibility_with_old_jsonl_format() {
        let manager = test_manager();
        let session = manager.create_session("test-project", "").unwrap();

        // Manually write old-format JSONL lines
        let mut file = OpenOptions::new()
            .append(true)
            .open(&session.file_path)
            .unwrap();

        let old_entry1 = serde_json::json!({
            "type": "message",
            "data": {
                "role": "user",
                "content": "Old format message 1"
            }
        });
        let old_entry2 = serde_json::json!({
            "type": "message",
            "data": {
                "role": "assistant",
                "content": "Old format message 2"
            }
        });

        writeln!(file, "{}", serde_json::to_string(&old_entry1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&old_entry2).unwrap()).unwrap();

        // Read entries - should parse old format correctly
        let entries = session.read_entries().unwrap();
        assert_eq!(entries.len(), 2);

        // Entries should have synthetic IDs
        assert!(entries[0].id.starts_with("synthetic-"));
        assert!(entries[1].id.starts_with("synthetic-"));

        // Parent-child relationship should be established
        assert!(entries[0].parent_id.is_none());
        assert_eq!(entries[1].parent_id, Some(entries[0].id.clone()));

        // Content should be preserved
        assert_eq!(entries[0].content, "Old format message 1");
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[1].content, "Old format message 2");
        assert_eq!(entries[1].role, "assistant");
    }

    #[test]
    fn session_metadata_serialization() {
        let metadata = SessionMetadata {
            model: Some("claude-sonnet-4".into()),
            token_count: Some(1500),
            working_directory: Some("/home/user/project".into()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let decoded: SessionMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.model, Some("claude-sonnet-4".into()));
        assert_eq!(decoded.token_count, Some(1500));
        assert_eq!(decoded.working_directory, Some("/home/user/project".into()));
    }

    #[test]
    fn session_entry_serialization() {
        let entry = SessionEntry {
            id: "test-id".into(),
            parent_id: Some("parent-id".into()),
            timestamp: Utc::now(),
            role: "user".into(),
            content: "Hello".into(),
            metadata: SessionMetadata {
                model: Some("claude".into()),
                token_count: None,
                working_directory: None,
            },
        };

        let json = serde_json::to_string(&entry).unwrap();
        let decoded: SessionEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, "test-id");
        assert_eq!(decoded.parent_id, Some("parent-id".into()));
        assert_eq!(decoded.role, "user");
        assert_eq!(decoded.content, "Hello");
        assert_eq!(decoded.metadata.model, Some("claude".into()));
    }

    #[test]
    fn session_new_has_single_empty_branch() {
        let id = Uuid::new_v4();
        let session = Session::new(
            id,
            "test".into(),
            String::new(),
            PathBuf::from("/tmp/test.jsonl"),
        );

        assert_eq!(session.branches.len(), 1);
        assert_eq!(session.list_branches().len(), 1);

        let branch = session.get_branch(&session.current_branch).unwrap();
        assert!(branch.entries.is_empty());
    }

    #[test]
    fn fork_from_nonexistent_entry_returns_error() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        let result = session.fork("nonexistent-id");
        assert!(result.is_err());
        match result.unwrap_err() {
            SessionError::EntryNotFound(id) => assert_eq!(id, "nonexistent-id"),
            other => panic!("expected EntryNotFound, got {other:?}"),
        }
    }

    #[test]
    fn list_sessions_scans_directory_correctly() {
        let manager = test_manager();

        // Create sessions in different projects
        let s1 = manager.create_session("project-alpha", "").unwrap();
        let s2 = manager.create_session("project-beta", "").unwrap();

        s1.append(&Message::User {
            content: "First message in alpha".into(),
        })
        .unwrap();

        s2.append(&Message::User {
            content: "First message in beta".into(),
        })
        .unwrap();
        s2.append(&Message::Assistant {
            content: "Reply in beta".into(),
            tool_calls: vec![],
        })
        .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);

        // Verify both sessions are found
        let alpha = sessions.iter().find(|s| s.id == s1.id).unwrap();
        let beta = sessions.iter().find(|s| s.id == s2.id).unwrap();

        assert_eq!(alpha.message_count, 1);
        assert_eq!(beta.message_count, 2);

        // Verify previews
        assert!(
            alpha
                .last_message_preview
                .contains("First message in alpha")
        );
        assert!(beta.last_message_preview.contains("Reply in beta"));
    }

    #[test]
    fn fork_from_specific_entry_includes_correct_history() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        session
            .append(&Message::User {
                content: "msg1".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "reply1".into(),
                tool_calls: vec![],
            })
            .unwrap();
        session
            .append(&Message::User {
                content: "msg2".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "reply2".into(),
                tool_calls: vec![],
            })
            .unwrap();

        let entries = session.read_entries().unwrap();
        assert_eq!(entries.len(), 4);

        let fork_from_id = entries[1].id.clone();
        let new_branch_id = session.fork(&fork_from_id).unwrap();

        let new_branch = session.get_branch(&new_branch_id).unwrap();
        assert_eq!(new_branch.entries.len(), 2);
        assert_eq!(new_branch.entries[0].content, "msg1");
        assert_eq!(new_branch.entries[1].content, "reply1");
    }

    #[test]
    fn fork_from_current_position_includes_all_entries() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        session
            .append(&Message::User {
                content: "only message".into(),
            })
            .unwrap();

        let entries = session.read_entries().unwrap();
        let last_entry_id = entries.last().unwrap().id.clone();

        let new_branch_id = session.fork(&last_entry_id).unwrap();
        let new_branch = session.get_branch(&new_branch_id).unwrap();

        assert_eq!(new_branch.entries.len(), 1);
        assert_eq!(new_branch.entries[0].content, "only message");
    }

    #[test]
    fn forked_session_branch_has_correct_root_id() {
        let manager = test_manager();
        let mut session = manager.create_session("test-project", "").unwrap();

        session
            .append(&Message::User {
                content: "root".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "child".into(),
                tool_calls: vec![],
            })
            .unwrap();

        let entries = session.read_entries().unwrap();
        let fork_point = entries[0].id.clone();

        let new_branch_id = session.fork(&fork_point).unwrap();
        let new_branch = session.get_branch(&new_branch_id).unwrap();

        assert_eq!(new_branch.root_id, fork_point);
    }

    #[test]
    fn arch_s5_update_index_reflects_new_session_in_list_recent() {
        let manager = test_manager();
        let session = manager
            .create_session("arch-s5-list", "")
            .expect("create_session");
        session
            .append(&Message::User {
                content: "hello arch s5".into(),
            })
            .unwrap();

        manager
            .update_index(&session)
            .expect("update_index should succeed");

        let listed = manager
            .list_recent(10)
            .expect("list_recent should succeed after index refresh");
        assert!(
            listed.iter().any(|s| s.id == session.id),
            "list_recent should surface the session after update_index; got {listed:?}"
        );
    }

    #[test]
    fn arch_s5_update_index_reflects_new_session_in_search() {
        let manager = test_manager();
        let session = manager
            .create_session("arch-s5-search", "")
            .expect("create_session");
        session
            .append(&Message::User {
                content: "searchable content alpha".into(),
            })
            .unwrap();

        manager.update_index(&session).expect("update_index");

        let hits = manager
            .search("alpha", 10)
            .expect("search should succeed after index refresh");
        let expected_id = session.id.to_string();
        assert!(
            hits.iter().any(|h| h.session_id == expected_id),
            "FTS5 search should find the session after update_index; got {hits:?}"
        );
    }

    #[test]
    fn arch_s6_fork_identity_sets_new_id_and_path() {
        let mut session = make_session_with_two_entries();
        let original_id = session.id;
        let original_path = session.file_path.clone();

        let new_id = Uuid::new_v4();
        let new_path = original_path
            .parent()
            .unwrap()
            .join(format!("{new_id}.jsonl"));
        let new_branch = Uuid::new_v4().to_string();

        session.with_fork_identity(new_id, new_path.clone(), new_branch.clone());

        assert_eq!(session.id, new_id, "id should be re-stamped to fork UUID");
        assert_ne!(
            session.id, original_id,
            "fork id must differ from source id"
        );
        assert_eq!(
            session.file_path, new_path,
            "file_path should point at the new JSONL"
        );
        assert_eq!(
            session.current_branch, new_branch,
            "current_branch should be the fork's branch id"
        );
    }

    #[test]
    fn arch_s6_fork_index_uses_new_identity() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(dir.path().to_path_buf());
        let mut source = manager.create_session("arch-s6-index", "").unwrap();
        source
            .append(&Message::User {
                content: "source entry".into(),
            })
            .unwrap();
        let source_id = source.id;
        let entries = source.read_entries().unwrap();
        let fork_point = entries.last().unwrap().id.clone();
        let branch_id = source.fork(&fork_point).unwrap();
        let fork_id = Uuid::new_v4();
        let fork_path = dir
            .path()
            .join("arch-s6-index")
            .join(format!("{fork_id}.jsonl"));
        std::fs::create_dir_all(fork_path.parent().unwrap()).unwrap();
        std::fs::write(&fork_path, b"").unwrap();
        source.with_fork_identity(fork_id, fork_path, branch_id);

        manager.update_index(&source).expect("index fork");
        let recent = manager.list_recent(10).expect("list_recent");
        let by_id: std::collections::HashSet<Uuid> = recent.iter().map(|s| s.id).collect();
        assert!(
            by_id.contains(&fork_id),
            "list_recent should contain fork id {fork_id}; got {by_id:?}"
        );
        assert!(
            !by_id.contains(&source_id),
            "list_recent should NOT contain the source id {source_id} under the fork key; got {by_id:?}"
        );
    }

    #[test]
    fn arch_s6_fork_file_receives_subsequent_appends() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(dir.path().to_path_buf());
        let mut session = manager.create_session("arch-s6-append", "").unwrap();
        session
            .append(&Message::User {
                content: "before fork".into(),
            })
            .unwrap();
        let entries = session.read_entries().unwrap();
        let fork_point = entries.last().unwrap().id.clone();
        let branch_id = session.fork(&fork_point).unwrap();
        let fork_id = Uuid::new_v4();
        let fork_path = dir
            .path()
            .join("arch-s6-append")
            .join(format!("{fork_id}.jsonl"));
        std::fs::create_dir_all(fork_path.parent().unwrap()).unwrap();
        std::fs::write(&fork_path, b"").unwrap();
        session.with_fork_identity(fork_id, fork_path.clone(), branch_id);

        session
            .append(&Message::Assistant {
                content: "after fork".into(),
                tool_calls: vec![],
            })
            .expect("append should write to fork file");

        let fork_contents = std::fs::read_to_string(&fork_path).expect("read fork file");
        assert!(
            fork_contents.contains("after fork"),
            "fork file should contain the new entry; got {fork_contents:?}"
        );
    }

    fn make_session_with_two_entries() -> Session {
        let dir = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(dir.path().to_path_buf());
        let session = manager
            .create_session("arch-s6-identity", "")
            .expect("create_session");
        session
            .append(&Message::User {
                content: "first".into(),
            })
            .unwrap();
        session
            .append(&Message::Assistant {
                content: "second".into(),
                tool_calls: vec![],
            })
            .unwrap();
        session
    }
}
