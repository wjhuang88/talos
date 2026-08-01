//! Structured transactional Session submissions (ADR-056 / TUI-044).

use serde::{Deserialize, Serialize};

use crate::message::ContentPart;

/// Maximum UTF-8 bytes accepted for one structured submission item.
pub const MAX_SUBMISSION_ITEM_BYTES: usize = 64 * 1024;
/// Maximum items retained in one interactive steering queue.
pub const MAX_STEERING_QUEUE_ITEMS: usize = 128;
/// Maximum UTF-8 text bytes retained in one interactive steering queue.
pub const MAX_STEERING_QUEUE_BYTES: usize = 1024 * 1024;
/// Maximum image attachments owned across running and pending work.
pub const MAX_STEERING_QUEUE_IMAGES: usize = 16;
/// Maximum declared image bytes owned across running and pending work.
pub const MAX_STEERING_QUEUE_IMAGE_BYTES: u64 = 100 * 1024 * 1024;
/// Maximum compatible items projected into one Actor Turn.
pub const MAX_SUBMISSION_BATCH_ITEMS: usize = 32;
/// Maximum UTF-8 text bytes projected into one Actor Turn.
pub const MAX_SUBMISSION_BATCH_BYTES: usize = 256 * 1024;
/// Maximum metadata bytes retained for one attachment.
pub const MAX_SUBMISSION_ATTACHMENT_METADATA_BYTES: usize = 16 * 1024;
/// Maximum metadata bytes retained across one structured submission.
pub const MAX_SUBMISSION_TOTAL_ATTACHMENT_METADATA_BYTES: usize = 64 * 1024;
/// Maximum image attachments accepted in one structured submission.
pub const MAX_SUBMISSION_IMAGE_COUNT: usize = 4;
/// Maximum declared bytes for one image attachment.
pub const MAX_SUBMISSION_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
/// Maximum declared image bytes across one structured submission.
pub const MAX_SUBMISSION_TOTAL_IMAGE_BYTES: u64 = 50 * 1024 * 1024;
/// Maximum durable pending submissions retained for one Session.
pub const MAX_PENDING_SUBMISSIONS: usize = 128;
/// Maximum durable pending text bytes retained for one Session.
pub const MAX_PENDING_SUBMISSION_BYTES: usize = 1024 * 1024;

/// Origin of a structured Session submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionSource {
    /// Interactive user input accepted by a product bridge.
    User,
    /// A scheduled follow-up produced by the Session scheduler.
    Scheduler,
    /// A legacy or external caller using a compatibility operation.
    Compatibility,
}

/// Dispatch semantics fixed before an item enters the authoritative queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionKind {
    /// A normal model-visible user Turn.
    UserTurn,
    /// A request-preview diagnostic that must not call the Provider.
    PreviewRequest,
}

/// One recoverable input item inside a structured submission.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmissionItem {
    /// Opaque producer-assigned item identity.
    pub id: String,
    /// Monotonic producer-side FIFO order.
    pub enqueue_sequence: u64,
    /// Dispatch kind fixed before queue admission.
    pub kind: SubmissionKind,
    /// Original text without delimiter rewriting.
    pub text: String,
    /// Attachments bound to this exact item before queue admission.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<ContentPart>,
}

impl SubmissionItem {
    /// Returns the original UTF-8 text size.
    #[must_use]
    pub fn text_bytes(&self) -> usize {
        self.text.len()
    }
}

/// An immutable compatible queue prefix prepared for one Actor Turn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredSubmission {
    /// Stable identity shared by preparation, retries, receipt and reconciliation.
    pub id: String,
    /// Source used by Actor arbitration.
    pub source: SubmissionSource,
    /// Runtime generation of the addressed logical Session.
    #[serde(default)]
    pub sender_generation: u64,
    /// Ordered, homogeneous, individually recoverable items.
    pub items: Vec<SubmissionItem>,
}

impl StructuredSubmission {
    /// Returns aggregate UTF-8 text bytes using saturating accounting.
    #[must_use]
    pub fn total_text_bytes(&self) -> usize {
        self.items
            .iter()
            .fold(0usize, |total, item| total.saturating_add(item.text_bytes()))
    }

    /// Returns attachment count and declared bytes using saturating accounting.
    #[must_use]
    pub fn image_totals(&self) -> (usize, u64) {
        self.items
            .iter()
            .flat_map(|item| &item.attachments)
            .fold((0usize, 0u64), |(count, bytes), part| match part {
                ContentPart::Image { byte_count, .. } => (
                    count.saturating_add(1),
                    bytes.saturating_add(*byte_count),
                ),
                ContentPart::Text { .. } => (count, bytes),
            })
    }

    /// Returns total attachment metadata bytes or `None` on arithmetic overflow.
    #[must_use]
    pub fn attachment_metadata_bytes(&self) -> Option<usize> {
        self.items
            .iter()
            .flat_map(|item| &item.attachments)
            .try_fold(0usize, |total, part| match part {
                ContentPart::Image {
                    path, mime, ..
                } => total
                    .checked_add(path.as_os_str().to_string_lossy().len())?
                    .checked_add(mime.len())?
                    .checked_add(std::mem::size_of::<u64>())?
                    .checked_add(32),
                ContentPart::Text { .. } => None,
            })
    }

    /// Returns the common dispatch kind for a non-empty homogeneous batch.
    #[must_use]
    pub fn common_kind(&self) -> Option<SubmissionKind> {
        let first = self.items.first()?.kind;
        self.items
            .iter()
            .all(|item| item.kind == first)
            .then_some(first)
    }

    /// Validates immutable identity, ordering, compatibility and hard bounds.
    pub fn validate(&self) -> Result<(), SubmissionRejectionReason> {
        if self.id.is_empty()
            || self.items.is_empty()
            || self.items.len() > MAX_SUBMISSION_BATCH_ITEMS
            || self.common_kind().is_none()
        {
            return Err(SubmissionRejectionReason::InvalidStructure);
        }

        let mut previous_sequence = None;
        let mut item_ids = std::collections::HashSet::with_capacity(self.items.len());
        for item in &self.items {
            if item.id.is_empty() {
                return Err(SubmissionRejectionReason::InvalidStructure);
            }
            if !item_ids.insert(item.id.as_str()) {
                return Err(SubmissionRejectionReason::Duplicate);
            }
            if item.text_bytes() > MAX_SUBMISSION_ITEM_BYTES {
                return Err(SubmissionRejectionReason::LimitExceeded);
            }
            if previous_sequence.is_some_and(|previous| item.enqueue_sequence <= previous) {
                return Err(SubmissionRejectionReason::InvalidStructure);
            }
            previous_sequence = Some(item.enqueue_sequence);
        }

        if self.total_text_bytes() > MAX_SUBMISSION_BATCH_BYTES {
            return Err(SubmissionRejectionReason::LimitExceeded);
        }
        if self.common_kind() == Some(SubmissionKind::PreviewRequest)
            && (self.items.len() != 1 || !self.items[0].attachments.is_empty())
        {
            return Err(SubmissionRejectionReason::InvalidStructure);
        }

        let (image_count, image_bytes) = self.image_totals();
        let metadata_bytes = self
            .attachment_metadata_bytes()
            .ok_or(SubmissionRejectionReason::InvalidStructure)?;
        let oversized_attachment = self
            .items
            .iter()
            .flat_map(|item| &item.attachments)
            .any(|part| match part {
                ContentPart::Image {
                    path,
                    mime,
                    byte_count,
                    ..
                } => {
                    let metadata = path
                        .as_os_str()
                        .to_string_lossy()
                        .len()
                        .saturating_add(mime.len())
                        .saturating_add(std::mem::size_of::<u64>())
                        .saturating_add(32);
                    *byte_count > MAX_SUBMISSION_IMAGE_BYTES
                        || metadata > MAX_SUBMISSION_ATTACHMENT_METADATA_BYTES
                }
                ContentPart::Text { .. } => true,
            });
        if image_count > MAX_SUBMISSION_IMAGE_COUNT
            || image_bytes > MAX_SUBMISSION_TOTAL_IMAGE_BYTES
            || metadata_bytes > MAX_SUBMISSION_TOTAL_ATTACHMENT_METADATA_BYTES
            || oversized_attachment
        {
            return Err(SubmissionRejectionReason::LimitExceeded);
        }

        Ok(())
    }
}

/// Durable state of an Actor-owned pending submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingSubmissionState {
    /// Accepted durably but not started.
    AcceptedPending,
    /// Correlated to an active model Turn.
    Running,
    /// Retained but automatic advancement is paused.
    PausedPending,
    /// The started Turn ended by explicit cancellation.
    TerminalCancelled,
    /// The started Turn ended with an error.
    TerminalError,
    /// Successful transcript commit completed.
    Committed,
}

/// Content-free reason a structured submission was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionRejectionReason {
    /// Durable pending storage is unavailable for this runtime.
    DurabilityUnavailable,
    /// The addressed Session is not current.
    WrongSession,
    /// The addressed Session generation is stale or unknown.
    WrongGeneration,
    /// The identity already exists with different immutable content.
    IdentityConflict,
    /// The exact identity was accepted already.
    Duplicate,
    /// The pending submission was explicitly cancelled before start.
    Cancelled,
    /// The submission is empty or mixes incompatible kinds.
    InvalidStructure,
    /// An item, batch, attachment, queue, or journal bound was exceeded.
    LimitExceeded,
    /// The complete Provider request would exceed its context budget.
    ContextBudgetExceeded,
    /// The Actor is shutting down or no longer accepts work.
    SessionClosed,
}

/// Result of durable Actor acceptance or reconciliation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SubmissionReceiptDisposition {
    /// The journal accepted this exact submission for the first time.
    AcceptedPending,
    /// The same immutable submission was accepted previously.
    AlreadyAccepted {
        /// Current durable state.
        state: PendingSubmissionState,
        /// Turn identity when the submission has started.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        turn_id: Option<String>,
    },
    /// The authoritative generation confirms no matching submission exists.
    NotAccepted,
    /// Admission failed before ownership transferred.
    Rejected {
        /// Bounded, content-free reason.
        reason: SubmissionRejectionReason,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn submission() -> StructuredSubmission {
        StructuredSubmission {
            id: "batch-1".into(),
            source: SubmissionSource::User,
            sender_generation: 1,
            items: vec![SubmissionItem {
                id: "item-1".into(),
                enqueue_sequence: 1,
                kind: SubmissionKind::UserTurn,
                text: "hello".into(),
                attachments: Vec::new(),
            }],
        }
    }

    #[test]
    fn valid_submission_preserves_item_boundaries() {
        let mut value = submission();
        value.items.push(SubmissionItem {
            id: "item-2".into(),
            enqueue_sequence: 2,
            kind: SubmissionKind::UserTurn,
            text: "world".into(),
            attachments: Vec::new(),
        });
        assert_eq!(value.validate(), Ok(()));
        let encoded = serde_json::to_string(&value).unwrap();
        let decoded: StructuredSubmission = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.items.len(), 2);
        assert_eq!(decoded.items[0].text, "hello");
        assert_eq!(decoded.items[1].text, "world");
    }

    #[test]
    fn duplicate_item_identity_fails_closed() {
        let mut value = submission();
        value.items.push(SubmissionItem {
            id: "item-1".into(),
            enqueue_sequence: 2,
            kind: SubmissionKind::UserTurn,
            text: "again".into(),
            attachments: Vec::new(),
        });
        assert_eq!(value.validate(), Err(SubmissionRejectionReason::Duplicate));
    }

    #[test]
    fn incompatible_kinds_and_regressive_sequence_fail_closed() {
        let mut value = submission();
        value.items.push(SubmissionItem {
            id: "item-2".into(),
            enqueue_sequence: 1,
            kind: SubmissionKind::PreviewRequest,
            text: "preview".into(),
            attachments: Vec::new(),
        });
        assert_eq!(
            value.validate(),
            Err(SubmissionRejectionReason::InvalidStructure)
        );
    }
}
