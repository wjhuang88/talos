//! Fixed host/executor ownership and one-shot authorized dispatch.

use super::{
    BrowserArtifactRef, BrowserContext, BrowserFailureCode, BrowserInvocationAuthorization,
    BrowserOutput, BrowserOutputError, BrowserPermissionResource, BrowserPermissionTarget,
    BrowserRequest, PreparedBrowserInvocation, ValidatedBrowserOutput,
};
use std::{
    future::{Future, poll_fn},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::Poll,
    time::Duration,
};

/// Stable host failure. No driver message or input is exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserHostError {
    /// Request or trusted state could not be admitted.
    #[error("browser admission rejected")]
    Admission,
    /// This backend cannot execute the exact operation/document pair safely.
    #[error("browser operation unsupported")]
    Unsupported,
    /// Invocation, host or permission capability did not match.
    #[error("browser permission denied")]
    Denied,
    /// Execution may have started. Never automatically replay this invocation.
    #[error("browser execution outcome indeterminate")]
    Indeterminate,
    /// A specific closed failure detected before executor dispatch.
    #[error("browser invocation rejected: {code:?}")]
    Rejected {
        /// Closed reason without supplied data.
        code: BrowserFailureCode,
        /// Known validated operation discriminator, if available.
        operation: Option<super::BrowserOperation>,
    },
}

impl BrowserHostError {
    /// Projects the trusted host error across the core prepared boundary.
    pub fn to_prepared(
        self,
        operation: Option<super::BrowserOperation>,
    ) -> talos_core::tool::PreparedInvocationError {
        use talos_core::tool::{PreparedFailureCode as F, PreparedInvocationError as E};
        let (code, operation) = match self {
            Self::Rejected {
                code,
                operation: rejected_operation,
            } => (code, rejected_operation.or(operation)),
            Self::Admission => (BrowserFailureCode::InvalidRequest, operation),
            Self::Unsupported => (BrowserFailureCode::UnsupportedOperation, operation),
            Self::Denied => (BrowserFailureCode::PermissionDenied, operation),
            Self::Indeterminate => (BrowserFailureCode::IndeterminateExecution, operation),
        };
        let code = match code {
            BrowserFailureCode::InvalidReference => F::InvalidReference,
            BrowserFailureCode::StaleFrameReference => F::StaleFrameReference,
            BrowserFailureCode::DetachedFrame => F::DetachedFrame,
            BrowserFailureCode::StaleElementReference => F::StaleElementReference,
            BrowserFailureCode::ContextUnavailable => F::ContextUnavailable,
            BrowserFailureCode::OriginNotAuthorized => F::OriginNotAuthorized,
            BrowserFailureCode::UnsupportedVersion => F::UnsupportedVersion,
            BrowserFailureCode::UnsupportedOperation => F::UnsupportedOperation,
            BrowserFailureCode::ResourceLimit => F::ResourceLimit,
            BrowserFailureCode::AdmissionExpired => F::AdmissionExpired,
            BrowserFailureCode::InvalidRequest => F::InvalidRequest,
            BrowserFailureCode::PermissionDenied => F::PermissionDenied,
            BrowserFailureCode::IndeterminateExecution => F::IndeterminateExecution,
        };
        let operation = operation
            .and_then(|operation| serde_json::to_value(operation).ok())
            .and_then(|value| value.as_str().and_then(E::browser_operation));
        E::Failure { code, operation }
    }
}

fn context_error(
    error: super::BrowserContextError,
    operation: Option<super::BrowserOperation>,
) -> BrowserHostError {
    use super::{BrowserContextError as C, BrowserTicketError as T};
    let code = match error {
        C::InvalidReference | C::Ticket(T::Consumed | T::Stale) => {
            BrowserFailureCode::InvalidReference
        }
        C::Pending | C::OriginUnavailable | C::Unavailable | C::Ticket(T::Unavailable) => {
            BrowserFailureCode::ContextUnavailable
        }
        C::ResourceLimit | C::Ticket(T::Capacity) => BrowserFailureCode::ResourceLimit,
        C::Ticket(T::Expired) => BrowserFailureCode::AdmissionExpired,
    };
    BrowserHostError::Rejected { code, operation }
}

/// Trusted backend boundary; not a sandbox against a malicious implementation.
#[async_trait::async_trait]
pub trait BrowserExecutor: Send {
    /// Checks exact operation/document support without running any browser action.
    /// Default deny: a backend must explicitly attest document isolation and redirect control.
    fn supports(&self, _request: &BrowserRequest, _target: &BrowserPermissionTarget) -> bool {
        false
    }

    /// Performs one action/capture on the bound document and checks identity before effects
    /// and before releasing observations. Updates trusted lifecycle state as required.
    /// Must cooperate with future cancellation; must not retry, detach or use global focus.
    async fn execute(
        &mut self,
        request: &BrowserRequest,
        target: &BrowserPermissionTarget,
        context: &mut BrowserContext,
    ) -> BrowserOutput;
}

/// Prepared work privately bound to a fixed executor instance as well as its context ticket.
pub struct BoundBrowserInvocation {
    host: Arc<()>,
    generation: u64,
    prepared: PreparedBrowserInvocation,
}

/// Trusted lifecycle invalidation handle retained by a host integration after composition.
/// It cannot mint permissions or change model-visible state; it only revokes pending work.
#[derive(Clone)]
pub struct BrowserLifecycleHandle {
    generation: Arc<AtomicU64>,
    changed: Arc<tokio::sync::Notify>,
}

impl BrowserLifecycleHandle {
    /// Revokes every pending invocation after navigation, detachment, or uncertain sync.
    pub fn invalidate_pending(&self) {
        let _ = self
            .generation
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                Some(value.saturating_add(1))
            });
        self.changed.notify_waiters();
    }
}

/// Owns one executor for its lifetime. Mutable dispatch serializes this session's operations.
pub struct BrowserHost<E: BrowserExecutor> {
    identity: Arc<()>,
    context: BrowserContext,
    executor: E,
    generation: Arc<AtomicU64>,
    synchronized_generation: u64,
    changed: Arc<tokio::sync::Notify>,
}

impl<E: BrowserExecutor> BrowserHost<E> {
    /// Creates an opt-in host. No tool registry is modified.
    pub fn new(executor: E) -> Self {
        Self {
            identity: Arc::new(()),
            context: BrowserContext::new(),
            executor,
            generation: Arc::new(AtomicU64::new(0)),
            synchronized_generation: 0,
            changed: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Returns a trusted handle for lifecycle subscriptions after the host is moved into a tool.
    pub fn lifecycle_handle(&self) -> BrowserLifecycleHandle {
        BrowserLifecycleHandle {
            generation: self.generation.clone(),
            changed: self.changed.clone(),
        }
    }

    /// Trusted lifecycle/discovery integration only; never expose this to model/plugin input.
    pub fn context_mut(&mut self) -> &mut BrowserContext {
        &mut self.context
    }

    /// Consumes a screenshot capability for the exact authorized resource and current generation.
    pub fn consume_screenshot(
        &self,
        reference: &BrowserArtifactRef,
        resource: &BrowserPermissionResource,
    ) -> Result<(Vec<u8>, std::time::Instant), BrowserOutputError> {
        self.context.artifact_store().consume(
            reference,
            resource,
            self.generation.load(Ordering::Acquire),
        )
    }

    /// Replaces all references after trusted host synchronization, never preserving old grants.
    /// Concurrent invalidation keeps the rebuilt context unavailable until another refresh.
    pub fn synchronize_context(&mut self, rebuild: impl FnOnce(&mut BrowserContext)) {
        let generation = self.generation.load(Ordering::Acquire);
        self.context.replace_session();
        if catch_unwind(AssertUnwindSafe(|| rebuild(&mut self.context))).is_err() {
            self.context.lose_synchronization();
            return;
        }
        self.synchronized_generation = generation;
    }

    /// Admits original JSON and backend readiness before any permission prompt.
    pub fn prepare(&mut self, raw: &str) -> Result<BoundBrowserInvocation, BrowserHostError> {
        let arguments =
            talos_core::tool::BrowserRawArguments::parse_original(raw).map_err(|error| {
                BrowserHostError::Rejected {
                    code: match error {
                        talos_core::tool::BrowserRawArgumentsError::TooLarge => {
                            BrowserFailureCode::ResourceLimit
                        }
                        _ => BrowserFailureCode::InvalidRequest,
                    },
                    operation: None,
                }
            })?;
        self.prepare_original(&arguments)
    }

    /// Admits a transport's original-argument carrier without reserializing parsed JSON.
    /// Syntax evidence does not replace semantic admission, lifecycle checks, or permission.
    pub fn prepare_original(
        &mut self,
        arguments: &talos_core::tool::BrowserRawArguments,
    ) -> Result<BoundBrowserInvocation, BrowserHostError> {
        let operation = arguments
            .fields()
            .get("operation")
            .and_then(serde_json::Value::as_str)
            .and_then(|name| {
                serde_json::from_value(serde_json::Value::String(name.to_owned())).ok()
            });
        let request = BrowserRequest::from_original(arguments).map_err(|error| {
            BrowserHostError::Rejected {
                code: match error {
                    super::BrowserRequestError::TooLarge => BrowserFailureCode::ResourceLimit,
                    super::BrowserRequestError::UnsupportedVersion => {
                        BrowserFailureCode::UnsupportedVersion
                    }
                    super::BrowserRequestError::Invalid => BrowserFailureCode::InvalidRequest,
                },
                operation,
            }
        })?;
        self.prepare_request(request)
    }

    fn prepare_request(
        &mut self,
        request: BrowserRequest,
    ) -> Result<BoundBrowserInvocation, BrowserHostError> {
        let generation = self.generation.load(Ordering::Acquire);
        if generation == u64::MAX || generation != self.synchronized_generation {
            self.context.lose_synchronization();
            return Err(BrowserHostError::Rejected {
                code: BrowserFailureCode::ContextUnavailable,
                operation: Some(request.operation()),
            });
        }
        let operation = Some(request.operation());
        let prepared = self
            .context
            .prepare_invocation(request)
            .map_err(|error| context_error(error, operation))?;
        let supported = catch_unwind(AssertUnwindSafe(|| {
            self.executor
                .supports(prepared.request(), prepared.target())
        }));
        if !matches!(supported, Ok(true)) {
            let _ = self.context.discard_invocation(prepared);
            if supported.is_err() {
                self.context.lose_synchronization();
            }
            return Err(BrowserHostError::Rejected {
                code: if supported.is_err() {
                    BrowserFailureCode::ContextUnavailable
                } else {
                    BrowserFailureCode::UnsupportedOperation
                },
                operation,
            });
        }
        Ok(BoundBrowserInvocation {
            host: self.identity.clone(),
            generation,
            prepared,
        })
    }

    /// Derives the exact challenge only for a valid invocation of this host.
    pub fn permission_resource(
        &self,
        invocation: &BoundBrowserInvocation,
    ) -> Result<BrowserPermissionResource, BrowserHostError> {
        if !Arc::ptr_eq(&self.identity, &invocation.host) {
            return Err(BrowserHostError::Denied);
        }
        if invocation.generation == u64::MAX
            || self.generation.load(Ordering::Acquire) != invocation.generation
        {
            return Err(BrowserHostError::Rejected {
                code: BrowserFailureCode::ContextUnavailable,
                operation: Some(invocation.prepared.request().operation()),
            });
        }
        self.context
            .invocation_permission_resource(&invocation.prepared)
            .map_err(|error| context_error(error, Some(invocation.prepared.request().operation())))
    }

    /// Retires cancellation or denial without invoking the executor.
    pub fn discard(&mut self, invocation: BoundBrowserInvocation) -> Result<(), BrowserHostError> {
        if !Arc::ptr_eq(&self.identity, &invocation.host) {
            return Err(BrowserHostError::Denied);
        }
        self.context
            .discard_invocation(invocation.prepared)
            .map_err(|error| context_error(error, None))
    }

    /// Consumes invocation and authorization, revalidates, then delegates at most once.
    /// Pass the business CancellationToken's `cancelled()` future as `cancelled`.
    /// Deadline, cancellation, panic or invalid output conservatively lose synchronization.
    pub async fn execute_prepared(
        &mut self,
        invocation: BoundBrowserInvocation,
        authorization: BrowserInvocationAuthorization,
        cancelled: impl Future<Output = ()> + Send,
    ) -> Result<ValidatedBrowserOutput, BrowserHostError> {
        let resource = match self.permission_resource(&invocation) {
            Ok(resource) => resource,
            Err(error) => {
                let _ = self.discard(invocation);
                return Err(error);
            }
        };
        let supported = catch_unwind(AssertUnwindSafe(|| {
            self.executor
                .supports(invocation.prepared.request(), invocation.prepared.target())
        }));
        if !matches!(supported, Ok(true)) {
            let _ = self.discard(invocation);
            if supported.is_err() {
                self.context.lose_synchronization();
            }
            return Err(BrowserHostError::Unsupported);
        }
        if authorization.consume(&resource).is_err() {
            let _ = self.discard(invocation);
            return Err(BrowserHostError::Denied);
        }
        let generation = invocation.generation;
        let changed = self.changed.clone().notified_owned();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if self.generation.load(Ordering::Acquire) != generation {
            let _ = self.discard(invocation);
            return Err(BrowserHostError::Denied);
        }
        let target = invocation.prepared.target().clone();
        let operation = invocation.prepared.request().operation();
        let request = self
            .context
            .consume_invocation(invocation.prepared)
            .map_err(|error| context_error(error, Some(operation)))?;
        let operation = request.operation();
        let mut guard = DispatchGuard {
            context: &mut self.context,
            completed: false,
        };
        guard
            .context
            .begin_artifact_scope(resource.clone(), generation);
        let output = {
            // Catch both synchronous future construction and panics while polling the backend.
            let executor = &mut self.executor;
            let context = &mut *guard.context;
            let request = &request;
            let target = &target;
            // Defer the trait call into the future body so construction and polling of
            // the backend future both occur inside the contained poll below.
            let mut future =
                Box::pin(async move { executor.execute(request, target, context).await });
            let contained = poll_fn(|cx| {
                if self.generation.load(Ordering::Acquire) != generation {
                    return Poll::Ready(Err(BrowserHostError::Indeterminate));
                }
                match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(cx))) {
                    Ok(Poll::Ready(output)) => Poll::Ready(Ok(output)),
                    Ok(Poll::Pending) => Poll::Pending,
                    Err(_) => Poll::Ready(Err(BrowserHostError::Indeterminate)),
                }
            });
            tokio::select! {
                biased;
                _ = &mut changed => return Err(BrowserHostError::Indeterminate),
                _ = cancelled => return Err(BrowserHostError::Indeterminate),
                result = tokio::time::timeout(Duration::from_secs(35), contained) => {
                    result.map_err(|_| BrowserHostError::Indeterminate)??
                }
            }
        };
        if self.generation.load(Ordering::Acquire) != generation {
            return Err(BrowserHostError::Indeterminate);
        }
        if matches!(
            &output,
            BrowserOutput::Failure {
                code: BrowserFailureCode::IndeterminateExecution,
                ..
            }
        ) {
            return Err(BrowserHostError::Indeterminate);
        }
        guard
            .context
            .validate_output_identity(&target, &output)
            .map_err(|_| BrowserHostError::Indeterminate)?;
        guard
            .context
            .artifact_store()
            .validate_release(&output, &resource, generation)
            .map_err(|_| BrowserHostError::Indeterminate)?;
        let validated = output
            .validate(Some(operation))
            .map_err(|_| BrowserHostError::Indeterminate)?;
        if matches!(target, BrowserPermissionTarget::Observe { .. }) {
            guard
                .context
                .validate_observation_release(&request, &target)
                .map_err(|_| BrowserHostError::Indeterminate)?;
        }
        if self.generation.load(Ordering::Acquire) != generation {
            return Err(BrowserHostError::Indeterminate);
        }
        guard.completed = true;
        Ok(validated)
    }
}

struct DispatchGuard<'a> {
    context: &'a mut BrowserContext,
    completed: bool,
}
impl Drop for DispatchGuard<'_> {
    fn drop(&mut self) {
        self.context.end_artifact_scope(self.completed);
        if !self.completed {
            self.context.lose_synchronization();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser_executor::{
        BrowserOperation, BrowserPermissionEvaluator, BrowserSuccess,
        ExactBrowserPermissionEvaluator,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Executor {
        calls: Arc<AtomicUsize>,
        panic: bool,
        supported: bool,
    }
    #[async_trait::async_trait]
    impl BrowserExecutor for Executor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            self.supported
        }
        async fn execute(
            &mut self,
            request: &BrowserRequest,
            _: &BrowserPermissionTarget,
            _: &mut BrowserContext,
        ) -> BrowserOutput {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert!(!self.panic, "simulated backend panic");
            BrowserOutput::Ack {
                operation: request.operation(),
                status: BrowserSuccess::Ok,
            }
        }
    }
    fn fixture(panic: bool, supported: bool) -> (BrowserHost<Executor>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            BrowserHost::new(Executor {
                calls: calls.clone(),
                panic,
                supported,
            }),
            calls,
        )
    }
    const WAIT: &str = r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#;

    #[test]
    fn rejection_codes_preserve_operation_and_never_dispatch() {
        let (mut host, calls) = fixture(false, true);
        for (raw, code, operation) in [
            (
                r#"{"protocolVersion":1,"operation":"tab-new"}"#,
                "UnsupportedVersion",
                "tab-new",
            ),
            (
                r#"{"protocolVersion":2,"operation":"fill","extra":"secret"}"#,
                "InvalidRequest",
                "fill",
            ),
            (
                r#"{"protocolVersion":2,"operation":"read","tabRef":"unknown","frameRef":"unknown"}"#,
                "InvalidReference",
                "read",
            ),
        ] {
            let error = host.prepare(raw).err().expect("rejected");
            let output = error.to_prepared(None).into_output();
            let value: serde_json::Value =
                serde_json::from_str(&output.result.content).expect("JSON");
            assert_eq!(value["code"], code);
            assert_eq!(value["operation"], operation);
            assert_eq!(value["outcome"], "notExecuted");
            assert!(!output.result.content.contains("secret"));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn lifecycle_refresh_cannot_hide_concurrent_invalidation_or_overflow() {
        let (mut host, _) = fixture(false, true);
        let lifecycle = host.lifecycle_handle();
        lifecycle.invalidate_pending();
        assert!(host.prepare(WAIT).is_err());
        host.synchronize_context(|_| lifecycle.invalidate_pending());
        assert!(host.prepare(WAIT).is_err());
        host.synchronize_context(|_| {});
        assert!(host.prepare(WAIT).is_ok());
        lifecycle.generation.store(u64::MAX - 1, Ordering::Release);
        lifecycle.invalidate_pending();
        lifecycle.invalidate_pending();
        assert_eq!(lifecycle.generation.load(Ordering::Acquire), u64::MAX);
        host.synchronize_context(|_| {});
        assert!(host.prepare(WAIT).is_err());
    }

    #[test]
    fn panicking_refresh_leaves_context_unavailable() {
        let (mut host, _) = fixture(false, true);
        host.synchronize_context(|_| panic!("broken host discovery"));
        assert!(host.prepare(WAIT).is_err());
        host.synchronize_context(|_| {});
        assert!(host.prepare(WAIT).is_ok());
    }

    struct RevokingExecutor {
        lifecycle: Option<BrowserLifecycleHandle>,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl BrowserExecutor for RevokingExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }

        async fn execute(
            &mut self,
            request: &BrowserRequest,
            _: &BrowserPermissionTarget,
            _: &mut BrowserContext,
        ) -> BrowserOutput {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.lifecycle
                .as_ref()
                .expect("fixture handle")
                .invalidate_pending();
            BrowserOutput::Ack {
                operation: request.operation(),
                status: BrowserSuccess::Ok,
            }
        }
    }

    #[tokio::test]
    async fn invalidation_during_dispatch_suppresses_success_without_replay() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut host = BrowserHost::new(RevokingExecutor {
            lifecycle: None,
            calls: calls.clone(),
        });
        host.executor.lifecycle = Some(host.lifecycle_handle());
        let invocation = host.prepare(WAIT).expect("prepare");
        let resource = host.permission_resource(&invocation).expect("resource");
        let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("approval");
        assert!(matches!(
            host.execute_prepared(invocation, authorization, std::future::pending())
                .await,
            Err(BrowserHostError::Indeterminate)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(host.prepare(WAIT).is_err());
    }

    #[test]
    fn abandoned_approval_cannot_authorize_or_exhaust_ticket_capacity() {
        let (mut host, calls) = fixture(false, true);
        for _ in 0..512 {
            let invocation = host.prepare(WAIT).expect("capacity reclaimed");
            let resource = host.permission_resource(&invocation).expect("resource");
            let evaluator = ExactBrowserPermissionEvaluator::for_resource(resource.clone());
            drop(invocation);
            assert!(evaluator.evaluate(&resource).is_err());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn original_carrier_requires_semantic_admission_and_exact_permission() {
        use talos_core::tool::BrowserRawArguments;

        let (mut host, calls) = fixture(false, true);
        for raw in [
            r#"{"protocolVersion":1,"operation":"wait-milliseconds","milliseconds":1}"#,
            r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":0}"#,
            r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1,"extra":true}"#,
        ] {
            let arguments = BrowserRawArguments::parse_original(raw).expect("valid syntax");
            assert!(matches!(
                host.prepare_original(&arguments),
                Err(BrowserHostError::Rejected { .. })
            ));
        }
        let arguments = BrowserRawArguments::parse_original(WAIT).expect("original");
        let invocation = host.prepare_original(&arguments).expect("admitted");
        let resource = host.permission_resource(&invocation).expect("resource");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("exact approval");
        host.execute_prepared(invocation, authorization, std::future::pending())
            .await
            .expect("one dispatch");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let (mut unsupported, calls) = fixture(false, false);
        assert!(matches!(
            unsupported.prepare_original(&arguments),
            Err(BrowserHostError::Rejected {
                code: BrowserFailureCode::UnsupportedOperation,
                ..
            })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    struct ReadExecutor {
        calls: Arc<AtomicUsize>,
        invalidate: bool,
    }

    #[async_trait::async_trait]
    impl BrowserExecutor for ReadExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }

        async fn execute(
            &mut self,
            _: &BrowserRequest,
            target: &BrowserPermissionTarget,
            context: &mut BrowserContext,
        ) -> BrowserOutput {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.invalidate {
                let BrowserPermissionTarget::Observe { scope, .. } = target else {
                    panic!("read fixture requires observation");
                };
                context
                    .invalidate_frame(&scope.tab, &scope.frame)
                    .expect("navigation after capture");
            }
            BrowserOutput::Read {
                text: "captured-document-content".to_owned(),
                truncated: false,
            }
        }
    }

    #[tokio::test]
    async fn observation_release_discards_content_after_document_change() {
        for invalidate in [false, true] {
            let calls = Arc::new(AtomicUsize::new(0));
            let mut host = BrowserHost::new(ReadExecutor {
                calls: calls.clone(),
                invalidate,
            });
            let tab = host.context_mut().insert_tab().expect("tab");
            let frame = host
                .context_mut()
                .insert_frame(
                    &tab,
                    None,
                    Some(
                        super::super::BrowserOrigin::from_effective_url("https://example.com")
                            .expect("origin"),
                    ),
                    true,
                )
                .expect("frame");
            let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":frame});
            let invocation = host.prepare(&raw.to_string()).expect("prepare");
            let resource = host.permission_resource(&invocation).expect("resource");
            let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .expect("approval");
            let result = host
                .execute_prepared(invocation, authorization, std::future::pending())
                .await;
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            if invalidate {
                assert!(matches!(result, Err(BrowserHostError::Indeterminate)));
                assert!(host.prepare(WAIT).is_err());
            } else {
                assert!(result.is_ok());
                assert!(host.prepare(WAIT).is_ok());
            }
        }
    }

    struct ScreenshotExecutor {
        mismatch: bool,
        navigate: bool,
        calls: Arc<AtomicUsize>,
        published: Option<BrowserArtifactRef>,
    }

    #[async_trait::async_trait]
    impl BrowserExecutor for ScreenshotExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }

        async fn execute(
            &mut self,
            _: &BrowserRequest,
            target: &BrowserPermissionTarget,
            context: &mut BrowserContext,
        ) -> BrowserOutput {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let mut png = std::io::Cursor::new(Vec::new());
            image::DynamicImage::new_rgb8(1, 1)
                .write_to(&mut png, image::ImageFormat::Png)
                .expect("fixture PNG");
            let bytes = png.into_inner();
            let byte_length = bytes.len() as u64;
            let artifact_ref = context.publish_screenshot(bytes, 1, 1).expect("publish");
            self.published = Some(artifact_ref.clone());
            if self.navigate {
                let BrowserPermissionTarget::Observe { scope, .. } = target else {
                    panic!("scope")
                };
                context
                    .invalidate_frame(&scope.tab, &scope.frame)
                    .expect("navigate");
            }
            BrowserOutput::Screenshot {
                artifact_ref,
                mime: super::super::BrowserScreenshotMime::Png,
                width: if self.mismatch { 2 } else { 1 },
                height: 1,
                byte_length,
            }
        }
    }

    #[tokio::test]
    async fn screenshot_host_checks_capture_identity_and_reclaims_rejected_results() {
        for (mismatch, navigate) in [(false, false), (true, false), (false, true)] {
            let calls = Arc::new(AtomicUsize::new(0));
            let mut host = BrowserHost::new(ScreenshotExecutor {
                mismatch,
                navigate,
                calls: calls.clone(),
                published: None,
            });
            let tab = host.context_mut().insert_tab().expect("tab");
            let frame = host
                .context_mut()
                .insert_frame(
                    &tab,
                    None,
                    Some(
                        super::super::BrowserOrigin::from_effective_url("https://example.com")
                            .expect("origin"),
                    ),
                    true,
                )
                .expect("frame");
            let raw = serde_json::json!({"protocolVersion":2,"operation":"screenshot","tabRef":tab,"frameRef":frame});
            let invocation = host.prepare(&raw.to_string()).expect("prepare");
            let resource = host.permission_resource(&invocation).expect("resource");
            let authorization = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .expect("approval");
            let result = host
                .execute_prepared(invocation, authorization, std::future::pending())
                .await;
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            let reference = host.executor.published.as_ref().expect("published");
            if mismatch || navigate {
                assert!(matches!(result, Err(BrowserHostError::Indeterminate)));
                assert!(host.consume_screenshot(reference, &resource).is_err());
            } else {
                let output = result.expect("success");
                assert!(!output.summary().contains("artifact_"));
                let (bytes, _) = host
                    .consume_screenshot(reference, &resource)
                    .expect("consume");
                let image = image::load_from_memory(&bytes).expect("decode");
                assert_eq!((image.width(), image.height()), (1, 1));
                assert!(host.consume_screenshot(reference, &resource).is_err());
            }
        }
    }

    struct PendingExecutor {
        calls: Arc<AtomicUsize>,
        dropped: Arc<AtomicUsize>,
    }
    struct PendingGuard(Arc<AtomicUsize>);
    impl Drop for PendingGuard {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[async_trait::async_trait]
    impl BrowserExecutor for PendingExecutor {
        fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
            true
        }
        async fn execute(
            &mut self,
            _: &BrowserRequest,
            _: &BrowserPermissionTarget,
            _: &mut BrowserContext,
        ) -> BrowserOutput {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let _guard = PendingGuard(self.dropped.clone());
            std::future::pending().await
        }
    }

    #[tokio::test]
    async fn invalidation_wakes_and_drops_a_parked_backend() {
        struct WakeCounter(AtomicUsize);
        impl std::task::Wake for WakeCounter {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let calls = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicUsize::new(0));
        let mut host = BrowserHost::new(PendingExecutor {
            calls: calls.clone(),
            dropped: dropped.clone(),
        });
        let lifecycle = host.lifecycle_handle();
        let invocation = host.prepare(WAIT).expect("prepare");
        let resource = host.permission_resource(&invocation).expect("resource");
        let approval = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("approval");
        let mut dispatch =
            Box::pin(host.execute_prepared(invocation, approval, std::future::pending()));
        let counter = Arc::new(WakeCounter(AtomicUsize::new(0)));
        let waker = std::task::Waker::from(counter.clone());
        let mut cx = std::task::Context::from_waker(&waker);
        assert!(dispatch.as_mut().poll(&mut cx).is_pending());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(dropped.load(Ordering::SeqCst), 0);
        lifecycle.invalidate_pending();
        assert!(counter.0.load(Ordering::SeqCst) > 0);
        assert!(matches!(
            dispatch.as_mut().poll(&mut cx),
            Poll::Ready(Err(BrowserHostError::Indeterminate))
        ));
        drop(dispatch);
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(host.prepare(WAIT).is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn deadline_and_midflight_cancellation_drop_backend_without_replay() {
        for cancel_after in [Some(Duration::from_secs(1)), None] {
            let calls = Arc::new(AtomicUsize::new(0));
            let dropped = Arc::new(AtomicUsize::new(0));
            let mut host = BrowserHost::new(PendingExecutor {
                calls: calls.clone(),
                dropped: dropped.clone(),
            });
            let invocation = host.prepare(WAIT).expect("prepare");
            let resource = host.permission_resource(&invocation).expect("resource");
            let approval = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .expect("approval");
            let started = tokio::time::Instant::now();
            let cancelled = async move {
                match cancel_after {
                    Some(delay) => tokio::time::sleep(delay).await,
                    None => std::future::pending().await,
                }
            };
            assert!(matches!(
                host.execute_prepared(invocation, approval, cancelled).await,
                Err(BrowserHostError::Indeterminate)
            ));
            assert_eq!(
                started.elapsed(),
                cancel_after.unwrap_or(Duration::from_secs(35))
            );
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(dropped.load(Ordering::SeqCst), 1);
            assert!(host.prepare(WAIT).is_err());
        }
    }

    #[tokio::test]
    async fn dropping_dispatch_after_first_poll_revokes_context() {
        let calls = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicUsize::new(0));
        let mut host = BrowserHost::new(PendingExecutor {
            calls: calls.clone(),
            dropped: dropped.clone(),
        });
        let invocation = host.prepare(WAIT).expect("prepare");
        let resource = host.permission_resource(&invocation).expect("resource");
        let approval = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("approval");
        let mut dispatch =
            Box::pin(host.execute_prepared(invocation, approval, std::future::pending()));
        poll_fn(|cx| {
            assert!(dispatch.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        drop(dispatch);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        assert!(host.prepare(WAIT).is_err());
    }
    fn approve(
        host: &BrowserHost<Executor>,
        invocation: &BoundBrowserInvocation,
    ) -> BrowserInvocationAuthorization {
        let resource = host.permission_resource(invocation).expect("resource");
        ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("approval")
    }

    #[tokio::test]
    async fn fixed_executor_dispatches_exactly_once_after_matching_approval() {
        let (mut host, calls) = fixture(false, true);
        let invocation = host.prepare(WAIT).expect("prepare");
        let approval = approve(&host, &invocation);
        let result = host
            .execute_prepared(invocation, approval, std::future::pending())
            .await
            .expect("execute");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(result.summary().contains("Ack"));
    }

    #[tokio::test]
    async fn backend_support_lost_during_approval_prevents_dispatch() {
        use std::sync::atomic::AtomicBool;

        struct ChangingExecutor {
            supported: Arc<AtomicBool>,
            calls: Arc<AtomicUsize>,
        }
        #[async_trait::async_trait]
        impl BrowserExecutor for ChangingExecutor {
            fn supports(&self, _: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
                self.supported.load(Ordering::SeqCst)
            }

            async fn execute(
                &mut self,
                request: &BrowserRequest,
                _: &BrowserPermissionTarget,
                _: &mut BrowserContext,
            ) -> BrowserOutput {
                self.calls.fetch_add(1, Ordering::SeqCst);
                BrowserOutput::Ack {
                    operation: request.operation(),
                    status: BrowserSuccess::Ok,
                }
            }
        }

        let supported = Arc::new(AtomicBool::new(true));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut host = BrowserHost::new(ChangingExecutor {
            supported: supported.clone(),
            calls: calls.clone(),
        });
        let invocation = host.prepare(WAIT).expect("prepare");
        let resource = host.permission_resource(&invocation).expect("resource");
        let approval = ExactBrowserPermissionEvaluator::for_resource(resource.clone())
            .evaluate(&resource)
            .expect("approval");
        supported.store(false, Ordering::SeqCst);

        assert!(matches!(
            host.execute_prepared(invocation, approval, std::future::pending())
                .await,
            Err(BrowserHostError::Unsupported)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(
            ExactBrowserPermissionEvaluator::for_resource(resource.clone())
                .evaluate(&resource)
                .is_err()
        );
    }

    #[tokio::test]
    async fn same_payload_different_invocation_and_foreign_host_never_execute() {
        let (mut host, calls) = fixture(false, true);
        let first = host.prepare(WAIT).expect("first");
        let second = host.prepare(WAIT).expect("second");
        let approval = approve(&host, &first);
        assert!(matches!(
            host.execute_prepared(second, approval, std::future::pending())
                .await,
            Err(BrowserHostError::Denied)
        ));
        let approval = approve(&host, &first);
        let (mut foreign, foreign_calls) = fixture(false, true);
        assert!(matches!(
            foreign
                .execute_prepared(first, approval, std::future::pending())
                .await,
            Err(BrowserHostError::Denied)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(foreign_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn stale_and_canceled_calls_never_poll_executor() {
        let (mut host, calls) = fixture(false, true);
        let invocation = host.prepare(WAIT).expect("prepare");
        let approval = approve(&host, &invocation);
        host.context_mut().replace_session();
        assert!(
            host.execute_prepared(invocation, approval, std::future::pending())
                .await
                .is_err()
        );
        let invocation = host.prepare(WAIT).expect("prepare");
        let approval = approve(&host, &invocation);
        assert!(
            host.execute_prepared(invocation, approval, std::future::ready(()))
                .await
                .is_err()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(host.prepare(WAIT).is_err());
    }

    #[tokio::test]
    async fn panic_is_indeterminate_and_disables_context_without_retry() {
        let (mut host, calls) = fixture(true, true);
        let invocation = host.prepare(WAIT).expect("prepare");
        let approval = approve(&host, &invocation);
        assert!(matches!(
            host.execute_prepared(invocation, approval, std::future::pending())
                .await,
            Err(BrowserHostError::Indeterminate)
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(host.prepare(WAIT).is_err());
    }

    #[test]
    fn unsupported_and_malformed_calls_fail_before_permission_resource() {
        let (mut host, calls) = fixture(false, false);
        assert!(matches!(
            host.prepare(WAIT),
            Err(BrowserHostError::Rejected {
                code: BrowserFailureCode::UnsupportedOperation,
                ..
            })
        ));
        assert!(matches!(
            host.prepare("{}"),
            Err(BrowserHostError::Rejected { .. })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            BrowserOperation::WaitMilliseconds.resource_class(),
            crate::browser_executor::BrowserResourceClass::SessionWait
        );
    }
}
