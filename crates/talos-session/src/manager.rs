use crate::jsonl::scan_file;
use crate::sqlite::{ForkInfo, IndexError, SearchResult, SessionIndex};
use crate::todo::{TodoError, TodoRepository};
use crate::topology::{workspace_dir_name, workspace_root_from_dir_name};
use crate::{Session, SessionError, SessionInfo};
use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Policy for selecting session cleanup candidates.
#[derive(Debug, Clone, Default)]
pub struct SessionCleanupPolicy {
    /// Workspace root to limit cleanup to. `None` scans all workspaces.
    pub workspace_root: Option<String>,
    /// Keep at most this many newest sessions after excluding protected IDs.
    pub max_sessions_per_workspace: Option<usize>,
    /// Delete sessions older than this many days after excluding protected IDs.
    pub max_age_days: Option<i64>,
    /// Session IDs that must never be selected for cleanup.
    pub protected_session_ids: Vec<Uuid>,
}

/// A session selected by a cleanup policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCleanupCandidate {
    /// Session identifier.
    pub id: Uuid,
    /// Workspace root associated with the session.
    pub workspace_root: String,
    /// JSONL file path to remove if cleanup is applied.
    pub file_path: PathBuf,
    /// File size in bytes at selection time.
    pub size_bytes: u64,
    /// Last modified timestamp used for retention decisions.
    pub timestamp: DateTime<Utc>,
    /// Human-readable reason this session was selected.
    pub reason: String,
}

/// Result of applying a session cleanup policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionCleanupReport {
    /// Candidates selected by the policy.
    pub candidates: Vec<SessionCleanupCandidate>,
    /// Number of candidates actually removed.
    pub removed: usize,
    /// Total bytes removed from JSONL files.
    pub bytes_removed: u64,
}

/// Manages sessions on disk.
#[derive(Debug, Clone)]
pub struct SessionManager {
    pub(crate) sessions_dir: PathBuf,
    index: Arc<Mutex<Option<SessionIndex>>>,
}

impl SessionManager {
    /// Create a new `SessionManager` with the default sessions directory (`~/.talos/sessions/`).
    pub fn new() -> Result<Self, SessionError> {
        let dir = Self::default_sessions_dir()?;
        let manager = Self {
            sessions_dir: dir,
            index: Arc::new(Mutex::new(None)),
        };
        // Repair any drift between on-disk JSONL and the SQLite index on
        // startup. Errors are non-fatal: a future operation that needs the
        // index will surface a precise cause.
        let _ = manager.reconcile_index();
        Ok(manager)
    }

    /// Return the default sessions directory without opening indexes or reconciling state.
    ///
    /// # Errors
    ///
    /// Returns an error if `$HOME` is unavailable.
    pub fn default_sessions_dir() -> Result<PathBuf, SessionError> {
        let home =
            std::env::var("HOME").map_err(|e| SessionError::IoError(std::io::Error::other(e)))?;
        Ok(PathBuf::from(home).join(".talos").join("sessions"))
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

    /// Open the colocated session todo repository and initialize its schema.
    ///
    /// The repository is session-scoped by data, not by database file: all todo rows carry a
    /// `session_id` and share one SQLite file under the sessions directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the SQLite database cannot be opened or initialized.
    pub fn todo_repository(&self) -> Result<TodoRepository, TodoError> {
        let repo = TodoRepository::new(&self.sessions_dir.join("todos.sqlite"))?;
        repo.init_schema()?;
        Ok(repo)
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

    /// Prepare a new session without creating the file on disk.
    ///
    /// Returns a [`Session`] with `persisted = false`. The JSONL file is
    /// not created until [`Session::ensure_persisted`] is called (triggered
    /// automatically on the first message append).
    pub fn defer_create_session(
        &self,
        project: &str,
        workspace_root: &str,
    ) -> Result<Session, SessionError> {
        let id = Uuid::new_v4();
        let project_dir = self.sessions_dir.join(workspace_dir_name(workspace_root));
        let file_path = project_dir.join(format!("{id}.jsonl"));

        Ok(Session::new_deferred(
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

    /// Checkpoint the session index WAL and truncate it where possible.
    pub fn checkpoint_index(&self) -> Result<(), IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.checkpoint_truncate()
    }

    /// Vacuum the session index database.
    pub fn vacuum_index(&self) -> Result<(), IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.vacuum()
    }

    /// Return forks originating from the given session ID.
    pub fn get_forks(&self, session_id: &str) -> Result<Vec<ForkInfo>, IndexError> {
        let guard = self.get_or_create_index()?;
        let index = guard.as_ref().expect("index just created");
        index.get_forks(session_id)
    }

    #[allow(clippy::collapsible_if)]
    pub fn reconcile_index(&self) -> Result<usize, IndexError> {
        let mut guard = self.get_or_create_index()?;
        let index = guard.as_mut().expect("index just created");

        let mut fixed = 0usize;

        let indexed_ids: std::collections::HashSet<String> =
            index.list_all_session_ids()?.into_iter().collect();

        let mut on_disk_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        if self.sessions_dir.exists() {
            for ws_entry in fs::read_dir(&self.sessions_dir)? {
                let ws_entry = ws_entry?;
                if !ws_entry.file_type()?.is_dir() {
                    continue;
                }
                let ws_dir = ws_entry.path();
                let workspace_root = workspace_root_from_dir_name(
                    &ws_dir.file_name().unwrap_or_default().to_string_lossy(),
                );

                for file_entry in fs::read_dir(&ws_dir)? {
                    let file_entry = file_entry?;
                    let path = file_entry.path();
                    if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                        continue;
                    }
                    let stem = match path.file_stem().and_then(|s| s.to_str()) {
                        Some(s) => s.to_string(),
                        None => continue,
                    };
                    on_disk_ids.insert(stem.clone());

                    let existing = index.get_session_info(&stem)?;
                    let (msg_count, _) = scan_file(&path).unwrap_or((0, String::new()));
                    let needs_reindex = match &existing {
                        None => true,
                        Some(info) => info.message_count != msg_count,
                    };
                    if needs_reindex && Uuid::parse_str(&stem).is_ok() {
                        if let Ok(id) = Uuid::parse_str(&stem) {
                            let project = ws_dir
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let mut session =
                                Session::new(id, project, workspace_root.to_string(), path.clone());
                            if let Ok(entries) = session.read_entries() {
                                if let Some(branch) =
                                    session.branches.get_mut(&session.current_branch)
                                {
                                    branch.entries = entries;
                                }
                            }
                            index.index_session(&session)?;
                            fixed += 1;
                        }
                    }
                }
            }
        }

        for orphan_id in indexed_ids.difference(&on_disk_ids) {
            index.delete_session(orphan_id)?;
            fixed += 1;
        }

        Ok(fixed)
    }

    #[allow(clippy::collapsible_if)]
    pub fn delete_session(&self, id: &Uuid) -> Result<(), SessionError> {
        let file_path = self.find_session_file(id)?;
        if file_path.exists() {
            fs::remove_file(&file_path)?;
        }
        if let Ok(mut guard) = self.get_or_create_index() {
            if let Some(index) = guard.as_mut() {
                let _ = index.delete_session(&id.to_string());
            }
        }
        Ok(())
    }

    /// Return sessions that would be removed by `policy` without deleting files.
    pub fn cleanup_candidates(
        &self,
        policy: &SessionCleanupPolicy,
    ) -> Result<Vec<SessionCleanupCandidate>, SessionError> {
        let mut by_workspace = self.collect_cleanup_sessions(policy)?;
        let protected: std::collections::HashSet<Uuid> =
            policy.protected_session_ids.iter().copied().collect();
        let cutoff = policy
            .max_age_days
            .map(|days| Utc::now() - Duration::days(days.max(0)));

        let mut candidates = Vec::new();
        for (workspace_root, sessions) in by_workspace.iter_mut() {
            sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));

            for session in sessions.iter() {
                if protected.contains(&session.id) {
                    continue;
                }
                if let Some(cutoff) = cutoff
                    && session.timestamp < cutoff
                {
                    candidates.push(SessionCleanupCandidate {
                        id: session.id,
                        workspace_root: workspace_root.clone(),
                        file_path: session.file_path.clone(),
                        size_bytes: session.size_bytes,
                        timestamp: session.timestamp,
                        reason: format!(
                            "older than {} day(s)",
                            policy.max_age_days.unwrap_or_default().max(0)
                        ),
                    });
                }
            }

            if let Some(max_sessions) = policy.max_sessions_per_workspace {
                let mut unprotected: Vec<_> = sessions
                    .iter()
                    .filter(|session| !protected.contains(&session.id))
                    .collect();
                unprotected
                    .sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| a.id.cmp(&b.id)));
                for session in unprotected.into_iter().skip(max_sessions) {
                    if candidates
                        .iter()
                        .any(|candidate| candidate.id == session.id)
                    {
                        continue;
                    }
                    candidates.push(SessionCleanupCandidate {
                        id: session.id,
                        workspace_root: workspace_root.clone(),
                        file_path: session.file_path.clone(),
                        size_bytes: session.size_bytes,
                        timestamp: session.timestamp,
                        reason: format!("exceeds max_sessions_per_workspace={max_sessions}"),
                    });
                }
            }
        }

        candidates.sort_by(|a, b| a.timestamp.cmp(&b.timestamp).then_with(|| a.id.cmp(&b.id)));
        Ok(candidates)
    }

    /// Apply a cleanup policy by deleting selected sessions and index rows.
    ///
    /// This is an explicit maintenance operation. It never removes IDs listed in
    /// `protected_session_ids`, even if callers accidentally include an active
    /// session in an otherwise matching policy.
    pub fn apply_cleanup(
        &self,
        policy: &SessionCleanupPolicy,
    ) -> Result<SessionCleanupReport, SessionError> {
        let candidates = self.cleanup_candidates(policy)?;
        let mut report = SessionCleanupReport {
            candidates,
            removed: 0,
            bytes_removed: 0,
        };

        for candidate in &report.candidates {
            if candidate.file_path.exists() {
                fs::remove_file(&candidate.file_path)?;
                report.removed += 1;
                report.bytes_removed += candidate.size_bytes;
            }

            if let Ok(mut guard) = self.get_or_create_index()
                && let Some(index) = guard.as_mut()
            {
                let _ = index.delete_session(&candidate.id.to_string());
            }
        }

        Ok(report)
    }

    #[allow(clippy::collapsible_if)]
    fn find_session_file(&self, id: &Uuid) -> Result<PathBuf, SessionError> {
        if self.sessions_dir.exists() {
            for ws_entry in fs::read_dir(&self.sessions_dir)? {
                let ws_entry = ws_entry?;
                if !ws_entry.file_type()?.is_dir() {
                    continue;
                }
                let candidate = ws_entry.path().join(format!("{id}.jsonl"));
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }
        Err(SessionError::SessionNotFound(*id))
    }

    fn collect_cleanup_sessions(
        &self,
        policy: &SessionCleanupPolicy,
    ) -> Result<std::collections::HashMap<String, Vec<CleanupSession>>, SessionError> {
        let mut by_workspace: std::collections::HashMap<String, Vec<CleanupSession>> =
            std::collections::HashMap::new();

        if !self.sessions_dir.exists() {
            return Ok(by_workspace);
        }

        if let Some(target) = &policy.workspace_root {
            let workspace_dir = self.sessions_dir.join(workspace_dir_name(target));
            if workspace_dir.exists() {
                Self::collect_cleanup_workspace(target, &workspace_dir, &mut by_workspace)?;
            }
            return Ok(by_workspace);
        }

        for ws_entry in fs::read_dir(&self.sessions_dir)? {
            let ws_entry = ws_entry?;
            if !ws_entry.file_type()?.is_dir() {
                continue;
            }
            let ws_dir = ws_entry.path();
            let workspace_root = workspace_root_from_dir_name(
                &ws_dir.file_name().unwrap_or_default().to_string_lossy(),
            );
            Self::collect_cleanup_workspace(&workspace_root, &ws_dir, &mut by_workspace)?;
        }

        Ok(by_workspace)
    }

    fn collect_cleanup_workspace(
        workspace_root: &str,
        workspace_dir: &Path,
        by_workspace: &mut std::collections::HashMap<String, Vec<CleanupSession>>,
    ) -> Result<(), SessionError> {
        for file_entry in fs::read_dir(workspace_dir)? {
            let file_entry = file_entry?;
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }
            let Some(id) = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            else {
                continue;
            };
            let metadata = fs::metadata(&path)?;
            let timestamp = metadata
                .modified()
                .ok()
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(Utc::now);
            by_workspace
                .entry(workspace_root.to_string())
                .or_default()
                .push(CleanupSession {
                    id,
                    file_path: path,
                    size_bytes: metadata.len(),
                    timestamp,
                });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct CleanupSession {
    id: Uuid,
    file_path: PathBuf,
    size_bytes: u64,
    timestamp: DateTime<Utc>,
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
