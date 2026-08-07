use std::collections::VecDeque;

use tokio::sync::mpsc;

use talos_core::session::{
    PendingSubmissionState, SessionEvent, StructuredSubmission, SubmissionReceipt,
    SubmissionReceiptDisposition, SubmissionRejectionReason, SubmissionSource,
    TurnCompletionStatus, TurnEventPayload,
};
use talos_session::PendingSubmissionError;

use super::{
    ActiveStructuredTurn, AppServerSession, MAX_RECENT_ITEM_IDS, MAX_RECENT_SUBMISSION_IDS,
    enqueue_by_source, queue_has_capacity, queue_totals_after, record_recent_identity,
    validate_submission,
};

impl AppServerSession {
    pub(super) fn reconcile_running_submissions(&self) {
        let records = match self.pending_store.recover_running() {
            Ok(records) => records,
            Err(error) => {
                self.emit_custody_error("failed to inspect running submissions", &error);
                return;
            }
        };
        for record in records {
            let Some(turn_id) = record.turn_id.as_deref() else {
                self.emit_custody_error(
                    "running submission has no Turn identity",
                    &record.submission.id,
                );
                continue;
            };
            match self
                .pending_store
                .mark_committed(&record.submission.id, turn_id)
            {
                Ok(()) => {}
                Err(PendingSubmissionError::InvalidTransition) => {
                    let _ = self.eq_tx.send(SessionEvent::Error {
                        message: format!(
                            "submission {} remains frozen in Running state because transcript outcome for Turn {turn_id} is ambiguous",
                            record.submission.id
                        ),
                    });
                }
                Err(error) => self.emit_custody_error(
                    "failed to reconcile Running submission with transcript outcome",
                    &error,
                ),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn restore_pending_submissions(
        &self,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
    ) -> bool {
        let records = match self.pending_store.recover_unstarted() {
            Ok(records) => records,
            Err(error) => {
                self.emit_custody_error("failed to recover pending submissions", &error);
                return true;
            }
        };
        let mut paused = false;
        for record in records {
            let submission = record.submission;
            if submission.sender_generation != self.session_generation {
                let _ = self.eq_tx.send(SessionEvent::Error {
                    message: format!(
                        "pending submission {} belongs to Session generation {} and remains frozen; current generation is {}",
                        submission.id, submission.sender_generation, self.session_generation
                    ),
                });
                continue;
            }
            if let Err(reason) = validate_submission(&submission) {
                let _ = self.eq_tx.send(SessionEvent::Error {
                    message: format!(
                        "pending submission {} failed validation during recovery: {reason:?}",
                        submission.id
                    ),
                });
                continue;
            }
            paused = true;
            let (images, image_bytes) = submission.image_totals();
            *pending_items = pending_items.saturating_add(submission.items.len());
            *pending_bytes = pending_bytes.saturating_add(submission.total_text_bytes());
            *pending_images = pending_images.saturating_add(images);
            *pending_image_bytes = pending_image_bytes.saturating_add(image_bytes);
            record_recent_identity(
                recent_submission_ids,
                submission.id.clone(),
                MAX_RECENT_SUBMISSION_IDS,
            );
            for item in &submission.items {
                record_recent_identity(recent_item_ids, item.id.clone(), MAX_RECENT_ITEM_IDS);
            }
            enqueue_by_source(pending, submission);
        }
        paused
    }

    pub(super) fn release_in_memory_pending_on_shutdown(
        &self,
        pending: &mut VecDeque<StructuredSubmission>,
    ) {
        for submission in pending.drain(..) {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::SessionClosed,
            );
        }
    }

    pub(super) fn finish_structured_turn(
        &self,
        active: &ActiveStructuredTurn,
        completion: &TurnCompletionStatus,
    ) -> bool {
        let transition = match completion {
            TurnCompletionStatus::Success { .. } => self
                .pending_store
                .mark_committed(&active.submission_id, &active.turn_id),
            TurnCompletionStatus::Cancelled => self.pending_store.mark_terminal(
                &active.submission_id,
                PendingSubmissionState::TerminalCancelled,
                &active.turn_id,
            ),
            TurnCompletionStatus::Error { .. } => self.pending_store.mark_terminal(
                &active.submission_id,
                PendingSubmissionState::TerminalError,
                &active.turn_id,
            ),
        };
        if let Err(error) = transition {
            self.emit_custody_error("failed to finalize structured turn custody", &error);
            return false;
        }
        if !matches!(completion, TurnCompletionStatus::Success { .. })
            && let Err(error) = self.pending_store.pause_unstarted()
        {
            self.emit_custody_error("failed to pause unstarted work after terminal turn", &error);
            return false;
        }
        let _ = self.eq_tx.send(SessionEvent::StructuredTurnEvent {
            session_id: self.session_id.clone(),
            session_generation: active.session_generation,
            source: active.source,
            submission_id: active.submission_id.clone(),
            receipt_id: active.receipt_id.clone(),
            turn_id: active.turn_id.clone(),
            sequence: 1,
            payload: TurnEventPayload::Completed {
                status: completion.clone(),
            },
        });
        true
    }

    pub(super) fn emit_custody_error(&self, context: &str, error: &impl std::fmt::Display) {
        let _ = self.eq_tx.send(SessionEvent::Error {
            message: format!("{context}: {error}"),
        });
    }

    pub(super) fn validate_current_generation(
        &self,
        submission: &StructuredSubmission,
    ) -> Result<(), SubmissionRejectionReason> {
        if submission.sender_generation != self.session_generation {
            return Err(SubmissionRejectionReason::WrongGeneration);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn accept_durable_submission(
        &self,
        submission: StructuredSubmission,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) -> bool {
        if let Err(reason) = self.validate_current_generation(&submission) {
            self.emit_admission_rejection(&submission, reason, receipt_tx);
            return false;
        }
        let (existing_receipt, existing_disposition) =
            match self.pending_store.reconcile(&submission) {
                Ok(result) => result,
                Err(error) => {
                    self.emit_durability_rejection(&submission, &error, receipt_tx);
                    return false;
                }
            };
        match existing_disposition {
            SubmissionReceiptDisposition::AlreadyAccepted { state, turn_id } => {
                self.emit_submission_receipt(
                    &submission,
                    existing_receipt,
                    SubmissionReceiptDisposition::AlreadyAccepted { state, turn_id },
                    receipt_tx,
                );
                return false;
            }
            SubmissionReceiptDisposition::Rejected { reason } => {
                self.emit_submission_receipt(
                    &submission,
                    existing_receipt,
                    SubmissionReceiptDisposition::Rejected { reason },
                    receipt_tx,
                );
                self.reject_submission(&submission.id, submission.sender_generation, reason);
                return false;
            }
            SubmissionReceiptDisposition::NotAccepted => {}
            SubmissionReceiptDisposition::AcceptedPending => {
                self.emit_custody_error(
                    "structured reconciliation returned an invalid first-accept state",
                    &submission.id,
                );
                return false;
            }
        }

        if let Err(reason) = validate_submission(&submission) {
            self.emit_admission_rejection(&submission, reason, receipt_tx);
            return false;
        }
        if submission
            .items
            .iter()
            .any(|item| recent_item_ids.contains(&item.id))
        {
            self.emit_admission_rejection(
                &submission,
                SubmissionRejectionReason::Duplicate,
                receipt_tx,
            );
            return false;
        }
        if !queue_has_capacity(
            &submission,
            *pending_items,
            *pending_bytes,
            *pending_images,
            *pending_image_bytes,
        ) {
            self.emit_admission_rejection(
                &submission,
                SubmissionRejectionReason::LimitExceeded,
                receipt_tx,
            );
            return false;
        }

        let (receipt_id, disposition) = match self.pending_store.accept(&submission) {
            Ok(result) => result,
            Err(error) => {
                self.emit_durability_rejection(&submission, &error, receipt_tx);
                return false;
            }
        };
        self.emit_submission_receipt(&submission, receipt_id, disposition.clone(), receipt_tx);
        match disposition {
            SubmissionReceiptDisposition::AcceptedPending => self.accept_submission(
                submission,
                pending,
                pending_items,
                pending_bytes,
                pending_images,
                pending_image_bytes,
                recent_submission_ids,
                recent_item_ids,
            ),
            SubmissionReceiptDisposition::AlreadyAccepted { .. }
            | SubmissionReceiptDisposition::NotAccepted => false,
            SubmissionReceiptDisposition::Rejected { reason } => {
                self.reject_submission(&submission.id, submission.sender_generation, reason);
                false
            }
        }
    }

    pub(super) fn reconcile_submission(
        &self,
        submission: &StructuredSubmission,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        let (receipt_id, disposition) = match self.pending_store.reconcile(submission) {
            Ok(result) => result,
            Err(error) => {
                self.emit_durability_rejection(submission, &error, receipt_tx);
                return;
            }
        };
        self.emit_submission_receipt(submission, receipt_id, disposition, receipt_tx);
    }

    pub(super) fn emit_admission_rejection(
        &self,
        submission: &StructuredSubmission,
        reason: SubmissionRejectionReason,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        self.emit_submission_receipt(
            submission,
            String::new(),
            SubmissionReceiptDisposition::Rejected { reason },
            receipt_tx,
        );
        self.reject_submission(&submission.id, submission.sender_generation, reason);
    }

    pub(super) fn emit_durability_rejection(
        &self,
        submission: &StructuredSubmission,
        error: &impl std::fmt::Display,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        self.emit_admission_rejection(
            submission,
            SubmissionRejectionReason::DurabilityUnavailable,
            receipt_tx,
        );
        self.emit_custody_error("structured submission durability failed", error);
    }

    pub(super) fn emit_submission_receipt(
        &self,
        submission: &StructuredSubmission,
        receipt_id: String,
        disposition: SubmissionReceiptDisposition,
        receipt_tx: Option<&mpsc::UnboundedSender<SubmissionReceipt>>,
    ) {
        let reservation_id = format!("reservation:{}", submission.id);
        let receipt = SubmissionReceipt {
            session_id: self.session_id.clone(),
            session_generation: submission.sender_generation,
            submission_id: submission.id.clone(),
            reservation_id,
            receipt_id,
            source: submission.source,
            item_count: submission.items.len(),
            total_text_bytes: submission.total_text_bytes(),
            disposition,
        };
        let _ = self.eq_tx.send(SessionEvent::SubmissionReceipt {
            session_id: receipt.session_id.clone(),
            session_generation: receipt.session_generation,
            submission_id: receipt.submission_id.clone(),
            reservation_id: receipt.reservation_id.clone(),
            receipt_id: receipt.receipt_id.clone(),
            source: receipt.source,
            item_count: receipt.item_count,
            total_text_bytes: receipt.total_text_bytes,
            disposition: receipt.disposition.clone(),
        });
        if let Some(receipt_tx) = receipt_tx {
            let _ = receipt_tx.send(receipt);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn accept_submission(
        &self,
        submission: StructuredSubmission,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
        recent_submission_ids: &mut VecDeque<String>,
        recent_item_ids: &mut VecDeque<String>,
    ) -> bool {
        if let Err(reason) = validate_submission(&submission) {
            self.reject_submission(&submission.id, submission.sender_generation, reason);
            return false;
        }
        if recent_submission_ids.contains(&submission.id)
            || submission
                .items
                .iter()
                .any(|item| recent_item_ids.contains(&item.id))
        {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::Duplicate,
            );
            return false;
        }
        let Some((next_items, next_bytes, next_images, next_image_bytes)) = queue_totals_after(
            &submission,
            *pending_items,
            *pending_bytes,
            *pending_images,
            *pending_image_bytes,
        ) else {
            self.reject_submission(
                &submission.id,
                submission.sender_generation,
                SubmissionRejectionReason::LimitExceeded,
            );
            return false;
        };
        let _ = self.eq_tx.send(SessionEvent::SubmissionQueued {
            session_id: self.session_id.clone(),
            submission_id: submission.id.clone(),
            sender_generation: submission.sender_generation,
            source: submission.source,
            item_count: submission.items.len(),
            total_text_bytes: submission.total_text_bytes(),
        });

        record_recent_identity(
            recent_submission_ids,
            submission.id.clone(),
            MAX_RECENT_SUBMISSION_IDS,
        );
        for item in &submission.items {
            record_recent_identity(recent_item_ids, item.id.clone(), MAX_RECENT_ITEM_IDS);
        }
        *pending_items = next_items;
        *pending_bytes = next_bytes;
        *pending_images = next_images;
        *pending_image_bytes = next_image_bytes;
        enqueue_by_source(pending, submission);
        true
    }

    pub(super) fn reject_submission(
        &self,
        submission_id: &str,
        sender_generation: u64,
        reason: SubmissionRejectionReason,
    ) {
        let _ = self.eq_tx.send(SessionEvent::SubmissionRejected {
            session_id: self.session_id.clone(),
            submission_id: submission_id.to_owned(),
            sender_generation,
            reason,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn cancel_paused_submission(
        &self,
        submission_id: &str,
        pending: &mut VecDeque<StructuredSubmission>,
        pending_items: &mut usize,
        pending_bytes: &mut usize,
        pending_images: &mut usize,
        pending_image_bytes: &mut u64,
    ) -> bool {
        let Some(front) = pending.front() else {
            return false;
        };
        if front.id != submission_id || front.sender_generation != self.session_generation {
            return false;
        }
        let record = match self.pending_store.get(submission_id) {
            Ok(Some(record)) if record.state == PendingSubmissionState::PausedPending => record,
            Ok(_) => return false,
            Err(error) => {
                self.emit_custody_error("failed to inspect paused submission", &error);
                return false;
            }
        };
        if let Err(error) = self.pending_store.cancel_unstarted(submission_id) {
            self.emit_custody_error("failed to terminalize paused submission", &error);
            return false;
        }
        let Some(submission) = pending.pop_front() else {
            return false;
        };
        let (images, image_bytes) = submission.image_totals();
        *pending_items = pending_items.saturating_sub(submission.items.len());
        *pending_bytes = pending_bytes.saturating_sub(submission.total_text_bytes());
        *pending_images = pending_images.saturating_sub(images);
        *pending_image_bytes = pending_image_bytes.saturating_sub(image_bytes);
        let _ = self.eq_tx.send(SessionEvent::SubmissionResolved {
            session_id: self.session_id.clone(),
            session_generation: self.session_generation,
            submission_id: submission_id.to_owned(),
            receipt_id: record.receipt_id,
            state: PendingSubmissionState::TerminalCancelled,
        });
        true
    }

    pub(super) fn pause_before_start(
        &self,
        submission: &StructuredSubmission,
        reason: SubmissionRejectionReason,
    ) {
        if submission.source == SubmissionSource::Compatibility {
            self.reject_submission(&submission.id, submission.sender_generation, reason);
            return;
        }
        if let Err(error) = self.pending_store.mark_paused(&submission.id) {
            self.emit_custody_error("failed to pause accepted pre-start submission", &error);
            return;
        }
        let receipt_id = match self.pending_store.get(&submission.id) {
            Ok(Some(record)) => record.receipt_id,
            Ok(None) => String::new(),
            Err(error) => {
                self.emit_custody_error("failed to reload paused pre-start submission", &error);
                String::new()
            }
        };
        let _ = self.eq_tx.send(SessionEvent::SubmissionPaused {
            session_id: self.session_id.clone(),
            session_generation: self.session_generation,
            submission_id: submission.id.clone(),
            receipt_id,
            reason,
        });
    }
}
