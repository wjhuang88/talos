use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitStatusSummary {
    pub branch: Option<String>,
    pub dirty: bool,
}

/// Each TUI owns its displayed snapshot; only worker ownership is process-wide.
#[derive(Default)]
pub(crate) struct GitSnapshot {
    value: Option<(Instant, String, Option<GitStatusSummary>)>,
}

impl GitSnapshot {
    /// Read-only lookup; never scans or changes state from the render path.
    pub(crate) fn get(&self, workspace: &str, now: Instant) -> Option<&GitStatusSummary> {
        let (collected, cached_workspace, summary) = self.value.as_ref()?;
        if cached_workspace != workspace || now >= *collected + Duration::from_secs(5) {
            return None;
        }
        if summary.is_some() {
            crate::render_metrics::snapshot_age(now.saturating_duration_since(*collected));
        }
        summary.as_ref()
    }

    pub(crate) fn publish(
        &mut self,
        workspace: String,
        summary: Option<GitStatusSummary>,
        now: Instant,
    ) -> bool {
        let changed = self
            .value
            .as_ref()
            .is_none_or(|(_, previous_workspace, previous)| {
                previous_workspace != &workspace || previous != &summary
            });
        self.value = Some((now, workspace, summary));
        changed
    }

    /// Unknown state has no expiry deadline, avoiding repeated expiry wakes.
    pub(crate) fn stale_deadline(&self, workspace: &str) -> Option<Instant> {
        let (collected, cached_workspace, summary) = self.value.as_ref()?;
        (cached_workspace == workspace && summary.is_some())
            .then_some(*collected + Duration::from_secs(5))
    }

    pub(crate) fn expire(&mut self, workspace: &str, now: Instant) -> bool {
        let Some((collected, cached_workspace, summary)) = self.value.as_mut() else {
            return false;
        };
        if cached_workspace == workspace && now >= *collected + Duration::from_secs(5) {
            return summary.take().is_some();
        }
        false
    }
}

pub(crate) fn compute_git_status(workspace_root: &Path) -> Option<GitStatusSummary> {
    let repo = match gix::discover(workspace_root) {
        Ok(repo) => repo,
        Err(error) => {
            use gix::discover::upwards::Error;
            return match error.downcast_any_ref::<Error>() {
                Some(
                    Error::NoGitRepository { .. }
                    | Error::NoGitRepositoryWithinCeiling { .. }
                    | Error::NoGitRepositoryWithinFs { .. },
                ) => {
                    // A successful absence check has no Git details to display.
                    Some(GitStatusSummary {
                        branch: None,
                        dirty: false,
                    })
                }
                _ => None,
            };
        }
    };

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

    let dirty = match iter.next() {
        Some(Ok(_)) => true,
        Some(Err(_)) => return None,
        None => false,
    };

    Some(GitStatusSummary { branch, dirty })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn snapshot_expiration_changes_display_once_and_is_workspace_scoped() {
        let workspace = "snapshot-expiration-test".to_string();
        let summary = Some(GitStatusSummary {
            branch: Some("main".into()),
            dirty: false,
        });
        let mut snapshot = GitSnapshot::default();
        let now = Instant::now();
        assert!(snapshot.publish(workspace.clone(), summary.clone(), now));
        assert!(!snapshot.publish(workspace.clone(), summary, now));
        let deadline = snapshot
            .stale_deadline(&workspace)
            .expect("successful snapshot expires");
        assert!(!snapshot.expire("another-workspace", deadline));
        assert!(!snapshot.expire(&workspace, deadline - Duration::from_nanos(1)));
        assert!(snapshot.expire(&workspace, deadline));
        assert!(!snapshot.expire(&workspace, deadline));
        assert!(snapshot.stale_deadline(&workspace).is_none());
        assert!(snapshot.get(&workspace, deadline).is_none());
    }

    #[test]
    fn sessions_do_not_replace_each_others_snapshots_and_refresh_extends_expiry() {
        let now = Instant::now();
        let summary = Some(GitStatusSummary {
            branch: Some("first".into()),
            dirty: true,
        });
        let mut first = GitSnapshot::default();
        let mut second = GitSnapshot::default();
        first.publish("same-workspace".into(), summary.clone(), now);
        second.publish("same-workspace".into(), None, now);
        assert_eq!(first.get("same-workspace", now), summary.as_ref());
        assert!(second.get("same-workspace", now).is_none());
        assert!(first.get("other-workspace", now).is_none());
        let refresh = now + Duration::from_secs(2);
        assert!(!first.publish("same-workspace".into(), summary.clone(), refresh));
        assert_eq!(
            first.get("same-workspace", now + Duration::from_secs(5)),
            summary.as_ref()
        );
        assert!(
            first
                .get("same-workspace", now + Duration::from_secs(7))
                .is_none()
        );
        assert!(first.expire("same-workspace", now + Duration::from_secs(7)));
        assert!(first.stale_deadline("same-workspace").is_none());
    }

    #[test]
    fn test_non_git_directory() {
        let td = tempdir().expect("operation should succeed");
        let summary = compute_git_status(td.path());
        assert_eq!(
            summary,
            Some(GitStatusSummary {
                branch: None,
                dirty: false
            })
        );
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
        let td = tempdir().expect("operation should succeed");
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").expect("operation should succeed");
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert!(!summary.dirty);
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
        let td = tempdir().expect("operation should succeed");
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").expect("operation should succeed");
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        fs::write(td.path().join("b.txt"), "dirty").expect("operation should succeed");

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert!(summary.dirty);
    }

    #[test]
    fn test_detached_head() {
        if !host_git_available() {
            eprintln!("skipping: host git not available");
            return;
        }
        let td = tempdir().expect("operation should succeed");
        run_cmd(td.path(), "git", &["init"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);

        fs::write(td.path().join("a.txt"), "hello").expect("operation should succeed");
        run_cmd(td.path(), "git", &["add", "a.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "init"]);

        fs::write(td.path().join("b.txt"), "hello2").expect("operation should succeed");
        run_cmd(td.path(), "git", &["add", "b.txt"]);
        run_cmd(td.path(), "git", &["commit", "-m", "second"]);

        run_cmd(td.path(), "git", &["checkout", "HEAD^"]);

        let summary = compute_git_status(td.path()).expect("Should get status");
        assert!(!summary.dirty);
        assert_eq!(summary.branch, None);
    }
}
