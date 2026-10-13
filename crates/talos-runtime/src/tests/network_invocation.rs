//! Real SDK actor/Agent dispatch with a native offline prepared fixture.
use super::*;
use talos_core::tool::{
    PreparedNetworkToolInvocation, ToolExecutionOutput, ToolNetworkAuthority, ToolNetworkEndpoint,
    ToolNetworkManifest, ToolNetworkPreparationError, ToolNetworkScope,
};

struct NetworkTool {
    configuration_id: uuid::Uuid,
    executed: Arc<AtomicUsize>,
}
struct PreparedNetwork {
    manifest: ToolNetworkManifest,
    profile: Vec<ToolPermissionFacet>,
    executed: Arc<AtomicUsize>,
    input: Value,
}
#[async_trait]
impl AgentTool for NetworkTool {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "offline SDK network carrier fixture"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})
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
        let scope = ToolNetworkScope::new(
            self.configuration_id,
            0,
            vec![ToolNetworkEndpoint::new(
                "https",
                "search.example",
                8443,
                "POST",
                "/search",
            )?],
            None,
            vec![],
            0,
        )?;
        Ok(Some(Box::new(PreparedNetwork {
            manifest: ToolNetworkManifest::new(scope),
            profile: vec![ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "search.example",
                ToolResourceKind::Domain,
            )],
            executed: self.executed.clone(),
            input: input.clone(),
        })))
    }
    async fn execute(&self, _: Value) -> ToolResult {
        panic!("SDK legacy route is forbidden")
    }
}
#[async_trait]
impl PreparedNetworkToolInvocation for PreparedNetwork {
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
        authority
            .check(
                self.manifest.invocation_id(),
                &self.manifest.scope().endpoints()[0],
            )
            .expect("Agent minted live authority");
        assert_eq!(
            self.input,
            serde_json::json!({"query":"SDK-approved-query"})
        );
        self.executed.fetch_add(1, Ordering::SeqCst);
        ToolExecutionOutput::success("SDK prepared execution")
    }
}

struct ScopedNetworkApproval(Arc<AtomicUsize>);
#[async_trait]
impl ApprovalHandler for ScopedNetworkApproval {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("argument-only approval cannot authorize a prepared network invocation")
    }
    async fn request_scoped_approval(
        &self,
        name: &str,
        _: &Value,
        _: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        assert_eq!(name, "web_search");
        let scope = preview.network_scope().expect("complete network scope");
        let endpoint = &scope.endpoints()[0];
        assert_eq!(
            (
                endpoint.scheme(),
                endpoint.host(),
                endpoint.port(),
                endpoint.method(),
                endpoint.path()
            ),
            ("https", "search.example", 8443, "POST", "/search")
        );
        self.0.fetch_add(1, Ordering::SeqCst);
        ApprovalChoice::ApproveOnce
    }
}

#[tokio::test]
async fn sdk_network_dispatch_requires_scoped_host_approval() {
    for scoped in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        let executed = Arc::new(AtomicUsize::new(0));
        let scoped_approvals = Arc::new(AtomicUsize::new(0));
        let legacy_records = Arc::new(StdMutex::new(Vec::new()));
        let handler: Arc<dyn ApprovalHandler> = if scoped {
            Arc::new(ScopedNetworkApproval(scoped_approvals.clone()))
        } else {
            Arc::new(RecordingApprovalHandler::new(
                ApprovalChoice::AlwaysApprove,
                legacy_records.clone(),
            ))
        };
        let provider = Arc::new(
            MockProvider::new()
                .with_tool_call(
                    "web_search",
                    serde_json::json!({"query":"SDK-approved-query"}),
                )
                .with_response("done"),
        );
        let mut runtime = RuntimeBuilder::new()
            .provider(provider)
            .workspace_root(root.path())
            .approval_handler(handler)
            .tool(Arc::new(NetworkTool {
                configuration_id: uuid::Uuid::new_v4(),
                executed: executed.clone(),
            }))
            .build()
            .expect("SDK runtime");
        runtime.submit("search").await.expect("submit");
        tokio::time::timeout(
            Duration::from_secs(5),
            collect_until_turn_completed(&mut runtime),
        )
        .await
        .expect("bounded actor run")
        .expect("completion");
        assert_eq!(executed.load(Ordering::SeqCst), usize::from(scoped));
        assert_eq!(scoped_approvals.load(Ordering::SeqCst), usize::from(scoped));
        assert!(legacy_records.lock().expect("records").is_empty());
        runtime.shutdown().await.expect("shutdown");
    }
}
