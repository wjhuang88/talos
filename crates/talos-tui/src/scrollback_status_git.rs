use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitStatusSummary {
    pub branch: Option<String>,
    pub dirty: bool,
}

static CACHE: Mutex<Option<(Instant, String, Option<GitStatusSummary>)>> = Mutex::new(None);

/// Returns the cached or computed git status summary for the given workspace path.
///
/// Refresh cadence is bounded to 500ms to avoid executing `gix` operations on every
/// ~50ms ratatui draw frame, which would degrade performance.
pub(crate) fn get_git_status(workspace: &str) -> Option<GitStatusSummary> {
    if workspace.is_empty() {
        return None;
    }

    let now = Instant::now();
    let ttl = Duration::from_millis(500);

    if let Ok(cache) = CACHE.lock()
        && let Some((ts, cached_workspace, summary)) = cache.as_ref()
        && cached_workspace == workspace
        && now.duration_since(*ts) < ttl
    {
        return summary.clone();
    }

    // gix is a native-code dependency (per ADR-010). Project hard constraint #9
    // (AGENTS.md) requires panic containment at the integration boundary so the
    // TUI draw loop can never crash on a repository read failure.
    let summary = catch_unwind(AssertUnwindSafe(|| {
        compute_git_status(Path::new(workspace))
    }))
    .unwrap_or(None);

    if let Ok(mut cache) = CACHE.lock() {
        *cache = Some((now, workspace.to_string(), summary.clone()));
    }

    summary
}

fn compute_git_status(workspace_root: &Path) -> Option<GitStatusSummary> {
    let repo = gix::discover(workspace_root).ok()?;

    let branch = repo
        .head_name()
        .ok()
        .flatten()
        .map(|name| name.shorten().to_string());

    let platform = repo
        .status(gix::progress::Discard)
        .ok()?
        .untracked_files(gix::status::UntrackedFiles::Files);

    let mut iter = platform
        .into_index_worktree_iter(Vec::<gix::bstr::BString>::new())
        .ok()?;

    let dirty = iter.next().is_some();

    Some(GitStatusSummary { branch, dirty })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_non_git_directory() {
        let td = tempdir().unwrap();
        let summary = get_git_status(td.path().to_str().unwrap());
        assert_eq!(summary, None);
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    /// Returns true when host `git` is available; tests that set up fixtures via
    /// host git skip themselves when this returns false (mirrors the convention
    /// used in `crates/talos-tools/src/git_tests.rs`).
    fn host_git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    fn run_cmd(dir: &Path, cmd: &str, args: &[&str]) {
        let status = Command::new(cmd)
            .args(args)
            .current_dir(dir)
            .status()
            .expect("Failed to execute command");
        assert!(status.success(), "Command failed");
    }

    #[test]
    fn test_clean_repo() {
        if !host_git_available() {
            eprintln!("skipping: host git not available");
            return;
        }
        let td = tempdir().unwrap();
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").unwrap();
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert_eq!(summary.dirty, false);
        // It should be either main or master
        assert!(
            summary.branch == Some("main".to_string())
                || summary.branch == Some("master".to_string())
        );
    }

    #[test]
    fn test_dirty_repo() {
        if !host_git_available() {
            eprintln!("skipping: host git not available");
            return;
        }
        let td = tempdir().unwrap();
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").unwrap();
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        fs::write(td.path().join("b.txt"), "dirty").unwrap();

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert_eq!(summary.dirty, true);
    }

    #[test]
    fn test_detached_head() {
        if !host_git_available() {
            eprintln!("skipping: host git not available");
            return;
        }
        let td = tempdir().unwrap();
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").unwrap();
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        fs::write(td.path().join("b.txt"), "hello2").unwrap();
        run_cmd(td.path(), "git", &["add", "b.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "second"]);

        run_cmd(td.path(), "git", &["checkout", "HEAD^"]);

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert_eq!(summary.dirty, false);
        assert_eq!(summary.branch, None);
    }
}
