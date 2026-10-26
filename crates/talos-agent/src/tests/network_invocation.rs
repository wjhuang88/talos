//! Real Agent-loop dispatch fixtures for the ADR-090 opt-in carrier; no HTTP/DNS.
use super::*;
mod auto_review;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{
    PreparedNetworkToolInvocation, ToolNetworkAuthority, ToolNetworkEndpoint, ToolNetworkManifest,
    ToolNetworkPreparationError, ToolNetworkScope,
};
use talos_permission::{
    InteractionCapability, PermissionContext, PermissionMode, PermissionSessionState,
};

struct NetworkFixture {
    configuration_id: uuid::Uuid,
    prepared: Arc<AtomicUsize>,
    executed: Arc<Mutex<Vec<Value>>>,
    missing: bool,
    revoke: Option<Arc<PermissionSessionState>>,
}

struct PreparedFixture {
    input: Value,
    manifest: ToolNetworkManifest,
    profile: Vec<ToolPermissionFacet>,
    executed: Arc<Mutex<Vec<Value>>>,
    revoke: Option<Arc<PermissionSessionState>>,
}

struct PendingNetworkTool {
    inner: NetworkFixture,
    started: Arc<tokio::sync::Notify>,
    dropped: Arc<AtomicUsize>,
}
struct PendingNetworkInvocation {
    inner: Box<dyn PreparedNetworkToolInvocation>,
    started: Arc<tokio::sync::Notify>,
    dropped: Arc<AtomicUsize>,
}
struct DropReceipt(Arc<AtomicUsize>);
impl Drop for DropReceipt {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
#[async_trait]
impl AgentTool for PendingNetworkTool {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn description(&self) -> &str {
        self.inner.description()
    }
    fn parameters(&self) -> Value {
        self.inner.parameters()
    }
    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }
    fn requires_network_invocation(&self) -> bool {
        true
    }
    fn prepare_network_invocation(
        &self,
        input: &Value,
    ) -> Result<Option<Box<dyn PreparedNetworkToolInvocation>>, ToolNetworkPreparationError> {
        let inner = self
            .inner
            .prepare_network_invocation(input)?
            .expect("fixture");
        Ok(Some(Box::new(PendingNetworkInvocation {
            inner,
            started: self.started.clone(),
            dropped: self.dropped.clone(),
        })))
    }
    async fn execute(&self, _: Value) -> ToolExecutionResult {
        panic!("legacy route")
    }
}
#[async_trait]
impl PreparedNetworkToolInvocation for PendingNetworkInvocation {
    fn manifest(&self) -> &ToolNetworkManifest {
        self.inner.manifest()
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        self.inner.permission_profile()
    }
    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput {
        let _receipt = DropReceipt(self.dropped.clone());
        authority
            .check(
                self.manifest().invocation_id(),
                &self.manifest().scope().endpoints()[0],
            )
            .expect("admitted");
        self.started.notify_one();
        std::future::pending().await
    }
}

#[tokio::test]
async fn network_prepared_caller_cancellation_drops_the_owned_pending_future() {
    let started = Arc::new(tokio::sync::Notify::new());
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(PendingNetworkTool {
        inner: NetworkFixture {
            configuration_id: uuid::Uuid::new_v4(),
            prepared: Arc::new(AtomicUsize::new(0)),
            executed: Arc::new(Mutex::new(Vec::new())),
            missing: false,
            revoke: None,
        },
        started: started.clone(),
        dropped: dropped.clone(),
    }));
    let agent = Agent::with_security(
        Arc::new(MockModel::new(events(
            serde_json::json!({"query":"private"}),
        ))),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
    )
    .with_permission_pipeline(
        Arc::new(PermissionSessionState::new(PermissionEngine::new())),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: Arc::new(Mutex::new(Vec::new())),
        })),
    );
    let task = tokio::spawn(async move { agent.run("search".into()).await });
    tokio::time::timeout(std::time::Duration::from_secs(2), started.notified())
        .await
        .expect("prepared execution starts");
    task.abort();
    assert!(task.await.expect_err("cancelled").is_cancelled());
    assert_eq!(
        dropped.load(Ordering::SeqCst),
        1,
        "owned prepared future dropped once; no HTTP cancellation proof"
    );
}

#[async_trait]
impl AgentTool for NetworkFixture {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "offline prepared search fixture"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"],"additionalProperties":false})
    }
    fn nature(&self) -> ToolNature {
        ToolNature::Network
    }
    fn requires_network_invocation(&self) -> bool {
        true
    }
    fn prepare_network_invocation(
        &self,
        input: &Value,
    ) -> Result<Option<Box<dyn PreparedNetworkToolInvocation>>, ToolNetworkPreparationError> {
        self.prepared.fetch_add(1, Ordering::SeqCst);
        if self.missing {
            return Ok(None);
        }
        let destination =
            ToolNetworkEndpoint::new("https", "search.example", 443, "GET", "/search")?;
        let scope =
            ToolNetworkScope::new(self.configuration_id, 0, vec![destination], None, vec![], 0)?;
        Ok(Some(Box::new(PreparedFixture {
            input: input.clone(),
            manifest: ToolNetworkManifest::new(scope),
            profile: vec![ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "search.example",
                ToolResourceKind::Domain,
            )],
            executed: self.executed.clone(),
            revoke: self.revoke.clone(),
        })))
    }
    async fn execute(&self, _: Value) -> ToolExecutionResult {
        panic!("legacy execution is forbidden")
    }
}

#[async_trait]
impl PreparedNetworkToolInvocation for PreparedFixture {
    fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.profile
    }
    async fn execute(
        self: Box<Self>,
        authority: Arc<dyn ToolNetworkAuthority>,
    ) -> ToolExecutionOutput {
        let destination = &self.manifest.scope().endpoints()[0];
        if let Err(error) = authority.check(self.manifest.invocation_id(), destination) {
            return ToolExecutionOutput::from_result(ToolExecutionResult::error(error.to_string()));
        }
        self.executed.lock().await.push(self.input);
        if let Some(state) = self.revoke {
            state.clear().expect("fixture revocation");
            assert!(
                authority
                    .check(self.manifest.invocation_id(), destination)
                    .is_err(),
                "future request start must fail"
            );
        }
        ToolExecutionOutput::success("prepared network invocation executed")
    }
}

fn events(input: Value) -> Vec<Vec<AgentEvent>> {
    vec![
        vec![
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "network-call".into(),
                    name: "web_search".into(),
                    input,
                },
                provenance: Default::default(),
                summary_fields: vec!["query".into()],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            },
        ],
        text_done_events("done"),
    ]
}

struct RevokeAfterPermission(Arc<PermissionSessionState>);
#[async_trait]
impl HookHandler for RevokeAfterPermission {
    fn name(&self) -> &str {
        "revoke-network-after-admission"
    }
    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::AfterPermissionCheck]
    }
    async fn on_event(&self, _: &HookContext, _: &mut HookEvent<'_>) -> HookResult {
        self.0.clear().expect("revoke before dispatch");
        HookResult::Continue
    }
}

#[tokio::test]
async fn network_prepared_hook_revocation_is_checked_before_first_logical_start() {
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let executed = Arc::new(Mutex::new(Vec::new()));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkFixture {
        configuration_id: uuid::Uuid::new_v4(),
        prepared: Arc::new(AtomicUsize::new(0)),
        executed: executed.clone(),
        missing: false,
        revoke: None,
    }));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(RevokeAfterPermission(state.clone())));
    let (model, captured) = CapturingMockModel::new(events(serde_json::json!({"query":"private"})));
    let agent = Agent::with_security_and_hooks(
        Arc::new(model),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
        Arc::new(hooks),
    )
    .with_permission_pipeline(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: requests.clone(),
        })),
    );
    agent
        .run("search".into())
        .await
        .expect("revocation reported");
    assert_eq!(requests.lock().await.len(), 1);
    assert!(executed.lock().await.is_empty());
    assert!(format!("{:?}", captured.lock().await).contains("network_authority_invalidated"));
}

#[tokio::test]
async fn network_prepared_dispatch_uses_approved_owned_input_and_live_authority() {
    let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let prepared = Arc::new(AtomicUsize::new(0));
    let executed = Arc::new(Mutex::new(Vec::new()));
    let input = serde_json::json!({"query":"approved query"});
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkFixture {
        configuration_id: uuid::Uuid::new_v4(),
        prepared: prepared.clone(),
        executed: executed.clone(),
        missing: false,
        revoke: Some(state.clone()),
    }));
    let agent = Agent::with_security(
        Arc::new(MockModel::new(events(input.clone()))),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
    )
    .with_permission_pipeline(
        state,
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: requests.clone(),
        })),
    );
    agent.run("search".into()).await.expect("real Agent run");
    assert_eq!(prepared.load(Ordering::SeqCst), 1);
    assert_eq!(*executed.lock().await, vec![input]);
    let requests = requests.lock().await;
    assert_eq!(
        requests.len(),
        1,
        "must exercise Ask/Once, not fixture policy Allow"
    );
    let scope = requests[0]
        .preview
        .network_scope()
        .expect("network preview");
    let destination = &scope.endpoints()[0];
    assert_eq!(
        (
            destination.host(),
            destination.port(),
            destination.method(),
            destination.path()
        ),
        ("search.example", 443, "GET", "/search")
    );
}

#[tokio::test]
async fn network_prepared_missing_pipeline_or_preparation_never_executes_legacy() {
    for (pipeline, missing) in [(false, false), (true, true)] {
        let prepared = Arc::new(AtomicUsize::new(0));
        let executed = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(NetworkFixture {
            configuration_id: uuid::Uuid::new_v4(),
            prepared: prepared.clone(),
            executed: executed.clone(),
            missing,
            revoke: None,
        }));
        let (model, captured) =
            CapturingMockModel::new(events(serde_json::json!({"query":"private"})));
        let mut agent =
            Agent::with_security(Arc::new(model), registry, None, None, PathBuf::from("/tmp"));
        if pipeline {
            agent = agent.with_permission_pipeline(
                Arc::new(PermissionSessionState::new(PermissionEngine::new())),
                PermissionContext::new(
                    PermissionMode::Interactive,
                    InteractionCapability::Available,
                ),
                Some(Arc::new(CaptureApprovalProjection {
                    requests: Arc::new(Mutex::new(Vec::new())),
                })),
            );
        }
        agent
            .run("search".into())
            .await
            .expect("error returned to model");
        assert!(executed.lock().await.is_empty());
        assert_eq!(prepared.load(Ordering::SeqCst), usize::from(pipeline));
        let history = format!("{:?}", captured.lock().await);
        assert!(history.contains(if pipeline {
            "network_preparation_unavailable"
        } else {
            "requires the Agent permission pipeline"
        }));
    }
}

#[tokio::test]
async fn network_prepared_deny_prevents_dispatch() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let prepared = Arc::new(AtomicUsize::new(0));
    let executed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkFixture {
        configuration_id: uuid::Uuid::new_v4(),
        prepared,
        executed: executed.clone(),
        missing: false,
        revoke: None,
    }));
    let engine = PermissionEngine::from_rules(vec![talos_permission::PermissionRule::new_nature(
        ToolNature::Network,
        Some("search.example".into()),
        Some(talos_permission::ResourceKind::Domain),
        PermissionDecision::Deny("fixture deny".into()),
    )]);
    let agent = Agent::with_security(
        Arc::new(MockModel::new(events(
            serde_json::json!({"query":"private"}),
        ))),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
    )
    .with_permission_pipeline(
        Arc::new(PermissionSessionState::new(engine)),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: requests.clone(),
        })),
    );
    agent.run("search".into()).await.expect("deny reported");
    assert!(executed.lock().await.is_empty());
    assert!(
        requests.lock().await.is_empty(),
        "Deny cannot reach approval"
    );
}

#[tokio::test]
async fn network_prepared_final_permission_hook_prevents_dispatch() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let executed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkFixture {
        configuration_id: uuid::Uuid::new_v4(),
        prepared: Arc::new(AtomicUsize::new(0)),
        executed: executed.clone(),
        missing: false,
        revoke: None,
    }));
    let mut hooks = HookRegistry::new();
    hooks.register(Arc::new(SkipFinalPermissionHook));
    let agent = Agent::with_security_and_hooks(
        Arc::new(MockModel::new(events(
            serde_json::json!({"query":"private"}),
        ))),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
        Arc::new(hooks),
    )
    .with_permission_pipeline(
        Arc::new(PermissionSessionState::new(PermissionEngine::new())),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: requests.clone(),
        })),
    );
    agent
        .run("search".into())
        .await
        .expect("hook denial reported");
    assert_eq!(
        requests.lock().await.len(),
        1,
        "approval was resolved before the final hook"
    );
    assert!(
        executed.lock().await.is_empty(),
        "an admitted authority does not bypass the hook"
    );
}

#[tokio::test]
async fn network_prepared_call_id_replay_does_not_repeat_execution() {
    let executed = Arc::new(Mutex::new(Vec::new()));
    let first_input = serde_json::json!({"query":"first"});
    let mut responses = events(first_input.clone());
    responses.insert(
        1,
        events(serde_json::json!({"query":"different"})).remove(0),
    );
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(NetworkFixture {
        configuration_id: uuid::Uuid::new_v4(),
        prepared: Arc::new(AtomicUsize::new(0)),
        executed: executed.clone(),
        missing: false,
        revoke: None,
    }));
    let agent = Agent::with_security(
        Arc::new(MockModel::new(responses)),
        registry,
        None,
        None,
        PathBuf::from("/tmp"),
    )
    .with_permission_pipeline(
        Arc::new(PermissionSessionState::new(PermissionEngine::new())),
        PermissionContext::new(
            PermissionMode::Interactive,
            InteractionCapability::Available,
        ),
        Some(Arc::new(CaptureApprovalProjection {
            requests: Arc::new(Mutex::new(Vec::new())),
        })),
    );
    agent
        .run("search".into())
        .await
        .expect("replay safely reported");
    assert_eq!(*executed.lock().await, vec![first_input]);
}
