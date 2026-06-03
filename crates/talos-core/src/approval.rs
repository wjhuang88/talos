//! Approval types for permission-gated tool execution.

use tokio::sync::oneshot;

/// User's choice when presented with an approval prompt.
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalChoice {
    /// Approve this tool call once.
    ApproveOnce,
    /// Always approve this tool (add a rule).
    AlwaysApprove,
    /// Deny the tool call.
    Deny,
}

/// Approval request sent from a TUI tool to the TUI event loop.
///
/// The tool sends this via an unbounded channel and awaits the response
/// on `response`. The TUI receives the request, shows the approval overlay,
/// and sends back the user's choice via the oneshot channel.
pub struct TuiApprovalRequest {
    /// Name of the tool requiring approval.
    pub tool_name: String,
    /// Formatted arguments for display.
    pub arguments: String,
    /// Channel to send the approval response back to the waiting tool.
    pub response: oneshot::Sender<ApprovalChoice>,
}
