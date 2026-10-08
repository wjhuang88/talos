//! Invocation-bound lifecycle ticket. Permission evaluation remains outside this type.

use std::time::{Duration, Instant};

use super::BrowserRequest;

/// Trusted host lifecycle epochs used to invalidate document-bound work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserLifecycle {
    session_epoch: u64,
    document_epoch: u64,
}

impl BrowserLifecycle {
    /// Creates the initial lifecycle state.
    pub const fn new() -> Self {
        Self {
            session_epoch: 0,
            document_epoch: 0,
        }
    }

    /// Returns the current session and document epochs.
    pub const fn epochs(self) -> (u64, u64) {
        (self.session_epoch, self.document_epoch)
    }

    /// Invalidates every outstanding ticket after session replacement or loss.
    pub fn replace_session(&mut self) {
        self.session_epoch = self.session_epoch.wrapping_add(1);
        self.document_epoch = self.document_epoch.wrapping_add(1);
    }

    /// Invalidates document-bound tickets after navigation or frame rebuild.
    pub fn change_document(&mut self) {
        self.document_epoch = self.document_epoch.wrapping_add(1);
    }
}

impl Default for BrowserLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded failure while validating or consuming a prepared invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserTicketError {
    /// The ticket's lifecycle epochs no longer match trusted host state.
    #[error("browser invocation context is stale")]
    Stale,
    /// The bounded ticket lifetime has elapsed.
    #[error("browser invocation ticket expired")]
    Expired,
}

/// A non-cloneable, invocation-bound request prepared for the permission composition root.
///
/// This object carries no permission decision and cannot be deserialized from model/plugin
/// input. Consuming it by value is the one-shot replay boundary.
#[derive(Debug)]
pub struct BrowserInvocationTicket {
    request: BrowserRequest,
    session_epoch: u64,
    document_epoch: u64,
    nonce: u128,
    expires_at: Instant,
}

impl BrowserInvocationTicket {
    /// Creates a ticket with the contract's bounded default lifetime.
    pub fn new(
        request: BrowserRequest,
        session_epoch: u64,
        document_epoch: u64,
        nonce: u128,
    ) -> Self {
        Self::with_ttl(
            request,
            session_epoch,
            document_epoch,
            nonce,
            Duration::from_secs(120),
        )
    }

    /// Creates a ticket with a caller-supplied testable lifetime.
    pub fn with_ttl(
        request: BrowserRequest,
        session_epoch: u64,
        document_epoch: u64,
        nonce: u128,
        ttl: Duration,
    ) -> Self {
        Self {
            request,
            session_epoch,
            document_epoch,
            nonce,
            expires_at: Instant::now() + ttl,
        }
    }

    /// Returns the request bound to this ticket.
    pub fn request(&self) -> &BrowserRequest {
        &self.request
    }

    /// Returns the invocation nonce for trusted composition bookkeeping.
    pub const fn nonce(&self) -> u128 {
        self.nonce
    }

    /// Verifies expiry and lifecycle epochs before authorization or dispatch.
    pub fn validate(
        &self,
        session_epoch: u64,
        document_epoch: u64,
    ) -> Result<(), BrowserTicketError> {
        if Instant::now() >= self.expires_at {
            return Err(BrowserTicketError::Expired);
        }
        if self.session_epoch != session_epoch || self.document_epoch != document_epoch {
            return Err(BrowserTicketError::Stale);
        }
        Ok(())
    }

    /// Consumes a ticket after the final trusted lifecycle check.
    pub fn consume(
        self,
        session_epoch: u64,
        document_epoch: u64,
    ) -> Result<BrowserRequest, BrowserTicketError> {
        self.validate(session_epoch, document_epoch)?;
        Ok(self.request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_executor::BrowserOperation;

    fn request() -> BrowserRequest {
        BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"tab-new"}"#).unwrap()
    }

    #[test]
    fn ticket_binds_epochs_and_consumes_once() {
        let ticket = BrowserInvocationTicket::new(request(), 4, 9, 42);
        assert_eq!(ticket.nonce(), 42);
        assert_eq!(ticket.request().operation(), BrowserOperation::TabNew);
        assert!(ticket.validate(4, 9).is_ok());
        let request = ticket.consume(4, 9).unwrap();
        assert_eq!(request.operation(), BrowserOperation::TabNew);
    }

    #[test]
    fn stale_and_expired_tickets_fail_closed() {
        let stale = BrowserInvocationTicket::new(request(), 4, 9, 1);
        assert_eq!(stale.validate(5, 9), Err(BrowserTicketError::Stale));
        let expired = BrowserInvocationTicket::with_ttl(request(), 4, 9, 2, Duration::ZERO);
        assert_eq!(expired.validate(4, 9), Err(BrowserTicketError::Expired));
    }

    #[test]
    fn lifecycle_changes_invalidate_document_work() {
        let mut lifecycle = BrowserLifecycle::new();
        let ticket =
            BrowserInvocationTicket::new(request(), lifecycle.epochs().0, lifecycle.epochs().1, 3);
        lifecycle.change_document();
        assert_eq!(
            ticket.validate(lifecycle.epochs().0, lifecycle.epochs().1),
            Err(BrowserTicketError::Stale)
        );
        lifecycle.replace_session();
        assert_eq!(lifecycle.epochs(), (1, 2));
    }
}
