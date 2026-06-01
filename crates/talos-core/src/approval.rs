//! Approval types for permission-gated tool execution.

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
