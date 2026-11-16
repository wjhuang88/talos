//! In-memory approval observations; these carry no permission authority.

use std::sync::{Arc, Mutex};
use talos_conversation::ApprovalPresentation;
use talos_core::ApprovalChoice;
use tokio::sync::{oneshot, watch};

#[derive(Default, Debug)]
struct Generation {
    epoch: u64,
    sequence: u64,
    exhausted: bool,
}

/// Synchronous invalidation shared by a host publication boundary and its TUI.
/// Invalidate while holding the host's session-publication fence, before publishing
/// replacement routes. Failed publication must not reactivate old tickets.
#[derive(Clone, Debug)]
pub struct ApprovalBinding {
    state: Arc<Mutex<Generation>>,
    changed: watch::Sender<u64>,
}

impl Default for ApprovalBinding {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(Generation::default())),
            changed: watch::channel(0).0,
        }
    }
}

impl ApprovalBinding {
    /// Permanently invalidates every ticket issued before this call.
    pub fn invalidate(&self) {
        let epoch = self.state.lock().ok().map(|mut state| {
            if let Some(epoch) = state.epoch.checked_add(1) {
                state.epoch = epoch;
            } else {
                state.exhausted = true;
            }
            state.epoch
        });
        // Wake waiters even after poison: current() then rejects the binding.
        self.changed.send_modify(|value| {
            if let Some(epoch) = epoch {
                *value = epoch;
            }
        });
    }

    /// Creates one indivisible display request and its uniquely correlated waiter.
    /// The caller must validate its session identity under the host publication fence.
    /// Counter exhaustion or a poisoned binding refuses creation.
    pub fn request(
        &self,
        tool_name: String,
        arguments: serde_json::Value,
        summary_fields: Vec<String>,
        presentation: Option<ApprovalPresentation>,
        preview: Option<String>,
    ) -> Option<(ObservedApprovalRequest, ApprovalWaiter)> {
        let mut state = self.state.lock().ok()?;
        if state.exhausted {
            return None;
        }
        let Some(sequence) = state.sequence.checked_add(1) else {
            state.exhausted = true;
            self.changed.send_modify(|_| {});
            return None;
        };
        state.sequence = sequence;
        let ticket = Ticket {
            binding: self.clone(),
            epoch: state.epoch,
            sequence,
        };
        let (tx, rx) = oneshot::channel();
        Some((
            ObservedApprovalRequest {
                tool_name,
                arguments,
                summary_fields,
                presentation,
                preview,
                responder: ObservedResponder {
                    ticket: ticket.clone(),
                    changed: self.changed.subscribe(),
                    tx,
                },
            },
            ApprovalWaiter { ticket, rx },
        ))
    }
}

#[derive(Clone, Debug)]
struct Ticket {
    binding: ApprovalBinding,
    epoch: u64,
    sequence: u64,
}

impl Ticket {
    fn current(&self) -> bool {
        self.binding
            .state
            .lock()
            .is_ok_and(|state| !state.exhausted && state.epoch == self.epoch)
    }
}

/// Observed provenance of one TUI response, separate from its permission choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApprovalResponseSource {
    /// A key selected a choice on this exact live approval surface.
    Human,
    /// The user explicitly cancelled this exact live approval surface.
    UserCancellation,
    /// The surface, binding or response channel was unavailable.
    Unavailable,
}

/// Opaque TUI observation. Only the actual input handler can produce Human results.
#[derive(Clone, Debug)]
pub struct ObservedApprovalResponse {
    choice: ApprovalChoice,
    source: ApprovalResponseSource,
    ticket: Option<Ticket>,
}

impl ObservedApprovalResponse {
    /// Returns the original choice; unavailable and cancelled responses are always Deny.
    pub fn choice(&self) -> &ApprovalChoice {
        &self.choice
    }
    /// Returns independently observed response provenance.
    pub fn source(&self) -> ApprovalResponseSource {
        self.source
    }
    /// Checks that this response still belongs to the current host publication.
    pub fn is_current(&self) -> bool {
        self.ticket.as_ref().is_some_and(Ticket::current)
    }
    fn unavailable() -> Self {
        Self {
            choice: ApprovalChoice::Deny,
            source: ApprovalResponseSource::Unavailable,
            ticket: None,
        }
    }
}

/// Unique response channel paired with one display request.
pub struct ApprovalWaiter {
    ticket: Ticket,
    rx: oneshot::Receiver<ObservedApprovalResponse>,
}

impl ApprovalWaiter {
    /// Monotonic request identity, independent of repeated tool names or arguments.
    pub fn request_id(&self) -> u64 {
        self.ticket.sequence
    }
    /// Waits for this request only; late results after invalidation become unavailable.
    pub async fn receive(mut self) -> ObservedApprovalResponse {
        let mut changed = self.ticket.binding.changed.subscribe();
        loop {
            if !self.ticket.current() {
                return ObservedApprovalResponse::unavailable();
            }
            tokio::select! {
                response = &mut self.rx => {
                    let response = response.unwrap_or_else(|_| ObservedApprovalResponse::unavailable());
                    return if self.ticket.current() { response } else { ObservedApprovalResponse::unavailable() };
                }
                result = changed.changed() => {
                    if result.is_err() { return ObservedApprovalResponse::unavailable(); }
                }
            }
        }
    }
}

/// Typed approval display and its sole responder. No legacy placeholder is emitted.
pub struct ObservedApprovalRequest {
    pub(crate) tool_name: String,
    pub(crate) arguments: serde_json::Value,
    pub(crate) summary_fields: Vec<String>,
    pub(crate) presentation: Option<ApprovalPresentation>,
    pub(crate) preview: Option<String>,
    pub(crate) responder: ObservedResponder,
}

pub(crate) struct ObservedResponder {
    ticket: Ticket,
    changed: watch::Receiver<u64>,
    tx: oneshot::Sender<ObservedApprovalResponse>,
}

impl ObservedResponder {
    pub(crate) fn expired(&self) -> bool {
        self.tx.is_closed() || !self.ticket.current()
    }
    pub(crate) async fn closed(&mut self) {
        loop {
            if self.expired() {
                return;
            }
            tokio::select! {
                _ = self.tx.closed() => return,
                result = self.changed.changed() => { if result.is_err() { return; } }
            }
        }
    }
    pub(crate) fn respond(self, choice: ApprovalChoice, cancelled: bool) -> bool {
        // Sending and invalidation share one fence; a retired request cannot be
        // relabelled as a human response even if its key event was already queued.
        let Ok(state) = self.ticket.binding.state.lock() else {
            return false;
        };
        if state.exhausted || state.epoch != self.ticket.epoch {
            return false;
        }
        self.tx
            .send(ObservedApprovalResponse {
                ticket: Some(self.ticket.clone()),
                choice: if cancelled {
                    ApprovalChoice::Deny
                } else {
                    choice
                },
                source: if cancelled {
                    ApprovalResponseSource::UserCancellation
                } else {
                    ApprovalResponseSource::Human
                },
            })
            .is_ok()
    }
}
