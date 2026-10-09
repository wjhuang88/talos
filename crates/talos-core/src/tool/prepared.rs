//! Owned invocation boundary for tools that cannot use parsed-JSON/path authorization.

use async_trait::async_trait;

use super::ToolExecutionOutput;

/// Closed failure vocabulary for the browser prepared-invocation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub enum PreparedFailureCode {
    /// Reference is unknown or no longer valid.
    InvalidReference,
    /// Frame generation changed.
    StaleFrameReference,
    /// Frame was detached.
    DetachedFrame,
    /// Element snapshot changed.
    StaleElementReference,
    /// Trusted context is unavailable.
    ContextUnavailable,
    /// Destination origin lacks authority.
    OriginNotAuthorized,
    /// Protocol version is unsupported.
    UnsupportedVersion,
    /// Exact operation cannot be performed safely.
    UnsupportedOperation,
    /// Bounded capacity was exceeded.
    ResourceLimit,
    /// Admission lifetime elapsed before execution.
    AdmissionExpired,
    /// Request failed schema or semantic validation.
    InvalidRequest,
    /// Exact authorization was not granted.
    PermissionDenied,
    /// Execution may have begun without a confirmed result.
    IndeterminateExecution,
}

/// Result of a prepared invocation with request-only provider attachments.
/// Kept separate from the existing exhaustive tool output structure for source compatibility.
pub struct PreparedExecutionOutput {
    /// Existing display and persistence projection.
    pub output: ToolExecutionOutput,
    /// Images owned by the immediately following provider request, never durable history.
    pub images: Vec<crate::provider::EphemeralImage>,
}

impl From<ToolExecutionOutput> for PreparedExecutionOutput {
    fn from(output: ToolExecutionOutput) -> Self {
        Self {
            output,
            images: Vec::new(),
        }
    }
}

/// Bounded failure without original arguments or backend diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PreparedInvocationError {
    /// No valid prepared route, current admission, or exact authorization is available.
    #[error("prepared invocation denied or unavailable")]
    Denied,
    /// A closed, content-free browser failure. Unknown operation strings are never projected.
    #[error("prepared invocation failed: {code:?}")]
    Failure {
        /// Stable failure category.
        code: PreparedFailureCode,
        /// Known discriminator; output independently validates the closed vocabulary.
        operation: Option<&'static str>,
    },
}

impl PreparedInvocationError {
    /// Recognizes only a closed browser discriminator without retaining arbitrary input.
    pub fn browser_operation(name: &str) -> Option<&'static str> {
        [
            "open",
            "read",
            "snapshot",
            "current-url",
            "screenshot",
            "click",
            "fill",
            "select",
            "hover",
            "check",
            "uncheck",
            "press",
            "scroll",
            "wait-for-element",
            "wait-milliseconds",
            "tab-new",
            "tab-list",
            "tab-close",
            "tab-switch",
            "window-close",
            "frame-tree",
        ]
        .into_iter()
        .find(|operation| *operation == name)
    }
    /// Renders a closed browser failure without arguments or backend diagnostics.
    /// Only indeterminate execution claims an unknown outcome; all other errors from this
    /// boundary establish that execution was not performed.
    pub fn into_output(self) -> ToolExecutionOutput {
        let (code, operation) = match self {
            Self::Denied => (PreparedFailureCode::PermissionDenied, None),
            Self::Failure { code, operation } => (code, operation),
        };
        let operation = operation.and_then(Self::browser_operation);
        ToolExecutionOutput::error(serde_json::json!({
            "kind": "Failure", "code": code, "operation": operation,
            "outcome": if code == PreparedFailureCode::IndeterminateExecution { "unknown" } else { "notExecuted" },
        }).to_string())
    }
}

/// An admitted invocation owned by one active request, never cloned or serialized.
///
/// Implementations retain the typed resource and dedicated permission resolver together.
/// Generic path/network grants must not be consulted. Dropping this object revokes admission.
#[async_trait]
pub trait PreparedToolInvocation: Send {
    /// Evaluates the exact admitted resource once and consumes this preparation.
    /// Must not execute a tool. The caller retains its final hook gate before dispatch.
    async fn authorize(
        self: Box<Self>,
    ) -> Result<Box<dyn AuthorizedToolInvocation>, PreparedInvocationError>;
}

/// One exact authorized invocation, still subject to final lifecycle validation.
#[async_trait]
pub trait AuthorizedToolInvocation: Send {
    /// Executes once and transfers any ephemeral attachments to the active request.
    /// Legacy implementers retain their existing behavior without attachments.
    async fn execute_with_attachments(self: Box<Self>) -> PreparedExecutionOutput {
        PreparedExecutionOutput {
            output: self.execute().await,
            images: Vec::new(),
        }
    }

    /// Consumes the authorization and dispatches at most once, without fallback or replay.
    /// Dropping the future must propagate cancellation to the owned execution.
    async fn execute(self: Box<Self>) -> ToolExecutionOutput;
}

#[cfg(test)]
mod failure_tests {
    use super::*;

    #[test]
    fn failure_projection_is_closed_and_effect_honest() {
        for (code, outcome) in [
            (PreparedFailureCode::InvalidRequest, "notExecuted"),
            (PreparedFailureCode::IndeterminateExecution, "unknown"),
        ] {
            let output = PreparedInvocationError::Failure {
                code,
                operation: Some("fill"),
            }
            .into_output();
            assert!(output.result.is_error);
            let value: serde_json::Value =
                serde_json::from_str(&output.result.content).expect("failure JSON");
            assert_eq!(value["operation"], "fill");
            assert_eq!(value["outcome"], outcome);
            assert_eq!(value["kind"], "Failure");
        }
        let output = PreparedInvocationError::Failure {
            code: PreparedFailureCode::InvalidRequest,
            operation: Some("secret-driver-error"),
        }
        .into_output();
        assert!(!output.result.content.contains("secret"));
        assert_eq!(
            PreparedInvocationError::browser_operation("hover"),
            Some("hover")
        );
    }
}
