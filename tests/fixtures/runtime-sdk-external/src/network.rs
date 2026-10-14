//! One direct Talos dependency, independently locked: new native prepared network API.
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use talos_runtime::{
    AgentTool, ApprovalChoice, ApprovalHandler, GrantPreview, PreparedNetworkToolInvocation,
    RuntimeBuilder, ToolExecutionOutput, ToolNature, ToolNetworkAuthority, ToolNetworkEndpoint,
    ToolNetworkManifest, ToolNetworkPreparationError, ToolNetworkScope, ToolPermissionFacet,
    ToolResourceKind, ToolResult, collect_until_turn_completed,
};

struct Fixture {
    scope: ToolNetworkScope,
    executed: Arc<AtomicUsize>,
}
struct Prepared {
    manifest: ToolNetworkManifest,
    facets: Vec<ToolPermissionFacet>,
    input: Value,
    executed: Arc<AtomicUsize>,
}
#[async_trait]
impl AgentTool for Fixture {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "offline externally compiled network carrier"
    }
    fn parameters(&self) -> Value {
        json!({"type":"object","properties":{"query":{"type":"string"}},"required":["query"]})
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
        Ok(Some(Box::new(Prepared {
            manifest: ToolNetworkManifest::new(self.scope.clone()),
            facets: vec![ToolPermissionFacet::with_resource(
                ToolNature::Network,
                "search.example",
                ToolResourceKind::Domain,
            )],
            input: input.clone(),
            executed: self.executed.clone(),
        })))
    }
    async fn execute(&self, _: Value) -> ToolResult {
        panic!("legacy dispatch forbidden")
    }
}
#[async_trait]
impl PreparedNetworkToolInvocation for Prepared {
    fn manifest(&self) -> &ToolNetworkManifest {
        &self.manifest
    }
    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
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
            .expect("live admitted authority");
        assert_eq!(self.input, json!({"query":"offline fixture"}));
        self.executed.fetch_add(1, Ordering::SeqCst);
        ToolExecutionOutput::success("external prepared invocation")
    }
}
struct ScopedHost;
#[async_trait]
impl ApprovalHandler for ScopedHost {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("unscoped callback")
    }
    async fn request_scoped_approval(
        &self,
        _: &str,
        _: &Value,
        _: &[String],
        preview: &GrantPreview,
    ) -> ApprovalChoice {
        let endpoint = &preview.network_scope().expect("network scope").endpoints()[0];
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
        ApprovalChoice::ApproveOnce
    }
}
pub async fn verify() -> Result<()> {
    let scope = ToolNetworkScope::new_configuration(
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
    let executed = Arc::new(AtomicUsize::new(0));
    let root = tempfile::tempdir()?;
    let mut runtime = RuntimeBuilder::new()
        .provider(Arc::new(
            crate::provider::FixtureProvider::new()
                .with_tool_call("web_search", json!({"query":"offline fixture"}))
                .with_response("done"),
        ))
        .workspace_root(root.path())
        .approval_handler(Arc::new(ScopedHost))
        .tool(Arc::new(Fixture {
            scope,
            executed: executed.clone(),
        }))
        .build()?;
    runtime.submit("offline search fixture").await?;
    tokio::time::timeout(
        Duration::from_secs(5),
        collect_until_turn_completed(&mut runtime),
    )
    .await?
    .ok_or_else(|| anyhow::anyhow!("no completion"))?;
    assert_eq!(executed.load(Ordering::SeqCst), 1);
    runtime.shutdown().await?;
    Ok(())
}
