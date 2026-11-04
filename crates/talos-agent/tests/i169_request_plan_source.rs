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
    ] {
        assert!(
            agent.contains(required),
            "Agent dispatch must consume the sealed plan without rebuilding: {required}"
        );
    }

    let (before_dispatch, dispatch) = agent
        .split_once("let provider_request = self.provider.stream_with_replay_context(")
        .expect("Agent must dispatch with sealed replay context");
    let (arguments, _) = dispatch
        .split_once(");")
        .expect("provider request arguments");
    assert!(
        arguments.trim_start().starts_with(
            "&plan.messages,\n                &plan.tool_definitions,\n                plan.tool_protocol,"
        ),
        "Agent must dispatch the sealed messages, tools and protocol directly"
    );
    let (_, replay_binding) = before_dispatch
        .rsplit_once("let replay = match private_replays.for_request(")
        .expect("Agent must bind replay before dispatch");
    assert!(replay_binding.trim_start().starts_with("&plan.messages,"));
    assert!(arguments.trim_end().ends_with("replay,"));

    assert!(
        session.contains(".prepare_session_turn(")
            || session.contains(".prepare_session_turn_at_revision("),
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
