//! Parent-tree evidence for recursive Pull discovery. These records identify
//! objects only; they do not authorize opening a child repository or fetching it.

use std::collections::BTreeMap;

use super::GitToolError;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Tree {
    pub(super) gitlinks: BTreeMap<String, String>,
    pub(super) modules_blob: Option<String>,
    pub(super) changed_gitlinks: BTreeMap<String, String>,
    pub(super) module_names: BTreeMap<String, String>,
}

/// A declared module's changed objects, in parent revision-walk order.
/// Unnamed gitlinks remain in Tree until separately admitted populated-path
/// discovery can resolve Git's legacy name fallback.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Changed {
    pub(super) commit: String,
    pub(super) path: String,
    pub(super) objects: std::collections::BTreeSet<String>,
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
                objects: std::collections::BTreeSet::new(),
            });
            changed.objects.insert(object.clone());
        }
    }
    Ok(result)
}

impl Tree {
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
        let invalid = || GitToolError::InvalidInput("Invalid Pull module declarations".into());
        if bytes.len() > 1024 * 1024 {
            return Err(invalid());
        }
        let names = std::panic::catch_unwind(|| {
            let file = gix::config::File::from_bytes_no_includes(
                bytes,
                gix::config::file::Metadata::default(),
                Default::default(),
            )
            .map_err(|_| invalid())?;
            let mut by_name = BTreeMap::new();
            for section in file.sections() {
                let header = section.header();
                if !header.name().eq_ignore_ascii_case(b"submodule") {
                    continue;
                }
                let Some(name) = header.subsection_name() else {
                    continue;
                };
                let name = std::str::from_utf8(name.as_ref()).map_err(|_| invalid())?;
                if name.is_empty() || name.contains(['\0', '\n', '\r']) {
                    return Err(invalid());
                }
                // Historical Git configuration keeps the first path for a
                // module name, including repeated sections.
                if by_name.contains_key(name) {
                    continue;
                }
                if section.body().value_implicit("path") == Some(None) {
                    return Err(invalid());
                }
                let Some(path) = section.body().values("path").into_iter().next() else {
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
            Ok(by_path)
        })
        .map_err(|_| invalid())??;
        self.module_names = names;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_named_changes_keep_first_revision_path_and_all_objects() {
        let mut trees = BTreeMap::new();
        for (commit, path, object) in [("z", "renamed", "b"), ("a", "old", "a")] {
            let mut tree = Tree::parse(b"", 40).expect("tree");
            tree.changed_gitlinks.insert(path.into(), object.repeat(40));
            tree.changed_gitlinks
                .insert("undeclared".into(), "c".repeat(40));
            tree.module_names.insert(path.into(), "Child".into());
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
