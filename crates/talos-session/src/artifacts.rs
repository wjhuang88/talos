//! Session-owned transcript and pending-submission artifact lifecycle.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, ErrorCode, OpenFlags};
use uuid::Uuid;

use crate::SessionError;

/// Default maximum number of filesystem directory entries inspected in one pass.
pub const DEFAULT_MAX_ORPHAN_SIDECAR_ENTRIES: usize = 4_096;
/// Default grace period before a transcript-less sidecar can be reconciled.
pub const DEFAULT_ORPHAN_SIDECAR_MINIMUM_AGE: Duration = Duration::from_secs(300);
const ORPHAN_SIDECAR_SCAN_STATE_FILE: &str = ".orphan-sidecar-scan-budget";

/// Successful deletion details for one Session-owned artifact operation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionArtifactCleanupReport {
    /// Number of files actually removed. Missing files do not increment this value.
    pub removed_artifacts: usize,
    /// Total bytes reported by filesystem metadata for removed files.
    pub bytes_removed: u64,
    /// Exact paths removed, in deletion order.
    pub removed_paths: Vec<PathBuf>,
}

impl SessionArtifactCleanupReport {
    pub(crate) fn merge(&mut self, other: Self) {
        self.removed_artifacts = self
            .removed_artifacts
            .saturating_add(other.removed_artifacts);
        self.bytes_removed = self.bytes_removed.saturating_add(other.bytes_removed);
        self.removed_paths.extend(other.removed_paths);
    }
}

/// Safety policy for bounded orphan-sidecar reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanSidecarReconciliationPolicy {
    /// Session IDs that are live, deferred, or otherwise protected by a caller.
    pub protected_session_ids: Vec<Uuid>,
    /// Maximum filesystem directory entries inspected in this pass.
    pub max_entries: usize,
    /// Minimum age required before a transcript-less sidecar can be removed.
    pub minimum_age: Duration,
}

impl Default for OrphanSidecarReconciliationPolicy {
    fn default() -> Self {
        Self {
            protected_session_ids: Vec::new(),
            max_entries: DEFAULT_MAX_ORPHAN_SIDECAR_ENTRIES,
            minimum_age: DEFAULT_ORPHAN_SIDECAR_MINIMUM_AGE,
        }
    }
}

/// One reconciliation failure retained without aborting the whole bounded pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanSidecarFailure {
    /// Session ID inferred from the strictly validated filename.
    pub session_id: Uuid,
    /// Exact SQLite sidecar path associated with the failed set.
    pub path: PathBuf,
    /// Content-free filesystem or SQLite diagnostic.
    pub error: String,
}

/// Result of scanning Session roots for transcript-less pending SQLite artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrphanSidecarReconciliationReport {
    /// Number of filesystem directory entries inspected, including unrelated names.
    pub scanned_entries: usize,
    /// Number of complete orphan sets for which at least one artifact was removed.
    pub removed_sets: usize,
    /// Number of individual SQLite/WAL/SHM files removed.
    pub removed_artifacts: usize,
    /// Total bytes removed from orphan sidecars.
    pub bytes_removed: u64,
    /// Number of sets skipped due to safety checks or concurrent disappearance.
    pub skipped_sets: usize,
    /// Failures retained for diagnosis; other candidate sets are still processed.
    pub failures: Vec<OrphanSidecarFailure>,
    /// Whether the configured scan bound stopped the pass before exhaustion.
    pub bounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionArtifactSet {
    transcript: PathBuf,
    sqlite: PathBuf,
    wal: PathBuf,
    shm: PathBuf,
}

impl SessionArtifactSet {
    fn for_transcript(transcript: &Path) -> Result<Self, SessionError> {
        let extension = transcript.extension().and_then(|value| value.to_str());
        if !matches!(extension, Some("tlog" | "jsonl")) {
            return Err(SessionError::ParseError(format!(
                "invalid Session transcript extension: {}",
                transcript.display()
            )));
        }
        let stem = transcript
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                SessionError::ParseError(format!(
                    "invalid Session transcript path: {}",
                    transcript.display()
                ))
            })?;
        Uuid::parse_str(stem).map_err(|_| {
            SessionError::ParseError(format!(
                "Session transcript filename is not a UUID: {}",
                transcript.display()
            ))
        })?;
        let sqlite = transcript.with_file_name(format!("{stem}.pending.sqlite"));
        Ok(Self {
            transcript: transcript.to_path_buf(),
            wal: PathBuf::from(format!("{}-wal", sqlite.display())),
            shm: PathBuf::from(format!("{}-shm", sqlite.display())),
            sqlite,
        })
    }

    fn for_workspace(workspace: &Path, id: Uuid) -> Self {
        let sqlite = workspace.join(format!("{id}.pending.sqlite"));
        Self {
            transcript: workspace.join(format!("{id}.tlog")),
            wal: PathBuf::from(format!("{}-wal", sqlite.display())),
            shm: PathBuf::from(format!("{}-shm", sqlite.display())),
            sqlite,
        }
    }

    fn sidecars(&self) -> [PathBuf; 3] {
        [self.wal.clone(), self.shm.clone(), self.sqlite.clone()]
    }
}

fn existing_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| fs::symlink_metadata(path).is_ok())
        .cloned()
        .collect()
}

fn remove_paths(paths: &[PathBuf]) -> Result<SessionArtifactCleanupReport, SessionError> {
    let mut report = SessionArtifactCleanupReport::default();
    for (index, path) in paths.iter().enumerate() {
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                if let Err(source) = fs::remove_file(path) {
                    return Err(SessionError::ArtifactCleanup {
                        path: path.clone(),
                        source,
                        removed: report.removed_paths,
                        remaining: existing_paths(&paths[index..]),
                    });
                }
                report.removed_artifacts = report.removed_artifacts.saturating_add(1);
                report.bytes_removed = report.bytes_removed.saturating_add(metadata.len());
                report.removed_paths.push(path.clone());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(SessionError::ArtifactCleanup {
                    path: path.clone(),
                    source,
                    removed: report.removed_paths,
                    remaining: existing_paths(&paths[index..]),
                });
            }
        }
    }
    Ok(report)
}

/// Removes WAL, SHM and pending SQLite in that order while retaining the transcript.
///
/// A failure leaves the transcript discoverable so `/delete` or retention can retry.
pub fn remove_session_sidecars_for_transcript(
    transcript_path: &Path,
) -> Result<SessionArtifactCleanupReport, SessionError> {
    let set = SessionArtifactSet::for_transcript(transcript_path)?;
    remove_paths(&set.sidecars())
}

/// Removes only the transcript, the final discoverability commit point.
pub fn remove_session_transcript(
    transcript_path: &Path,
) -> Result<SessionArtifactCleanupReport, SessionError> {
    let set = SessionArtifactSet::for_transcript(transcript_path)?;
    remove_paths(&[set.transcript])
}

/// Removes the complete Session-owned filesystem set with transcript last.
pub fn remove_session_artifacts_for_transcript(
    transcript_path: &Path,
) -> Result<SessionArtifactCleanupReport, SessionError> {
    let mut report = remove_session_sidecars_for_transcript(transcript_path)?;
    report.merge(remove_session_transcript(transcript_path)?);
    Ok(report)
}

fn parse_sidecar_session_id(name: &str) -> Option<Uuid> {
    [
        ".pending.sqlite-wal",
        ".pending.sqlite-shm",
        ".pending.sqlite",
    ]
    .iter()
    .find_map(|suffix| name.strip_suffix(suffix))
    .and_then(|stem| Uuid::parse_str(stem).ok())
}

fn path_is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

fn sidecars_are_old_enough(set: &SessionArtifactSet, minimum_age: Duration) -> bool {
    set.sidecars()
        .iter()
        .filter_map(|path| fs::symlink_metadata(path).ok())
        .all(|metadata| {
            metadata
                .modified()
                .ok()
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age >= minimum_age)
        })
}

fn sqlite_sidecar_is_busy(path: &Path) -> Result<bool, SessionError> {
    if !path.exists() {
        return Ok(false);
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| SessionError::OrphanReconciliation {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    connection.busy_timeout(Duration::ZERO).map_err(|error| {
        SessionError::OrphanReconciliation {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;
    match connection.execute_batch("BEGIN EXCLUSIVE; ROLLBACK;") {
        Ok(()) => Ok(false),
        Err(rusqlite::Error::SqliteFailure(failure, _))
            if matches!(
                failure.code,
                ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
            ) =>
        {
            Ok(true)
        }
        Err(error) => Err(SessionError::OrphanReconciliation {
            path: path.to_path_buf(),
            message: error.to_string(),
        }),
    }
}

fn orphan_scan_state_path(canonical_root: &Path) -> PathBuf {
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
    if fs::symlink_metadata(&state_path)
        .is_ok_and(|metadata| metadata.file_type().is_symlink() || !metadata.file_type().is_file())
    {
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
    sessions_dir: &Path,
    policy: &OrphanSidecarReconciliationPolicy,
) -> Result<OrphanSidecarReconciliationReport, SessionError> {
    let mut report = OrphanSidecarReconciliationReport::default();
    if !sessions_dir.exists() {
        return Ok(report);
    }
    let canonical_root = sessions_dir.canonicalize()?;
    let protected: HashSet<Uuid> = policy.protected_session_ids.iter().copied().collect();
    let base_limit = policy.max_entries.max(1);
    let limit = read_orphan_scan_limit(&canonical_root, base_limit)?;
    let mut sets: BTreeMap<(PathBuf, Uuid), SessionArtifactSet> = BTreeMap::new();
    let mut blocked: BTreeSet<(PathBuf, Uuid)> = BTreeSet::new();

    'workspaces: for workspace_entry in fs::read_dir(&canonical_root)? {
        let workspace_entry = workspace_entry?;
        if workspace_entry.file_name().to_str() == Some(ORPHAN_SIDECAR_SCAN_STATE_FILE) {
            continue;
        }
        if report.scanned_entries >= limit {
            report.bounded = true;
            break;
        }
        report.scanned_entries = report.scanned_entries.saturating_add(1);
        let metadata = fs::symlink_metadata(workspace_entry.path())?;
        if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let workspace = workspace_entry.path().canonicalize()?;
        if !workspace.starts_with(&canonical_root) {
            continue;
        }
        for entry in fs::read_dir(&workspace)? {
            if report.scanned_entries >= limit {
                report.bounded = true;
                break 'workspaces;
            }
            report.scanned_entries = report.scanned_entries.saturating_add(1);
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(id) = parse_sidecar_session_id(name) else {
                continue;
            };
            let key = (workspace.clone(), id);
            if fs::symlink_metadata(entry.path()).is_ok_and(|value| value.file_type().is_symlink())
            {
                blocked.insert(key.clone());
            }
            sets.entry(key)
                .or_insert_with(|| SessionArtifactSet::for_workspace(&workspace, id));
        }
    }

    for ((workspace, id), set) in sets {
        if blocked.contains(&(workspace.clone(), id)) || protected.contains(&id) {
            report.skipped_sets = report.skipped_sets.saturating_add(1);
            continue;
        }
        let tlog = workspace.join(format!("{id}.tlog"));
        let jsonl = workspace.join(format!("{id}.jsonl"));
        if fs::symlink_metadata(&tlog).is_ok() || fs::symlink_metadata(&jsonl).is_ok() {
            report.skipped_sets = report.skipped_sets.saturating_add(1);
            continue;
        }
        if set.sidecars().iter().any(|path| path_is_symlink(path))
            || !sidecars_are_old_enough(&set, policy.minimum_age)
        {
            report.skipped_sets = report.skipped_sets.saturating_add(1);
            continue;
        }
        match sqlite_sidecar_is_busy(&set.sqlite) {
            Ok(true) => {
                report.skipped_sets = report.skipped_sets.saturating_add(1);
                continue;
            }
            Err(error) => {
                report.failures.push(OrphanSidecarFailure {
                    session_id: id,
                    path: set.sqlite.clone(),
                    error: error.to_string(),
                });
                continue;
            }
            Ok(false) => {}
        }
        match remove_paths(&set.sidecars()) {
            Ok(cleanup) if cleanup.removed_artifacts > 0 => {
                report.removed_sets = report.removed_sets.saturating_add(1);
                report.removed_artifacts = report
                    .removed_artifacts
                    .saturating_add(cleanup.removed_artifacts);
                report.bytes_removed = report.bytes_removed.saturating_add(cleanup.bytes_removed);
            }
            Ok(_) => {
                report.skipped_sets = report.skipped_sets.saturating_add(1);
            }
            Err(error) => report.failures.push(OrphanSidecarFailure {
                session_id: id,
                path: set.sqlite,
                error: error.to_string(),
            }),
        }
    }

    if report.bounded {
        persist_next_orphan_scan_limit(&canonical_root, limit)?;
    } else {
        clear_orphan_scan_limit(&canonical_root)?;
    }

    Ok(report)
}
