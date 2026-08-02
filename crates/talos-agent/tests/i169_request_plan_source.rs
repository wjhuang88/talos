use std::fs;
use std::path::Path;

fn read_source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

#[test]
fn sealed_provider_plan_owns_budgeted_dispatch_inputs() {
    let source = read_source("src/request_plan.rs");

    for required in [
        "struct ProviderRequestPlan",
        "messages: Vec<Message>",
        "tool_definitions: Vec<ToolDefinition>",
        "estimated_tokens: u32",
        "struct PreparedSessionTurn",
        "initial_plan: ProviderRequestPlan",
    ] {
        assert!(
            source.contains(required),
            "sealed Provider request source contract is missing: {required}"
        );
    }
}

#[test]
fn actor_and_provider_dispatch_share_the_sealed_plan() {
    let agent = read_source("src/lib.rs");
    let session = read_source("src/session.rs");
    let forwarding = read_source("src/session/turn.rs");

    for required in [
        "let mut initial_plan = Some(initial_plan);",
        "if let Some(plan) = initial_plan.take()",
        "stream_with_tools(&plan.messages, &plan.tool_definitions)",
    ] {
        assert!(
            agent.contains(required),
            "Agent dispatch must consume the sealed plan without rebuilding: {required}"
        );
    }

    assert!(
        session.contains(".prepare_session_turn("),
        "Actor must seal the initial Provider request before Turn start"
    );
    assert!(
        forwarding.contains("prepared: PreparedSessionTurn"),
        "Turn forwarding must carry the already sealed request plan"
    );
    assert!(
        !forwarding.contains("request_context_limit"),
        "Turn forwarding must not re-budget a rebuilt initial request"
    );
}
