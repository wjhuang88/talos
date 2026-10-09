//! External opt-in consumer. This executes a deterministic backend, not a browser.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use talos_runtime::{AgentTool, BrowserRawArguments};
use talos_tools::browser_executor::*;

struct Executor(Arc<AtomicUsize>);

#[async_trait::async_trait]
impl BrowserExecutor for Executor {
    fn supports(&self, request: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
        request.operation() == BrowserOperation::WaitMilliseconds
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut host = BrowserHost::new(Executor(calls.clone()));
    let raw = r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#;
    let invocation = host.prepare(raw)?;
    let resource = host.permission_resource(&invocation)?;
    assert!(DenyBrowserPermissionEvaluator.evaluate(&resource).is_err());
    let authorization =
        ExactBrowserPermissionEvaluator::for_resource(resource.clone()).evaluate(&resource)?;
    let output = host
        .execute_prepared(invocation, authorization, std::future::pending())
        .await?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(output.model_json())?["status"],
        "ok"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // The managed adapter implements the Runtime-exported trait without internal imports.
    let managed = ManagedBrowserTool::new(host);
    let tool: &dyn AgentTool = &managed;
    assert_eq!(tool.name(), "browser");
    let arguments = BrowserRawArguments::parse_original(raw)?;
    let prepared = tool.prepare_original_invocation(arguments).await?;
    assert!(prepared.authorize().await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    println!("external browser host: explicit authorization and default denial passed");
    Ok(())
}
