//! Explicit opt-in bridge for cross-crate I293 caller-boundary tests.
//!
//! This module is compiled only by the non-default `search-integration-test` feature. It exposes
//! the real dormant `GuardedSearchRouterTool` as an `AgentTool` trait object plus a probe that holds
//! execution after the real prepared composition has started but before any candidate network
//! future is polled. Production registration, defaults and network behavior are unchanged.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use talos_core::tool::{AgentTool, ToolNetworkPreparationError};
use tokio::sync::Notify;

use super::composition::SearchCompositionConfiguration;
use super::router::SearchRouterPolicy;
use super::router_tool::GuardedSearchRouterTool;
use super::transport::TransportLimits;

/// Test-only receipt for observing entry/drop of the real prepared search execution future.
#[derive(Debug, Default)]
pub struct SearchRouterIntegrationProbe {
    starts: AtomicUsize,
    drops: AtomicUsize,
    started: Notify,
}

impl SearchRouterIntegrationProbe {
    /// Number of prepared search executions that reached the deterministic pre-network hold point.
    #[must_use]
    pub fn starts(&self) -> usize {
        self.starts.load(Ordering::SeqCst)
    }

    /// Number of held prepared search execution futures released by caller-owned drop.
    #[must_use]
    pub fn drops(&self) -> usize {
        self.drops.load(Ordering::SeqCst)
    }

    /// Waits until at least one prepared search execution reaches the pre-network hold point.
    pub async fn wait_started(&self) {
        loop {
            if self.starts() > 0 {
                return;
            }
            let notified = self.started.notified();
            if self.starts() > 0 {
                return;
            }
            notified.await;
        }
    }

    pub(crate) fn enter(self: &Arc<Self>) -> SearchRouterIntegrationReceipt {
        self.starts.fetch_add(1, Ordering::SeqCst);
        self.started.notify_waiters();
        SearchRouterIntegrationReceipt(Arc::clone(self))
    }
}

pub(crate) struct SearchRouterIntegrationReceipt(Arc<SearchRouterIntegrationProbe>);

impl Drop for SearchRouterIntegrationReceipt {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}

/// Builds the real dormant prepared-only SearchRouter tool with a deterministic pre-network hold.
///
/// The returned tool is never registered by production composition. Its direct compatibility
/// candidate set, prepared manifest/profile, bounded transport limits and router policy are the real
/// I293 implementations; only the final pre-network hold is test-specific.
pub fn i293_guarded_search_router_tool_fixture(
    probe: Arc<SearchRouterIntegrationProbe>,
) -> Result<Arc<dyn AgentTool>, ToolNetworkPreparationError> {
    let configuration = SearchCompositionConfiguration::new(None)
        .map_err(|_| ToolNetworkPreparationError::Unavailable)?;
    let limits = TransportLimits::new(Duration::from_secs(5), 5, 1_048_576)
        .map_err(|_| ToolNetworkPreparationError::Invalid)?;
    let policy = SearchRouterPolicy::new(Duration::from_millis(50), 1, 20, 8192)
        .map_err(|_| ToolNetworkPreparationError::Invalid)?;
    Ok(Arc::new(
        GuardedSearchRouterTool::new(configuration, limits, policy).with_integration_probe(probe),
    ))
}
