//! Deterministic host composition, not a native-browser implementation.
//! Run: cargo run -p talos-tools --locked --features network --example i295_browser_host

use talos_tools::browser_executor::*;

struct FixtureExecutor {
    root: BrowserFrameRef,
    child: BrowserFrameRef,
}

#[async_trait::async_trait]
impl BrowserExecutor for FixtureExecutor {
    fn supports(&self, request: &BrowserRequest, _: &BrowserPermissionTarget) -> bool {
        matches!(
            request.operation(),
            BrowserOperation::FrameTree | BrowserOperation::Press
        )
    }

    async fn execute(
        &mut self,
        request: &BrowserRequest,
        target: &BrowserPermissionTarget,
        context: &mut BrowserContext,
    ) -> BrowserOutput {
        match target {
            BrowserPermissionTarget::FrameInventory { .. } => BrowserOutput::Frames {
                entries: vec![
                    BrowserFrameEntry {
                        frame_ref: self.root.clone(),
                        parent_frame_ref: None,
                        origin: "https://parent.example".into(),
                        readiness: BrowserReadiness::Ready,
                    },
                    BrowserFrameEntry {
                        frame_ref: self.child.clone(),
                        parent_frame_ref: Some(self.root.clone()),
                        origin: "https://child.example".into(),
                        readiness: BrowserReadiness::Ready,
                    },
                ],
                truncated: false,
            },
            BrowserPermissionTarget::Interact { scope, .. }
                if context.resolve(&scope.tab, &scope.frame).as_ref() == Ok(scope) =>
            {
                // The deterministic backend has no asynchronous navigation or global focus.
                // A real executor must perform a document-bound action here, not merely
                // check an epoch before dispatching a global keyboard event.
                BrowserOutput::Ack {
                    operation: request.operation(),
                    status: BrowserSuccess::Ok,
                }
            }
            _ => BrowserOutput::Failure {
                code: BrowserFailureCode::StaleFrameReference,
                operation: Some(request.operation()),
                outcome: BrowserOutcome::NotExecuted,
            },
        }
    }
}

async fn approved_once(
    host: &mut BrowserHost<FixtureExecutor>,
    raw: &str,
) -> Result<ValidatedBrowserOutput, Box<dyn std::error::Error>> {
    let invocation = host.prepare(raw)?;
    let resource = host.permission_resource(&invocation)?;
    // Fixture policy explicitly accepts this one exact challenge. Production hosts must
    // obtain their browser-aware policy decision or human approval before issuing it.
    let authorization =
        ExactBrowserPermissionEvaluator::for_resource(resource.clone()).evaluate(&resource)?;
    Ok(host
        .execute_prepared(invocation, authorization, std::future::pending())
        .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Trusted lifecycle discovery supplies identities; model input cannot populate them.
    let mut context = BrowserContext::new();
    let tab = context.insert_tab()?;
    let root = context.insert_frame(
        &tab,
        None,
        Some(BrowserOrigin::from_effective_url("https://parent.example")?),
        true,
    )?;
    let child = context.insert_frame(
        &tab,
        Some(&root),
        Some(BrowserOrigin::from_effective_url("https://child.example")?),
        true,
    )?;
    let mut host = BrowserHost::new(FixtureExecutor {
        root,
        child: child.clone(),
    });
    *host.context_mut() = context;
    let discovery = approved_once(
        &mut host,
        &serde_json::json!({
            "protocolVersion": 2, "operation": "frame-tree", "tabRef": tab,
        })
        .to_string(),
    )
    .await?;
    let frames: serde_json::Value = serde_json::from_str(discovery.model_json())?;
    let discovered_child = &frames["entries"][1]["frameRef"];
    let action = serde_json::json!({
        "protocolVersion": 2, "operation": "press", "tabRef": tab,
        "frameRef": discovered_child, "key": "Enter",
    })
    .to_string();
    let result = approved_once(&mut host, &action).await?;
    println!("{}", result.summary());
    // Navigation invalidates the old identity; it cannot silently retarget a new document.
    host.context_mut().invalidate_frame(&tab, &child)?;
    assert!(host.prepare(&action).is_err());
    println!("child discovery, exact action, and stale-reference rejection passed");
    Ok(())
}
