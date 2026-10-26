//! Private assessment material retained by a prepared file mutation.

use super::PreparedPlanIdentity;

/// Maximum bytes in each captured before/after image admitted for assessment.
pub const MAX_FILE_MUTATION_REVIEW_BYTES: usize = 2 * 1024 * 1024;

/// Reasons complete file-change evidence cannot be retained for assessment.
#[derive(Debug, thiserror::Error)]
pub enum FileMutationEvidenceError {
    /// A complete before or after image exceeds the review limit.
    #[error("file mutation review image exceeds 2 MiB")]
    ImageLimit,
    /// A relative path cannot identify the retained target independently.
    #[error("file mutation review requires an absolute target")]
    RelativeTarget,
}

/// Exact text change captured after read admission by a file-tool implementation.
///
/// This material may contain private file contents. It is assessment input, not
/// display output, and must pass the evaluator's secret and total-payload guards.
#[derive(Clone, serde::Serialize, schemars::JsonSchema)]
pub struct PreparedFileMutationDescription {
    /// Actual target retained by the prepared operation.
    pub path: std::path::PathBuf,
    /// Complete original text, never a truncated preview.
    pub before: String,
    /// Complete replacement text; None means deletion of the original file.
    pub after: Option<String>,
}

impl std::fmt::Debug for PreparedFileMutationDescription {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedFileMutationDescription")
            .field("contents", &"<private>")
            .finish_non_exhaustive()
    }
}

/// Exact before/after evidence bound to a single retained preparation.
///
/// Constructing this value grants no authority and does not establish trusted
/// implementation ownership. Dispatch must independently verify registered
/// built-in ownership before offering this evidence for automatic review.
#[derive(Clone)]
pub struct PreparedFileMutationEvidence {
    identity: PreparedPlanIdentity,
    description: PreparedFileMutationDescription,
}

impl std::fmt::Debug for PreparedFileMutationEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedFileMutationEvidence")
            .field("contents", &"<private>")
            .finish_non_exhaustive()
    }
}

impl PreparedFileMutationEvidence {
    /// Binds complete bounded content to a retained plan without granting access.
    /// The producer must obtain read permission before capturing the original.
    pub fn new(
        identity: PreparedPlanIdentity,
        description: PreparedFileMutationDescription,
    ) -> Result<Self, FileMutationEvidenceError> {
        if description.before.len() > MAX_FILE_MUTATION_REVIEW_BYTES
            || description
                .after
                .as_ref()
                .is_some_and(|text| text.len() > MAX_FILE_MUTATION_REVIEW_BYTES)
        {
            return Err(FileMutationEvidenceError::ImageLimit);
        }
        if !description.path.is_absolute() {
            return Err(FileMutationEvidenceError::RelativeTarget);
        }
        Ok(Self {
            identity,
            description,
        })
    }

    /// Tests whether this evidence belongs to the exact plan being admitted.
    pub fn belongs_to(&self, identity: &PreparedPlanIdentity) -> bool {
        self.identity.same_plan(identity)
    }

    /// Returns complete private assessment input, not a human display projection.
    pub fn description(&self) -> &PreparedFileMutationDescription {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn description() -> PreparedFileMutationDescription {
        PreparedFileMutationDescription {
            path: std::env::temp_dir().join("private-review-target"),
            before: "private old content".into(),
            after: Some("private new content".into()),
        }
    }

    #[test]
    fn copied_content_does_not_rebind_the_plan_or_leak_through_debug() {
        let identity = PreparedPlanIdentity::new();
        let evidence = PreparedFileMutationEvidence::new(identity.clone(), description())
            .expect("bounded absolute file evidence");
        assert!(evidence.clone().belongs_to(&identity));
        assert!(!evidence.belongs_to(&PreparedPlanIdentity::new()));
        assert!(!format!("{evidence:?}").contains("private old content"));
        assert!(!format!("{:?}", evidence.description()).contains("private old content"));
        let projection = serde_json::to_value(evidence.description()).expect("assessment input");
        assert!(projection.get("identity").is_none());
    }

    #[test]
    fn neither_oversized_image_can_be_silently_truncated_for_review() {
        let mut original = description();
        original.before = "x".repeat(MAX_FILE_MUTATION_REVIEW_BYTES + 1);
        assert!(PreparedFileMutationEvidence::new(PreparedPlanIdentity::new(), original).is_err());
        let mut updated = description();
        updated.after = Some("x".repeat(MAX_FILE_MUTATION_REVIEW_BYTES + 1));
        assert!(PreparedFileMutationEvidence::new(PreparedPlanIdentity::new(), updated).is_err());
    }
}
