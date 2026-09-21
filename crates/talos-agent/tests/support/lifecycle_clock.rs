use std::time::Duration;

/// Keep Tokio's paused clock from advancing while real OS readiness is pending.
/// Tokio inhibits automatic advancement while a spawn_blocking task is outstanding.
pub struct ReadinessClock {
    release: std::sync::mpsc::Sender<()>,
    worker: tokio::task::JoinHandle<Result<(), std::sync::mpsc::RecvTimeoutError>>,
}

impl ReadinessClock {
    /// Hold the paused clock until explicit release, drop, or the wall-clock backstop.
    pub async fn hold() -> Self {
        let (release, wait) = std::sync::mpsc::channel();
        let (started, ready) = tokio::sync::oneshot::channel();
        let worker = tokio::task::spawn_blocking(move || {
            let _ = started.send(());
            // A wall-clock backstop also bounds runtime shutdown after a test panic.
            wait.recv_timeout(Duration::from_secs(60))
        });
        ready.await.expect("readiness clock worker starts");
        Self { release, worker }
    }

    /// Release the guard and verify that its wall-clock backstop did not expire.
    pub async fn release(self) {
        self.release
            .send(())
            .expect("readiness watchdog did not expire");
        self.worker
            .await
            .expect("readiness clock worker joins")
            .expect("readiness watchdog did not expire");
    }

    /// Cross deadlines created from std::Instant after the Tokio clock was paused.
    pub async fn advance_past_execution_deadline(&self, execution_timeout: Duration) {
        let target = tokio::time::Instant::from_std(std::time::Instant::now() + execution_timeout);
        let delta = target.saturating_duration_since(tokio::time::Instant::now());
        tokio::time::advance(delta).await;
    }
}
