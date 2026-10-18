//! Bounded frontier discovery for Pull graph reads. The caller must authorize
//! every frontier before inspection, either by exact paths or the dedicated
//! identity-bound metadata scope. Neither mode permits traversing links.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{GitToolError, snapshot::PathIdentity};

/// Cumulative file-evidence allowance shared by every inventory in one Pull.
/// Path traversal and transient handle accounting are separate remaining work.
#[derive(Clone, Debug)]
pub(super) struct ReferenceBudget(Arc<Mutex<(usize, usize)>>);

impl Default for ReferenceBudget {
    fn default() -> Self {
        Self(Arc::new(Mutex::new((4096, 4 * 1024 * 1024))))
    }
}

impl ReferenceBudget {
    fn capture(&self, path: &Path) -> Result<super::snapshot::RegularEvidence, GitToolError> {
        let mut remaining = self.0.lock().map_err(|_| invalid())?;
        let mut allowance = super::snapshot::Budget {
            entries: remaining.0,
            bytes: remaining.1,
        };
        let result = super::snapshot::RegularEvidence::capture(path, &mut allowance);
        // Failed captures also consume any work already charged. A continuation
        // or replacement inventory must never restore the invocation allowance.
        *remaining = (allowance.entries, allowance.bytes);
        result.map_err(|_| invalid())
    }
}

#[derive(Debug)]
pub(super) struct Inventory {
    reference_budget: ReferenceBudget,
    pending: Vec<PathBuf>,
    visited: BTreeMap<PathBuf, Option<PathIdentity>>,
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
    alternates: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    references: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    worktree_targets: BTreeMap<PathBuf, Option<WorktreeTarget>>,
    worktree_common_targets: BTreeSet<PathBuf>,
    worktree_reference_roots: BTreeSet<PathBuf>,
}

/// Resolution evidence for an existing target or a missing checkout suffix.
/// Common directories still require an existing target before refs discovery.
#[derive(Debug, PartialEq, Eq)]
struct WorktreeTarget {
    resolved: PathBuf,
    ancestors: Vec<PathIdentity>,
}

impl WorktreeTarget {
    fn capture(path: &Path, allow_missing: bool) -> Result<Self, GitToolError> {
        let mut spelling = path.to_owned();
        let mut ancestors = Vec::new();
        'resolve: loop {
            let mut ancestor = spelling.as_path();
            let mut suffix = Vec::new();
            let mut resolved = loop {
                match ancestor.canonicalize() {
                    Ok(resolved) => break resolved,
                    Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => {
                        // A dangling link is not an absent checkout component.
                        match std::fs::symlink_metadata(ancestor) {
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                            _ => return Err(invalid()),
                        }
                        match ancestor.components().next_back().ok_or_else(invalid)? {
                            std::path::Component::Normal(name) => suffix.push(name.to_owned()),
                            std::path::Component::ParentDir => suffix.push("..".into()),
                            _ => return Err(invalid()),
                        }
                        ancestor = ancestor.parent().ok_or_else(invalid)?;
                    }
                    Err(_) => return Err(invalid()),
                }
            };
            ancestors.push(PathIdentity::capture(ancestor).map_err(|_| invalid())?);
            let mut remaining = suffix.into_iter().rev();
            while let Some(name) = remaining.next() {
                if name == ".." {
                    // A cancelled missing component may expose an existing
                    // symlink. Re-resolve before interpreting any later `..`.
                    // Each pass removes a ParentDir; use a loop, not recursion.
                    resolved.pop();
                    resolved.extend(remaining);
                    spelling = resolved;
                    continue 'resolve;
                }
                resolved.push(name);
            }
            return Ok(Self {
                resolved,
                ancestors,
            });
        }
    }
}

fn invalid() -> GitToolError {
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
    }

    pub(super) fn with_reference_budget(
        git_dir: &Path,
        common_dir: &Path,
        reference_budget: ReferenceBudget,
    ) -> Self {
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
                pending.push(root.join(name));
            }
        }
        pending.sort();
        pending.dedup();
        Self {
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

    pub(super) fn paths(&self) -> Vec<PathBuf> {
        self.visited
            .keys()
            .cloned()
            .chain(self.pending.iter().cloned())
            .chain(self.worktree_targets.keys().cloned())
            .collect()
    }

    pub(super) fn complete(&self) -> bool {
        self.pending.is_empty()
    }

    /// Paths not covered by the bound metadata roots, plus raw pointer targets
    /// whose approved normalization must be checked separately before binding.
    pub(super) fn exact_paths(&self, gitdir: &Path, common: &Path) -> Vec<PathBuf> {
        self.paths()
            .into_iter()
            .filter(|path| {
                self.worktree_targets.contains_key(path)
                    || (!path.starts_with(gitdir) && !path.starts_with(common))
            })
            .collect()
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
        for (path, expected) in &self.worktree_targets {
            if let Some(expected) = expected
                && WorktreeTarget::capture(path, !self.worktree_common_targets.contains(path))
                    .ok()
                    .as_ref()
                    != Some(expected)
            {
                return Err(invalid());
            }
        }
        for (path, expected) in &self.visited {
            if &identity(path)? != expected {
                return Err(invalid());
            }
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
                    path.starts_with(root.join("refs"))
                        || path.starts_with(root.join("reftable"))
                        || path == &root.join("packed-refs")
                        || path == &root.join("HEAD")
                })
                || path
                    .strip_prefix(common.join("worktrees"))
                    .ok()
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
            let boundary = [gitdir, common]
                .iter()
                .any(|root| path == &root.join("shallow") || path == &root.join("info/grafts"));
            // get_worktrees() checks the main worktree's core.bare separately
            // even when its config.worktree is absent from the current chain.
            let worktree_config = path == &common.join("config.worktree");
            let pointer = path
                .strip_prefix(common.join("worktrees"))
                .ok()
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
            let Some(relative) = path.strip_prefix(common.join("worktrees")).ok() else {
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
            let target = path.parent().ok_or_else(invalid)?.join(text);
            if !gitdir {
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

    pub(super) fn bind_worktree_targets(
        &mut self,
        admitted: impl Fn(&Path) -> Option<PathBuf>,
    ) -> Result<bool, GitToolError> {
        // Check all admissions before touching any newly discovered target.
        let mappings = self
            .worktree_targets
            .keys()
            .map(|path| {
                admitted(path)
                    .map(|normalized| (path.clone(), normalized))
                    .ok_or_else(invalid)
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        self.revalidate()?;
        for (path, identity) in &mut self.worktree_targets {
            let current =
                WorktreeTarget::capture(path, !self.worktree_common_targets.contains(path))?;
            if current.resolved != mappings[path] {
                return Err(invalid());
            }
            if identity.is_none() {
                *identity = Some(current);
            }
        }
        self.revalidate()?;
        // Resolving a common directory does not grant access to its reference
        // store. Publish only names here; the next advance requires fresh grants
        // before inspecting their type, contents, or descendants.
        let mut added = false;
        for raw in &self.worktree_common_targets {
            let root = &mappings[raw];
            self.worktree_reference_roots.insert(root.clone());
            for name in ["refs", "packed-refs", "reftable"] {
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
        let mut budget = super::snapshot::Budget {
            entries: 4,
            bytes: 256 * 1024,
        };
        let mut evidence = Vec::new();
        for path in paths {
            let entry = super::snapshot::RegularEvidence::capture(&path, &mut budget)
                .map_err(|_| invalid())?;
            if !matches!(&entry.entry, super::snapshot::Entry::File(_, bytes) if bytes.is_empty()) {
                return Err(GitToolError::InvalidInput(
                    "Pull child alternate object stores require separate discovery and admission"
                        .into(),
                ));
            }
            evidence.push((path, entry));
        }
        self.revalidate()?;
        self.alternates = evidence;
        self.revalidate()
    }

    pub(super) fn advance(&mut self, admitted: impl Fn(&Path) -> bool) -> Result<(), GitToolError> {
        // Check every resource before metadata access, including previously
        // visited entries used for revalidation. Never traverse an unapproved
        // child simply because its parent was approved.
        if self.paths().iter().any(|path| !admitted(path)) {
            return Err(invalid());
        }
        self.revalidate()?;
        let mut next = Vec::new();
        for path in &self.pending {
            let before = identity(path)?;
            if before.is_some()
                && std::fs::symlink_metadata(path)
                    .map_err(|_| invalid())?
                    .is_dir()
            {
                let entries = children(path)?;
                for child in &entries {
                    if self.visited.len() + self.pending.len() + next.len() >= 4096 {
                        return Err(invalid());
                    }
                    // Only directory-entry names are consumed here. Child
                    // identity and type wait for the next exact-path admission.
                    next.push(child.clone());
                }
                self.directories.insert(path.clone(), entries);
            }
            if identity(path)? != before {
                return Err(invalid());
            }
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

fn children(path: &Path) -> Result<Vec<PathBuf>, GitToolError> {
    let mut entries = Vec::new();
    for child in std::fs::read_dir(path).map_err(|_| invalid())? {
        if entries.len() >= 4096 {
            return Err(invalid());
        }
        entries.push(child.map_err(|_| invalid())?.path());
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

#[cfg(test)]
#[path = "pull_inventory_tests.rs"]
mod tests;
