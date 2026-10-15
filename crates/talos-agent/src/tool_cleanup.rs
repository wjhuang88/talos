//! Per-Agent invocation receipts retained before dispatch and after cancellation.

use futures_util::FutureExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use talos_core::tool::{ToolCleanupError, ToolCleanupReceipt};

type TurnReceipt = (talos_plugin::TurnId, Arc<dyn ToolCleanupReceipt>);

#[derive(Clone, Default)]
pub(crate) struct ToolCleanup {
    receipts: Arc<Mutex<Vec<TurnReceipt>>>,
}

impl ToolCleanup {
    pub(crate) fn register(
        &self,
        turn: talos_plugin::TurnId,
        receipt: Arc<dyn ToolCleanupReceipt>,
    ) {
        let mut receipts = self
            .receipts
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !receipts
            .iter()
            .any(|(id, old)| *id == turn && Arc::ptr_eq(old, &receipt))
        {
            receipts.push((turn, receipt));
        }
    }

    // Call only after the actual Agent task has finished/joined, so dispatch
    // cannot register another stage while this snapshot is being awaited.
    pub(crate) async fn wait(
        &self,
        turn: talos_plugin::TurnId,
        budget: Duration,
    ) -> Result<(), ToolCleanupError> {
        let receipts = self
            .receipts
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        let deadline = tokio::time::Instant::now() + budget;
        for (id, receipt) in receipts {
            if id != turn {
                continue;
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let observed = std::panic::AssertUnwindSafe(async { receipt.wait(remaining).await })
                .catch_unwind();
            tokio::time::timeout_at(deadline, observed)
                .await
                .map_err(|_| ToolCleanupError::Timeout)?
                .map_err(|_| ToolCleanupError::Unconfirmed)??;
            self.receipts
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .retain(|(id, registered)| *id != turn || !Arc::ptr_eq(registered, &receipt));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Pending;
    #[async_trait::async_trait]
    impl ToolCleanupReceipt for Pending {
        async fn wait(&self, _: Duration) -> Result<(), ToolCleanupError> {
            std::future::pending().await
        }
    }
    struct Panics;
    struct Complete;
    #[async_trait::async_trait]
    impl ToolCleanupReceipt for Complete {
        async fn wait(&self, _: Duration) -> Result<(), ToolCleanupError> {
            Ok(())
        }
    }
    #[async_trait::async_trait]
    impl ToolCleanupReceipt for Panics {
        async fn wait(&self, _: Duration) -> Result<(), ToolCleanupError> {
            panic!("fixture");
        }
    }
    #[tokio::test]
    async fn waiting_is_bounded_and_panics_do_not_escape() {
        let pending = ToolCleanup::default();
        pending.register(talos_plugin::TurnId(1), Arc::new(Pending));
        assert_eq!(
            pending
                .wait(talos_plugin::TurnId(1), Duration::from_millis(1))
                .await,
            Err(ToolCleanupError::Timeout)
        );
        let failed = ToolCleanup::default();
        failed.register(talos_plugin::TurnId(1), Arc::new(Panics));
        assert_eq!(
            failed
                .wait(talos_plugin::TurnId(1), Duration::from_secs(1))
                .await,
            Err(ToolCleanupError::Unconfirmed)
        );
        ToolCleanup::default()
            .wait(talos_plugin::TurnId(1), Duration::from_secs(1))
            .await
            .expect("independent scope");
    }

    #[tokio::test]
    async fn turns_are_isolated_and_only_confirmed_receipts_are_released() {
        let cleanup = ToolCleanup::default();
        cleanup.register(talos_plugin::TurnId(1), Arc::new(Pending));
        cleanup.register(talos_plugin::TurnId(2), Arc::new(Complete));
        cleanup
            .wait(talos_plugin::TurnId(2), Duration::from_secs(1))
            .await
            .expect("independent turn");
        assert_eq!(cleanup.receipts.lock().expect("receipts").len(), 1);
        assert_eq!(
            cleanup
                .wait(talos_plugin::TurnId(1), Duration::from_millis(1))
                .await,
            Err(ToolCleanupError::Timeout)
        );
        assert_eq!(cleanup.receipts.lock().expect("receipts").len(), 1);
    }
}
