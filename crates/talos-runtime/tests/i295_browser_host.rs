//! Real embedded Runtime composition with a deterministic executor, not a native browser.

use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use talos_runtime::{
    AgentEvent, BrowserRawArguments, LanguageModel, Message, ProviderInvocationEvent,
    ProviderInvocationStream, ProviderProgress, ProviderResult, Receiver, RuntimeBuilder,
    StopReason, ToolDefinition, ToolProtocol, TurnCompletionStatus, Usage,
    collect_until_turn_completed,
};
use talos_tools::browser_executor::*;
use tokio::sync::mpsc;

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

struct Approve(Arc<AtomicUsize>);

struct PendingExecutor {
    started: Arc<tokio::sync::Notify>,
    calls: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
}

struct ExecutionGuard(Arc<AtomicUsize>);
impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[async_trait]
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
        let _guard = ExecutionGuard(self.dropped.clone());
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        std::future::pending().await
    }
}

#[tokio::test]
async fn runtime_shutdown_drops_browser_execution_and_invalidates_host() {
    let started = Arc::new(tokio::sync::Notify::new());
    let calls = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    let tool = Arc::new(
        ManagedBrowserTool::new(BrowserHost::new(PendingExecutor {
            started: started.clone(),
            calls: calls.clone(),
            dropped: dropped.clone(),
        }))
        .with_permission_resolver(Arc::new(Approve(Arc::new(AtomicUsize::new(0))))),
    );
    let runtime = RuntimeBuilder::new()
        .provider(Arc::new(Model {
            requests: AtomicUsize::new(0),
            registered: true,
            allowed: true,
        }))
        .tool(tool.clone())
        .build()
        .expect("runtime");
    runtime
        .submit("start pending browser call")
        .await
        .expect("submit");
    tokio::time::timeout(std::time::Duration::from_secs(5), started.notified())
        .await
        .expect("backend started");
    tokio::time::timeout(std::time::Duration::from_secs(10), runtime.shutdown())
        .await
        .expect("bounded shutdown")
        .expect("shutdown");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    let arguments = BrowserRawArguments::parse_original(
        r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#,
    )
    .expect("arguments");
    assert!(tool.prepare_invocation(&arguments).await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[async_trait]
impl BrowserPermissionResolver for Approve {
    async fn resolve(
        &self,
        resource: &BrowserPermissionResource,
    ) -> Result<BrowserInvocationAuthorization, BrowserAuthorizationError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        ExactBrowserPermissionEvaluator::for_resource(resource.clone()).evaluate(resource)
    }
}

struct Model {
    requests: AtomicUsize,
    registered: bool,
    allowed: bool,
}

#[async_trait]
impl LanguageModel for Model {
    async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        panic!("must use integrity-aware ingress")
    }
    async fn stream_with_invocation_integrity(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        _: ToolProtocol,
        _: mpsc::UnboundedSender<ProviderProgress>,
        protected: &[String],
    ) -> ProviderResult<ProviderInvocationStream> {
        assert_eq!(
            protected.iter().any(|name| name == "browser"),
            self.registered
        );
        assert_eq!(
            tools.iter().any(|tool| tool.name == "browser"),
            self.registered
        );
        let (tx, rx) = mpsc::channel(4);
        if self.registered && self.requests.fetch_add(1, Ordering::SeqCst) == 0 {
            tx.send(ProviderInvocationEvent::BrowserToolCall {
                id: "host-call".into(),
                name: "browser".into(),
                arguments: BrowserRawArguments::parse_original(
                    r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#,
                )
                .expect("valid raw arguments"),
            })
            .await
            .expect("receiver");
            tx.send(ProviderInvocationEvent::Legacy(AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            }))
            .await
            .expect("receiver");
        } else {
            if self.registered {
                let result = messages
                    .iter()
                    .find_map(|message| match message {
                        Message::Tool { result } if result.tool_use_id == "host-call" => {
                            Some(result)
                        }
                        _ => None,
                    })
                    .expect("paired result");
                assert_eq!(result.is_error, !self.allowed);
            }
            tx.send(ProviderInvocationEvent::Legacy(AgentEvent::TextDelta {
                delta: "done".into(),
            }))
            .await
            .expect("receiver");
            tx.send(ProviderInvocationEvent::Legacy(AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            }))
            .await
            .expect("receiver");
        }
        Ok(ProviderInvocationStream::original(rx))
    }
}

#[tokio::test]
async fn runtime_browser_is_opt_in_and_requires_exact_browser_approval() {
    for (registered, allowed) in [(false, false), (true, false), (true, true)] {
        let executions = Arc::new(AtomicUsize::new(0));
        let approvals = Arc::new(AtomicUsize::new(0));
        let mut builder = RuntimeBuilder::new().provider(Arc::new(Model {
            requests: AtomicUsize::new(0),
            registered,
            allowed,
        }));
        if registered {
            let mut tool = ManagedBrowserTool::new(BrowserHost::new(Executor(executions.clone())));
            if allowed {
                tool = tool.with_permission_resolver(Arc::new(Approve(approvals.clone())));
            }
            builder = builder.tool(Arc::new(tool));
        }
        let mut runtime = builder.build().expect("runtime builds");
        runtime.submit("exercise bound host").await.expect("submit");
        let terminal = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            collect_until_turn_completed(&mut runtime),
        )
        .await
        .expect("bounded turn")
        .expect("terminal");
        assert!(matches!(terminal, TurnCompletionStatus::Success { .. }));
        assert_eq!(executions.load(Ordering::SeqCst), usize::from(allowed));
        assert_eq!(approvals.load(Ordering::SeqCst), usize::from(allowed));
        runtime.shutdown().await.expect("shutdown");
    }
}
