//! Explicit Git directory arguments. Permission/evidence paths stay canonical.

use std::{borrow::Cow, path::Path};

use super::GitToolError;

/// Git for Windows does not reliably accept Rust's verbatim path spelling as
/// --git-dir/--work-tree. Use a conventional spelling only after proving that
/// it resolves to the same directory. This grants no new path authority.
pub(super) fn directory_argument(path: &Path) -> Result<Cow<'_, Path>, GitToolError> {
    #[cfg(not(windows))]
    {
        Ok(Cow::Borrowed(path))
    }
    #[cfg(windows)]
    {
        use std::path::{Component, PathBuf, Prefix};

        let invalid = || {
            GitToolError::InvalidInput(
                "Git directory argument cannot preserve admitted path identity".into(),
            )
        };
        let identity = super::snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
        let canonical = path.canonicalize().map_err(|_| invalid())?;
        let mut components = canonical.components();
        let spelling = match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::VerbatimDisk(drive) => PathBuf::from(format!("{}:\\", char::from(drive))),
                Prefix::VerbatimUNC(server, share) => {
                    let mut spelling = std::ffi::OsString::from("\\\\");
                    spelling.push(server);
                    spelling.push("\\");
                    spelling.push(share);
                    spelling.push("\\");
                    PathBuf::from(spelling)
                }
                _ => return Err(invalid()),
            },
            _ => return Err(invalid()),
        };
        let mut spelling = spelling;
        for component in components {
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    // Git and Rust can apply different Win32 normalization.
                    // Do not convert verbatim-only names or DOS device aliases.
                    let name_text = name.to_str().ok_or_else(invalid)?;
                    let stem = name_text.split('.').next().unwrap_or_default();
                    let reserved = ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"]
                        .iter()
                        .any(|device| stem.eq_ignore_ascii_case(device))
                        || ["COM", "LPT"].iter().any(|prefix| {
                            stem.get(..3)
                                .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                                && matches!(
                                    stem.get(3..),
                                    Some(
                                        "1" | "2"
                                            | "3"
                                            | "4"
                                            | "5"
                                            | "6"
                                            | "7"
                                            | "8"
                                            | "9"
                                            | "¹"
                                            | "²"
                                            | "³"
                                    )
                                )
                        });
                    if name_text.ends_with(['.', ' ']) || name_text.contains(':') || reserved {
                        return Err(invalid());
                    }
                    spelling.push(name);
                }
                _ => return Err(invalid()),
            }
        }
        // Removing a verbatim prefix can reinterpret names (e.g. trailing
        // dots/spaces or DOS device names). Never rely on lexical equivalence.
        identity.revalidate(&spelling).map_err(|_| invalid())?;
        if !spelling.is_dir() {
            return Err(invalid());
        }
        Ok(Cow::Owned(spelling))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn explicit_git_directory_argument_preserves_canonical_identity() {
        let root = tempfile::tempdir().expect("fixture");
        let directory = root.path().join("directory with spaces-目录");
        std::fs::create_dir(&directory).expect("directory");
        let canonical = directory.canonicalize().expect("canonical");
        let argument = directory_argument(&canonical).expect("compatible spelling");
        assert!(
            !argument
                .as_os_str()
                .to_string_lossy()
                .starts_with("\\\\?\\")
        );
        assert_eq!(argument.canonicalize().expect("same target"), canonical);
        let identity = super::super::snapshot::PathIdentity::capture(&canonical).expect("identity");
        identity
            .revalidate(&argument)
            .expect("same directory identity");
        assert!(directory_argument(&directory.join("missing")).is_err());
        let file = directory.join("file");
        std::fs::write(&file, "data").expect("file");
        assert!(directory_argument(&file).is_err());
        let verbatim_root = root.path().canonicalize().expect("canonical fixture");
        for name in ["trailing.", "trailing "] {
            let verbatim_directory = verbatim_root.join(name);
            std::fs::create_dir(&verbatim_directory).expect("verbatim directory");
            assert!(directory_argument(&verbatim_directory).is_err());
        }
    }

    #[test]
    fn explicit_git_directory_arguments_run_native_git() {
        let root = tempfile::tempdir().expect("fixture");
        let worktree = root.path().join("Git directory with spaces-目录");
        std::fs::create_dir(&worktree).expect("worktree");
        assert!(
            std::process::Command::new("git")
                .current_dir(&worktree)
                .arg("init")
                .output()
                .expect("init")
                .status
                .success()
        );
        let canonical = worktree.canonicalize().expect("canonical worktree");
        let gitdir = worktree
            .join(".git")
            .canonicalize()
            .expect("canonical gitdir");
        let output = std::process::Command::new("git")
            .current_dir(&canonical)
            .arg("--git-dir")
            .arg(
                directory_argument(&gitdir)
                    .expect("gitdir argument")
                    .as_os_str(),
            )
            .arg("--work-tree")
            .arg(
                directory_argument(&canonical)
                    .expect("worktree argument")
                    .as_os_str(),
            )
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .expect("native explicit-root invocation");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"true\n");
    }
}
