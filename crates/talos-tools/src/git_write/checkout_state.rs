//! Target-tree and parent-path evidence for prepared checkout.

use super::{GitToolError, discover_repo};
use std::collections::BTreeMap;

/// Identity of a local branch resolved without invoking host Git. Remote
/// guessing and revision expressions are resolved separately after discovery.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct LocalTarget {
    pub reference: String,
    pub commit: String,
}

impl LocalTarget {
    pub(super) fn capture(
        workdir: &std::path::Path,
        branch: &str,
    ) -> Result<Option<Self>, GitToolError> {
        std::panic::catch_unwind(|| {
            super::validate_git_positional(branch)?;
            let reference = format!("refs/heads/{branch}");
            let repo = discover_repo(workdir)?;
            let Some(mut found) = repo
                .try_find_reference(reference.as_str())
                .map_err(|error| GitToolError::Git(error.to_string()))?
            else {
                return Ok(None);
            };
            let commit = found
                .peel_to_id()
                .map_err(|error| GitToolError::Git(error.to_string()))?;
            let header = repo
                .find_header(commit.detach())
                .map_err(|error| GitToolError::Git(error.to_string()))?;
            if header.kind() != gix::objs::Kind::Commit {
                return Err(GitToolError::InvalidInput(
                    "Checkout branch does not reference a commit".into(),
                ));
            }
            Ok(Some(Self {
                reference,
                commit: commit.to_string(),
            }))
        })
        .unwrap_or_else(|_| {
            Err(GitToolError::Git(
                "Checkout branch inspection failed".into(),
            ))
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct TargetTree {
    pub commit: String,
    pub tree: String,
    pub entries: BTreeMap<String, (u16, String)>,
    pub attributes: BTreeMap<String, Vec<u8>>,
}

impl TargetTree {
    // Accept an already resolved object ID, never a revision expression. Branch
    // and remote-guessing identity remain separate evidence in the caller.
    pub(super) fn capture(workdir: &std::path::Path, commit: &str) -> Result<Self, GitToolError> {
        std::panic::catch_unwind(|| {
            let id = gix::ObjectId::from_hex(commit.as_bytes()).map_err(|_| {
                GitToolError::InvalidInput("Invalid checkout target object ID".into())
            })?;
            let repo = discover_repo(workdir)?;
            let header = repo
                .find_header(id)
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            if header.kind() != gix::objs::Kind::Commit || header.size() > 4 * 1024 * 1024 {
                return Err(GitToolError::InvalidInput(
                    "Checkout target must be a bounded commit object".into(),
                ));
            }
            let commit = repo
                .find_object(id)
                .map_err(|e| GitToolError::Git(e.to_string()))?
                .peel_to_commit()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            // Obtain the ID without loading the root tree: its header must
            // pass the same byte budget as every descendant before allocation.
            let tree_id = commit
                .tree_id()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            let mut entries = BTreeMap::new();
            let mut attributes = BTreeMap::new();
            let mut attribute_bytes = 0usize;
            let mut pending = vec![(String::new(), tree_id.detach())];
            let mut visited = 0usize;
            let mut tree_bytes = 0u64;
            while let Some((prefix, tree_id)) = pending.pop() {
                let header = repo
                    .find_header(tree_id)
                    .map_err(|e| GitToolError::Git(e.to_string()))?;
                tree_bytes = tree_bytes
                    .checked_add(header.size())
                    .filter(|size| *size <= 4 * 1024 * 1024)
                    .ok_or_else(|| {
                        GitToolError::InvalidInput("Checkout trees exceed byte limit".into())
                    })?;
                let current = repo
                    .find_object(tree_id)
                    .map_err(|e| GitToolError::Git(e.to_string()))?
                    .try_into_tree()
                    .map_err(|e| GitToolError::Git(e.to_string()))?;
                for entry in current.iter() {
                    let entry = entry.map_err(|e| GitToolError::Git(e.to_string()))?;
                    visited += 1;
                    if visited > 4096 {
                        return Err(GitToolError::InvalidInput(
                            "Checkout target exceeds entry limit".into(),
                        ));
                    }
                    let name = std::str::from_utf8(entry.filename()).map_err(|_| {
                        GitToolError::InvalidInput("Checkout target path is not UTF-8".into())
                    })?;
                    if name.is_empty()
                        || name.contains('\0')
                        || name == "."
                        || name == ".."
                        || name.eq_ignore_ascii_case(".git")
                        || name.contains(['/', '\\'])
                    {
                        return Err(GitToolError::InvalidInput(
                            "Unsafe checkout target path".into(),
                        ));
                    }
                    let path = if prefix.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{prefix}/{name}")
                    };
                    if path.len() > 4096 {
                        return Err(GitToolError::InvalidInput(
                            "Checkout target path exceeds byte limit".into(),
                        ));
                    }
                    if entry.mode().is_tree() {
                        pending.push((path, entry.object_id()));
                        continue;
                    }
                    if name == ".gitattributes" && entry.mode().is_blob() {
                        let header = repo
                            .find_header(entry.object_id())
                            .map_err(|e| GitToolError::Git(e.to_string()))?;
                        let length = usize::try_from(header.size()).map_err(|_| {
                            GitToolError::InvalidInput(
                                "Checkout attributes exceed byte limit".into(),
                            )
                        })?;
                        attribute_bytes = attribute_bytes
                            .checked_add(length)
                            .filter(|bytes| *bytes <= 4 * 1024 * 1024)
                            .ok_or_else(|| {
                                GitToolError::InvalidInput(
                                    "Checkout attributes exceed byte limit".into(),
                                )
                            })?;
                        let object = entry
                            .object()
                            .map_err(|e| GitToolError::Git(e.to_string()))?;
                        attributes.insert(path.clone(), object.data.clone());
                    }
                    entries.insert(path, (entry.mode().value(), entry.object_id().to_string()));
                }
            }
            Ok(Self {
                commit: commit.id.to_string(),
                tree: tree_id.to_string(),
                entries,
                attributes,
            })
        })
        .unwrap_or_else(|_| {
            Err(GitToolError::Git(
                "Checkout target inspection failed".into(),
            ))
        })
    }
}

/// Ancestors are evidence even when a target file does not yet exist. Never
/// follow an ancestor link during capture; the final entry remains a Git entry.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct TargetParents(Vec<(std::path::PathBuf, super::snapshot::PathIdentity)>);

impl TargetParents {
    pub(super) fn capture(
        workdir: &std::path::Path,
        paths: &[String],
    ) -> Result<Self, GitToolError> {
        if paths.len() > 4096 || paths.iter().any(|path| path.len() > 4096) {
            return Err(GitToolError::InvalidInput(
                "Checkout parent evidence exceeds limit".into(),
            ));
        }
        let root = workdir
            .canonicalize()
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;
        let mut parents = BTreeMap::new();
        for path in paths {
            let mut current = workdir.to_path_buf();
            let components: Vec<_> = std::path::Path::new(path).components().collect();
            if components.is_empty()
                || components
                    .iter()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
            {
                return Err(GitToolError::InvalidInput(
                    "Invalid checkout source path".into(),
                ));
            }
            for component in components.iter().take(components.len() - 1) {
                current.push(component);
                match std::fs::symlink_metadata(&current) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err(GitToolError::InvalidInput("Checkout source ancestor is a symlink; use a separately authorized invocation".into()));
                    }
                    Ok(metadata) if metadata.is_dir() => {
                        let resolved = current
                            .canonicalize()
                            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;
                        if !resolved.starts_with(&root) {
                            return Err(GitToolError::InvalidInput(
                                "Checkout ancestor escaped the worktree during inspection".into(),
                            ));
                        }
                        if parents.len() >= 4096 && !parents.contains_key(&current) {
                            return Err(GitToolError::InvalidInput(
                                "Checkout parent evidence exceeds limit".into(),
                            ));
                        }
                        parents.insert(
                            current.clone(),
                            super::snapshot::PathIdentity::capture(&current)
                                .map_err(|e| GitToolError::InvalidInput(e.to_string()))?,
                        );
                    }
                    // A file blocking a target directory is captured separately
                    // as a conflict source. No descendants can be read through it.
                    Ok(_) => break,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                    Err(error) => return Err(GitToolError::InvalidInput(error.to_string())),
                }
            }
        }
        parents.insert(
            workdir.to_path_buf(),
            super::snapshot::PathIdentity::capture(workdir)
                .map_err(|e| GitToolError::InvalidInput(e.to_string()))?,
        );
        Ok(Self(parents.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_target_binds_reference_and_detects_movement() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        git(&["commit", "--allow-empty", "-m", "first"]);
        let first = LocalTarget::capture(root.path(), "main")
            .expect("resolve")
            .expect("branch");
        assert_eq!(first.reference, "refs/heads/main");
        std::fs::write(root.path().join("missing"), "not a branch").expect("file");
        assert!(
            LocalTarget::capture(root.path(), "missing")
                .expect("absent")
                .is_none()
        );
        git(&["commit", "--allow-empty", "-m", "second"]);
        assert_ne!(
            Some(first),
            LocalTarget::capture(root.path(), "main").expect("moved")
        );
        assert!(LocalTarget::capture(root.path(), "--detach").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn missing_target_parents_bind_identity_and_refuse_ancestor_links() {
        let root = tempfile::tempdir().expect("workspace");
        let outside = tempfile::tempdir().expect("outside");
        let paths = vec!["new/deeper/file".to_owned()];
        let missing = TargetParents::capture(root.path(), &paths).expect("missing parents");
        std::fs::create_dir(root.path().join("new")).expect("parent");
        let present = TargetParents::capture(root.path(), &paths).expect("present parent");
        assert_ne!(missing, present);
        std::fs::rename(root.path().join("new"), root.path().join("old")).expect("move");
        std::fs::create_dir(root.path().join("new")).expect("replacement");
        assert_ne!(
            present,
            TargetParents::capture(root.path(), &paths).expect("new identity")
        );
        std::fs::remove_dir(root.path().join("new")).expect("remove");
        std::os::unix::fs::symlink(outside.path(), root.path().join("new")).expect("link");
        assert!(
            TargetParents::capture(root.path(), &paths)
                .expect_err("escape")
                .to_string()
                .contains("ancestor")
        );
        assert!(TargetParents::capture(root.path(), &["../outside".into()]).is_err());
    }

    use super::*;
    #[test]
    fn target_tree_limit_counts_directories_and_rejects_oversized_attributes() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str], input: &[u8]| {
            use std::io::Write;
            let mut child = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("git");
            child
                .stdin
                .take()
                .expect("stdin")
                .write_all(input)
                .expect("input");
            let output = child.wait_with_output().expect("git result");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["init"], b"");
        git(&["config", "user.name", "Fixture"], b"");
        git(&["config", "user.email", "fixture@example.invalid"], b"");
        let empty = git(&["mktree"], b"");
        let records = (0..4097)
            .map(|index| format!("040000 tree {empty}\td{index:04}\n"))
            .collect::<String>();
        let wide = git(&["mktree"], records.as_bytes());
        let commit = git(&["commit-tree", &wide, "-m", "wide"], b"");
        assert!(
            TargetTree::capture(root.path(), &commit)
                .expect_err("directory budget")
                .to_string()
                .contains("entry limit")
        );
        let blob = git(
            &["hash-object", "-w", "--stdin"],
            &vec![b'x'; 4 * 1024 * 1024 + 1],
        );
        let tree = git(
            &["mktree"],
            format!("100644 blob {blob}\t.gitattributes\n").as_bytes(),
        );
        let commit = git(&["commit-tree", &tree, "-m", "large attributes"], b"");
        assert!(
            TargetTree::capture(root.path(), &commit)
                .expect_err("attribute budget")
                .to_string()
                .contains("byte limit")
        );
    }

    #[test]
    fn target_attributes_come_from_target_tree_not_current_worktree() {
        let root = tempfile::tempdir().expect("fixture");
        let git = |args: &[&str]| {
            let output = std::process::Command::new("git")
                .args(args)
                .current_dir(root.path())
                .output()
                .expect("git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout)
                .expect("utf8")
                .trim()
                .to_owned()
        };
        git(&["init"]);
        git(&["config", "user.name", "Fixture"]);
        git(&["config", "user.email", "fixture@example.invalid"]);
        git(&["config", "commit.gpgSign", "false"]);
        std::fs::write(root.path().join("file"), "target").expect("file");
        std::fs::write(
            root.path().join(".gitattributes"),
            "file filter=target-helper\n",
        )
        .expect("attributes");
        git(&["add", "."]);
        git(&["commit", "-m", "target"]);
        let id = git(&["rev-parse", "HEAD"]);
        std::fs::write(root.path().join(".gitattributes"), "file -filter\n")
            .expect("different current attributes");
        let target = TargetTree::capture(root.path(), &id).expect("target");
        assert_eq!(
            target.attributes[".gitattributes"],
            b"file filter=target-helper\n"
        );
        assert_eq!(target.entries.len(), 2);
        assert!(
            TargetTree::capture(root.path(), "HEAD").is_err(),
            "only resolved IDs accepted"
        );
        assert_eq!(
            target,
            TargetTree::capture(root.path(), &id).expect("immutable target")
        );
    }
}
