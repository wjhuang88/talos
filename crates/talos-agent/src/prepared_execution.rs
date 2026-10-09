//! Agent-owned final gates for original-argument invocations.

use futures_util::FutureExt;
use talos_core::{
    message::ToolCall,
    tool::{BrowserRawArguments, PreparedFailureCode, PreparedInvocationError},
};
use talos_permission::PermissionDecision;
use talos_plugin::{HookContext, HookEvent, HookOutcome};

use crate::Agent;

fn failure(
    code: PreparedFailureCode,
    operation: Option<&'static str>,
) -> talos_core::tool::PreparedExecutionOutput {
    PreparedInvocationError::Failure { code, operation }
        .into_output()
        .into()
}

impl Agent {
    pub(crate) async fn execute_original_tool(
        &self,
        context: &HookContext,
        call: &ToolCall,
        arguments: BrowserRawArguments,
    ) -> talos_core::tool::PreparedExecutionOutput {
        let operation = arguments
            .fields()
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .and_then(PreparedInvocationError::browser_operation);
        let Some(tool) = self.tools.get(&call.name) else {
            return failure(PreparedFailureCode::UnsupportedOperation, operation);
        };
        if !tool.requires_original_invocation() {
            return failure(PreparedFailureCode::UnsupportedOperation, operation);
        }
        let deadline = tokio::time::Instant::now() + self.permission_deadline;
        let admission = tokio::time::timeout_at(
            deadline,
            std::panic::AssertUnwindSafe(async {
                tool.prepare_original_invocation(arguments).await
            })
            .catch_unwind(),
        )
        .await;
        let prepared = match admission {
            Ok(Ok(Ok(prepared))) => prepared,
            Ok(Ok(Err(error))) => return error.into_output().into(),
            _ => return failure(PreparedFailureCode::ContextUnavailable, operation),
        };
        for event in [
            HookEvent::OnToolCallProposed { call },
            HookEvent::BeforePermissionCheck { call },
        ] {
            if !matches!(
                tokio::time::timeout_at(
                    deadline,
                    self.hook_registry.dispatch_permission_gate(context, event)
                )
                .await,
                Ok(HookOutcome::Continue(_))
            ) {
                return failure(PreparedFailureCode::PermissionDenied, operation);
            }
        }
        let authorization = tokio::time::timeout_at(
            deadline,
            std::panic::AssertUnwindSafe(async { prepared.authorize().await }).catch_unwind(),
        )
        .await;
        let decision = if matches!(&authorization, Ok(Ok(Ok(_)))) {
            PermissionDecision::Allow
        } else {
            PermissionDecision::Deny("browser authorization unavailable".into())
        };
        if !matches!(
            tokio::time::timeout_at(
                deadline,
                self.hook_registry.dispatch_permission_gate(
                    context,
                    HookEvent::AfterPermissionCheck { call, decision }
                )
            )
            .await,
            Ok(HookOutcome::Continue(_))
        ) {
            return failure(PreparedFailureCode::PermissionDenied, operation);
        }
        let authorized = match authorization {
            Ok(Ok(Ok(authorized))) => authorized,
            Ok(Ok(Err(error))) => return error.into_output().into(),
            _ => return failure(PreparedFailureCode::PermissionDenied, operation),
        };
        if self
            .execution_ledger
            .reserve(context.turn_id, call)
            .is_err()
        {
            return failure(PreparedFailureCode::InvalidReference, operation);
        }
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(35),
            std::panic::AssertUnwindSafe(async { authorized.execute_with_attachments().await })
                .catch_unwind(),
        )
        .await;
        self.execution_ledger.complete(context.turn_id, call);
        match result {
            Ok(Ok(output)) => output,
            _ => failure(PreparedFailureCode::IndeterminateExecution, operation),
        }
    }
}
