//! Exact operation-family targets produced from trusted host state.

use super::{
    BrowserElementRef, BrowserFrameRef, BrowserFrameScope, BrowserInvocationTicket,
    BrowserOperation, BrowserOrigin, BrowserRequest, BrowserSnapshotRef, BrowserTabRef,
};

/// Trusted document origin usable for lifecycle and inventory, never inherited authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserDocumentOrigin {
    /// Host-verified effective HTTP origin.
    Http(BrowserOrigin),
    /// Host-verified opaque document, bound to its non-reused frame generation.
    OpaqueDocument(BrowserFrameRef),
}

/// Exact typed permission target. Session/executor identity and epochs are bound by the ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserPermissionTarget {
    /// Bounded session inventory only.
    SessionInventory,
    /// Create one empty tab.
    SessionCreate,
    /// Bounded delay only.
    SessionWait {
        /// Exact admitted delay.
        milliseconds: u16,
    },
    /// Close the bound session window.
    SessionClose,
    /// Close or switch one explicit tab.
    TabLifecycle {
        /// Exact tab generation.
        tab: BrowserTabRef,
        /// Exact close/switch operation.
        operation: BrowserOperation,
    },
    /// Bounded frame structure disclosure.
    FrameInventory {
        /// Exact tab generation.
        tab: BrowserTabRef,
        /// Host-verified current top document origin.
        top_origin: BrowserDocumentOrigin,
    },
    /// Top-level navigation only.
    Navigate {
        /// Exact tab generation.
        tab: BrowserTabRef,
        /// Host-verified current top document origin.
        current_origin: BrowserDocumentOrigin,
        /// Exact approved destination origin; redirects cannot widen it.
        destination: BrowserOrigin,
    },
    /// One explicit frame-local observation.
    Observe {
        /// Independently resolved top and frame origins.
        scope: BrowserFrameScope,
        /// Exact observation operation.
        operation: BrowserOperation,
        /// Exact element tuple for wait-for-element, absent for other observations.
        element: Option<(BrowserSnapshotRef, BrowserElementRef)>,
    },
    /// One exact frame-local interaction.
    Interact {
        /// Independently resolved top and frame origins.
        scope: BrowserFrameScope,
        /// Exact interaction operation.
        operation: BrowserOperation,
        /// Exact element tuple when the operation targets a captured element.
        element: Option<(BrowserSnapshotRef, BrowserElementRef)>,
    },
}

/// Non-cloneable admitted invocation. It conveys identity, not execution permission.
#[derive(Debug)]
pub struct PreparedBrowserInvocation {
    pub(crate) ticket: BrowserInvocationTicket,
    pub(crate) target: BrowserPermissionTarget,
}

impl PreparedBrowserInvocation {
    /// Returns the immutable normalized request; values may contain sensitive user input.
    pub fn request(&self) -> &BrowserRequest {
        self.ticket.request()
    }

    /// Returns the target derived from trusted host state.
    pub fn target(&self) -> &BrowserPermissionTarget {
        &self.target
    }
}
