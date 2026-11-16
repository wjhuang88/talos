//! Bounded native change notifications. All registration and teardown stay off the UI thread.
use notify::{EventKind, RecursiveMode, Watcher};
use std::{
    collections::HashSet,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

static BUSY: AtomicBool = AtomicBool::new(false);
const CHANGE: u8 = 1;
const REBUILD: u8 = 2;
const FAILED: u8 = 4;
const MAX_WATCHES: usize = 8192;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct State {
    pub revision: u64,
    pub healthy: bool,
}

pub(crate) struct Watch {
    updates: tokio::sync::watch::Receiver<State>,
    stop: Arc<AtomicBool>,
    wake: mpsc::SyncSender<()>,
}

struct Slot;
impl Drop for Slot {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}

impl Watch {
    pub(crate) fn start(workspace: PathBuf) -> std::io::Result<Option<Self>> {
        // notify 8.2 kqueue internally re-registers recursively after directory
        // events, bypassing our ignore rules and watch budget. Keep polling until
        // a bounded macOS backend is available; never instantiate that backend.
        if cfg!(target_os = "macos") {
            return Ok(None);
        }
        if BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(None);
        }
        let slot = Slot;
        let (updates_tx, updates) = tokio::sync::watch::channel(State::default());
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let (wake, receiver) = mpsc::sync_channel(1);
        let callback_wake = wake.clone();
        std::thread::Builder::new()
            .name("talos-git-watch".into())
            .spawn(move || {
                let _slot = slot;
                let result = catch_unwind(AssertUnwindSafe(|| {
                    run(
                        &workspace,
                        &worker_stop,
                        callback_wake,
                        receiver,
                        &updates_tx,
                    )
                }));
                if !matches!(result, Ok(Ok(()))) {
                    updates_tx.send_modify(|state| {
                        state.healthy = false;
                        state.revision += 1;
                    });
                }
            })?;
        Ok(Some(Self {
            updates,
            stop,
            wake,
        }))
    }

    pub(crate) async fn changed(&mut self) -> Option<State> {
        self.updates.changed().await.ok()?;
        Some(*self.updates.borrow_and_update())
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = self.wake.try_send(());
    }
}

fn run(
    workspace: &Path,
    stop: &AtomicBool,
    wake: mpsc::SyncSender<()>,
    receiver: mpsc::Receiver<()>,
    updates: &tokio::sync::watch::Sender<State>,
) -> Result<(), ()> {
    let flags = Arc::new(AtomicU8::new(0));
    let callback_flags = flags.clone();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        let bits = match event {
            Err(_) => FAILED,
            Ok(event) if event.need_rescan() => FAILED,
            Ok(event)
                if event.paths.iter().any(|path| {
                    matches!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some(".gitignore" | "exclude" | "index" | "config")
                    )
                }) && !matches!(event.kind, EventKind::Access(_)) =>
            {
                CHANGE | REBUILD
            }
            Ok(event) => match event.kind {
                EventKind::Access(_) => 0,
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(notify::event::ModifyKind::Name(_)) => CHANGE | REBUILD,
                _ => CHANGE,
            },
        };
        if bits != 0 {
            callback_flags.fetch_or(bits, Ordering::Release);
            let _ = wake.try_send(());
        }
    })
    .map_err(|_| ())?;
    let mut registered = HashSet::new();
    register(&mut watcher, workspace, stop, &mut registered)?;
    if stop.load(Ordering::Acquire) {
        return Ok(());
    }
    updates.send_modify(|state| {
        state.healthy = true;
        state.revision += 1;
    });
    let mut rebuild_at: Option<Instant> = None;
    loop {
        let received = match rebuild_at {
            Some(deadline) => {
                receiver.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            }
            None => receiver
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        if matches!(received, Err(mpsc::RecvTimeoutError::Disconnected)) {
            break;
        }
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        let bits = flags.swap(0, Ordering::AcqRel);
        if bits & FAILED != 0 {
            return Err(());
        }
        if bits & CHANGE != 0 {
            // Publish invalidation before refreshing directory coverage. Callbacks
            // only set bits: event storms cannot grow a queue of paths.
            updates.send_modify(|state| {
                state.revision += 1;
            });
        }
        if bits & REBUILD != 0 {
            rebuild_at.get_or_insert(Instant::now() + Duration::from_millis(500));
        }
        if rebuild_at.is_some_and(|deadline| Instant::now() >= deadline) {
            rebuild_at = None;
            register(&mut watcher, workspace, stop, &mut registered)?;
            // New files can precede registration of a new directory.
            updates.send_modify(|state| {
                state.revision += 1;
            });
        }
    }
    Ok(())
}

fn register(
    watcher: &mut notify::RecommendedWatcher,
    workspace: &Path,
    stop: &AtomicBool,
    registered: &mut HashSet<PathBuf>,
) -> Result<(), ()> {
    let start = Instant::now();
    let mut desired = HashSet::new();
    // Non-recursive watches exclude ignored build trees; watch hidden source
    // directories too. Symlinks are not followed.
    for entry in ignore::WalkBuilder::new(workspace)
        .hidden(false)
        .ignore(false)
        .follow_links(false)
        .build()
    {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(2) {
            return Err(());
        }
        let entry = entry.map_err(|_| ())?;
        if entry
            .file_type()
            .is_some_and(|kind| kind.is_dir() || (cfg!(target_os = "macos") && kind.is_file()))
        {
            desired.insert(entry.into_path());
            if desired.len() > MAX_WATCHES {
                return Err(());
            }
        }
    }
    // A linked worktree's index/HEAD and its common refs live outside its root.
    if let Ok(repo) = gix::discover(workspace) {
        if let Some(root) = repo.workdir() {
            let index = repo.index_or_empty().map_err(|_| ())?;
            for entry in index.entries() {
                if stop.load(Ordering::Acquire) {
                    return Ok(());
                }
                if start.elapsed() > Duration::from_secs(2) {
                    return Err(());
                }
                let relative = gix::path::from_bstr(entry.path(&index)).map_err(|_| ())?;
                let mut directory = root.join(relative.as_ref());
                #[cfg(target_os = "macos")]
                if directory.is_file() {
                    desired.insert(directory.clone());
                }
                directory.pop();
                while directory.starts_with(root) {
                    if directory.is_dir() {
                        desired.insert(directory.clone());
                    }
                    if desired.len() > MAX_WATCHES {
                        return Err(());
                    }
                    if directory == root || !directory.pop() {
                        break;
                    }
                }
            }
        }
        for root in [repo.git_dir(), repo.common_dir()] {
            desired.insert(root.to_owned());
            #[cfg(target_os = "macos")]
            for name in ["HEAD", "index", "packed-refs", "config"] {
                let path = root.join(name);
                if path.is_file() {
                    desired.insert(path);
                }
            }
            let info = root.join("info");
            if info.is_dir() {
                #[cfg(target_os = "macos")]
                if info.join("exclude").is_file() {
                    desired.insert(info.join("exclude"));
                }
                desired.insert(info);
            }
            let refs = root.join("refs");
            if refs.is_dir() {
                for entry in ignore::WalkBuilder::new(refs)
                    .hidden(false)
                    .standard_filters(false)
                    .build()
                {
                    if stop.load(Ordering::Acquire) {
                        return Ok(());
                    }
                    if start.elapsed() > Duration::from_secs(2) {
                        return Err(());
                    }
                    let entry = entry.map_err(|_| ())?;
                    if entry.file_type().is_some_and(|kind| {
                        kind.is_dir() || (cfg!(target_os = "macos") && kind.is_file())
                    }) {
                        desired.insert(entry.into_path());
                        if desired.len() > MAX_WATCHES {
                            return Err(());
                        }
                    }
                }
            }
        }
    }
    // A path may name a new inode after delete/recreate. Re-register even
    // unchanged path strings; scans after rebuild cover the notification gap.
    for path in registered.drain() {
        let _ = watcher.unwatch(&path);
    }
    for path in &desired {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(2) {
            return Err(());
        }
        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|_| ())?;
    }
    *registered = desired;
    Ok(())
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    async fn start(path: PathBuf) -> Watch {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(watch) = Watch::start(path.clone()).expect("spawn") {
                    return watch;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("previous watcher exits")
    }

    async fn next(watch: &mut Watch) -> State {
        tokio::time::timeout(Duration::from_secs(5), watch.changed())
            .await
            .expect("notification")
            .expect("watch alive")
    }

    #[tokio::test]
    async fn native_watch_observes_changes_and_quiet_ignored_tree() {
        let dir = tempfile::tempdir().expect("fixture");
        gix::init(dir.path()).expect("repo");
        std::fs::write(dir.path().join(".gitignore"), "build/\n").expect("ignore");
        std::fs::create_dir(dir.path().join("build")).expect("build directory");
        let mut watch = start(dir.path().to_owned()).await;
        assert!(next(&mut watch).await.healthy);
        std::fs::write(dir.path().join("build/ignored"), "ignored").expect("file");
        assert!(
            tokio::time::timeout(Duration::from_millis(300), watch.changed())
                .await
                .is_err()
        );
        std::fs::write(dir.path().join("tracked"), "changed").expect("file");
        assert!(next(&mut watch).await.healthy);
        drop(watch);
    }

    #[tokio::test]
    async fn tracked_file_in_ignored_directory_is_watched() {
        let dir = tempfile::tempdir().expect("fixture");
        gix::init(dir.path()).expect("repo");
        std::fs::create_dir(dir.path().join("build")).expect("directory");
        let file = dir.path().join("build/tracked");
        std::fs::write(&file, "original").expect("file");
        std::fs::write(dir.path().join(".gitignore"), "build/\n").expect("ignore");
        assert!(
            std::process::Command::new("git")
                .args(["add", "-f", "build/tracked"])
                .current_dir(dir.path())
                .status()
                .expect("fixture git")
                .success()
        );
        let mut watch = start(dir.path().to_owned()).await;
        assert!(next(&mut watch).await.healthy);
        std::fs::write(file, "modified").expect("modify");
        assert!(next(&mut watch).await.healthy);
    }

    #[tokio::test]
    async fn replaced_directory_keeps_observing_file_writes() {
        let dir = tempfile::tempdir().expect("fixture");
        gix::init(dir.path()).expect("repo");
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).expect("directory");
        std::fs::write(nested.join("file"), "first").expect("file");
        let mut watch = start(dir.path().to_owned()).await;
        assert!(next(&mut watch).await.healthy);
        std::fs::remove_dir_all(&nested).expect("remove fixture directory");
        std::fs::create_dir(&nested).expect("replacement");
        std::fs::write(nested.join("file"), "replacement").expect("file");
        tokio::time::sleep(Duration::from_millis(900)).await;
        let drain_deadline = Instant::now() + Duration::from_secs(3);
        loop {
            assert!(
                Instant::now() < drain_deadline,
                "notifications did not settle"
            );
            match tokio::time::timeout(Duration::from_millis(100), watch.changed()).await {
                Err(_) => break,
                Ok(Some(state)) => assert!(state.healthy, "watch failed"),
                Ok(None) => panic!("watch stopped"),
            }
        }
        std::fs::write(nested.join("file"), "later write").expect("write");
        assert!(next(&mut watch).await.healthy);
    }

    #[tokio::test]
    async fn removing_ignore_rule_registers_existing_directory() {
        let dir = tempfile::tempdir().expect("fixture");
        gix::init(dir.path()).expect("repo");
        let nested = dir.path().join("build");
        std::fs::create_dir(&nested).expect("directory");
        let file = nested.join("existing");
        std::fs::write(&file, "initial").expect("file");
        let rules = dir.path().join(".gitignore");
        std::fs::write(&rules, "build/\n").expect("ignore");
        let mut watch = start(dir.path().to_owned()).await;
        assert!(next(&mut watch).await.healthy);
        std::fs::write(rules, "").expect("remove ignore rule");
        tokio::time::sleep(Duration::from_millis(900)).await;
        let drain_deadline = Instant::now() + Duration::from_secs(3);
        loop {
            assert!(
                Instant::now() < drain_deadline,
                "notifications did not settle"
            );
            match tokio::time::timeout(Duration::from_millis(100), watch.changed()).await {
                Err(_) => break,
                Ok(Some(state)) => assert!(state.healthy, "watch failed"),
                Ok(None) => panic!("watch stopped"),
            }
        }
        std::fs::write(file, "changed").expect("modify newly watched file");
        assert!(next(&mut watch).await.healthy);
    }

    #[tokio::test]
    async fn missing_watch_root_reports_failure() {
        let dir = tempfile::tempdir().expect("fixture");
        let mut watch = start(dir.path().join("missing")).await;
        assert!(!next(&mut watch).await.healthy);
    }
}
