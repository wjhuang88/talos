//! Production protected WebSearch never downgrades to ordinary invocation admission.
use super::*;
use crate::auto_resolver::{BuiltinNetworkAssessmentRequest, OrdinaryInvocationAssessmentRequest};
use std::sync::atomic::{AtomicUsize, Ordering};

const DIRECT_ROUTE_CHILD: &str = "TALOS_SEARCH_DENY_DIRECT_ROUTE_CHILD";
const PROXY_ENVIRONMENT: [&str; 6] = [
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "HTTP_PROXY",
    "http_proxy",
];

fn rerun_without_proxy(filter: &str) -> bool {
    if std::env::var_os(DIRECT_ROUTE_CHILD).is_some() {
        return false;
    }
    let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
    command
        .arg(filter)
        .arg("--nocapture")
        .env(DIRECT_ROUTE_CHILD, "1");
    for name in PROXY_ENVIRONMENT {
        command.env_remove(name);
    }
    let output = command.output().expect("isolated direct-route child");
    assert!(
        output.status.success(),
        "direct-route child failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}

struct Assessor(Arc<AtomicUsize>);
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("search must not use legacy read-only review".into())
    }

    async fn assess_ordinary_invocation(
        &self,
        _: OrdinaryInvocationAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("search must not use ordinary HTTP review".into())
    }

    async fn assess_builtin_network(
        &self,
        _: BuiltinNetworkAssessmentRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("explicit Deny must win before Auto assessment".into())
    }
}

struct Human(Arc<AtomicUsize>);
#[async_trait]
impl ApprovalResolver for Human {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: std::time::Duration,
    ) -> Result<talos_core::ApprovalChoice, ApprovalResolverError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(talos_core::ApprovalChoice::ApproveOnce)
    }
}

#[tokio::test]
async fn protected_web_search_explicit_deny_never_downgrades_or_reaches_network() {
    if rerun_without_proxy("protected_web_search_explicit_deny_never_downgrades_or_reaches_network")
    {
        return;
    }
    for name in PROXY_ENVIRONMENT {
        assert!(std::env::var_os(name).is_none(), "{name} must be isolated");
    }

    let root = tempfile::tempdir().expect("workspace");
    let mut registry = ToolRegistry::new();
    registry
        .register_builtin_contributions(talos_tools::network_tool_contributions())
        .expect("production registration");
    let search = registry.get("web_search").expect("protected WebSearch");
    assert!(registry.is_builtin_instance("web_search"));
    assert!(
        search.requires_network_invocation(),
        "production web_search must use ADR-090 prepared authority"
    );

    let mut engine = PermissionEngine::with_workspace_root(root.path().into());
    engine.add_rule(talos_permission::PermissionRule::new(
        "web_search",
        None,
        PermissionDecision::Deny("explicit search denial".into()),
    ));
    let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
    let assessed = Arc::new(AtomicUsize::new(0));
    let humans = Arc::new(AtomicUsize::new(0));
    let resolver = AutoPermissionResolver::new(
        Arc::new(Assessor(assessed.clone())),
        Arc::new(Human(humans.clone())),
        ManagedWorkspaceLease::for_permission_session(root.path(), state.clone()).expect("lease"),
        std::time::Duration::from_secs(2),
        AutoPermissionControl::new(true),
    );

    // A valid query proves the prepared production route can be constructed. The explicit Deny
    // must terminate before execution, so this fixture performs no DNS/HTTP.
    let model = MockModel::new(vec![
        vec![
            AgentEvent::ToolCall {
                call: ToolCall {
                    id: "search".into(),
                    name: "web_search".into(),
                    input: serde_json::json!({"query":"current Rust documentation"}),
                },
                provenance: Default::default(),
                summary_fields: vec![],
            },
            AgentEvent::TurnEnd {
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
            },
        ],
        text_done_events("done"),
    ]);
    let mut agent = Agent::with_security(Arc::new(model), registry, None, None, root.path().into())
        .with_permission_pipeline(
            state.clone(),
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
    agent.set_tool_presentation_policy(ToolPresentationPolicy::with_families([
        ToolFamily::Network,
        ToolFamily::AdvancedNetwork,
    ]));

    let (tx, mut rx) = mpsc::unbounded_channel();
    agent
        .run_streaming("search current Rust documentation".into(), vec![], tx)
        .await
        .expect("run");

    let mut results = 0;
    while let Ok(event) = rx.try_recv() {
        if let AgentEvent::ToolResult { result, .. } = event {
            results += 1;
            assert!(result.is_error, "{}", result.content);
            assert!(
                result.content.contains("Permission denied"),
                "{}",
                result.content
            );
            assert!(!result.content.contains("current Rust documentation"));
        }
    }
    assert_eq!(results, 1);
    assert_eq!(assessed.load(Ordering::SeqCst), 0);
    assert_eq!(humans.load(Ordering::SeqCst), 0);
    assert_eq!(state.grant_count().expect("grants"), 0);
}
