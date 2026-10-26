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
