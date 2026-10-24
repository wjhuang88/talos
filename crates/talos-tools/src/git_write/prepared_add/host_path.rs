//! Explicit Git directory arguments. Permission/evidence paths stay canonical.

use std::{borrow::Cow, path::Path};

use super::GitToolError;

/// Compare already admitted canonical executable paths; this performs no new
/// filesystem reads. Unknown Git installations retain the separate-execution
/// fallback until their runtime effects have been reviewed.
#[cfg(windows)]
pub(super) fn validate_receiver_shell(git: &Path, shell: &Path) -> Result<(), GitToolError> {
    use std::path::{Component, Prefix};

    let local = matches!(git.components().next(), Some(Component::Prefix(prefix))
        if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)));
    if local
        && git.is_absolute()
        && git.ends_with(r"ucrt64\libexec\git-core\git.exe")
        && let Some(root) = git.ancestors().nth(4)
        // Exact canonical comparison rejects an escaping shell junction;
        // never canonicalize an unapproved expected path.
        && shell == root.join(r"usr\bin\sh.exe")
    {
        return Ok(());
    }
    Err(GitToolError::InvalidInput(
        "Receiver transport requires a separately bound shell in the supported Git for Windows installation".into(),
    ))
}

/// Represent the bound Git executable for an MSYS shell without changing its
/// file identity. Only the fixed git.exe leaf under a local drive is supported.
#[cfg(windows)]
pub(super) fn receiver_program(path: &Path) -> Result<String, GitToolError> {
    use std::path::{Component, Prefix};

    let invalid = || {
        GitToolError::InvalidInput(
            "Receiver Git executable cannot preserve its admitted Windows identity".into(),
        )
    };
    if path.file_name() != Some(std::ffi::OsStr::new("git.exe"))
        || !matches!(path.components().next(), Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        || !path.is_absolute()
        || !path.is_file()
    {
        return Err(invalid());
    }
    let identity = super::snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
    let parent = path.parent().ok_or_else(invalid)?;
    let spelling = directory_argument(parent)?.join("git.exe");
    // A local-drive junction may resolve to a network share. Check the
    // converted spelling as well as the original, before MSYS interprets it.
    if !matches!(spelling.components().next(), Some(Component::Prefix(prefix))
        if matches!(prefix.kind(), Prefix::Disk(_)))
        || !spelling.is_absolute()
    {
        return Err(invalid());
    }
    identity.revalidate(&spelling).map_err(|_| invalid())?;
    let text = spelling.to_str().ok_or_else(invalid)?.replace('\\', "/");
    if text.contains(['\0', '\n', '\r']) {
        return Err(invalid());
    }
    Ok(text)
}

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
    fn receiver_shell_requires_exact_supported_installation() {
        let git = Path::new(r"\\?\C:\Program Files\Git\ucrt64\libexec\git-core\git.exe");
        let shell = Path::new(r"\\?\C:\Program Files\Git\usr\bin\sh.exe");
        validate_receiver_shell(git, shell).expect("same installation");
        // A rebound invocation spelling must not replace retained canonical
        // evidence when deriving the installation root.
        assert!(
            validate_receiver_shell(
                Path::new(r"C:\Program Files\Git\ucrt64\libexec\git-core\git.exe"),
                shell,
            )
            .is_err()
        );
        for other in [
            r"\\?\C:\Other Git\usr\bin\sh.exe",
            r"\\?\C:\Program Files\Git\usr\bin\bash.exe",
            r"\\?\D:\Program Files\Git\usr\bin\sh.exe",
        ] {
            assert!(validate_receiver_shell(git, Path::new(other)).is_err());
        }
        assert!(
            validate_receiver_shell(Path::new(r"\\?\C:\Program Files\Git\cmd\git.exe"), shell,)
                .is_err()
        );
        assert!(
            validate_receiver_shell(Path::new(r"ucrt64\libexec\git-core\git.exe"), shell,).is_err()
        );
    }

    #[test]
    fn receiver_program_preserves_identity_and_rejects_unsupported_forms() {
        let root = tempfile::tempdir().expect("fixture");
        let directory = root.path().join("Git space-'quote-目录");
        std::fs::create_dir(&directory).expect("directory");
        let program = directory.join("git.exe");
        std::fs::write(&program, b"fixture executable").expect("program");
        let canonical = program.canonicalize().expect("canonical");
        let represented = receiver_program(&canonical).expect("shell path");
        assert!(!represented.contains('\\'));
        assert!(!represented.starts_with("//?/"));
        assert_eq!(
            Path::new(&represented).canonicalize().expect("same file"),
            canonical
        );
        assert!(receiver_program(Path::new(r"\\server\share\git.exe")).is_err());
        assert!(receiver_program(Path::new(r"C:git.exe")).is_err());
        let other = directory.join("other.exe");
        std::fs::write(&other, b"fixture executable").expect("other");
        assert!(receiver_program(&other).is_err());
        let bad_directory = root.path().canonicalize().expect("root").join("trailing.");
        std::fs::create_dir(&bad_directory).expect("verbatim directory");
        let bad_program = bad_directory.join("git.exe");
        std::fs::write(&bad_program, b"fixture executable").expect("verbatim program");
        assert!(receiver_program(&bad_program).is_err());
    }

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
