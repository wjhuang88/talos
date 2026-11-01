//! Bounded compaction-operation identity and receipt bookkeeping for the Actor.

use std::collections::VecDeque;

pub use talos_core::session::CompactOutcome;

/// Runtime-selected operation origin; models cannot choose a manual identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactOrigin {
    /// User/host manual compaction sequence.
    Manual,
    /// Runtime-issued sequence for model checkpoint requests.
    Model,
}

impl CompactOrigin {
    fn index(self) -> usize {
        match self {
            Self::Manual => 0,
            Self::Model => 1,
        }
    }
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
    highest: [Option<u64>; 2],
    pending: Option<(CompactOrigin, u64)>,
    receipts: VecDeque<((CompactOrigin, u64), CompactOutcome)>,
}

impl CompactOperations {
    /// Start a receipt window for a runtime generation.
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            highest: [None, None],
            pending: None,
            receipts: VecDeque::new(),
        }
    }

    /// Admit one ID without consuming IDs rejected as busy or stale.
    pub fn admit(&mut self, generation: u64, id: u64) -> CompactAdmission {
        self.admit_from(generation, CompactOrigin::Manual, id)
    }

    /// Admit a runtime-issued origin-qualified ID into the one shared pending slot.
    /// Independent monotonic sequences prevent model operations from consuming
    /// manual IDs. The origin and model IDs must be selected by the runtime.
    pub fn admit_from(
        &mut self,
        generation: u64,
        origin: CompactOrigin,
        id: u64,
    ) -> CompactAdmission {
        let key = (origin, id);
        if generation != self.generation {
            return CompactAdmission::StaleGeneration;
        }
        if let Some((_, outcome)) = self.receipts.iter().find(|(receipt, _)| *receipt == key) {
            return CompactAdmission::Completed(*outcome);
        }
        if self.pending == Some(key) {
            return CompactAdmission::Pending;
        }
        if self.highest[origin.index()].is_some_and(|highest| id <= highest) {
            return CompactAdmission::Expired;
        }
        if self.pending.is_some() {
            return CompactAdmission::Busy;
        }
        self.highest[origin.index()] = Some(id);
        self.pending = Some(key);
        CompactAdmission::Accepted
    }

    /// Observe an existing operation without admitting or consuming a new ID.
    pub(crate) fn observe(&self, generation: u64, id: u64) -> CompactAdmission {
        self.observe_from(generation, CompactOrigin::Manual, id)
    }

    /// Observe an origin-qualified operation without allocating an ID.
    pub fn observe_from(
        &self,
        generation: u64,
        origin: CompactOrigin,
        id: u64,
    ) -> CompactAdmission {
        let key = (origin, id);
        if generation != self.generation {
            return CompactAdmission::StaleGeneration;
        }
        if let Some((_, outcome)) = self.receipts.iter().find(|(receipt, _)| *receipt == key) {
            return CompactAdmission::Completed(*outcome);
        }
        if self.pending == Some(key) {
            CompactAdmission::Pending
        } else {
            CompactAdmission::Expired
        }
    }

    /// Record the unique terminal result. Late or mismatched completions fail.
    /// The caller must not mutate projection state if this returns false.
    pub fn finish(&mut self, generation: u64, id: u64, outcome: CompactOutcome) -> bool {
        self.finish_from(generation, CompactOrigin::Manual, id, outcome)
    }

    /// Finish exactly the pending origin-qualified operation, never another
    /// origin's operation with the same numeric ID.
    pub fn finish_from(
        &mut self,
        generation: u64,
        origin: CompactOrigin,
        id: u64,
        outcome: CompactOutcome,
    ) -> bool {
        let key = (origin, id);
        if generation != self.generation || self.pending != Some(key) {
            return false;
        }
        self.pending = None;
        if self.receipts.len() == 64 {
            self.receipts.pop_front();
        }
        self.receipts.push_back((key, outcome));
        true
    }

    /// Cancel pending work and clear receipts when replacing a generation.
    /// Returns the old operation key so the Actor can emit its one cancellation.
    pub fn replace_generation(&mut self, generation: u64) -> Option<(u64, u64)> {
        self.replace_generation_with_origin(generation)
            .map(|(generation, _, id)| (generation, id))
    }

    /// Replace a mixed-origin generation and retain the origin in its cancellation
    /// key. Mixed-origin Actors must use this instead of the legacy manual helper.
    pub fn replace_generation_with_origin(
        &mut self,
        generation: u64,
    ) -> Option<(u64, CompactOrigin, u64)> {
        if generation == self.generation {
            return None;
        }
        let cancelled = self
            .pending
            .map(|(origin, id)| (self.generation, origin, id));
        *self = Self::new(generation);
        cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_and_manual_share_capacity_but_never_alias_receipts_or_sequences() {
        let mut ledger = CompactOperations::new(7);
        assert_eq!(ledger.admit(7, 1), CompactAdmission::Accepted);
        assert_eq!(
            ledger.admit_from(7, CompactOrigin::Model, 1),
            CompactAdmission::Busy
        );
        assert!(!ledger.finish_from(7, CompactOrigin::Model, 1, CompactOutcome::Cancelled));
        assert!(ledger.finish(7, 1, CompactOutcome::Cancelled));
        assert_eq!(
            ledger.admit_from(7, CompactOrigin::Model, 1),
            CompactAdmission::Accepted
        );
        assert_eq!(ledger.admit(7, 2), CompactAdmission::Busy);
        assert!(!ledger.finish(7, 1, CompactOutcome::Failed));
        assert_eq!(
            ledger.observe_from(7, CompactOrigin::Model, 1),
            CompactAdmission::Pending
        );
        assert!(ledger.finish_from(7, CompactOrigin::Model, 1, CompactOutcome::Failed));
        assert_eq!(
            ledger.admit(7, 1),
            CompactAdmission::Completed(CompactOutcome::Cancelled)
        );
        assert_eq!(
            ledger.admit_from(7, CompactOrigin::Model, 1),
            CompactAdmission::Completed(CompactOutcome::Failed)
        );
        assert_eq!(
            ledger.admit_from(7, CompactOrigin::Model, u64::MAX),
            CompactAdmission::Accepted
        );
        assert!(ledger.finish_from(7, CompactOrigin::Model, u64::MAX, CompactOutcome::Failed));
        assert_eq!(ledger.admit(7, 2), CompactAdmission::Accepted);
    }

    #[test]
    fn mixed_generation_reset_retains_cancel_origin_and_shared_receipt_bound() {
        let mut ledger = CompactOperations::new(1);
        for id in 0..70 {
            let origin = if id % 2 == 0 {
                CompactOrigin::Manual
            } else {
                CompactOrigin::Model
            };
            assert_eq!(ledger.admit_from(1, origin, id), CompactAdmission::Accepted);
            assert!(ledger.finish_from(1, origin, id, CompactOutcome::Cancelled));
        }
        assert_eq!(ledger.receipts.len(), 64);
        assert_eq!(
            ledger.admit_from(1, CompactOrigin::Model, 1),
            CompactAdmission::Expired
        );
        assert_eq!(
            ledger.admit_from(1, CompactOrigin::Model, 70),
            CompactAdmission::Accepted
        );
        assert_eq!(
            ledger.replace_generation_with_origin(2),
            Some((1, CompactOrigin::Model, 70))
        );
        assert!(!ledger.finish_from(1, CompactOrigin::Model, 70, CompactOutcome::Failed));
        assert_eq!(
            ledger.admit_from(2, CompactOrigin::Model, 1),
            CompactAdmission::Accepted
        );
        assert_eq!(ledger.admit(2, 1), CompactAdmission::Busy);
    }

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
            assert!(ledger.finish(1, id, CompactOutcome::NotEligible { before: 0 }));
        }
        assert_eq!(ledger.receipts.len(), 64);
        assert_eq!(ledger.admit(1, 0), CompactAdmission::Expired);
        assert_eq!(
            ledger.admit(1, 1),
            CompactAdmission::Completed(CompactOutcome::NotEligible { before: 0 })
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
