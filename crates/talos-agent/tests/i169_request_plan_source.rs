use std::fs;
use std::path::Path;

#[test]
fn sealed_provider_plan_owns_budgeted_dispatch_inputs() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/request_plan.rs"))
            .expect("read request_plan.rs");

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
