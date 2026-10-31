//! Bounded manual-operation identity and receipt bookkeeping for the Actor.

use std::collections::VecDeque;

/// Content-free terminal facts retained for duplicate manual requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactOutcome {
    /// A validated projection was atomically installed.
    Applied { before: u32, after: u32 },
    /// Whole-request pressure was below the manual eligibility threshold.
    NotEligible,
    /// Checkpoint generation or validation failed without changing context.
    Failed,
    /// Cancellation won before atomic application.
    Cancelled,
}

/// Result of admitting a generation-qualified monotonic operation ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactAdmission {
    /// A newly accepted operation owns the single pending slot.
    Accepted,
    /// This operation already owns the pending slot; do not run it twice.
    Pending,
    /// A different operation currently owns the pending slot.
    Busy,
    /// A retained terminal receipt is returned without execution.
    Completed(CompactOutcome),
    /// An old ID has fallen out of the receipt window or was already superseded.
    Expired,
    /// The caller addresses a different session generation.
    StaleGeneration,
}

/// One pending operation and at most 64 terminal receipts for one generation.
///
/// The Actor owns this ledger and calls `finish` only at the projection swap or
/// an earlier terminal failure/cancellation. It is not a second transcript store.
#[derive(Debug)]
pub struct CompactOperations {
    generation: u64,
    highest: Option<u64>,
    pending: Option<u64>,
    receipts: VecDeque<(u64, CompactOutcome)>,
}

impl CompactOperations {
    /// Start a receipt window for a runtime generation.
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            highest: None,
            pending: None,
            receipts: VecDeque::new(),
        }
    }

    /// Admit one ID without consuming IDs rejected as busy or stale.
    pub fn admit(&mut self, generation: u64, id: u64) -> CompactAdmission {
        if generation != self.generation {
            return CompactAdmission::StaleGeneration;
        }
        if let Some((_, outcome)) = self.receipts.iter().find(|(receipt, _)| *receipt == id) {
            return CompactAdmission::Completed(*outcome);
        }
        if self.pending == Some(id) {
            return CompactAdmission::Pending;
        }
        if self.highest.is_some_and(|highest| id <= highest) {
            return CompactAdmission::Expired;
        }
        if self.pending.is_some() {
            return CompactAdmission::Busy;
        }
        self.highest = Some(id);
        self.pending = Some(id);
        CompactAdmission::Accepted
    }

    /// Record the unique terminal result. Late or mismatched completions fail.
    /// The caller must not mutate projection state if this returns false.
    pub fn finish(&mut self, generation: u64, id: u64, outcome: CompactOutcome) -> bool {
        if generation != self.generation || self.pending != Some(id) {
            return false;
        }
        self.pending = None;
        if self.receipts.len() == 64 {
            self.receipts.pop_front();
        }
        self.receipts.push_back((id, outcome));
        true
    }

    /// Cancel pending work and clear receipts when replacing a generation.
    /// Returns the old operation key so the Actor can emit its one cancellation.
    pub fn replace_generation(&mut self, generation: u64) -> Option<(u64, u64)> {
        if generation == self.generation {
            return None;
        }
        let cancelled = self.pending.map(|id| (self.generation, id));
        *self = Self::new(generation);
        cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_and_busy_requests_never_reexecute_or_consume_ids() {
        let mut ledger = CompactOperations::new(7);
        assert_eq!(ledger.admit(6, 1), CompactAdmission::StaleGeneration);
        assert_eq!(ledger.admit(7, 1), CompactAdmission::Accepted);
        assert_eq!(ledger.admit(7, 1), CompactAdmission::Pending);
        assert_eq!(ledger.admit(7, 2), CompactAdmission::Busy);
        assert!(!ledger.finish(7, 2, CompactOutcome::Failed));
        assert!(ledger.finish(7, 1, CompactOutcome::Cancelled));
        assert!(!ledger.finish(
            7,
            1,
            CompactOutcome::Applied {
                before: 100,
                after: 50
            }
        ));
        assert_eq!(
            ledger.admit(7, 1),
            CompactAdmission::Completed(CompactOutcome::Cancelled)
        );
        assert_eq!(ledger.admit(7, 2), CompactAdmission::Accepted);
    }

    #[test]
    fn receipt_eviction_rejects_replay_and_preserves_maximum_id() {
        let mut ledger = CompactOperations::new(1);
        for id in 0..65 {
            assert_eq!(ledger.admit(1, id), CompactAdmission::Accepted);
            assert!(ledger.finish(1, id, CompactOutcome::NotEligible));
        }
        assert_eq!(ledger.receipts.len(), 64);
        assert_eq!(ledger.admit(1, 0), CompactAdmission::Expired);
        assert_eq!(
            ledger.admit(1, 1),
            CompactAdmission::Completed(CompactOutcome::NotEligible)
        );
        assert_eq!(ledger.admit(1, u64::MAX), CompactAdmission::Accepted);
        assert!(ledger.finish(1, u64::MAX, CompactOutcome::Failed));
        assert_eq!(ledger.admit(1, u64::MAX - 1), CompactAdmission::Expired);
    }

    #[test]
    fn generation_replacement_invalidates_pending_results_and_receipts() {
        let mut ledger = CompactOperations::new(1);
        assert_eq!(ledger.admit(1, 10), CompactAdmission::Accepted);
        assert_eq!(ledger.replace_generation(1), None);
        assert_eq!(ledger.admit(1, 10), CompactAdmission::Pending);
        assert_eq!(ledger.replace_generation(2), Some((1, 10)));
        assert!(!ledger.finish(
            1,
            10,
            CompactOutcome::Applied {
                before: 100,
                after: 50
            }
        ));
        assert_eq!(ledger.admit(1, 10), CompactAdmission::StaleGeneration);
        assert_eq!(ledger.admit(2, 1), CompactAdmission::Accepted);
    }
}
