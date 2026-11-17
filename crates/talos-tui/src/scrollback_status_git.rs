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
    lifetime: Option<Duration>,
}

impl GitSnapshot {
    pub(crate) fn set_lifetime(&mut self, lifetime: Duration) {
        self.lifetime = Some(lifetime);
    }

    /// Read-only lookup; never scans or changes state from the render path.
    pub(crate) fn get(&self, workspace: &str, now: Instant) -> Option<&GitStatusSummary> {
        let (collected, cached_workspace, summary) = self.value.as_ref()?;
        if cached_workspace != workspace
            || now >= *collected + self.lifetime.unwrap_or(Duration::from_secs(5))
        {
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
            .then_some(*collected + self.lifetime.unwrap_or(Duration::from_secs(5)))
    }

    pub(crate) fn expire(&mut self, workspace: &str, now: Instant) -> bool {
        let Some((collected, cached_workspace, summary)) = self.value.as_mut() else {
            return false;
        };
        if cached_workspace == workspace
            && now >= *collected + self.lifetime.unwrap_or(Duration::from_secs(5))
        {
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

    let dirty = worktree_dirty(&repo)?;

    Some(GitStatusSummary { branch, dirty })
}

// Use the synchronous plumbing visitor: the serial status iterator buffers all
// changes before yielding, while its parallel variant detaches on early drop.
fn worktree_dirty(repo: &gix::Repository) -> Option<bool> {
    use gix::status::plumbing::index_as_worktree as status;
    use std::sync::atomic::{AtomicBool, Ordering};
    let (pipeline, index) = repo.filter_pipeline(None).ok()?;
    let (filter, stack) = pipeline.into_parts();
    let pathspec = repo
        .pathspec(
            false,
            None::<&str>,
            true,
            &index,
            gix::worktree::stack::state::attributes::Source::WorktreeThenIdMapping,
        )
        .ok()?
        .into_parts()
        .0;
    let stop = AtomicBool::new(false);
    let mut visitor = DirtyVisitor(&stop);
    let result = gix::status::plumbing::index_as_worktree(
        &index,
        repo.workdir()?,
        &mut visitor,
        status::traits::FastEq,
        gix::status::index_worktree::BuiltinSubmoduleStatus::new(
            repo.clone().into_sync(),
            gix::status::Submodule::default(),
        )
        .ok()?,
        repo.objects.clone().into_arc().ok()?,
        &mut gix::progress::Discard,
        status::Context {
            pathspec,
            stack,
            filter,
            should_interrupt: &stop,
        },
        status::Options {
            fs: repo.filesystem_options().ok()?,
            thread_limit: Some(1),
            stat: repo.stat_options().ok()?,
            fscache: cfg!(windows),
        },
    );
    if stop.load(Ordering::Relaxed) {
        return Some(true);
    }
    result.ok()?;
    let mut untracked = UntrackedVisitor(false);
    repo.dirwalk(
        &index,
        None::<&str>,
        &stop,
        repo.dirwalk_options()
            .ok()?
            .emit_untracked(gix::dir::walk::EmissionMode::Matching),
        &mut untracked,
    )
    .ok()?;
    Some(untracked.0)
}

struct DirtyVisitor<'a>(&'a std::sync::atomic::AtomicBool);
impl<'index> gix::status::plumbing::index_as_worktree::VisitEntry<'index> for DirtyVisitor<'_> {
    type ContentChange = ();
    type SubmoduleStatus = gix::submodule::Status;
    fn visit_entry(
        &mut self,
        _: &'index [gix::index::Entry],
        _: &'index gix::index::Entry,
        _: usize,
        _: &'index gix::bstr::BStr,
        status: gix::status::plumbing::index_as_worktree::EntryStatus<(), Self::SubmoduleStatus>,
    ) {
        if !matches!(
            status,
            gix::status::plumbing::index_as_worktree::EntryStatus::NeedsUpdate(_)
        ) {
            self.0.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

struct UntrackedVisitor(bool);
impl gix::dir::walk::Delegate for UntrackedVisitor {
    fn emit(
        &mut self,
        entry: gix::dir::EntryRef<'_>,
        _: Option<gix::dir::entry::Status>,
    ) -> gix::dir::walk::Action {
        if entry.status == gix::dir::entry::Status::Untracked {
            self.0 = true;
            gix::dir::walk::Action::Break(())
        } else {
            gix::dir::walk::Action::Continue(())
        }
    }
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
    fn tracked_and_untracked_changes_restore_to_clean() {
        let td = tempdir().expect("fixture");
        run_cmd(td.path(), "git", &["init", "-b", "main"]);
        run_cmd(td.path(), "git", &["config", "user.name", "Test"]);
        run_cmd(td.path(), "git", &["config", "user.email", "test@test.com"]);
        fs::write(td.path().join(".gitignore"), "ignored/\n").expect("ignore");
        fs::write(td.path().join("tracked"), "original").expect("file");
        run_cmd(td.path(), "git", &["add", "."]);
        run_cmd(td.path(), "git", &["commit", "-m", "fixture"]);
        let dirty = || compute_git_status(td.path()).expect("known status").dirty;
        assert!(!dirty());
        fs::create_dir(td.path().join("ignored")).expect("directory");
        fs::write(td.path().join("ignored/generated"), "ignored").expect("file");
        assert!(!dirty(), "ignored files do not count");
        fs::write(td.path().join("tracked"), "changed").expect("modify");
        assert!(dirty());
        fs::write(td.path().join("tracked"), "original").expect("restore");
        let file = fs::File::options()
            .write(true)
            .open(td.path().join("tracked"))
            .expect("open");
        file.set_modified(
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(946684800),
        )
        .expect("timestamp");
        assert!(!dirty(), "stat changes alone are clean");
        fs::remove_file(td.path().join("tracked")).expect("delete");
        assert!(dirty());
        fs::write(td.path().join("tracked"), "original").expect("restore");
        fs::create_dir(td.path().join("nested")).expect("directory");
        fs::write(td.path().join("nested/new"), "untracked").expect("file");
        assert!(dirty());
        fs::remove_file(td.path().join("nested/new")).expect("remove");
        assert!(!dirty(), "empty directory is clean");
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
