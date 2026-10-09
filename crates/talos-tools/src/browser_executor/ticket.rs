//! Invocation-bound lifecycle ticket. Permission evaluation remains outside this type.

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use super::BrowserRequest;

/// Trusted host lifecycle epochs used to invalidate document-bound work.
#[derive(Debug)]
pub struct BrowserLifecycle {
    identity: Arc<()>,
    outstanding: BTreeMap<u64, Reservation>,
    next_nonce: u64,
    session_epoch: u64,
    document_epoch: u64,
    available: bool,
}

#[derive(Debug)]
struct Reservation {
    expires_at: Instant,
    validity: Validity,
}

/// Either owner dropping this guard revokes all copies of the challenge.
#[derive(Debug)]
struct Validity(Arc<AtomicBool>);

impl Drop for Validity {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl BrowserLifecycle {
    /// Creates the initial lifecycle state.
    pub fn new() -> Self {
        Self {
            identity: Arc::new(()),
            outstanding: BTreeMap::new(),
            next_nonce: 0,
            session_epoch: 0,
            document_epoch: 0,
            available: true,
        }
    }

    /// Returns the current session and document epochs.
    pub const fn epochs(&self) -> (u64, u64) {
        (self.session_epoch, self.document_epoch)
    }

    /// Invalidates every outstanding ticket after session replacement or loss.
    pub fn replace_session(&mut self) {
        match self.session_epoch.checked_add(1) {
            Some(epoch) => self.session_epoch = epoch,
            None => self.available = false,
        }
        self.change_document();
    }

    /// Invalidates document-bound tickets after navigation or frame rebuild.
    pub fn change_document(&mut self) {
        self.outstanding.clear();
        match self.document_epoch.checked_add(1) {
            Some(epoch) => self.document_epoch = epoch,
            None => self.available = false,
        }
    }

    /// Reports whether epoch bookkeeping remains usable.
    pub const fn is_available(&self) -> bool {
        self.available
    }

    /// Disables this domain after loss of trusted host synchronization.
    /// A new domain is required after host resynchronization.
    pub fn lose_synchronization(&mut self) {
        self.available = false;
        self.outstanding.clear();
    }

    /// Reserves a local ticket. This performs no origin admission or permission evaluation.
    pub fn prepare(
        &mut self,
        request: BrowserRequest,
    ) -> Result<BrowserInvocationTicket, BrowserTicketError> {
        self.prepare_at(request, Instant::now())
    }

    fn prepare_at(
        &mut self,
        request: BrowserRequest,
        now: Instant,
    ) -> Result<BrowserInvocationTicket, BrowserTicketError> {
        if !self.available {
            return Err(BrowserTicketError::Unavailable);
        }
        self.outstanding.retain(|_, reservation| {
            reservation.expires_at > now && reservation.validity.0.load(Ordering::Acquire)
        });
        if self.outstanding.len() >= 256 {
            return Err(BrowserTicketError::Capacity);
        }
        let nonce = self
            .next_nonce
            .checked_add(1)
            .ok_or(BrowserTicketError::Unavailable)?;
        let expires_at = now
            .checked_add(Duration::from_secs(120))
            .ok_or(BrowserTicketError::Unavailable)?;
        self.next_nonce = nonce;
        let live = Arc::new(AtomicBool::new(true));
        self.outstanding.insert(
            nonce,
            Reservation {
                expires_at,
                validity: Validity(live.clone()),
            },
        );
        Ok(BrowserInvocationTicket {
            request,
            identity: self.identity.clone(),
            session_epoch: self.session_epoch,
            document_epoch: self.document_epoch,
            nonce,
            expires_at,
            validity: Validity(live),
        })
    }

    /// Validates the issuing domain, registry membership, expiry and lifecycle.
    pub fn validate(&self, ticket: &BrowserInvocationTicket) -> Result<(), BrowserTicketError> {
        self.validate_at(ticket, Instant::now())
    }

    fn validate_at(
        &self,
        ticket: &BrowserInvocationTicket,
        now: Instant,
    ) -> Result<(), BrowserTicketError> {
        if !self.available {
            return Err(BrowserTicketError::Unavailable);
        }
        if !Arc::ptr_eq(&self.identity, &ticket.identity)
            || self.session_epoch != ticket.session_epoch
            || self.document_epoch != ticket.document_epoch
        {
            return Err(BrowserTicketError::Stale);
        }
        if now >= ticket.expires_at {
            return Err(BrowserTicketError::Expired);
        }
        if !ticket.validity.0.load(Ordering::Acquire)
            || self.outstanding.get(&ticket.nonce).map(|r| r.expires_at) != Some(ticket.expires_at)
        {
            return Err(BrowserTicketError::Consumed);
        }
        Ok(())
    }

    /// Consumes and invalidates a local reservation, without granting execution authority.
    /// Returns no executable request; authorized dispatch must use a separate permission gate.
    pub fn discard(&mut self, ticket: BrowserInvocationTicket) -> Result<(), BrowserTicketError> {
        let result = self.validate(&ticket);
        if Arc::ptr_eq(&self.identity, &ticket.identity) {
            self.outstanding.remove(&ticket.nonce);
        }
        result
    }

    pub(crate) fn consume(
        &mut self,
        ticket: BrowserInvocationTicket,
    ) -> Result<BrowserRequest, BrowserTicketError> {
        let result = self.validate(&ticket);
        if Arc::ptr_eq(&self.identity, &ticket.identity) {
            self.outstanding.remove(&ticket.nonce);
        }
        result?;
        Ok(ticket.request)
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
    /// Trusted lifecycle synchronization is unavailable or counters are exhausted.
    #[error("browser context unavailable")]
    Unavailable,
    /// The 256 outstanding reservation limit has been reached.
    #[error("browser ticket capacity exhausted")]
    Capacity,
    /// The reservation was already consumed or discarded.
    #[error("browser ticket already consumed")]
    Consumed,
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
/// input. Registry membership prevents reuse; this is not an authorization capability.
#[derive(Debug)]
pub struct BrowserInvocationTicket {
    validity: Validity,
    identity: Arc<()>,
    request: BrowserRequest,
    session_epoch: u64,
    document_epoch: u64,
    nonce: u64,
    expires_at: Instant,
}

/// Private identity copied into an authorization challenge, never supplied by a caller.
#[derive(Debug, Clone)]
pub(crate) struct InvocationBinding {
    live: Arc<AtomicBool>,
    identity: Arc<()>,
    nonce: u64,
    session_epoch: u64,
    document_epoch: u64,
    expires_at: Instant,
}

impl PartialEq for InvocationBinding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
            && self.nonce == other.nonce
            && self.session_epoch == other.session_epoch
            && self.document_epoch == other.document_epoch
            && self.expires_at == other.expires_at
    }
}

impl Eq for InvocationBinding {}

impl InvocationBinding {
    pub(crate) fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire) && Instant::now() < self.expires_at
    }
}

impl BrowserInvocationTicket {
    pub(crate) fn binding(&self) -> InvocationBinding {
        InvocationBinding {
            live: self.validity.0.clone(),
            identity: self.identity.clone(),
            nonce: self.nonce,
            session_epoch: self.session_epoch,
            document_epoch: self.document_epoch,
            expires_at: self.expires_at,
        }
    }
    /// Returns the request bound to this ticket.
    pub fn request(&self) -> &BrowserRequest {
        &self.request
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_executor::BrowserOperation;

    fn request() -> BrowserRequest {
        BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"tab-new"}"#)
            .expect("valid test fixture")
    }

    #[test]
    fn ticket_binds_epochs_and_consumes_once() {
        let mut host = BrowserLifecycle::new();
        let ticket = host.prepare(request()).expect("valid test fixture");
        assert_eq!(ticket.request().operation(), BrowserOperation::TabNew);
        let duplicate = BrowserInvocationTicket {
            validity: Validity(ticket.validity.0.clone()),
            identity: ticket.identity.clone(),
            request: request(),
            session_epoch: ticket.session_epoch,
            document_epoch: ticket.document_epoch,
            nonce: ticket.nonce,
            expires_at: ticket.expires_at,
        };
        assert!(host.discard(ticket).is_ok());
        assert_eq!(host.discard(duplicate), Err(BrowserTicketError::Consumed));
    }

    #[test]
    fn dropped_or_invalidated_owners_revoke_challenges_immediately() {
        let mut host = BrowserLifecycle::new();
        for _ in 0..512 {
            let ticket = host
                .prepare(request())
                .expect("dropped tickets free capacity");
            let binding = ticket.binding();
            assert!(binding.is_live());
            drop(ticket);
            assert!(!binding.is_live());
        }
        let ticket = host.prepare(request()).expect("ticket");
        let binding = ticket.binding();
        host.change_document();
        assert!(!binding.is_live());
        drop(ticket);

        let ticket = host.prepare(request()).expect("ticket");
        let binding = ticket.binding();
        host.discard(ticket).expect("discard");
        assert!(!binding.is_live());

        let ticket = host.prepare(request()).expect("ticket");
        let binding = ticket.binding();
        drop(host);
        assert!(!binding.is_live());
        drop(ticket);
    }

    #[test]
    fn stale_and_expired_tickets_fail_closed() {
        let mut host = BrowserLifecycle::new();
        let now = Instant::now();
        let ticket = host.prepare_at(request(), now).expect("valid test fixture");
        assert_eq!(
            BrowserLifecycle::new().validate(&ticket),
            Err(BrowserTicketError::Stale)
        );
        assert_eq!(
            host.validate_at(&ticket, now + Duration::from_secs(120)),
            Err(BrowserTicketError::Expired)
        );
    }

    #[test]
    fn lifecycle_changes_invalidate_document_work() {
        let mut lifecycle = BrowserLifecycle::new();
        let ticket = lifecycle.prepare(request()).expect("valid test fixture");
        lifecycle.change_document();
        assert_eq!(lifecycle.validate(&ticket), Err(BrowserTicketError::Stale));
        lifecycle.replace_session();
        assert_eq!(lifecycle.epochs(), (1, 2));
    }

    #[test]
    fn epochs_never_wrap_into_old_context() {
        let mut lifecycle = BrowserLifecycle::new();
        lifecycle.session_epoch = u64::MAX;
        lifecycle.document_epoch = u64::MAX;
        lifecycle.replace_session();
        assert!(!lifecycle.is_available());
        assert_eq!(lifecycle.epochs(), (u64::MAX, u64::MAX));
        assert_eq!(
            lifecycle
                .prepare(request())
                .expect_err("invalid test fixture must be rejected"),
            BrowserTicketError::Unavailable
        );
    }

    #[test]
    fn capacity_expiry_and_loss_are_fail_closed() {
        let mut host = BrowserLifecycle::new();
        let now = Instant::now();
        let tickets: Vec<_> = (0..256)
            .map(|_| host.prepare_at(request(), now).expect("valid test fixture"))
            .collect();
        assert_eq!(
            host.prepare_at(request(), now)
                .expect_err("invalid test fixture must be rejected"),
            BrowserTicketError::Capacity
        );
        let fresh = host
            .prepare_at(request(), now + Duration::from_secs(120))
            .expect("valid test fixture");
        assert_eq!(host.outstanding.len(), 1);
        assert_ne!(tickets[0].nonce, fresh.nonce);
        host.lose_synchronization();
        assert_eq!(host.validate(&fresh), Err(BrowserTicketError::Unavailable));
        assert_eq!(
            host.prepare(request())
                .expect_err("invalid test fixture must be rejected"),
            BrowserTicketError::Unavailable
        );
    }

    #[test]
    fn foreign_discard_cannot_remove_local_reservation() {
        let mut first = BrowserLifecycle::new();
        let mut second = BrowserLifecycle::new();
        let foreign = first.prepare(request()).expect("valid test fixture");
        let local = second.prepare(request()).expect("valid test fixture");
        assert_eq!(foreign.nonce, local.nonce);
        assert_eq!(second.discard(foreign), Err(BrowserTicketError::Stale));
        assert!(second.validate(&local).is_ok());
        assert!(second.discard(local).is_ok());
        assert!(second.outstanding.is_empty());
    }

    #[test]
    fn nonce_exhaustion_never_reuses_an_identity() {
        let mut host = BrowserLifecycle::new();
        host.next_nonce = u64::MAX;
        assert_eq!(
            host.prepare(request())
                .expect_err("invalid test fixture must be rejected"),
            BrowserTicketError::Unavailable
        );
        host.replace_session();
        assert_eq!(
            host.prepare(request())
                .expect_err("invalid test fixture must be rejected"),
            BrowserTicketError::Unavailable
        );
        assert!(host.outstanding.is_empty());
    }
}
