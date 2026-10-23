//! Component-wise pointer resolution. Callers must obtain the dedicated Pull
//! resolution authorization before capture and separately admit `resolved()`.
//! This module performs no directory enumeration or file-content reads.

use std::collections::VecDeque;
use std::path::{Component, Path, PathBuf};

use super::GitToolError;

/// Invocation-owned cumulative allowance, never reset by a continuation.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Budget {
    pub(super) entries: usize,
    pub(super) bytes: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        }
    }
}

impl Budget {
    // Queue text is explicit scratch evidence. Reserve before allocating its
    // owned components; consumed discovery allowance is never refunded.
    fn charge_components(&mut self, path: &Path) -> Result<(), GitToolError> {
        let bytes = path
            .components()
            .try_fold(0usize, |total, component| {
                total.checked_add(component.as_os_str().as_encoded_bytes().len())
            })
            .ok_or_else(invalid)?;
        self.charge(0, bytes)
    }

    pub(super) fn charge(&mut self, entries: usize, bytes: usize) -> Result<(), GitToolError> {
        let remaining_entries = self.entries.checked_sub(entries).ok_or_else(invalid)?;
        let remaining_bytes = self.bytes.checked_sub(bytes).ok_or_else(invalid)?;
        self.entries = remaining_entries;
        self.bytes = remaining_bytes;
        Ok(())
    }
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput(
        "Git path resolution changed, is unsupported, or exceeded its limit".into(),
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Identity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(windows)]
    handle: same_file::Handle,
}

impl Identity {
    fn capture(path: &Path, metadata: &std::fs::Metadata) -> Result<Self, GitToolError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let _ = path;
            Ok(Self {
                device: metadata.dev(),
                inode: metadata.ino(),
            })
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            };
            let _ = metadata;
            // Metadata-only access, including the link itself, rather than a
            // read-content handle or an implicit traversal through its target.
            let handle = std::panic::catch_unwind(|| {
                let file = std::fs::OpenOptions::new()
                    .access_mode(0)
                    .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(path)?;
                same_file::Handle::from_file(file)
            })
            .map_err(|_| invalid())?
            .map_err(|_| invalid())?;
            Ok(Self { handle })
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (path, metadata);
            Err(invalid())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Missing,
    Directory(Identity),
    File(Identity),
    Link(Identity, PathBuf),
}

fn inspect(path: &Path) -> Result<State, GitToolError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            return Ok(State::Missing);
        }
        Err(_) => return Err(invalid()),
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            && !metadata.file_type().is_symlink()
        {
            return Err(invalid());
        }
    }
    let identity = Identity::capture(path, &metadata)?;
    if metadata.file_type().is_symlink() {
        Ok(State::Link(
            identity,
            std::fs::read_link(path).map_err(|_| invalid())?,
        ))
    } else if metadata.is_dir() {
        Ok(State::Directory(identity))
    } else if metadata.is_file() {
        Ok(State::File(identity))
    } else {
        Err(invalid())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Record {
    path: PathBuf,
    state: State,
}

/// Retains traversal order, including links, missing suffixes and ancestors
/// later cancelled by `..`. Revalidation never discovers or replaces records.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Resolution {
    resolved: PathBuf,
    records: Vec<Record>,
    original_leaf_link: bool,
}

impl Resolution {
    pub(super) fn resolved(&self) -> &Path {
        &self.resolved
    }

    pub(super) fn original_leaf_is_link(&self) -> bool {
        self.original_leaf_link
    }

    #[cfg(test)]
    fn capture(
        path: &Path,
        allow_missing: bool,
        budget: &mut Budget,
    ) -> Result<Self, GitToolError> {
        Self::capture_endpoint(path, Endpoint::Directory, allow_missing, budget)
    }

    /// Resolves a local auxiliary file path, allowing only its final leaf to
    /// be absent. Parent traversal and every earlier component must exist as a
    /// directory, and regular-file contents remain a separate Read operation.
    pub(super) fn capture_file(path: &Path, budget: &mut Budget) -> Result<Self, GitToolError> {
        Self::capture_endpoint(path, Endpoint::File, true, budget)
    }

    fn capture_endpoint(
        path: &Path,
        endpoint: Endpoint,
        allow_missing: bool,
        budget: &mut Budget,
    ) -> Result<Self, GitToolError> {
        if !path.is_absolute() {
            return Err(invalid());
        }
        reject_network_path(path)?;
        budget.charge_components(path)?;
        let components: Vec<_> = path.components().collect();
        let mut pending: VecDeque<_> = path
            .components()
            .enumerate()
            .map(|(index, component)| {
                (
                    component.as_os_str().to_owned(),
                    allow_missing && (endpoint == Endpoint::Directory || index < components.len()),
                    endpoint == Endpoint::File && index + 1 == components.len(),
                )
            })
            .collect();
        let mut result = Self {
            resolved: PathBuf::new(),
            records: Vec::new(),
            original_leaf_link: false,
        };
        let mut hops = 0usize;
        let mut missing_suffix = false;
        while let Some((part, missing_allowed, original_leaf)) = pending.pop_front() {
            let component = Path::new(&part).components().next().ok_or_else(invalid)?;
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    result.resolved.push(&part);
                    // Verbatim disk prefixes have an implicit root in Rust,
                    // but `\\?\C:` is not the complete filesystem root. Wait
                    // for an explicit RootDir before inspecting its identity.
                    let root_follows = pending.front().is_some_and(|(next, _, _)| {
                        matches!(
                            Path::new(next).components().next(),
                            Some(Component::RootDir)
                        )
                    });
                    if result.resolved.is_absolute() && !root_follows {
                        result.record(budget)?;
                    }
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if missing_suffix {
                        return Err(invalid());
                    }
                    if matches!(
                        result.records.last(),
                        Some(Record {
                            path,
                            state: State::File(_),
                        }) if path == &result.resolved
                    ) {
                        return Err(invalid());
                    }
                    result.resolved.pop();
                }
                Component::Normal(_) => {
                    result.resolved.push(&part);
                    result.record(budget)?;
                    match &result.records.last().ok_or_else(invalid)?.state {
                        State::Missing if !missing_allowed => return Err(invalid()),
                        State::Missing if endpoint == Endpoint::File => {
                            missing_suffix = true;
                        }
                        State::File(_) if endpoint == Endpoint::File && !pending.is_empty() => {
                            // A later leaf below a file is absent in the
                            // filesystem namespace (ENOENT/ENOTDIR). Retain
                            // that ancestor identity and absence before any
                            // future checkout may replace the file by a dir.
                            missing_suffix = true;
                        }
                        State::File(_) if endpoint != Endpoint::File => {
                            return Err(invalid());
                        }
                        State::Link(_, target) => {
                            reject_network_path(target)?;
                            hops += 1;
                            if hops > 64 {
                                return Err(invalid());
                            }
                            result.resolved.pop();
                            if original_leaf {
                                result.original_leaf_link = true;
                            }
                            if target.is_absolute() {
                                result.resolved.clear();
                            }
                            // Drive-relative Windows link targets depend on hidden
                            // per-drive state and are outside this capability.
                            if target.has_root() != target.is_absolute() {
                                return Err(invalid());
                            }
                            #[cfg(windows)]
                            if matches!(target.components().next(), Some(Component::Prefix(_)))
                                && !target.is_absolute()
                            {
                                return Err(invalid());
                            }
                            if target.components().next().is_none() {
                                return Err(invalid());
                            }
                            budget.charge_components(target)?;
                            // A dangling link is not a missing checkout suffix.
                            for component in target.components().rev() {
                                pending.push_front((
                                    component.as_os_str().to_owned(),
                                    false,
                                    false,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        result.revalidate()?;
        if endpoint == Endpoint::File
            && !matches!(
                result.records.last().map(|record| &record.state),
                Some(State::Missing | State::File(_) | State::Directory(_))
            )
        {
            return Err(invalid());
        }
        Ok(result)
    }

    fn record(&mut self, budget: &mut Budget) -> Result<(), GitToolError> {
        // Reserve before opening metadata/identity handles. On Windows charge
        // the retained handle and a conservative temporary comparison slot too.
        let entries = if cfg!(windows) { 3 } else { 1 };
        budget.charge(
            entries,
            self.resolved.as_os_str().as_encoded_bytes().len() + 32,
        )?;
        let state = inspect(&self.resolved)?;
        if let State::Link(_, target) = &state {
            budget.charge(0, target.as_os_str().as_encoded_bytes().len())?;
        }
        self.records.push(Record {
            path: self.resolved.clone(),
            state,
        });
        Ok(())
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        for record in &self.records {
            if inspect(&record.path)? != record.state {
                return Err(invalid());
            }
        }
        Ok(())
    }

    pub(super) fn related_paths(&self) -> impl Iterator<Item = &Path> {
        self.records.iter().map(|record| record.path.as_path())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endpoint {
    Directory,
    File,
}

fn reject_network_path(path: &Path) -> Result<(), GitToolError> {
    #[cfg(unix)]
    let network = path.as_os_str().as_encoded_bytes().starts_with(b"//");
    #[cfg(windows)]
    let network = matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(
                prefix.kind(),
                std::path::Prefix::UNC(..)
                    | std::path::Prefix::VerbatimUNC(..)
                    | std::path::Prefix::DeviceNS(..)
            )
    );
    #[cfg(not(any(unix, windows)))]
    let network = false;
    if network { Err(invalid()) } else { Ok(()) }
}

#[cfg(test)]
#[path = "pull_resolution_tests.rs"]
mod tests;
