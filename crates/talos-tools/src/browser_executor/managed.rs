//! Explicit browser tool composition. Parsed-JSON execution is never an authority path.

use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    AgentTool, AuthorizedToolInvocation, BrowserRawArguments, PreparedInvocationError,
    PreparedToolInvocation, ToolExecutionOutput, ToolResult, ToolResultProjection,
};
use tokio::sync::Mutex;

use super::{
    BoundBrowserInvocation, BrowserExecutor, BrowserHost, BrowserHostError,
    BrowserInvocationAuthorization, BrowserOutput, BrowserPermissionResolver,
    BrowserPermissionResource, BrowserRequest, DenyBrowserPermissionEvaluator,
    ValidatedBrowserOutput,
};

/// Opt-in browser v2 tool backed by one fixed host and executor.
///
/// Registering this tool does not grant authority. A trusted composition root must prepare
/// original arguments, evaluate the exact resource and consume its one-shot authorization.
/// Legacy execution methods always deny, including when supplied path authorizations.
pub struct ManagedBrowserTool<E: BrowserExecutor> {
    host: Arc<Mutex<BrowserHost<E>>>,
    lifecycle: super::BrowserLifecycleHandle,
    resolver: Arc<dyn BrowserPermissionResolver>,
}

impl<E: BrowserExecutor> ManagedBrowserTool<E> {
    /// Takes ownership of a configured host. No default inventory is modified.
    pub fn new(host: BrowserHost<E>) -> Self {
        let lifecycle = host.lifecycle_handle();
        Self {
            host: Arc::new(Mutex::new(host)),
            lifecycle,
            resolver: Arc::new(DenyBrowserPermissionEvaluator),
        }
    }

    /// Installs an explicit browser-aware resolver. Generic permission rules are not forwarded.
    pub fn with_permission_resolver(
        mut self,
        resolver: Arc<dyn BrowserPermissionResolver>,
    ) -> Self {
        self.resolver = resolver;
        self
    }

    /// Returns a trusted revocation handle for the host integration.
    pub fn lifecycle_handle(&self) -> super::BrowserLifecycleHandle {
        self.lifecycle.clone()
    }

    /// Rebuilds trusted discovery after invalidation. Old references and approvals stay revoked.
    /// Invalidation itself never waits for this lock; rebuilding waits for active dispatch to end.
    pub async fn synchronize_context(&self, rebuild: impl FnOnce(&mut super::BrowserContext)) {
        self.host.lock().await.synchronize_context(rebuild);
    }

    /// Performs semantic admission and backend readiness before permission evaluation.
    /// The returned resource is derived from the same non-cloneable invocation.
    pub async fn prepare_invocation(
        &self,
        arguments: &BrowserRawArguments,
    ) -> Result<(BoundBrowserInvocation, BrowserPermissionResource), BrowserHostError> {
        let mut host = self.host.lock().await;
        let invocation = host.prepare_original(arguments)?;
        let resource = host.permission_resource(&invocation)?;
        Ok((invocation, resource))
    }

    /// Consumes an exact authorization and invocation once; never retries the backend.
    /// Cancellation while waiting for the host also drops and revokes the invocation.
    pub async fn execute_prepared(
        &self,
        invocation: BoundBrowserInvocation,
        authorization: BrowserInvocationAuthorization,
        cancelled: impl Future<Output = ()> + Send,
    ) -> Result<ValidatedBrowserOutput, BrowserHostError> {
        tokio::pin!(cancelled);
        let mut host = tokio::select! {
            biased;
            _ = &mut cancelled => return Err(BrowserHostError::Denied),
            host = self.host.lock() => host,
        };
        host.execute_prepared(invocation, authorization, cancelled)
            .await
    }
}

#[async_trait]
impl<E: BrowserExecutor + 'static> AgentTool for ManagedBrowserTool<E> {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Explicit host-bound frame-aware browser v2 operations"
    }

    fn parameters(&self) -> Value {
        BrowserRequest::schema()
    }

    fn requires_original_invocation(&self) -> bool {
        true
    }

    async fn prepare_original_invocation(
        &self,
        arguments: BrowserRawArguments,
    ) -> Result<Box<dyn PreparedToolInvocation>, PreparedInvocationError> {
        let (invocation, resource) = self
            .prepare_invocation(&arguments)
            .await
            .map_err(|error| error.to_prepared(None))?;
        Ok(Box::new(Prepared {
            host: self.host.clone(),
            resolver: self.resolver.clone(),
            invocation,
            resource,
        }))
    }

    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::error("browser permission denied: prepared invocation required")
    }

    fn project_input(&self, _input: &Value) -> Value {
        serde_json::json!({"protocolVersion": 2})
    }

    fn project_result(&self, result: &ToolResult) -> ToolResultProjection {
        ToolResultProjection {
            model_content: result.content.clone(),
            display_content: "browser invocation completed".into(),
            persistence_content: "browser invocation completed".into(),
        }
    }
}

struct Prepared<E: BrowserExecutor> {
    host: Arc<Mutex<BrowserHost<E>>>,
    resolver: Arc<dyn BrowserPermissionResolver>,
    invocation: BoundBrowserInvocation,
    resource: BrowserPermissionResource,
}

#[async_trait]
impl<E: BrowserExecutor + 'static> PreparedToolInvocation for Prepared<E> {
    async fn authorize(
        self: Box<Self>,
    ) -> Result<Box<dyn AuthorizedToolInvocation>, PreparedInvocationError> {
        let current = self
            .host
            .lock()
            .await
            .permission_resource(&self.invocation)
            .map_err(|error| error.to_prepared(Some(self.resource.operation)))?;
        if current != self.resource {
            return Err(BrowserHostError::Denied.to_prepared(Some(self.resource.operation)));
        }
        let authorization = self
            .resolver
            .resolve(&self.resource)
            .await
            .map_err(|error| {
                let code = match error {
                    super::BrowserAuthorizationError::InvalidRequest => {
                        super::BrowserFailureCode::InvalidRequest
                    }
                    _ => super::BrowserFailureCode::PermissionDenied,
                };
                BrowserHostError::Rejected {
                    code,
                    operation: Some(self.resource.operation),
                }
                .to_prepared(None)
            })?;
        let current = self
            .host
            .lock()
            .await
            .permission_resource(&self.invocation)
            .map_err(|error| error.to_prepared(Some(self.resource.operation)))?;
        if !authorization.matches(&current) {
            return Err(BrowserHostError::Denied.to_prepared(Some(self.resource.operation)));
        }
        Ok(Box::new(Authorized {
            host: self.host,
            invocation: self.invocation,
            authorization,
        }))
    }
}

struct Authorized<E: BrowserExecutor> {
    host: Arc<Mutex<BrowserHost<E>>>,
    invocation: BoundBrowserInvocation,
    authorization: BrowserInvocationAuthorization,
}

#[async_trait]
impl<E: BrowserExecutor + 'static> AuthorizedToolInvocation for Authorized<E> {
    async fn execute(self: Box<Self>) -> ToolExecutionOutput {
        self.execute_with_attachments().await.output
    }

    async fn execute_with_attachments(
        self: Box<Self>,
    ) -> talos_core::tool::PreparedExecutionOutput {
        let resource = self.authorization.resource().clone();
        let mut host = self.host.lock().await;
        match host
            .execute_prepared(self.invocation, self.authorization, std::future::pending())
            .await
        {
            Ok(output) if output.is_failure() => {
                ToolExecutionOutput::error(output.model_json()).into()
            }
            Ok(output) => {
                let mut result: talos_core::tool::PreparedExecutionOutput =
                    ToolExecutionOutput::success(output.model_json()).into();
                if let Ok(BrowserOutput::Screenshot {
                    artifact_ref,
                    mime: _,
                    width: _,
                    height: _,
                    byte_length: _,
                }) = serde_json::from_str(output.model_json())
                {
                    let Ok((bytes, expires_at)) = host.consume_screenshot(&artifact_ref, &resource)
                    else {
                        return BrowserHostError::Indeterminate
                            .to_prepared(Some(resource.operation))
                            .into_output()
                            .into();
                    };
                    let Some(image) = talos_core::provider::EphemeralImage::png(bytes, expires_at)
                    else {
                        return BrowserHostError::Indeterminate
                            .to_prepared(Some(resource.operation))
                            .into_output()
                            .into();
                    };
                    result.images.push(image);
                }
                result
            }
            Err(error) => error
                .to_prepared(Some(resource.operation))
                .into_output()
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_executor::{
        BrowserContext, BrowserOutput, BrowserPermissionEvaluator, BrowserPermissionTarget,
        BrowserScreenshotMime, BrowserSuccess, ExactBrowserPermissionEvaluator,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct Executor(Arc<AtomicUsize>);

    #[async_trait]
    impl BrowserExecutor for Executor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }

        async fn execute(
            &mut self,
            request: &BrowserRequest,
            _: &BrowserPermissionTarget,
            _: &mut BrowserContext,
        ) -> BrowserOutput {
            self.0.fetch_add(1, Ordering::SeqCst);
            BrowserOutput::Ack {
                operation: request.operation(),
                status: BrowserSuccess::Ok,
            }
        }
    }

    fn arguments() -> BrowserRawArguments {
        BrowserRawArguments::parse_original(
            r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#,
        )
        .expect("valid original request")
    }

    struct FailingExecutor;

    struct ScreenshotExecutor;

    #[async_trait]
    impl BrowserExecutor for ScreenshotExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }
        async fn execute(
            &mut self,
            _: &BrowserRequest,
            _: &BrowserPermissionTarget,
            context: &mut BrowserContext,
        ) -> BrowserOutput {
            let mut png = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgb8(1, 1)
                .write_to(&mut png, image::ImageFormat::Png)
                .expect("encode");
            let bytes = png.into_inner();
            let byte_length = bytes.len() as u64;
            let artifact_ref = context.publish_screenshot(bytes, 1, 1).expect("publish");
            BrowserOutput::Screenshot {
                artifact_ref,
                mime: BrowserScreenshotMime::Png,
                width: 1,
                height: 1,
                byte_length,
            }
        }
    }

    #[tokio::test]
    async fn managed_screenshot_delivers_transient_bytes_without_relocking_host() {
        let mut host = BrowserHost::new(ScreenshotExecutor);
        let tab = host.context_mut().insert_tab().expect("tab");
        let frame = host
            .context_mut()
            .insert_frame(
                &tab,
                None,
                Some(
                    crate::browser_executor::BrowserOrigin::from_effective_url(
                        "https://example.com",
                    )
                    .expect("origin"),
                ),
                true,
            )
            .expect("frame");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"screenshot","tabRef":tab,"frameRef":frame});
        let tool = ManagedBrowserTool::new(host).with_permission_resolver(Arc::new(ApproveOnce));
        let prepared = tool
            .prepare_original_invocation(
                BrowserRawArguments::parse_original(&raw.to_string()).expect("raw"),
            )
            .await
            .expect("prepare");
        let authorized = prepared.authorize().await.expect("approve");
        let mut output = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            authorized.execute_with_attachments(),
        )
        .await
        .expect("must not deadlock");
        assert!(!output.output.result.is_error);
        assert!(output.output.next_provider_parts.is_empty());
        assert_eq!(output.images.len(), 1);
        let (bytes, _) = output
            .images
            .pop()
            .expect("image")
            .into_live_png()
            .expect("live");
        let decoded = image::load_from_memory(&bytes).expect("valid PNG");
        assert_eq!((decoded.width(), decoded.height()), (1, 1));
        let projection = tool.project_result(&output.output.result);
        assert!(!projection.persistence_content.contains("artifact_"));
        assert_eq!(projection.persistence_content, projection.display_content);
        assert_eq!(
            serde_json::to_string(&output.output.next_provider_parts).expect("durable empty parts"),
            "[]"
        );
    }

    #[async_trait]
    impl BrowserExecutor for FailingExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }

        async fn execute(
            &mut self,
            request: &BrowserRequest,
            _: &BrowserPermissionTarget,
            _: &mut BrowserContext,
        ) -> BrowserOutput {
            BrowserOutput::Failure {
                code: crate::browser_executor::BrowserFailureCode::OriginNotAuthorized,
                operation: Some(request.operation()),
                outcome: crate::browser_executor::BrowserOutcome::NotExecuted,
            }
        }
    }

    #[tokio::test]
    async fn typed_backend_failure_is_not_reported_as_tool_success() {
        let tool = ManagedBrowserTool::new(BrowserHost::new(FailingExecutor))
            .with_permission_resolver(Arc::new(ApproveOnce));
        let output = tool
            .prepare_original_invocation(arguments())
            .await
            .expect("prepare")
            .authorize()
            .await
            .expect("authorize")
            .execute()
            .await;
        assert!(output.result.is_error);
        let failure: Value = serde_json::from_str(&output.result.content).expect("typed failure");
        assert_eq!(failure["code"], "OriginNotAuthorized");
        assert_eq!(failure["outcome"], "notExecuted");
        let projection = tool.project_result(&output.result);
        assert!(!projection.display_content.contains("OriginNotAuthorized"));
        assert_eq!(projection.display_content, projection.persistence_content);
    }

    struct ApproveOnce;

    struct RevokeDuringApproval(super::super::BrowserLifecycleHandle);

    #[async_trait]
    impl BrowserPermissionResolver for RevokeDuringApproval {
        async fn resolve(
            &self,
            resource: &BrowserPermissionResource,
        ) -> Result<
            BrowserInvocationAuthorization,
            crate::browser_executor::BrowserAuthorizationError,
        > {
            let approval = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(resource)?;
            self.0.invalidate_pending();
            Ok(approval)
        }
    }

    #[tokio::test]
    async fn public_lifecycle_handle_revokes_approval_without_dispatch_or_host_lock() {
        let calls = Arc::new(AtomicUsize::new(0));
        let tool = ManagedBrowserTool::new(BrowserHost::new(Executor(calls.clone())));
        let guard = tool.host.lock().await;
        let lifecycle = tool.lifecycle_handle();
        drop(guard);
        let tool = tool.with_permission_resolver(Arc::new(RevokeDuringApproval(lifecycle)));
        let prepared = tool
            .prepare_original_invocation(arguments())
            .await
            .expect("prepare");
        assert!(prepared.authorize().await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(tool.prepare_original_invocation(arguments()).await.is_err());
        tool.synchronize_context(|_| {}).await;
        assert!(tool.prepare_original_invocation(arguments()).await.is_ok());
    }

    struct CountingResolver(Arc<AtomicUsize>);

    #[async_trait]
    impl BrowserPermissionResolver for CountingResolver {
        async fn resolve(
            &self,
            resource: &BrowserPermissionResource,
        ) -> Result<
            BrowserInvocationAuthorization,
            crate::browser_executor::BrowserAuthorizationError,
        > {
            self.0.fetch_add(1, Ordering::SeqCst);
            ExactBrowserPermissionEvaluator::for_resource(resource.clone()).evaluate(resource)
        }
    }

    #[tokio::test]
    async fn stale_preparation_never_invokes_approval_resolver() {
        let calls = Arc::new(AtomicUsize::new(0));
        let approvals = Arc::new(AtomicUsize::new(0));
        let tool = ManagedBrowserTool::new(BrowserHost::new(Executor(calls.clone())))
            .with_permission_resolver(Arc::new(CountingResolver(approvals.clone())));
        let prepared = tool
            .prepare_original_invocation(arguments())
            .await
            .expect("prepared");
        tool.host.lock().await.context_mut().lose_synchronization();
        assert!(prepared.authorize().await.is_err());
        assert_eq!(approvals.load(Ordering::SeqCst), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[async_trait]
    impl BrowserPermissionResolver for ApproveOnce {
        async fn resolve(
            &self,
            resource: &BrowserPermissionResource,
        ) -> Result<
            BrowserInvocationAuthorization,
            crate::browser_executor::BrowserAuthorizationError,
        > {
            ExactBrowserPermissionEvaluator::for_resource(resource.clone()).evaluate(resource)
        }
    }

    #[tokio::test]
    async fn object_safe_route_defaults_to_deny_and_requires_explicit_browser_resolver() {
        let calls = Arc::new(AtomicUsize::new(0));
        let tool = ManagedBrowserTool::new(BrowserHost::new(Executor(calls.clone())));
        let prepared = tool
            .prepare_original_invocation(arguments())
            .await
            .expect("prepared");
        assert!(prepared.authorize().await.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let tool = tool.with_permission_resolver(Arc::new(ApproveOnce));
        let prepared = tool
            .prepare_original_invocation(arguments())
            .await
            .expect("prepared");
        let authorized = prepared.authorize().await.expect("exact approval");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(!authorized.execute().await.result.is_error);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn legacy_variants_deny_and_prepared_path_executes_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let tool = ManagedBrowserTool::new(BrowserHost::new(Executor(calls.clone())));
        let input = serde_json::json!({"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1});
        assert!(tool.execute(input.clone()).await.is_error);
        assert!(tool.execute_authorized(input.clone(), &[]).await.is_error);
        assert!(
            tool.execute_with_output(input.clone())
                .await
                .result
                .is_error
        );
        assert!(
            tool.execute_authorized_with_output(input, &[])
                .await
                .result
                .is_error
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let (invocation, resource) = tool
            .prepare_invocation(&arguments())
            .await
            .expect("prepared");
        let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("exact approval");
        tool.execute_prepared(invocation, authorization, std::future::pending())
            .await
            .expect("one execution");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cancellation_revokes_prepared_work_without_dispatch() {
        let calls = Arc::new(AtomicUsize::new(0));
        let tool = ManagedBrowserTool::new(BrowserHost::new(Executor(calls.clone())));
        let (invocation, resource) = tool
            .prepare_invocation(&arguments())
            .await
            .expect("prepared");
        let evaluator = ExactBrowserPermissionEvaluator::for_resource(resource.clone());
        let authorization = evaluator.evaluate(&resource).expect("exact approval");
        assert!(
            tool.execute_prepared(invocation, authorization, std::future::ready(()))
                .await
                .is_err()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(
            ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .is_err()
        );
    }
}
