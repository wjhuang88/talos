//! Parent-tree evidence for recursive Pull discovery. These records identify
//! objects only; they do not authorize opening a child repository or fetching it.

use std::collections::BTreeMap;

use super::GitToolError;

/// Invalid modes are retained until candidate selection. Historical invalid
/// modes use demand fallback; selected invalid current configuration must refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Recursion {
    Off,
    On,
    OnDemand,
    Invalid,
}

impl Recursion {
    fn parse(value: Option<&[u8]>) -> Self {
        if value == Some(b"on-demand".as_slice()) {
            Self::OnDemand
        } else {
            match super::config_state::git_boolean(value) {
                Ok(true) => Self::On,
                Ok(false) => Self::Off,
                Err(_) => Self::Invalid,
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Tree {
    pub(super) gitlinks: BTreeMap<String, String>,
    pub(super) modules_blob: Option<String>,
    pub(super) changed_gitlinks: BTreeMap<String, String>,
    pub(super) module_names: BTreeMap<String, String>,
    pub(super) module_recursion: BTreeMap<String, Recursion>,
    pub(super) declared_names: std::collections::BTreeSet<String>,
}

/// Current index candidates and sparse subtrees. Construction reads no paths.
pub(super) struct Index {
    pub(super) gitlinks: BTreeMap<String, String>,
    pub(super) sparse: BTreeMap<String, String>,
    pub(super) modules_blob: Option<String>,
}

impl Index {
    pub(super) fn parse(index: &gix::index::State, width: usize) -> Result<Self, GitToolError> {
        std::panic::catch_unwind(|| {
            let mut listing = Vec::new();
            let mut sparse = BTreeMap::new();
            for entry in index.entries() {
                if entry.stage_raw() != 0 {
                    return Err(GitToolError::InvalidInput(
                        "Pull index has unresolved stages".into(),
                    ));
                }
                let mode = entry.mode.bits();
                let raw = entry.path(index);
                if raw == b".gitmodules" {
                    if !matches!(mode, 0o100644 | 0o100755) {
                        return Err(GitToolError::InvalidInput(
                            "Pull index module declarations are not regular".into(),
                        ));
                    }
                    listing.extend_from_slice(
                        format!("{mode:o} blob {}\t.gitmodules\0", entry.id).as_bytes(),
                    );
                    continue;
                }
                if !matches!(mode, 0o040000 | 0o160000) {
                    continue;
                }
                let path = if mode == 0o040000 {
                    raw.strip_suffix(b"/").unwrap_or(raw)
                } else {
                    raw
                };
                // Reuse strict tree path/OID validation for both candidate kinds.
                listing.extend_from_slice(format!("160000 commit {}\t", entry.id).as_bytes());
                listing.extend_from_slice(path);
                listing.push(0);
                if mode == 0o040000 {
                    let name = std::str::from_utf8(path).map_err(|_| {
                        GitToolError::InvalidInput("Invalid Pull sparse path".into())
                    })?;
                    sparse.insert(name.to_owned(), entry.id.to_string());
                }
            }
            let tree = Tree::parse(&listing, width)?;
            let mut gitlinks = tree.gitlinks;
            for path in sparse.keys() {
                gitlinks.remove(path);
            }
            Ok(Self {
                gitlinks,
                sparse,
                modules_blob: tree.modules_blob,
            })
        })
        .map_err(|_| GitToolError::Git("Pull index child decoder panicked".into()))?
    }
}

/// A declared module's changed objects, in parent revision-walk order.
/// Unnamed gitlinks remain in Tree until separately admitted populated-path
/// discovery can resolve Git's legacy name fallback.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Changed {
    pub(super) commit: String,
    pub(super) path: String,
    pub(super) objects: std::collections::BTreeSet<String>,
    pub(super) historical_recursion: Option<Recursion>,
    pub(super) effective_recursion: Option<Recursion>,
}

pub(super) fn collect_named_changes(
    commits: &[String],
    trees: &BTreeMap<String, Tree>,
) -> Result<BTreeMap<String, Changed>, GitToolError> {
    let mut result: BTreeMap<String, Changed> = BTreeMap::new();
    for commit in commits {
        let tree = trees.get(commit).ok_or_else(|| {
            GitToolError::InvalidInput("Missing Pull child tree during aggregation".into())
        })?;
        for (path, object) in &tree.changed_gitlinks {
            let Some(name) = tree.module_names.get(path) else {
                continue;
            };
            let changed = result.entry(name.clone()).or_insert_with(|| Changed {
                commit: commit.clone(),
                path: path.clone(),
                historical_recursion: tree.module_recursion.get(name).copied(),
                effective_recursion: None,
                objects: std::collections::BTreeSet::new(),
            });
            changed.objects.insert(object.clone());
        }
    }
    Ok(result)
}

impl Tree {
    /// Select declared current-index candidates without opening child paths.
    /// Unmapped paths remain in the caller's index inventory for separately
    /// admitted legacy discovery. Selection does not mark module names seen:
    /// native Git does that only after an initialized repository yields a task.
    pub(super) fn select_current(
        &self,
        index: &BTreeMap<String, String>,
        changed: &BTreeMap<String, Changed>,
        policy: impl Fn(&str, Option<Recursion>) -> Result<Recursion, GitToolError>,
    ) -> Result<BTreeMap<String, Recursion>, GitToolError> {
        let mut selected = BTreeMap::new();
        for path in index.keys() {
            let Some(name) = self.module_names.get(path) else {
                continue;
            };
            let mode = policy(name, self.module_recursion.get(name).copied())?;
            match mode {
                Recursion::On => {}
                Recursion::OnDemand if changed.contains_key(name) => {}
                Recursion::Invalid => {
                    return Err(GitToolError::InvalidInput(
                        "Invalid current Pull child recursion policy".into(),
                    ));
                }
                _ => continue,
            }
            selected.insert(path.clone(), mode);
        }
        Ok(selected)
    }

    /// Parse an admitted `ls-tree -r -z --full-tree` listing without filesystem
    /// discovery, config includes, path expansion, or child repository access.
    pub(super) fn parse(bytes: &[u8], width: usize) -> Result<Self, GitToolError> {
        let invalid = || GitToolError::InvalidInput("Invalid Pull child tree evidence".into());
        if !matches!(width, 40 | 64) || bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let mut tree = Self {
            gitlinks: BTreeMap::new(),
            modules_blob: None,
            module_names: BTreeMap::new(),
            declared_names: std::collections::BTreeSet::new(),
            module_recursion: BTreeMap::new(),
            changed_gitlinks: BTreeMap::new(),
        };
        if bytes.is_empty() {
            return Ok(tree);
        }
        let mut paths = std::collections::BTreeSet::new();
        for record in bytes
            .strip_suffix(&[0])
            .ok_or_else(invalid)?
            .split(|b| *b == 0)
        {
            let tab = record
                .iter()
                .position(|b| *b == b'\t')
                .ok_or_else(invalid)?;
            let fields: Vec<_> = record[..tab].split(|b| *b == b' ').collect();
            if fields.len() != 3
                || fields[2].len() != width
                || !fields[2].iter().all(u8::is_ascii_hexdigit)
                || fields[2].iter().all(|b| *b == b'0')
            {
                return Err(invalid());
            }
            let name = std::str::from_utf8(&record[tab + 1..]).map_err(|_| invalid())?;
            if name.is_empty()
                || name.contains('\\')
                || std::path::Path::new(name)
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
                || name.split('/').any(|part| {
                    matches!(part, "" | "." | "..") || part.eq_ignore_ascii_case(".git")
                })
                || !paths.insert(name)
            {
                return Err(invalid());
            }
            let oid = std::str::from_utf8(fields[2])
                .map_err(|_| invalid())?
                .to_ascii_lowercase();
            match (fields[0], fields[1]) {
                (b"160000", b"commit") if name != ".gitmodules" => {
                    tree.gitlinks.insert(name.to_owned(), oid);
                }
                (b"100644" | b"100755", b"blob") => {
                    if name == ".gitmodules" {
                        tree.modules_blob = Some(oid);
                    }
                }
                (b"120000", b"blob") if name != ".gitmodules" => {}
                _ => return Err(invalid()),
            }
        }
        Ok(tree)
    }

    /// Retain only new-side gitlinks named by Git's combined diff. Deleted
    /// links and regular files are not child-fetch candidates. No paths are opened.
    pub(super) fn bind_changes(&mut self, bytes: &[u8]) -> Result<(), GitToolError> {
        let invalid = || GitToolError::InvalidInput("Invalid Pull child change listing".into());
        if bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let mut changed = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        if !bytes.is_empty() {
            for path in bytes
                .strip_suffix(&[0])
                .ok_or_else(invalid)?
                .split(|b| *b == 0)
            {
                let path = std::str::from_utf8(path).map_err(|_| invalid())?;
                if path.contains('\\')
                    || path.split('/').any(|part| {
                        matches!(part, "" | "." | "..") || part.eq_ignore_ascii_case(".git")
                    })
                    || !seen.insert(path)
                {
                    return Err(invalid());
                }
                if let Some(oid) = self.gitlinks.get(path) {
                    changed.insert(path.to_owned(), oid.clone());
                }
            }
        }
        self.changed_gitlinks = changed;
        Ok(())
    }

    /// Resolve path-to-name declarations from an already admitted blob. Git
    /// ignores includes in historical .gitmodules; never resolve them here.
    pub(super) fn bind_modules(&mut self, bytes: &[u8]) -> Result<(), GitToolError> {
        self.bind_module_declarations(bytes, false)
    }

    /// Parse admitted declarations; current files overwrite and reject invalid policy early.
    pub(super) fn bind_module_declarations(
        &mut self,
        bytes: &[u8],
        current: bool,
    ) -> Result<(), GitToolError> {
        let invalid = || GitToolError::InvalidInput("Invalid Pull module declarations".into());
        if bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let (names, declared_names, module_recursion) = std::panic::catch_unwind(|| {
            use gix::config::parse::EventRef as Event;
            let file =
                gix::config::parse::Events::from_bytes(bytes, None).map_err(|_| invalid())?;
            let mut by_name = BTreeMap::new();
            let mut recursion = BTreeMap::new();
            let mut declared = std::collections::BTreeSet::new();
            for section in file.sections() {
                let Event::SectionHeader {
                    name: section_name,
                    subsection_name,
                    ..
                } = section.header()
                else {
                    return Err(invalid());
                };
                if !section_name.eq_ignore_ascii_case(b"submodule") {
                    continue;
                }
                let Some(name) = subsection_name else {
                    continue;
                };
                let name = std::str::from_utf8(name.as_ref()).map_err(|_| invalid())?;
                if name.is_empty() || name.contains(['\0', '\n', '\r']) {
                    return Err(invalid());
                }
                declared.insert(name.to_owned());
                // Preserve the first declaration, including whether it had '='.
                // Body value queries select different duplicates for implicit keys.
                let mut is_path = false;
                let mut is_recursion = false;
                let mut explicit = false;
                let mut raw = Vec::new();
                let mut first_path = None;
                for event in section.body() {
                    match event {
                        Event::SectionValueName(key) => {
                            is_path = key.eq_ignore_ascii_case(b"path")
                                && (current
                                    || (!by_name.contains_key(name) && first_path.is_none()));
                            is_recursion = key.eq_ignore_ascii_case(b"fetchrecursesubmodules")
                                && (current || !recursion.contains_key(name));
                            explicit = false;
                            raw.clear();
                        }
                        Event::KeyValueSeparator => explicit = true,
                        Event::ValueNotDone(value) if is_path || is_recursion => {
                            raw.extend_from_slice(value.as_ref());
                        }
                        Event::Value(value) | Event::ValueDone(value)
                            if is_path || is_recursion =>
                        {
                            raw.extend_from_slice(value.as_ref());
                            let normalized = gix::config::value::normalize(raw.as_slice());
                            if is_path {
                                if !explicit {
                                    return Err(invalid());
                                }
                                first_path = Some(normalized.into_owned());
                            } else {
                                let mode =
                                    Recursion::parse(explicit.then_some(normalized.as_ref()));
                                if current && mode == Recursion::Invalid {
                                    return Err(invalid());
                                }
                                recursion.insert(name.to_owned(), mode);
                            }
                            is_path = false;
                            is_recursion = false;
                        }
                        _ => {}
                    }
                }
                let Some(path) = first_path else {
                    continue;
                };
                let path = std::str::from_utf8(path.as_ref()).map_err(|_| invalid())?;
                if path.starts_with('-')
                    || path.is_empty()
                    || path.contains('\\')
                    || path.split('/').any(|part| {
                        matches!(part, "" | "." | "..") || part.eq_ignore_ascii_case(".git")
                    })
                {
                    return Err(invalid());
                }
                by_name.insert(name.to_owned(), path.to_owned());
            }
            let mut by_path = BTreeMap::new();
            for (name, path) in by_name {
                if by_path.insert(path, name).is_some() {
                    return Err(invalid());
                }
            }
            Ok((by_path, declared, recursion))
        })
        .map_err(|_| invalid())??;
        self.module_names = names;
        self.declared_names = declared_names;
        self.module_recursion = module_recursion;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_current_selection_uses_names_and_preserves_legacy_inventory() {
        let mut modules = Tree::parse(b"", 40).expect("empty tree");
        modules.bind_module_declarations(
            b"[submodule \"renamed\"]\npath = current\nfetchRecurseSubmodules = on-demand\n[submodule \"always\"]\npath = unchanged\nfetchRecurseSubmodules = true\n[submodule \"off\"]\npath = disabled\nfetchRecurseSubmodules = false\n", true,
        ).expect("modules");
        let index = ["current", "unchanged", "disabled", "legacy"]
            .into_iter()
            .map(|path| (path.to_owned(), "a".repeat(40)))
            .collect();
        let mut historical = Tree::parse(b"", 40).expect("history");
        historical
            .changed_gitlinks
            .insert("old".into(), "b".repeat(40));
        historical
            .module_names
            .insert("old".into(), "renamed".into());
        let changed = collect_named_changes(
            &["commit".into()],
            &BTreeMap::from([("commit".into(), historical)]),
        )
        .expect("changes");
        let selected = modules
            .select_current(&index, &changed, |_, mode| {
                Ok(mode.unwrap_or(Recursion::OnDemand))
            })
            .expect("selection");
        assert_eq!(
            selected,
            BTreeMap::from([
                ("current".into(), Recursion::OnDemand),
                ("unchanged".into(), Recursion::On)
            ])
        );
        assert!(index.contains_key("legacy"));
        let selected = modules
            .select_current(&index, &BTreeMap::new(), |_, mode| {
                Ok(mode.unwrap_or(Recursion::OnDemand))
            })
            .expect("unchanged");
        assert_eq!(selected.len(), 1);
        assert!(selected.contains_key("unchanged"));
        assert!(
            modules
                .select_current(&index, &changed, |_, _| Ok(Recursion::Invalid))
                .is_err()
        );
    }

    #[test]
    fn pull_index_children_preserve_sparse_trees_and_reject_ambiguous_paths() {
        use gix::index::entry::{Flags, Mode};
        for (kind, width) in [(gix::hash::Kind::Sha1, 40), (gix::hash::Kind::Sha256, 64)] {
            let oid = gix::hash::ObjectId::from_hex("a".repeat(width).as_bytes()).expect("OID");
            let mut index = gix::index::State::new(kind);
            let push = |index: &mut gix::index::State, mode, path: &str| {
                index.dangerously_push_entry(
                    Default::default(),
                    oid,
                    Flags::empty(),
                    mode,
                    gix::bstr::BStr::new(path),
                );
            };
            push(&mut index, Mode::COMMIT, "nested/child");
            push(&mut index, Mode::DIR, "sparse/");
            push(&mut index, Mode::FILE, "ordinary");
            push(&mut index, Mode::FILE, ".gitmodules");
            let candidates = Index::parse(&index, width).expect("candidates");
            assert_eq!(candidates.gitlinks.len(), 1);
            assert_eq!(candidates.gitlinks["nested/child"], oid.to_string());
            assert_eq!(candidates.sparse["sparse"], oid.to_string());
            assert_eq!(candidates.modules_blob, Some(oid.to_string()));
            let mut linked_modules = gix::index::State::new(kind);
            push(&mut linked_modules, Mode::SYMLINK, ".gitmodules");
            assert!(Index::parse(&linked_modules, width).is_err());
            index.entries_mut()[2].flags = Flags::STAGE_MASK;
            assert!(
                Index::parse(&index, width).is_err(),
                "conflict even on ordinary files"
            );
            index.entries_mut()[2].flags = Flags::empty();
            push(&mut index, Mode::COMMIT, "nested/child");
            assert!(Index::parse(&index, width).is_err(), "duplicate candidate");
            for path in ["../outside", "a/.git/b", "/absolute", "a\\b"] {
                let mut invalid = gix::index::State::new(kind);
                push(&mut invalid, Mode::COMMIT, path);
                assert!(Index::parse(&invalid, width).is_err(), "{path}");
            }
        }
    }

    #[test]
    fn pull_named_changes_keep_first_revision_path_and_all_objects() {
        let mut trees = BTreeMap::new();
        for (commit, path, object) in [("z", "renamed", "b"), ("a", "old", "a")] {
            let mut tree = Tree::parse(b"", 40).expect("tree");
            tree.changed_gitlinks.insert(path.into(), object.repeat(40));
            tree.changed_gitlinks
                .insert("undeclared".into(), "c".repeat(40));
            tree.module_names.insert(path.into(), "Child".into());
            tree.module_recursion.insert(
                "Child".into(),
                if commit == "z" {
                    Recursion::Off
                } else {
                    Recursion::On
                },
            );
            // Module names are case sensitive.
            tree.changed_gitlinks.insert("lower".into(), "d".repeat(40));
            tree.module_names.insert("lower".into(), "child".into());
            trees.insert(commit.into(), tree);
        }
        let commits = vec!["z".into(), "a".into()];
        let changed = collect_named_changes(&commits, &trees).expect("changes");
        assert_eq!(changed.len(), 2);
        assert_eq!(changed["Child"].commit, "z");
        assert_eq!(changed["Child"].path, "renamed");
        assert_eq!(changed["Child"].historical_recursion, Some(Recursion::Off));
        assert_eq!(changed["Child"].objects.len(), 2);
        assert_eq!(changed["child"].objects.len(), 1);
        assert!(collect_named_changes(&["missing".into()], &trees).is_err());
    }

    #[test]
    fn pull_combined_changes_exclude_links_inherited_from_a_parent() {
        let root = tempfile::tempdir().expect("repository");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .current_dir(root.path())
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", root.path().join("absent"))
                .env("GIT_AUTHOR_NAME", "Fixture")
                .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
                .env("GIT_COMMITTER_NAME", "Fixture")
                .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
                .args(args)
                .output()
                .expect("Git");
            assert!(output.status.success(), "{output:?}");
            output.stdout
        };
        git(&["init", "-q"]);
        let text = |bytes: Vec<u8>| String::from_utf8(bytes).expect("OID").trim().to_owned();
        let empty = text(git(&["write-tree"]));
        let base = text(git(&["commit-tree", &empty, "-m", "base"]));
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{base},child"),
        ]);
        let first_tree = text(git(&["write-tree"]));
        let first = text(git(&[
            "commit-tree",
            &first_tree,
            "-p",
            &base,
            "-m",
            "first",
        ]));
        git(&[
            "update-index",
            "--cacheinfo",
            &format!("160000,{first},child"),
        ]);
        let second_tree = text(git(&["write-tree"]));
        let second = text(git(&[
            "commit-tree",
            &second_tree,
            "-p",
            &base,
            "-m",
            "second",
        ]));
        git(&[
            "update-index",
            "--cacheinfo",
            &format!("160000,{second},child"),
        ]);
        let novel_tree = text(git(&["write-tree"]));
        for (tree_oid, expected) in [
            (&first_tree, false),
            (&second_tree, false),
            (&novel_tree, true),
        ] {
            let commit = text(git(&[
                "commit-tree",
                tree_oid,
                "-p",
                &first,
                "-p",
                &second,
                "-m",
                "merge",
            ]));
            let listing = git(&["ls-tree", "-r", "-z", "--full-tree", &commit]);
            let mut tree = Tree::parse(&listing, 40).expect("tree");
            let changes = git(&[
                "diff-tree",
                "-r",
                "--cc",
                "--name-only",
                "-z",
                "--no-commit-id",
                "--no-renames",
                "--no-ext-diff",
                "--no-textconv",
                "--ignore-submodules=none",
                &commit,
                "--",
            ]);
            tree.bind_changes(&changes).expect("changes");
            assert_eq!(!tree.changed_gitlinks.is_empty(), expected);
            let retained = tree.changed_gitlinks.clone();
            for bad in [
                b"child".as_slice(),
                b"../escape\0",
                b"child\0child\0",
                b"/absolute\0",
            ] {
                assert!(tree.bind_changes(bad).is_err());
                assert_eq!(tree.changed_gitlinks, retained);
            }
            tree.bind_changes(b"deleted\0ordinary\0")
                .expect("non-links");
            assert!(tree.changed_gitlinks.is_empty());
        }
    }

    #[test]
    fn pull_current_modules_overwrite_and_refuse_invalid_values_atomically() {
        let mut tree = Tree::parse(b"", 40).expect("tree");
        tree.bind_module_declarations(
            br#"[include]
path = /must/not/be/read
[submodule "child"]
path = old
path = intermediate
fetchRecurseSubmodules = true
[submodule "child"]
path = final/child
fetchRecurseSubmodules = on-demand
[submodule "Child"]
path = other
fetchRecurseSubmodules
fetchRecurseSubmodules =
"#,
            true,
        )
        .expect("current declarations");
        assert_eq!(tree.module_names.len(), 2);
        assert_eq!(tree.module_names["final/child"], "child");
        assert_eq!(tree.module_names["other"], "Child");
        assert_eq!(tree.module_recursion["child"], Recursion::OnDemand);
        assert_eq!(tree.module_recursion["Child"], Recursion::Off);
        let retained_names = tree.module_names.clone();
        let retained_recursion = tree.module_recursion.clone();
        for bytes in [
            b"[submodule \"child\"]\npath = valid\npath\n".as_slice(),
            b"[submodule \"child\"]\nfetchRecurseSubmodules = invalid\nfetchRecurseSubmodules = true\n",
        ] {
            assert!(tree.bind_module_declarations(bytes, true).is_err());
            assert_eq!(tree.module_names, retained_names);
            assert_eq!(tree.module_recursion, retained_recursion);
        }
    }

    #[test]
    fn pull_historical_recursion_is_first_per_key_across_sections() {
        let mut tree = Tree::parse(b"", 40).expect("tree");
        tree.bind_modules(
            br#"[submodule "child"]
path = nested/child
[submodule "child"]
fetchRecurseSubmodules = on-demand
fetchRecurseSubmodules = false
[submodule "implicit"]
fetchRecurseSubmodules
[submodule "empty"]
fetchRecurseSubmodules =
[submodule "invalid"]
fetchRecurseSubmodules = unsupported
fetchRecurseSubmodules = true
[submodule "Child"]
fetchRecurseSubmodules = true
"#,
        )
        .expect("historical policy");
        assert_eq!(tree.module_recursion["child"], Recursion::OnDemand);
        assert_eq!(tree.module_recursion["implicit"], Recursion::On);
        assert_eq!(tree.module_recursion["empty"], Recursion::Off);
        assert_eq!(tree.module_recursion["invalid"], Recursion::Invalid);
        assert_eq!(tree.module_recursion["Child"], Recursion::On);
        assert_eq!(tree.module_names["nested/child"], "child");
    }

    #[test]
    fn pull_module_first_path_preserves_implicit_declaration_order() {
        let mut tree = Tree::parse(b"", 40).expect("tree");
        tree.bind_modules(b"[submodule \"child\"]\npath = \"nested/child\"\npath\n[submodule \"name-only\"]\nurl = ignored\n".as_slice())
            .expect("later implicit duplicate ignored");
        assert_eq!(tree.module_names["nested/child"], "child");
        assert!(tree.declared_names.contains("name-only"));
        let retained = tree.module_names.clone();
        assert!(
            tree.bind_modules(b"[submodule \"child\"]\npath\npath = later\n")
                .is_err()
        );
        assert_eq!(tree.module_names, retained);
        assert!(tree.declared_names.contains("name-only"));
    }

    #[test]
    fn pull_module_blob_maps_names_without_following_includes() {
        let mut tree = Tree::parse(b"", 40).expect("empty tree");
        tree.bind_modules(b"[include]\npath = /does/not/exist\n[submodule \"Child\"]\npath = missing/child\npath = ignored\n[submodule \"Child\"]\npath = also-ignored\n").expect("historical declarations");
        assert_eq!(
            tree.module_names.get("missing/child").map(String::as_str),
            Some("Child")
        );
        assert_eq!(tree.module_names.len(), 1);
        for bytes in [
            b"[submodule \"child\"]\npath\n".as_slice(),
            b"[submodule \"child\"]\npath = ../outside\n",
            b"[submodule \"child\"]\npath = /outside\n",
            b"[submodule \"child\"]\npath = -option\n",
            b"[submodule \"child\"]\npath = a/.git/b\n",
            b"[submodule \"one\"]\npath = child\n[submodule \"two\"]\npath = child\n",
            b"[submodule",
        ] {
            assert!(tree.bind_modules(bytes).is_err(), "{bytes:?}");
            assert_eq!(tree.module_names.len(), 1, "failed binding is atomic");
        }
        assert!(tree.bind_modules(&vec![b'x'; 1024 * 1024 + 1]).is_err());
    }

    #[test]
    fn pull_child_tree_retains_objects_without_opening_paths() {
        for width in [40, 64] {
            let oid = "a".repeat(width);
            let bytes = format!(
                "100644 blob {oid}\t.gitmodules\0160000 commit {oid}\tmissing/child\0100644 blob {oid}\tfile\0"
            );
            let tree = Tree::parse(bytes.as_bytes(), width).expect("tree");
            assert_eq!(tree.modules_blob.as_deref(), Some(oid.as_str()));
            assert_eq!(tree.gitlinks.get("missing/child"), Some(&oid));
            assert_eq!(tree.gitlinks.len(), 1);
            for name in [
                "../outside",
                "/absolute",
                "a//b",
                "a/./b",
                "a/.git/b",
                "a\\b",
                "",
            ] {
                assert!(
                    Tree::parse(format!("160000 commit {oid}\t{name}\0").as_bytes(), width)
                        .is_err()
                );
            }
            for mode in ["120000 blob", "160000 commit", "040000 tree"] {
                assert!(
                    Tree::parse(format!("{mode} {oid}\t.gitmodules\0").as_bytes(), width).is_err()
                );
            }
            assert!(Tree::parse(format!("160000 commit {oid}\tchild").as_bytes(), width).is_err());
            assert!(
                Tree::parse(
                    format!("160000 commit {oid}\tchild\0160000 commit {oid}\tchild\0").as_bytes(),
                    width
                )
                .is_err()
            );
        }
        assert!(Tree::parse(&vec![0; 1024 * 1024 + 1], 40).is_err());
    }
}
