use super::*;

#[test]
fn shared_builtins_keep_explicit_ownership_in_product_registries() {
    let print = build_print_tool_registry(Vec::new());
    let (tx, _rx) = mpsc::unbounded_channel();
    let tui = build_tui_tool_registry(
        Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
        PathBuf::from("."),
        Uuid::new_v4(),
        Vec::new(),
    );
    let mcp = build_mcp_tool_registry();
    assert!(mcp.is_builtin_instance("status"));
    for registry in [&print, &tui, &mcp] {
        for name in [
            "write",
            "edit",
            "delete",
            "web_search",
            "http_request",
            "git_pull",
            "git_push",
        ] {
            assert!(registry.get(name).is_some(), "missing {name}");
            assert!(
                registry.is_builtin_instance(name),
                "lost ownership of {name}"
            );
        }
    }
}

#[test]
fn caller_supplied_scheduler_instances_do_not_gain_builtin_ownership() {
    let (tools, _pending) = talos_agent::create_scheduler_tools();
    let names: Vec<_> = tools.iter().map(|tool| tool.name().to_owned()).collect();
    assert!(!names.is_empty());
    let registry = build_print_tool_registry(tools);
    for name in names {
        assert!(registry.get(&name).is_some());
        assert!(!registry.is_builtin_instance(&name));
    }
}
