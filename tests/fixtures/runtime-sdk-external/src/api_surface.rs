//! Compile the less common canonical signatures without another Talos dependency.

use async_trait::async_trait;
use serde_json::Value;
use talos_runtime::{
    AgentError, AgentTool, ApprovalChoice, ApprovalHandler, BackgroundCleanupOutcome,
    BackgroundJobId, BackgroundJobLauncher, BackgroundJobPermit, BackgroundJobRequest,
    BackgroundJobState, BackgroundJobTerminalSummary, BackgroundOutputChunk,
    BackgroundOutputStream, BackgroundProcessControl, BackgroundProcessEvent,
    BackgroundProcessExit, GrantPreview, GrantPreviewFacet, GrantScope, LaunchedBackgroundJob,
    PendingSubmissionState, ProviderError, RuntimeError, SessionError, SessionEvent,
    StructuredSubmission, SubmissionItem, SubmissionKind, SubmissionReceiptDisposition,
    SubmissionRejectionReason, SubmissionSource, ToolContinuation, ToolExecutionAdmission,
    ToolExecutionAuthorization, ToolExecutionOutput, ToolNature, ToolResourceKind, ToolResult,
};

struct SurfaceTool;

#[async_trait]
impl AgentTool for SurfaceTool {
    fn name(&self) -> &str {
        "surface_only"
    }
    fn description(&self) -> &str {
        "Signature coverage; never launches a process"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type": "object"})
    }

    fn execution_admission(&self, _: &Value) -> Result<ToolExecutionAdmission, String> {
        Ok(ToolExecutionAdmission::Foreground)
    }

    async fn execute(&self, _: Value) -> ToolResult {
        ToolResult::error("signature fixture is not an execution tool")
    }

    async fn execute_background_authorized_with_output(
        &self,
        _: Value,
        _permit: Box<dyn BackgroundJobPermit>,
        _: &[ToolExecutionAuthorization],
    ) -> ToolExecutionOutput {
        ToolExecutionOutput::error("background execution deliberately unsupported")
    }

    async fn execute_authorized_with_output(
        &self,
        _: Value,
        _: &[ToolExecutionAuthorization],
    ) -> ToolExecutionOutput {
        ToolExecutionOutput::from_result(continuation_result())
    }
}

fn continuation_result() -> ToolResult {
    let continuation: ToolContinuation = ToolContinuation::disclose_tool("surface_only", "fixture");
    ToolResult {
        content: "fixture".into(),
        is_error: false,
        continuations: vec![continuation],
    }
}

struct ScopedApproval;

#[async_trait]
impl ApprovalHandler for ScopedApproval {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        ApprovalChoice::Deny
    }

    async fn request_scoped_approval(
        &self,
        _: &str,
        _: &Value,
        _: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        let _: GrantScope = preview.scope();
        let facets: &[GrantPreviewFacet] = preview.facets();
        for facet in facets {
            let _: ToolNature = facet.nature;
            let _: ToolResourceKind = facet.resource_kind;
        }
        ApprovalChoice::Deny
    }
}

// These signatures cover the background permit's recursive trait closure without
// inventing a runtime supervisor or launching an operating-system process.
fn background_signature(
    _launcher: Option<Box<dyn BackgroundJobLauncher>>,
    _control: Option<std::sync::Arc<dyn BackgroundProcessControl>>,
    _launched: Option<LaunchedBackgroundJob>,
) {
    let output = BackgroundProcessEvent::Output(BackgroundOutputChunk {
        stream: BackgroundOutputStream::Stdout,
        bytes: vec![],
        captured_at: std::time::SystemTime::UNIX_EPOCH,
    });
    let exit = BackgroundProcessEvent::Exited(BackgroundProcessExit {
        code: Some(0),
        success: true,
    });
    assert!(matches!(output, BackgroundProcessEvent::Output(_)));
    assert!(matches!(exit, BackgroundProcessEvent::Exited(_)));
}

fn inspect_event(event: SessionEvent) {
    match event {
        SessionEvent::StructuredSubmissionStarted { submission, .. }
        | SessionEvent::StructuredSubmissionInjected { submission, .. } => {
            let submission: StructuredSubmission = submission;
            let _: SubmissionSource = submission.source;
            for item in submission.items {
                let item: SubmissionItem = item;
                let _: SubmissionKind = item.kind;
            }
        }
        SessionEvent::SubmissionReceipt { disposition, .. } => {
            let disposition: SubmissionReceiptDisposition = disposition;
            match disposition {
                SubmissionReceiptDisposition::AlreadyAccepted { state, .. } => {
                    let _: PendingSubmissionState = state;
                }
                SubmissionReceiptDisposition::Rejected { reason } => {
                    let _: SubmissionRejectionReason = reason;
                }
                _ => {}
            }
        }
        SessionEvent::BackgroundJobTerminal { summary, .. } => {
            let summary: BackgroundJobTerminalSummary = summary;
            let _: BackgroundJobId = summary.job_id;
            let state: BackgroundJobState = summary.state;
            let cleanup: BackgroundCleanupOutcome = summary.cleanup_outcome;
            assert!(state.is_terminal());
            assert!(matches!(cleanup, BackgroundCleanupOutcome::Natural));
        }
        _ => {}
    }
}

fn inspect_error(error: RuntimeError) {
    match error {
        RuntimeError::Agent(AgentError::ProviderError(provider)) => {
            let _: ProviderError = provider;
        }
        RuntimeError::Session(session) => {
            let session: SessionError = session;
            assert!(matches!(session, SessionError::IoError(_)));
        }
        _ => panic!("unexpected fixture error"),
    }
}

pub fn verify() {
    let tool: &dyn AgentTool = &SurfaceTool;
    assert!(matches!(
        tool.execution_admission(&Value::Null),
        Ok(ToolExecutionAdmission::Foreground)
    ));
    let _: &dyn ApprovalHandler = &ScopedApproval;
    let admission = ToolExecutionAdmission::Background(BackgroundJobRequest {
        tool_name: "surface_only".into(),
        timeout: std::time::Duration::from_secs(1),
    });
    assert!(matches!(admission, ToolExecutionAdmission::Background(_)));
    assert_eq!(continuation_result().continuations.len(), 1);
    background_signature(None, None, None);
    inspect_event(SessionEvent::StructuredSubmissionStarted {
        session_id: "fixture".into(),
        session_generation: 1,
        submission: StructuredSubmission {
            id: "submission".into(),
            source: SubmissionSource::Compatibility,
            sender_generation: 1,
            items: vec![SubmissionItem {
                id: "item".into(),
                enqueue_sequence: 0,
                kind: SubmissionKind::UserTurn,
                text: "fixture".into(),
                attachments: vec![],
            }],
        },
        receipt_id: "receipt".into(),
        turn_id: "turn".into(),
    });
    for disposition in [
        SubmissionReceiptDisposition::AlreadyAccepted {
            state: PendingSubmissionState::Committed,
            turn_id: Some("turn".into()),
        },
        SubmissionReceiptDisposition::Rejected {
            reason: SubmissionRejectionReason::SessionClosed,
        },
    ] {
        inspect_event(SessionEvent::SubmissionReceipt {
            session_id: "fixture".into(),
            session_generation: 1,
            submission_id: "submission".into(),
            reservation_id: "reservation".into(),
            receipt_id: "receipt".into(),
            source: SubmissionSource::Compatibility,
            item_count: 1,
            total_text_bytes: 7,
            disposition,
        });
    }
    inspect_event(SessionEvent::BackgroundJobTerminal {
        session_id: "fixture".into(),
        session_generation: 1,
        summary: BackgroundJobTerminalSummary {
            job_id: BackgroundJobId::new("job"),
            tool_name: "surface_only".into(),
            state: BackgroundJobState::Completed,
            exit_code: Some(0),
            stdout_bytes: 0,
            stderr_bytes: 0,
            earliest_cursor: 0,
            next_cursor: 0,
            truncated: false,
            started_at_unix_ms: 0,
            finished_at_unix_ms: 1,
            cleanup_outcome: BackgroundCleanupOutcome::Natural,
            cleanup_error: None,
        },
    });
    inspect_error(RuntimeError::Agent(AgentError::ProviderError(
        ProviderError::InvalidResponse("fixture".into()),
    )));
    inspect_error(RuntimeError::Session(SessionError::IoError(
        std::io::Error::other("fixture"),
    )));
}
