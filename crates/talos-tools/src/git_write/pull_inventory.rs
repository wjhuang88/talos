//! Bounded frontier discovery for Pull graph reads. The caller must authorize
//! every frontier before inspection, either by exact paths or the dedicated
//! identity-bound metadata scope. Neither mode permits traversing links.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{GitToolError, snapshot::PathIdentity};

#[derive(Debug)]
pub(super) struct Inventory {
    pending: Vec<PathBuf>,
    visited: BTreeMap<PathBuf, Option<PathIdentity>>,
    directories: BTreeMap<PathBuf, Vec<PathBuf>>,
    alternates: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    references: Vec<(PathBuf, super::snapshot::RegularEvidence)>,
    worktree_targets: BTreeMap<PathBuf, Option<PathIdentity>>,
}

fn invalid() -> GitToolError {
    GitToolError::InvalidInput(
        "Pull graph inventory changed, redirected, or exceeded its limit".into(),
    )
}

impl Inventory {
    pub(super) fn new(git_dir: &Path, common_dir: &Path) -> Self {
        let mut pending = Vec::new();
        for root in [git_dir, common_dir] {
            for name in [
                "HEAD",
                "refs",
                "logs",
                "objects",
                "info",
                "shallow",
                "packed-refs",
                "worktrees",
            ] {
                pending.push(root.join(name));
            }
        }
        pending.sort();
        pending.dedup();
        Self {
            pending,
            visited: BTreeMap::new(),
            directories: BTreeMap::new(),
            alternates: Vec::new(),
            references: Vec::new(),
            worktree_targets: BTreeMap::new(),
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
                && PathIdentity::capture(path).ok().as_ref() != Some(expected)
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
            if &children(path)? != expected {
                return Err(invalid());
            }
        }
        let mut budget = super::snapshot::Budget {
            entries: 4,
            bytes: 256 * 1024,
        };
        for (path, expected) in &self.alternates {
            if &super::snapshot::RegularEvidence::capture(path, &mut budget)
                .map_err(|_| invalid())?
                != expected
            {
                return Err(invalid());
            }
        }
        let mut budget = super::snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        for (path, expected) in &self.references {
            if &super::snapshot::RegularEvidence::capture(path, &mut budget)
                .map_err(|_| invalid())?
                != expected
            {
                return Err(invalid());
            }
        }
        Ok(())
    }

    /// Bind references, linked-worktree pointers and shallow/graft boundaries,
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
        let mut budget = super::snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        let mut evidence = Vec::new();
        for (path, identity) in &self.visited {
            let reference = [gitdir, common].iter().any(|root| {
                path.starts_with(root.join("refs"))
                    || path == &root.join("packed-refs")
                    || path == &root.join("HEAD")
            }) || path
                .strip_prefix(common.join("worktrees"))
                .ok()
                .is_some_and(|relative| {
                    // Linked worktree refs are per-worktree roots for --all.
                    // The first component is the registered worktree identifier.
                    let mut components = relative.components();
                    components.next().is_some()
                        && components.next().is_some_and(|name| {
                            name.as_os_str() == "HEAD" || name.as_os_str() == "refs"
                        })
                });
            let boundary = [gitdir, common]
                .iter()
                .any(|root| path == &root.join("shallow") || path == &root.join("info/grafts"));
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
            if (reference || boundary || pointer)
                && identity.is_some()
                && !self.directories.contains_key(path)
            {
                let entry = super::snapshot::RegularEvidence::capture(path, &mut budget)
                    .map_err(|_| invalid())?;
                if reference && let super::snapshot::Entry::File(_, bytes) = &entry.entry {
                    super::receiver_state::validate_symbolic_target(bytes)?;
                }
                evidence.push((path.clone(), entry));
            }
        }
        self.revalidate()?;
        self.references = evidence;
        self.revalidate()
    }

    /// Parse only retained pointer bytes. Target filesystem access waits for
    /// the next exact-path admission; these targets are not recursive grants.
    pub(super) fn discover_worktree_targets(
        &mut self,
        common: &Path,
    ) -> Result<bool, GitToolError> {
        self.revalidate()?;
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
            super::remote_state::validate_local_path_syntax(Path::new(text))?;
            // get_linked_worktree() only resolves a relative backlink. An
            // absolute backlink is retained as text, even for a pruned checkout.
            if gitdir && Path::new(text).is_absolute() {
                continue;
            }
            let target = path.parent().ok_or_else(invalid)?.join(text);
            if !self.worktree_targets.contains_key(&target) {
                if self.visited.len() + self.worktree_targets.len() >= 4096 {
                    return Err(invalid());
                }
                self.worktree_targets.insert(target, None);
                added = true;
            }
        }
        self.revalidate()?;
        Ok(added)
    }

    pub(super) fn bind_worktree_targets(
        &mut self,
        admitted: impl Fn(&Path) -> Option<PathBuf>,
    ) -> Result<(), GitToolError> {
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
            if path.canonicalize().map_err(|_| invalid())? != mappings[path] {
                return Err(invalid());
            }
            if identity.is_none() {
                *identity = Some(PathIdentity::capture(path).map_err(|_| invalid())?);
            }
        }
        self.revalidate()
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
mod tests {
    use super::*;

    #[test]
    fn linked_pointer_targets_wait_for_fresh_admission() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let registration = root.join("worktrees/linked");
        std::fs::create_dir_all(&registration).expect("registration");
        let target = root.join("separately-admitted");
        std::fs::write(
            registration.join("gitdir"),
            "../../separately-admitted/.git \t\n",
        )
        .expect("pointer");
        std::fs::write(registration.join("commondir"), "../..\r\n").expect("common");
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("frontier");
        }
        inventory
            .bind_references(&root, &root)
            .expect("pointer evidence");
        let previous = inventory.paths();
        // Target does not exist: discovery must not resolve it.
        assert!(
            inventory
                .discover_worktree_targets(&root)
                .expect("raw targets")
        );
        let raw_target = registration.join("../../separately-admitted");
        assert!(inventory.worktree_targets.contains_key(&raw_target));
        assert!(
            inventory
                .worktree_targets
                .contains_key(&registration.join("../.."))
        );
        assert!(
            inventory
                .bind_worktree_targets(|path| previous
                    .contains(&path.to_owned())
                    .then(|| path.to_owned()))
                .is_err()
        );
        assert!(inventory.worktree_targets.values().all(Option::is_none));
        std::fs::create_dir(&target).expect("target created independently");
        assert!(
            inventory
                .bind_worktree_targets(|_| Some(root.clone()))
                .is_err()
        );
        inventory
            .bind_worktree_targets(|path| path.canonicalize().ok())
            .expect("fresh admission");
        assert!(
            !inventory
                .discover_worktree_targets(&root)
                .expect("stable targets")
        );
        inventory.revalidate().expect("unchanged targets");
        std::fs::rename(&target, root.join("saved-target")).expect("move target");
        std::fs::create_dir(&target).expect("replacement target");
        #[cfg(any(unix, windows))]
        assert!(inventory.revalidate().is_err());
    }

    #[test]
    fn absolute_worktree_backlink_does_not_resolve_pruned_checkout() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let registration = root.join("worktrees/pruned");
        std::fs::create_dir_all(&registration).expect("registration");
        let missing = root.join("missing/.git");
        std::fs::write(
            registration.join("gitdir"),
            format!("{}\n", missing.display()),
        )
        .expect("pointer");
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("frontier");
        }
        inventory.bind_references(&root, &root).expect("evidence");
        assert!(
            !inventory
                .discover_worktree_targets(&root)
                .expect("absolute text only")
        );
        assert!(inventory.worktree_targets.is_empty());
        inventory
            .bind_worktree_targets(|_| None)
            .expect("no target access");
    }

    #[test]
    fn linked_worktree_references_require_admission_and_retain_contents() {
        for leaf in ["HEAD", "refs/heads/private", "gitdir", "commondir"] {
            let temp = tempfile::tempdir().expect("fixture");
            let root = temp.path().canonicalize().expect("root");
            let path = root.join("worktrees/linked").join(leaf);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("directories");
            std::fs::write(&path, format!("{}\n", "a".repeat(40))).expect("reference");
            let mut inventory = Inventory::new(&root, &root);
            let initial = inventory.paths();
            assert!(initial.contains(&root.join("worktrees")));
            inventory
                .advance(|path| initial.contains(&path.to_owned()))
                .expect("root admission");
            assert!(
                inventory
                    .advance(|path| initial.contains(&path.to_owned()))
                    .is_err()
            );
            assert!(inventory.bind_references(&root, &root).is_err());
            while !inventory.complete() {
                inventory
                    .advance(|_| true)
                    .expect("fresh frontier admission");
            }
            inventory
                .bind_references(&root, &root)
                .expect("bind references");
            assert!(inventory.references.iter().any(|(bound, _)| bound == &path));
            std::fs::write(&path, format!("{}\n", "b".repeat(40))).expect("in-place change");
            assert!(inventory.revalidate().is_err());
        }
    }

    #[test]
    fn graph_boundaries_bind_in_place_changes_and_absence() {
        for name in ["shallow", "info/grafts"] {
            for exists in [false, true] {
                let temp = tempfile::tempdir().expect("fixture");
                let root = temp.path().canonicalize().expect("root");
                std::fs::create_dir(root.join("info")).expect("info");
                let path = root.join(name);
                if exists {
                    std::fs::write(&path, format!("{}\n", "a".repeat(40)))
                        .expect("initial graph boundary");
                }
                let mut inventory = Inventory::new(&root, &root);
                assert!(inventory.bind_references(&root, &root).is_err());
                while !inventory.complete() {
                    inventory.advance(|_| true).expect("admitted frontier");
                }
                inventory.bind_references(&root, &root).expect("bind graph");
                inventory.revalidate().expect("unchanged graph");
                std::fs::write(&path, format!("{}\n", "b".repeat(40)))
                    .expect("change graph boundary");
                assert!(inventory.revalidate().is_err(), "{name}, exists={exists}");
            }
        }
    }

    #[test]
    fn replacement_symbolic_target_waits_for_full_reference_frontier() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("refs/replace")).expect("replacement directory");
        std::fs::create_dir_all(root.join("refs/heads/a/b")).expect("target directory");
        std::fs::write(
            root.join("refs/replace/object"),
            b"ref: refs/heads/a/b/target\n",
        )
        .expect("symbolic replacement");
        std::fs::write(root.join("refs/heads/a/b/target"), b"replacement\n").expect("target");
        let mut inventory = Inventory::new(&root, &root);
        for _ in 0..3 {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        assert!(inventory.subtree_complete(&root.join("refs/replace")));
        assert!(!inventory.subtree_complete(&root.join("refs")));
        let old_paths = inventory.paths();
        inventory.advance(|_| true).expect("target name discovery");
        assert!(
            inventory
                .advance(|path| old_paths.iter().any(|old| old == path))
                .is_err()
        );
        assert!(!inventory.subtree_complete(&root.join("refs")));
        inventory.advance(|_| true).expect("target admission");
        assert!(inventory.subtree_complete(&root.join("refs")));
    }

    #[test]
    fn replacement_refs_wait_for_descendants_and_absence_is_bound() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
        std::fs::write(root.join("refs/replace/object"), b"replacement").expect("ref");
        let mut inventory = Inventory::new(&root, &root);
        inventory.advance(|_| true).expect("roots");
        assert!(!inventory.subtree_complete(&root.join("refs/replace")));
        inventory.advance(|_| true).expect("replace directory");
        assert!(!inventory.subtree_complete(&root.join("refs/replace")));
        inventory.advance(|_| true).expect("reference");
        assert!(inventory.subtree_complete(&root.join("refs/replace")));
        assert!(inventory.subtree_complete(&root.join("refs/missing")));
        std::fs::create_dir(root.join("refs/missing")).expect("new subtree");
        assert!(inventory.revalidate().is_err());
    }

    #[test]
    fn child_symbolic_reference_cannot_escape_retained_namespace() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
        std::fs::write(root.join("refs/replace/object"), b"ref: FETCH_HEAD\n")
            .expect("pseudo target");
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        assert!(inventory.bind_references(&root, &root).is_err());
    }

    #[test]
    fn child_metadata_waits_for_its_own_admission_and_changes_refuse() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir(root.join("objects")).expect("objects");
        std::fs::create_dir(root.join("objects/aa")).expect("fanout");
        std::fs::write(root.join("objects/aa/object"), b"object").expect("object");
        let mut inventory = Inventory::new(&root, &root);
        assert!(!inventory.subtree_complete(&root.join("objects")));
        let initial = inventory.paths();
        inventory
            .advance(|path| initial.iter().any(|allowed| allowed == path))
            .expect("first frontier");
        assert!(inventory.paths().contains(&root.join("objects/aa")));
        assert!(!inventory.subtree_complete(&root.join("objects")));
        assert!(
            inventory
                .advance(|path| initial.iter().any(|allowed| allowed == path))
                .is_err()
        );
        inventory.advance(|_| true).expect("second frontier");
        assert!(!inventory.subtree_complete(&root.join("objects")));
        inventory.advance(|_| true).expect("files");
        assert!(inventory.subtree_complete(&root.join("objects")));
        assert!(inventory.complete());
        std::fs::write(root.join("shallow"), b"new").expect("new leaf");
        assert!(inventory.revalidate().is_err());
    }

    #[test]
    fn child_alternates_wait_for_inventory_and_bind_in_place_changes() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("objects/info")).expect("objects");
        let path = root.join("objects/info/alternates");
        std::fs::write(&path, b"").expect("empty");
        let mut inventory = Inventory::new(&root, &root);
        assert!(inventory.bind_alternates(&root, &root).is_err());
        while !inventory.complete() {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        inventory
            .bind_alternates(&root, &root)
            .expect("empty alternates");
        std::fs::write(&path, b"/unadmitted/object/store\n").expect("redirect");
        assert!(inventory.revalidate().is_err());
        let mut changed = Inventory::new(&root, &root);
        while !changed.complete() {
            changed.advance(|_| true).expect("admitted frontier");
        }
        assert!(changed.bind_alternates(&root, &root).is_err());
    }

    #[test]
    fn child_absent_alternates_are_bound_by_directory_listing() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("objects/info")).expect("objects");
        let mut inventory = Inventory::new(&root, &root);
        while !inventory.complete() {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        inventory
            .bind_alternates(&root, &root)
            .expect("absent alternates");
        assert!(inventory.alternates.is_empty());
        std::fs::write(
            root.join("objects/info/http-alternates"),
            b"http://unadmitted.invalid\n",
        )
        .expect("new redirect");
        assert!(inventory.revalidate().is_err());
    }

    #[test]
    fn child_reference_contents_bind_in_place_symbolic_target_changes() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("refs/replace")).expect("refs");
        std::fs::create_dir_all(root.join("refs/heads")).expect("heads");
        let target = root.join("refs/heads/target");
        std::fs::write(&target, format!("{}\n", "a".repeat(40))).expect("target");
        std::fs::write(
            root.join("refs/replace/object"),
            b"ref: refs/heads/target\n",
        )
        .expect("symbolic replacement");
        let mut inventory = Inventory::new(&root, &root);
        assert!(inventory.bind_references(&root, &root).is_err());
        while !inventory.complete() {
            inventory.advance(|_| true).expect("admitted frontier");
        }
        inventory
            .bind_references(&root, &root)
            .expect("reference bytes");
        std::fs::write(&target, format!("{}\n", "b".repeat(40))).expect("in-place change");
        assert!(inventory.revalidate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_child_is_not_traversed_during_parent_discovery() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir(root.join("objects")).expect("objects");
        std::os::unix::fs::symlink("/unapproved", root.join("objects/escape")).expect("link");
        let mut inventory = Inventory::new(&root, &root);
        inventory.advance(|_| true).expect("names only");
        assert!(inventory.advance(|_| true).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn discovered_directory_replacement_refuses_even_with_closed_scope() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let metadata = root.join("metadata");
        let external = root.join("external");
        std::fs::create_dir(&metadata).expect("metadata");
        std::fs::create_dir_all(metadata.join("refs/heads")).expect("refs");
        std::fs::create_dir(&external).expect("external");
        std::fs::write(external.join("outside"), b"unapproved reference").expect("outside");
        let mut inventory = Inventory::new(&metadata, &metadata);
        inventory.advance(|_| true).expect("discover refs");
        inventory.advance(|_| true).expect("discover heads");
        std::fs::rename(metadata.join("refs/heads"), metadata.join("saved-heads"))
            .expect("move discovered directory");
        std::os::unix::fs::symlink(&external, metadata.join("refs/heads"))
            .expect("redirect discovered directory");
        assert!(inventory.revalidate().is_err());
        assert!(inventory.advance(|_| true).is_err());
        assert!(
            !inventory
                .paths()
                .contains(&metadata.join("refs/heads/outside"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn redirected_metadata_parent_refuses_before_initial_enumeration() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        let external = root.join("external");
        std::fs::create_dir_all(external.join("refs")).expect("external refs");
        std::fs::write(external.join("refs/outside"), b"unapproved reference").expect("outside");
        let redirected = root.join("metadata");
        std::os::unix::fs::symlink(&external, &redirected).expect("redirect root");
        let mut inventory = Inventory::new(&redirected, &redirected);
        assert!(inventory.advance(|_| true).is_err());
        assert!(!inventory.paths().contains(&redirected.join("refs/outside")));
    }

    #[test]
    fn added_directory_entry_invalidates_discovered_inventory() {
        let temp = tempfile::tempdir().expect("fixture");
        let root = temp.path().canonicalize().expect("root");
        std::fs::create_dir(root.join("objects")).expect("objects");
        let mut inventory = Inventory::new(&root, &root);
        inventory.advance(|_| true).expect("initial inventory");
        assert!(inventory.complete());
        std::fs::write(root.join("objects/unadmitted"), b"new object").expect("new entry");
        assert!(inventory.revalidate().is_err());
        assert!(inventory.advance(|_| true).is_err());
    }
}
