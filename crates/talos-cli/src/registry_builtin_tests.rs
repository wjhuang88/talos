use super::*;

#[test]
fn product_scheduler_factory_cannot_relabel_existing_external_instances() {
    let mut registry = ToolRegistry::new();
    let (tools, _pending) = talos_agent::create_scheduler_tools();
    for tool in tools {
        registry.register(tool);
    }
    assert!(register_builtin_scheduler(&mut registry).is_err());
    assert_eq!(registry.list().len(), 4);
    for tool in registry.list() {
        assert!(!registry.is_builtin_instance(tool.name()));
    }
}

#[test]
fn product_scheduler_factory_registers_exact_owned_instances() {
    let mut registry = ToolRegistry::new();
    let _pending = register_builtin_scheduler(&mut registry).expect("scheduler registration");
    let expected = std::collections::BTreeSet::from([
        "delay",
        "schedule",
        "list_scheduled_tasks",
        "cancel_scheduled_task",
    ]);
    let tools = registry.list();
    assert_eq!(expected, tools.iter().map(|tool| tool.name()).collect());
    for tool in tools {
        assert!(registry.is_builtin_instance(tool.name()));
    }
}

// Coverage inventory only, never a permission allowlist. Any newly registered
// product tool must receive an explicit disposition in I308 before this changes.
#[test]
fn product_registries_have_no_uninventoried_native_operations() {
    let print = build_print_tool_registry(Vec::new());
    let (tx, _rx) = mpsc::unbounded_channel();
    let tui = build_tui_tool_registry(
        Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
        PathBuf::from("."),
        Uuid::new_v4(),
        Vec::new(),
    );
    let mcp = build_mcp_tool_registry();
    let shared = [
        "exec",
        "read",
        "write",
        "edit",
        "ls",
        "delete",
        "document_extract",
        "grep",
        "glob",
        "diff",
        "stat",
        "tree",
        "save_url",
        "fetch_url",
        "http_request",
        "web_search",
        "find_symbol",
        "find_references",
        "list_symbols",
        "list_imports",
        "git_status",
        "git_diff",
        "git_log",
        "git_show",
        "git_branch_list",
        "git_add",
        "git_commit",
        "git_push",
        "git_pull",
        "git_checkout",
    ];
    let todos = [
        "todo_create",
        "todo_create_batch",
        "todo_update_batch",
        "todo_update_status",
        "todo_update",
        "todo_delete",
        "todo_add_dependency",
        "todo_remove_dependency",
        "todo_query",
    ];
    for (profile, registry) in [("print", &print), ("tui", &tui), ("mcp", &mcp)] {
        let mut expected: std::collections::BTreeSet<&str> = shared.into_iter().collect();
        expected.insert(if cfg!(windows) { "powershell" } else { "bash" });
        if profile == "mcp" {
            expected.insert("status");
        } else {
            expected.insert("read_image");
            expected.extend(todos);
        }
        let tools = registry.list();
        let actual = tools.iter().map(|tool| tool.name()).collect();
        assert_eq!(
            expected, actual,
            "{profile} inventory changed; update coverage disposition"
        );
        for tool in tools {
            assert!(
                registry.is_builtin_instance(tool.name()),
                "{profile}: {} lost native ownership",
                tool.name()
            );
        }
    }
}

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
