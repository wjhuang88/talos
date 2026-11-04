//! Talos Conversation Engine.
//!
//! Owns all conversation state and event processing, sitting between the
//! Agent Loop and the UI Loop. Communicates via typed async channels.
//!
//! # Support Boundary (experimental, product-oriented)
//!
//! This crate is currently Talos's product-level, UI-independent state and
//! projection layer. It is published on crates.io, but being published does
//! NOT constitute a general-purpose UI SDK support commitment. The API is
//! experimental pre-1.0, and external direct users should NOT assume
//! Talos-independent UI compatibility.
//!
//! Under [ADR-052](../../docs/decisions/052-sdk-publication-and-composition-boundary.md) a
//! general-purpose third-party UI SDK remains deferred; this crate is not
//! marketed as a supported UI framework. The existing conversation/TUI
//! layering (single-direction session → conversation → UI projection) is
//! preserved and must not be broken.
//!
//! Within that boundary, the pre-1.0 integration surface is:
//!
//! - typed conversation inputs and UI outputs are public integration surfaces;
//! - terminal rendering, keyboard handling, and visual layout are owned by `talos-tui`;
//! - provider execution and tool execution are owned by the runtime/agent layers;
//! - command registration is reusable, but built-in command semantics may still evolve before 1.0;
//! - consumers should expect additive events and fields while the external contract settles.

mod command_registry;
mod engine;
#[cfg(test)]
mod engine_tests;
mod governance_summary;
mod project_tools;
mod types;
mod validation;

pub use command_registry::{
    AvailabilityPredicate, CommandDefinition, CommandExecutionMode, CommandOrigin, CommandRegistry,
    always_available, command_registry,
};
pub use engine::{
    ConversationEngine, build_extension_snapshot, build_extension_snapshot_with_plugins,
};
pub use governance_summary::{
    GovernanceValidationReport, collect_governance_validation, format_governance_summary,
};
pub use project_tools::{agile_summary_available, project_validation_available};
pub use types::{
    ChatMessage, ConnectPickerData, ConnectPickerItem, ContentOutput, CopyScope,
    CredentialRequestData, CredentialResponseData, ExtensionSnapshot, HookDeclarationDiagnostic,
    HookSnapshot, LoadedPluginDiagnostic, McpServerDiagnostic, MessageRole, MessageSource,
    MessageStatus, ModelInfo, ModelPickerData, ModelPickerItem, ModelPickerVariantItem,
    ModelSwitchRequest, PluginObservation, ProviderSetupItem, ScrollbackState,
    SessionDeleteRequest, SessionForkRequest, SessionNewRequest, SessionPickerItem,
    SessionResumeRequest, SkillCommandRequest, SkillDiagnostic, StatusSnapshot, SteeringQueueEntry,
    SteeringQueueSnapshot, StreamMessage, TipKind, TodoCommandAction, TodoCommandRequest,
    TodoExportFormat, TodoPanelData, TodoPanelRow, ToolActivity, ToolCallDisplay, ToolCallInfo,
    ToolResultDisplay, TurnPhase, UiOutput, UserInput,
};
pub use validation::{
    EvidenceStatus, FindingSeverity, ProjectType, ProjectTypeDetectorDescriptor, ValidationCheck,
    ValidationEvidence, ValidationEvidenceRecord, ValidationExecutionMode, ValidationFinding,
    ValidationHostExecutor, ValidationHostFailure, ValidationHostFuture, ValidationHostOutput,
    ValidationPlan, ValidationProfile, collect_validation_plan, detect_project_types,
    project_type_detector_descriptors, render_json_evidence, render_json_plan,
    render_text_evidence, render_text_plan, run_governance_validation,
    run_validation_plan_with_executor,
};

/// Format content-free checkpoint facts for transient user-visible status.
pub fn format_compact_status(id: u64, state: talos_core::session::CompactState) -> String {
    use talos_core::session::{CompactOutcome, CompactState};
    let status = match state {
        CompactState::Queued => "queued until a safe boundary".into(),
        CompactState::Started => "started".into(),
        CompactState::Busy => "busy: another compaction is pending".into(),
        CompactState::Expired => "expired operation ID".into(),
        CompactState::StaleGeneration => "rejected: session generation changed".into(),
        CompactState::Completed { outcome, replayed } => {
            let result = match outcome {
                CompactOutcome::Applied { before, after } => {
                    format!("applied: estimated tokens {before} → {after}")
                }
                CompactOutcome::NotEligible { before } => {
                    format!("skipped: estimated tokens {before}, below threshold")
                }
                CompactOutcome::Failed => {
                    "failed during preparation: estimate unavailable; prior context retained".into()
                }
                CompactOutcome::FailedWithEstimate { before, reason } => {
                    use talos_core::session::CompactFailureReason;
                    let reason = match reason {
                        CompactFailureReason::Preparation => "preparation or validation rejected",
                        CompactFailureReason::Provider => "provider failure",
                        CompactFailureReason::Timeout => "deadline exceeded",
                        CompactFailureReason::CommitRejected => "candidate installation rejected",
                    };
                    format!("failed ({reason}): estimated tokens {before}; prior context retained")
                }
                CompactOutcome::Cancelled => "cancelled: prior context retained".into(),
                CompactOutcome::CancelledWithEstimate { before } => {
                    format!("cancelled: prior context retained; estimated tokens {before}")
                }
            };
            if replayed {
                format!("{result} (previous result)")
            } else {
                result
            }
        }
    };
    format!("Compaction {id}: {status}")
}

/// Format automatic checkpoint metadata without revealing derived context.
pub fn format_automatic_compact_status(
    state: talos_core::session::AutomaticCompactState,
) -> String {
    use talos_core::session::AutomaticCompactState;
    let status = match state {
        AutomaticCompactState::Started { before } => format!("started: estimated tokens {before}"),
        AutomaticCompactState::Applied { before, after } => {
            format!("applied: estimated tokens {before} → {after}")
        }
        AutomaticCompactState::SkippedSourceUnavailable { before } => {
            format!("skipped: no verified eligible source; estimated tokens {before}")
        }
        AutomaticCompactState::Failed { before, reason } => {
            use talos_core::session::AutomaticCompactFailure;
            let reason = match reason {
                AutomaticCompactFailure::Provider => "provider failure",
                AutomaticCompactFailure::Preparation => "preparation or validation rejected",
                AutomaticCompactFailure::CommitRejected => "candidate installation rejected",
            };
            format!("failed ({reason}): prior context retained; estimated tokens {before}")
        }
        AutomaticCompactState::Cancelled { before } => {
            format!("cancelled: prior context retained; estimated tokens {before}")
        }
    };
    format!("Automatic compaction {status}")
}

#[cfg(test)]
mod compact_status_tests {
    #[test]
    fn manual_failure_distinguishes_measured_request_from_unavailable_estimate() {
        use talos_core::session::{CompactFailureReason, CompactOutcome, CompactState};
        let render = |outcome| {
            super::format_compact_status(
                7,
                CompactState::Completed {
                    outcome,
                    replayed: false,
                },
            )
        };
        let unknown = render(CompactOutcome::Failed);
        assert!(unknown.contains("estimate unavailable"));
        assert!(!unknown.contains("tokens 0"));
        let measured = render(CompactOutcome::FailedWithEstimate {
            before: 24_001,
            reason: CompactFailureReason::Timeout,
        });
        assert!(measured.contains("deadline exceeded"));
        assert!(measured.contains("estimated tokens 24001"));
        assert!(measured.contains("prior context retained"));
    }
}
