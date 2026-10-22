//! Bounded frontier discovery for Pull graph reads. The caller must authorize
//! every frontier before inspection, either by exact paths or the dedicated
//! identity-bound metadata scope. Neither mode permits traversing links.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{GitToolError, snapshot::PathIdentity};

/// Cumulative file, directory-name and pointer-resolution allowance for Pull.
/// Full transient-resource and identity-handle accounting remain separate.
#[derive(Clone, Debug)]
pub(super) struct ReferenceBudget(Arc<Mutex<(usize, usize)>>);

impl Default for ReferenceBudget {
    fn default() -> Self {
        Self(Arc::new(Mutex::new((4096, 4 * 1024 * 1024))))
    }
}

impl ReferenceBudget {
    fn retain_path(&self, path: &Path) -> Result<(), GitToolError> {
        self.retain_path_bytes(path.as_os_str().as_encoded_bytes().len())
    }

    fn retain_child_name(&self, parent: &Path, name: &std::ffi::OsStr) -> Result<(), GitToolError> {
        let bytes = parent
            .as_os_str()
            .as_encoded_bytes()
            .len()
            .checked_add(1)
            .and_then(|bytes| bytes.checked_add(name.as_encoded_bytes().len()))
            .ok_or_else(invalid)?;
        self.retain_path_bytes(bytes)
    }

    fn retain_path_bytes(&self, bytes: usize) -> Result<(), GitToolError> {
        let mut remaining = self.0.lock().map_err(|_| invalid())?;
        remaining.0 = remaining.0.checked_sub(1).ok_or_else(invalid)?;
        remaining.1 = remaining.1.checked_sub(bytes).ok_or_else(invalid)?;
        Ok(())
    }

    fn resolve(
        &self,
        path: &Path,
        allow_missing: bool,
    ) -> Result<super::pull_resolution::Resolution, GitToolError> {
        let mut remaining = self.0.lock().map_err(|_| invalid())?;
        let mut allowance = super::pull_resolution::Budget {
            entries: remaining.0,
            bytes: remaining.1,
        };
        let result =
            super::pull_resolution::Resolution::capture(path, allow_missing, &mut allowance);
        *remaining = (allowance.entries, allowance.bytes);
        result
    }

    pub(super) fn capture(
        &self,
        path: &Path,
    ) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        self.capture_limited(path, usize::MAX)
    }

    fn capture_limited(
        &self,
        path: &Path,
        max_bytes: usize,
    ) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        let mut remaining = self.0.lock().map_err(|_| invalid())?;
        let available_bytes = remaining.1.min(max_bytes);
        let mut allowance = super::snapshot::Budget {
            entries: remaining.0,
            bytes: available_bytes,
        };
        let result = super::snapshot::RegularEvidence::capture(path, &mut allowance);
        // Failed captures also consume any work already charged. A continuation
        // or replacement inventory must never restore the invocation allowance.
        remaining.0 = allowance.entries;
        remaining.1 -= available_bytes
            .checked_sub(allowance.bytes)
            .ok_or_else(invalid)?;
        result.map_err(|_| invalid())
    }
}

#[derive(Debug)]
pub(super) struct Inventory {
    stored_modules_only: bool,
    reference_budget: ReferenceBudget,
    pending: Vec<PathBuf>,
    visited: BTreeMap<PathBuf, Option<PathIdentity>>,
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
    alternates: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    references: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    worktree_targets: BTreeMap<PathBuf, Option<super::pull_resolution::Resolution>>,
    worktree_common_targets: BTreeSet<PathBuf>,
    worktree_reference_roots: BTreeSet<PathBuf>,
}

#[cfg_attr(test, track_caller)]
fn invalid() -> GitToolError {
    // Keep production errors opaque, but identify the rejecting invariant in
    // test failures without logging repository paths or reference contents.
    #[cfg(test)]
    eprintln!(
        "Pull inventory rejection at {}",
        std::panic::Location::caller()
    );
    GitToolError::InvalidInput(
        "Pull graph inventory changed, redirected, or exceeded its limit".into(),
    )
}

// Native reftable readers join manifest entries directly to the backend path.
// Keep every entry a single filename on both Unix and Windows.
fn validate_table_names(bytes: &[u8]) -> Result<(), GitToolError> {
    for name in bytes.split(|byte| *byte == b'\n') {
        let stem = name.split(|byte| *byte == b'.').next().unwrap_or_default();
        let device = [
            b"CON".as_slice(),
            b"PRN",
            b"AUX",
            b"NUL",
            b"CONIN$",
            b"CONOUT$",
        ]
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
            || (stem.len() == 4
                && (stem[..3].eq_ignore_ascii_case(b"COM")
                    || stem[..3].eq_ignore_ascii_case(b"LPT"))
                && matches!(stem[3], b'1'..=b'9'));
        if name
            .last()
            .is_some_and(|byte| matches!(byte, b'.' | b' ' | b'\r'))
            || device
            || name == b"."
            || name == b".."
            || name.iter().any(|byte| {
                !byte.is_ascii()
                    || byte.is_ascii_control()
                    || matches!(
                        byte,
                        b' ' | b'/' | b'\\' | b':' | b'<' | b'>' | b'"' | b'|' | b'?' | b'*'
                    )
            })
        {
            return Err(invalid());
        }
    }
    Ok(())
}

impl Inventory {
    #[cfg(test)]
    pub(super) fn new(git_dir: &Path, common_dir: &Path) -> Self {
        Self::with_reference_budget(git_dir, common_dir, ReferenceBudget::default())
            .expect("fixture inventory allowance")
    }

    pub(super) fn with_reference_budget(
        git_dir: &Path,
        common_dir: &Path,
        reference_budget: ReferenceBudget,
    ) -> Result<Self, GitToolError> {
        let mut pending = Vec::new();
        for root in [git_dir, common_dir] {
            for name in [
                "HEAD",
                "refs",
                "reftable",
                "logs",
                "objects",
                "info",
                "shallow",
                "packed-refs",
                "worktrees",
                "config.worktree",
            ] {
                // Charge before retaining the joined spelling. Duplicate roots
                // still consume discovery work even though the frontier dedups.
                reference_budget.retain_child_name(root, std::ffi::OsStr::new(name))?;
                pending.push(root.join(name));
            }
        }
        pending.sort();
        pending.dedup();
        Ok(Self::from_pending(pending, reference_budget, false))
    }

    fn from_pending(
        pending: Vec<PathBuf>,
        reference_budget: ReferenceBudget,
        stored_modules_only: bool,
    ) -> Self {
        Self {
            stored_modules_only,
            reference_budget,
            pending,
            visited: BTreeMap::new(),
            directories: BTreeMap::new(),
            alternates: Vec::new(),
            references: Vec::new(),
            worktree_targets: BTreeMap::new(),
            worktree_common_targets: BTreeSet::new(),
            worktree_reference_roots: BTreeSet::new(),
        }
    }

    // Names and identities only. Repository contents are deliberately not
    // traversed: discovering a module never admits its config, objects or refs.
    pub(super) fn stored_modules(
        common: &Path,
        reference_budget: ReferenceBudget,
    ) -> Result<Self, GitToolError> {
        reference_budget.retain_child_name(common, std::ffi::OsStr::new("modules"))?;
        Ok(Self::from_pending(
            vec![common.join("modules")],
            reference_budget,
            true,
        ))
    }

    pub(super) fn stored_module_roots(&self) -> impl Iterator<Item = &Path> {
        self.directories.iter().filter_map(|(path, entries)| {
            let head = entries
                .iter()
                .find(|entry| entry.file_name().is_some_and(|name| name == "HEAD"));
            (self.stored_modules_only
                && head.is_some_and(|head| {
                    self.visited.get(head).is_some_and(Option::is_some)
                        && !self.directories.contains_key(head)
                }))
            .then_some(path.as_path())
        })
    }

    #[cfg(test)]
    pub(super) fn paths(&self) -> Vec<PathBuf> {
        self.iter_paths().map(Path::to_owned).collect()
    }

    pub(super) fn capture_paths(&self) -> Result<Vec<PathBuf>, GitToolError> {
        self.iter_paths()
            .map(|path| {
                self.reference_budget.retain_path(path)?;
                Ok(path.to_owned())
            })
            .collect()
    }

    pub(super) fn iter_paths(&self) -> impl Iterator<Item = &Path> {
        self.visited
            .keys()
            .map(PathBuf::as_path)
            .chain(self.pending.iter().map(PathBuf::as_path))
            .chain(
                self.worktree_targets
                    .values()
                    .filter_map(|target| target.as_ref().map(|target| target.resolved())),
            )
    }

    pub(super) fn complete(&self) -> bool {
        self.pending.is_empty()
    }

    /// External frontiers and resolved targets. Raw pointer spellings never
    /// enter Path facets or the general lexical permission normalizer.
    #[cfg(test)]
    pub(super) fn exact_paths(&self, gitdir: &Path, common: &Path) -> Vec<PathBuf> {
        self.iter_exact_paths(gitdir, common)
            .map(Path::to_owned)
            .collect()
    }

    pub(super) fn iter_exact_paths<'a>(
        &'a self,
        gitdir: &'a Path,
        common: &'a Path,
    ) -> impl Iterator<Item = &'a Path> {
        self.iter_paths().filter(move |path| {
            self.worktree_targets
                .values()
                .flatten()
                .any(|target| target.resolved() == *path)
                || (!path.starts_with(gitdir) && !path.starts_with(common))
        })
    }

    pub(super) fn subtree_complete(&self, root: &Path) -> bool {
        let absent = root.parent().is_some_and(|parent| {
            self.directories
                .get(parent)
                .is_some_and(|children| !children.iter().any(|child| child == root))
                || self.visited.get(parent) == Some(&None)
        });
        (self.visited.contains_key(root) || absent)
            && !self.pending.iter().any(|path| path.starts_with(root))
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        for target in self.worktree_targets.values().flatten() {
            target.revalidate()?;
        }
        for (path, expected) in &self.visited {
            revalidate_identity(path, expected.as_ref())?;
        }
        for (path, expected) in &self.directories {
            revalidate_children(path, expected)?;
        }
        for (path, expected) in self.alternates.iter().chain(&self.references) {
            expected.revalidate(path).map_err(|_| invalid())?;
        }
        Ok(())
    }

    /// Bind references, linked-worktree pointers/config and shallow/graft boundaries,
    /// including symbolic replacement refs, under one shared evidence budget,
    /// without recursively opening descendants absent from the admitted inventory.
    pub(super) fn bind_references(
        &mut self,
        gitdir: &Path,
        common: &Path,
    ) -> Result<(), GitToolError> {
        if !self.complete() {
            return Err(invalid());
        }
        self.revalidate()?;
        // Existing evidence has just been revalidated in place. Keep its
        // identity handles and bytes rather than capturing a second full copy.
        // Their initial capture already consumed the invocation allowance.
        let mut evidence = Vec::new();
        for (path, identity) in &self.visited {
            if self
                .references
                .binary_search_by(|(retained, _)| retained.cmp(path))
                .is_ok()
            {
                continue;
            }
            let reference = [gitdir, common]
                .into_iter()
                .chain(self.worktree_reference_roots.iter().map(PathBuf::as_path))
                .any(|root| {
                    path.strip_prefix(root).is_ok_and(|relative| {
                        relative.starts_with("refs")
                            || relative.starts_with("reftable")
                            || relative == Path::new("packed-refs")
                            || relative == Path::new("HEAD")
                    })
                })
                || path
                    .strip_prefix(common)
                    .ok()
                    .and_then(|relative| relative.strip_prefix("worktrees").ok())
                    .is_some_and(|relative| {
                        // Linked worktree refs are per-worktree roots for --all.
                        // The first component is the registered worktree identifier.
                        let mut components = relative.components();
                        components.next().is_some()
                            && components.next().is_some_and(|name| {
                                matches!(
                                    name.as_os_str().to_str(),
                                    Some("HEAD" | "refs" | "packed-refs" | "reftable")
                                )
                            })
                    });
            let boundary = [gitdir, common].iter().any(|root| {
                path.strip_prefix(root).is_ok_and(|relative| {
                    relative == Path::new("shallow") || relative == Path::new("info/grafts")
                })
            });
            // get_worktrees() checks the main worktree's core.bare separately
            // even when its config.worktree is absent from the current chain.
            let worktree_config = path.strip_prefix(common) == Ok(Path::new("config.worktree"));
            let pointer = path
                .strip_prefix(common)
                .ok()
                .and_then(|relative| relative.strip_prefix("worktrees").ok())
                .is_some_and(|relative| {
                    let mut components = relative.components();
                    components.next().is_some()
                        && components.next().is_some_and(|name| {
                            name.as_os_str() == "gitdir" || name.as_os_str() == "commondir"
                        })
                        && components.next().is_none()
                });
            if (reference || boundary || pointer || worktree_config)
                && identity.is_some()
                && !self.directories.contains_key(path)
            {
                let entry = self.reference_budget.capture(path)?;
                if reference && let super::snapshot::Entry::File(_, bytes) = &entry.entry {
                    super::receiver_state::validate_symbolic_target(bytes)?;
                    if path.file_name().is_some_and(|name| name == "tables.list")
                        && path
                            .parent()
                            .and_then(Path::file_name)
                            .is_some_and(|name| name == "reftable")
                    {
                        validate_table_names(bytes)?;
                    }
                }
                self.reference_budget.retain_path(path)?;
                evidence.push((path.clone(), entry));
            }
        }
        self.revalidate()?;
        self.references.extend(evidence);
        self.references
            .sort_by(|(left, _), (right, _)| left.cmp(right));
        self.revalidate()
    }

    /// Parse only retained pointer bytes, without filesystem revalidation.
    /// Callers separately revalidate the admitted source evidence before use;
    /// target resolution and content reads require their respective admissions.
    pub(super) fn discover_worktree_targets(
        &mut self,
        common: &Path,
    ) -> Result<bool, GitToolError> {
        let mut added = false;
        for (path, evidence) in &self.references {
            let Some(relative) = path
                .strip_prefix(common)
                .ok()
                .and_then(|relative| relative.strip_prefix("worktrees").ok())
            else {
                continue;
            };
            let mut components = relative.components();
            let _id = components.next();
            let Some(leaf) = components.next() else {
                continue;
            };
            if components.next().is_some() {
                continue;
            }
            let gitdir = leaf.as_os_str() == "gitdir";
            if !gitdir && leaf.as_os_str() != "commondir" {
                continue;
            }
            let super::snapshot::Entry::File(_, bytes) = &evidence.entry else {
                return Err(invalid());
            };
            let text = std::str::from_utf8(bytes).map_err(|_| invalid())?;
            let text = if gitdir {
                text.trim_end_matches(|c: char| c.is_ascii_whitespace())
                    .strip_suffix("/.git")
                    .unwrap_or_else(|| text.trim_end_matches(|c: char| c.is_ascii_whitespace()))
            } else {
                text.trim_end_matches(['\r', '\n'])
            };
            if text.is_empty() || text.contains(['\0', '\r', '\n']) {
                return Err(invalid());
            }
            // Checkout backlinks use Git's forgiving relative resolver below,
            // including interleaved missing components and ParentDir. Shared
            // metadata pointers retain the stricter receiver path syntax.
            if !gitdir {
                super::remote_state::validate_local_path_syntax(Path::new(text))?;
            }
            // get_linked_worktree() only resolves a relative backlink. An
            // absolute backlink is retained as text, even for a pruned checkout.
            if gitdir && Path::new(text).is_absolute() {
                continue;
            }
            let parent = path.parent().ok_or_else(invalid)?;
            self.reference_budget
                .retain_child_name(parent, std::ffi::OsStr::new(text))?;
            let target = parent.join(text);
            if !gitdir && !self.worktree_common_targets.contains(&target) {
                self.reference_budget.retain_path(&target)?;
                self.worktree_common_targets.insert(target.clone());
            }
            if !self.worktree_targets.contains_key(&target) {
                if self.visited.len() + self.worktree_targets.len() >= 4096 {
                    return Err(invalid());
                }
                self.worktree_targets.insert(target, None);
                added = true;
            }
        }
        Ok(added)
    }

    pub(super) fn has_worktree_targets(&self) -> bool {
        !self.worktree_targets.is_empty()
    }

    /// Caller must consume a current prepared proof covering the dedicated
    /// resolution capability before entering. No target content is read here.
    pub(super) fn resolve_worktree_targets(&mut self) -> Result<bool, GitToolError> {
        self.revalidate()?;
        let mut discovered = false;
        for (path, target) in &mut self.worktree_targets {
            if target.is_none() {
                *target = Some(
                    self.reference_budget
                        .resolve(path, !self.worktree_common_targets.contains(path))?,
                );
                discovered = true;
            }
        }
        self.revalidate()?;
        Ok(discovered)
    }

    pub(super) fn bind_worktree_targets(
        &mut self,
        admitted: impl Fn(&Path) -> Option<PathBuf>,
    ) -> Result<bool, GitToolError> {
        // Resolution is separately authorized; binding only consumes exact
        // resolved target grants. Never send raw spellings to a normalizer.
        for target in self.worktree_targets.values() {
            let resolved = target.as_ref().ok_or_else(invalid)?.resolved();
            self.reference_budget.retain_path(resolved)?;
            let normalized = admitted(resolved).ok_or_else(invalid)?;
            if normalized != resolved {
                return Err(invalid());
            }
        }
        self.revalidate()?;
        // Resolving a common directory does not grant access to its reference
        // store. Publish only names here; the next advance requires fresh grants
        // before inspecting their type, contents, or descendants.
        let mut added = false;
        for raw in &self.worktree_common_targets {
            let root = self
                .worktree_targets
                .get(raw)
                .and_then(Option::as_ref)
                .ok_or_else(invalid)?
                .resolved();
            if !self.worktree_reference_roots.contains(root) {
                self.reference_budget.retain_path(root)?;
                self.worktree_reference_roots.insert(root.to_owned());
            }
            for name in ["refs", "packed-refs", "reftable"] {
                self.reference_budget
                    .retain_child_name(root, std::ffi::OsStr::new(name))?;
                let path = root.join(name);
                if !self.visited.contains_key(&path) && !self.pending.contains(&path) {
                    if self.visited.len() + self.pending.len() + self.worktree_targets.len() >= 4096
                    {
                        return Err(invalid());
                    }
                    self.pending.push(path);
                    added = true;
                }
            }
        }
        self.pending.sort();
        Ok(added)
    }

    /// Bind object-store redirects only after the complete frontier was admitted.
    /// Missing leaves are proved by the retained parent directory listing, so
    /// an absent, unadmitted descendant is never opened or stat'ed here.
    pub(super) fn bind_alternates(
        &mut self,
        gitdir: &Path,
        common: &Path,
    ) -> Result<(), GitToolError> {
        if !self.complete() {
            return Err(invalid());
        }
        self.revalidate()?;
        let mut paths = Vec::new();
        for root in [gitdir, common] {
            if !self.subtree_complete(&root.join("objects")) {
                return Err(invalid());
            }
            for name in ["alternates", "http-alternates"] {
                let path = root.join("objects/info").join(name);
                if self.visited.get(&path).is_some_and(Option::is_some) && !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        let mut evidence = Vec::new();
        for path in paths {
            if self
                .alternates
                .iter()
                .any(|(retained, _)| retained == &path)
            {
                continue;
            }
            let entry = self.reference_budget.capture_limited(&path, 256 * 1024)?;
            if !matches!(&entry.entry, super::snapshot::Entry::File(_, bytes) if bytes.is_empty()) {
                return Err(GitToolError::InvalidInput(
                    "Pull child alternate object stores require separate discovery and admission"
                        .into(),
                ));
            }
            evidence.push((path, entry));
        }
        self.revalidate()?;
        self.alternates.extend(evidence);
        self.revalidate()
    }

    fn retain_frontier(&self, path: &Path, next: &mut Vec<PathBuf>) -> Result<(), GitToolError> {
        if self.visited.len() + self.pending.len() + next.len() >= 4096 {
            return Err(invalid());
        }
        self.reference_budget.retain_path(path)?;
        next.push(path.to_owned());
        Ok(())
    }

    pub(super) fn advance(&mut self, admitted: impl Fn(&Path) -> bool) -> Result<(), GitToolError> {
        // Check every resource before metadata access, including previously
        // visited entries used for revalidation. Never traverse an unapproved
        // child simply because its parent was approved.
        if self.iter_paths().any(|path| !admitted(path)) {
            return Err(invalid());
        }
        self.revalidate()?;
        let mut next = Vec::new();
        for path in &self.pending {
            self.reference_budget.retain_path(path)?;
            let before = identity(path)?;
            if before.is_some()
                && std::fs::symlink_metadata(path)
                    .map_err(|_| invalid())?
                    .is_dir()
            {
                let entries = children(path, &self.reference_budget)?;
                let marker = self
                    .stored_modules_only
                    .then(|| {
                        entries
                            .iter()
                            .find(|entry| entry.file_name().is_some_and(|name| name == "HEAD"))
                    })
                    .flatten();
                for child in &entries {
                    // Inspect a possible HEAD's type under its own admission
                    // before deciding this is a repository. A namespace can
                    // legitimately contain a module whose name is HEAD.
                    if marker.is_some_and(|head| head != child) {
                        continue;
                    }
                    self.retain_frontier(child, &mut next)?;
                }
                self.reference_budget.retain_path(path)?;
                self.directories.insert(path.clone(), entries);
            }
            if self.stored_modules_only
                && path.file_name().is_some_and(|name| name == "HEAD")
                && let Some(entries) = path
                    .parent()
                    .and_then(|parent| self.directories.get(parent))
            {
                let namespace = self.directories.contains_key(path);
                for child in entries {
                    if child != path
                        && (namespace || child.file_name().is_some_and(|name| name == "modules"))
                    {
                        self.retain_frontier(child, &mut next)?;
                    }
                }
            }
            revalidate_identity(path, before.as_ref())?;
            self.visited.insert(path.clone(), before);
        }
        next.sort();
        next.dedup();
        self.pending = next;
        self.revalidate()
    }
}

// Compare against the retained sorted set without constructing a second full
// path inventory. New entries fail immediately, rather than being retained as
// a replacement discovery baseline during revalidation.
fn revalidate_children(path: &Path, expected: &[PathBuf]) -> Result<(), GitToolError> {
    if expected.len() > 4096 {
        return Err(invalid());
    }
    let mut seen = vec![false; expected.len()];
    let mut count = 0usize;
    for child in std::fs::read_dir(path).map_err(|_| invalid())? {
        let child = child.map_err(|_| invalid())?.path();
        let index = expected.binary_search(&child).map_err(|_| invalid())?;
        if seen[index] {
            return Err(invalid());
        }
        seen[index] = true;
        count += 1;
    }
    if count != expected.len() {
        return Err(invalid());
    }
    Ok(())
}

fn children(path: &Path, budget: &ReferenceBudget) -> Result<Vec<PathBuf>, GitToolError> {
    let mut entries = Vec::new();
    for child in std::fs::read_dir(path).map_err(|_| invalid())? {
        if entries.len() >= 4096 {
            return Err(invalid());
        }
        let child = child.map_err(|_| invalid())?;
        let name = child.file_name();
        budget.retain_child_name(path, &name)?;
        entries.push(path.join(name));
    }
    entries.sort();
    Ok(entries)
}

fn identity(path: &Path) -> Result<Option<PathIdentity>, GitToolError> {
    let parent = path.parent().ok_or_else(invalid)?;
    if parent.canonicalize().map_err(|_| invalid())? != parent {
        return Err(invalid());
    }
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Ok(metadata) if metadata.is_file() || metadata.is_dir() => {
            if path.canonicalize().map_err(|_| invalid())? != path {
                return Err(invalid());
            }
            PathIdentity::capture(path).map(Some).map_err(|_| invalid())
        }
        _ => Err(invalid()),
    }
}

fn revalidate_identity(path: &Path, expected: Option<&PathIdentity>) -> Result<(), GitToolError> {
    let parent = path.parent().ok_or_else(invalid)?;
    if parent.canonicalize().map_err(|_| invalid())? != parent {
        return Err(invalid());
    }
    match (std::fs::symlink_metadata(path), expected) {
        (Err(error), None) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        (Ok(metadata), Some(expected)) if metadata.is_file() || metadata.is_dir() => {
            expected.revalidate(path).map_err(|_| invalid())
        }
        _ => Err(invalid()),
    }
}

#[cfg(test)]
#[path = "pull_inventory_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "pull_directory_budget_tests.rs"]
mod directory_budget_tests;

#[cfg(test)]
#[path = "pull_module_inventory_tests.rs"]
mod module_tests;
