//! One process-wide scan slot: timed-out work retains ownership until it returns.
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::oneshot;

use crate::scrollback_status_git::GitStatusSummary;

static SCAN_BUSY: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub(crate) static TEST_SCAN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[cfg(test)]
pub(crate) async fn wait_for_test_slot() {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while SCAN_BUSY.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("worker must release its slot");
}

struct ScanSlot;
impl Drop for ScanSlot {
    fn drop(&mut self) {
        SCAN_BUSY.store(false, Ordering::Release);
    }
}

pub(crate) struct ScanResult {
    pub workspace: String,
    pub generation: u64,
    pub summary: Option<GitStatusSummary>,
    pub collected_at: std::time::Instant,
}

pub(crate) struct Scan {
    receiver: oneshot::Receiver<ScanResult>,
    cancelled: Arc<AtomicBool>,
}

impl Scan {
    /// A busy slot is not an error and must not create another worker.
    pub(crate) fn start(workspace: String, generation: u64) -> std::io::Result<Option<Self>> {
        Self::start_with(workspace, generation, |workspace| {
            crate::scrollback_status_git::compute_git_status(std::path::Path::new(workspace))
        })
    }

    pub(crate) fn start_with(
        workspace: String,
        generation: u64,
        scan: impl FnOnce(&str) -> Option<GitStatusSummary> + Send + 'static,
    ) -> std::io::Result<Option<Self>> {
        if SCAN_BUSY
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(None);
        }
        let slot = ScanSlot;
        let (sender, receiver) = oneshot::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        // Builder failure drops the closure (and slot). A detached worker never
        // blocks UI shutdown; it still owns the global slot across TUI sessions.
        std::thread::Builder::new()
            .name("talos-git-status".into())
            .spawn(move || {
                let _slot = slot;
                if worker_cancelled.load(Ordering::Acquire) {
                    return;
                }
                let _metric = crate::render_metrics::measure(crate::render_metrics::Stage::GitScan);
                let summary = catch_unwind(AssertUnwindSafe(|| scan(&workspace))).unwrap_or(None);
                if !worker_cancelled.load(Ordering::Acquire) {
                    let _ = sender.send(ScanResult {
                        workspace,
                        generation,
                        summary,
                        collected_at: std::time::Instant::now(),
                    });
                }
            })?;
        Ok(Some(Self {
            receiver,
            cancelled,
        }))
    }

    pub(crate) async fn result(&mut self) -> Option<ScanResult> {
        (&mut self.receiver).await.ok()
    }
}

impl Drop for Scan {
    fn drop(&mut self) {
        // Cancellation suppresses publication, not termination of native work.
        self.cancelled.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn abandoned_worker_keeps_slot_until_actual_return_and_panic_is_contained() {
        let _serial = TEST_SCAN_LOCK.lock().await;
        wait_for_test_slot().await;
        let (release, wait) = std::sync::mpsc::channel();
        let (entered, entry) = oneshot::channel();
        let first = Scan::start_with("old".into(), 1, move |_| {
            let _ = entered.send(());
            let _ = wait.recv();
            None
        })
        .expect("spawn")
        .expect("free slot");
        entry.await.expect("worker started");
        drop(first);
        assert!(
            Scan::start_with("new".into(), 2, |_| None)
                .expect("spawn")
                .is_none()
        );
        release.send(()).expect("release worker");
        // Wait on the ownership invariant, bounded by a timeout rather than sleeps.
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while SCAN_BUSY.load(Ordering::Acquire) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("worker released slot");
        let mut next = Scan::start_with("new".into(), 2, |_| panic!("dependency panic"))
            .expect("spawn")
            .expect("free slot");
        let result = next.result().await.expect("panic becomes unknown");
        assert_eq!(result.workspace, "new");
        assert_eq!(result.generation, 2);
        assert!(result.summary.is_none());
        wait_for_test_slot().await;
    }
}
