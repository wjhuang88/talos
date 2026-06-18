use crate::jsonl::scan_file;
use crate::sqlite::{IndexError, SearchResult, SessionIndex};
use crate::topology::{workspace_dir_name, workspace_root_from_dir_name};
use crate::{Session, SessionError, SessionInfo};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Manages sessions on disk.
#[derive(Debug, Clone)]
pub struct SessionManager {
    pub(crate) sessions_dir: PathBuf,
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

                let mut session = Session::new(
                    *id,
                    dir_name.clone(),
                    workspace_root_from_dir_name(&dir_name),
                    file_path,
                );
                session.created_at = created_at;

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

                    let (message_count, last_preview) = scan_file(&path)?;

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

                let (message_count, last_preview) = scan_file(&path)?;

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
