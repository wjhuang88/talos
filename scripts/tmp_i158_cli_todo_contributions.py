from pathlib import Path

path = Path("crates/talos-cli/src/registry.rs")
text = path.read_text()

old_core_import = '''use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult,
};
'''
new_core_import = '''use talos_core::tool::{
    AgentTool, ToolAuthorizationScope, ToolBackend, ToolContribution, ToolExecutionAuthorization,
    ToolExecutionOutput, ToolFamily, ToolPermissionFacet, ToolRegistry, ToolResult,
};
'''
if old_core_import not in text:
    raise SystemExit("core import block not found")
text = text.replace(old_core_import, new_core_import, 1)

old_session_import = '''use talos_session::{
    SessionManager, TodoAddDependencyTool, TodoCreateBatchTool, TodoCreateTool, TodoDeleteTool,
    TodoQueryTool, TodoRemoveDependencyTool, TodoUpdateBatchTool, TodoUpdateStatusTool,
    TodoUpdateTool,
};
'''
new_session_import = '''use talos_session::{SessionManager, todo_tool_contributions_for_sessions_dir};
'''
if old_session_import not in text:
    raise SystemExit("session import block not found")
text = text.replace(old_session_import, new_session_import, 1)

old_helpers = '''fn default_todo_tools(session_id: Uuid) -> Vec<Arc<dyn AgentTool>> {
    let Ok(sessions_dir) = SessionManager::default_sessions_dir() else {
        return Vec::new();
    };
    todo_tools_for_sessions_dir(&sessions_dir, session_id)
}

fn todo_tools_for_sessions_dir(sessions_dir: &Path, session_id: Uuid) -> Vec<Arc<dyn AgentTool>> {
    vec![
        Arc::new(TodoCreateTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoCreateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoUpdateStatusTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoUpdateTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoUpdateBatchTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoDeleteTool::from_sessions_dir(sessions_dir, session_id)),
        Arc::new(TodoAddDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoRemoveDependencyTool::from_sessions_dir(
            sessions_dir,
            session_id,
        )),
        Arc::new(TodoQueryTool::from_sessions_dir(sessions_dir, session_id)),
    ]
}
'''
new_helpers = '''fn default_todo_tool_contributions(session_id: Uuid) -> Vec<ToolContribution> {
    let Ok(sessions_dir) = SessionManager::default_sessions_dir() else {
        return Vec::new();
    };
    todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id)
}
'''
if old_helpers not in text:
    raise SystemExit("todo helper block not found")
text = text.replace(old_helpers, new_helpers, 1)

old_print = '''    for tool in default_todo_tools(ephemeral_session_id) {
        registry.register(Arc::new(PermissionAwareTool {
            inner: tool,
            approval: approval.clone(),
            print_mode: true,
        }));
    }
'''
new_print = '''    for contribution in default_todo_tool_contributions(ephemeral_session_id) {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: true,
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
'''
if old_print not in text:
    raise SystemExit("print todo loop not found")
text = text.replace(old_print, new_print, 1)

old_tui = '''    for tool in default_todo_tools(session_id) {
        registry.register(Arc::new(TuiPermissionAwareTool {
            inner: tool,
            approval: approval_handler.clone(),
        }));
    }
'''
new_tui = '''    for contribution in default_todo_tool_contributions(session_id) {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: approval_handler.clone(),
            })
        });
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
'''
if old_tui not in text:
    raise SystemExit("TUI todo loop not found")
text = text.replace(old_tui, new_tui, 1)

old_print_test = '''        let mut print_registry = ToolRegistry::new();
        for tool in todo_tools_for_sessions_dir(&sessions_dir, session_id) {
            print_registry.register(tool);
        }
'''
new_print_test = '''        let mut print_registry = ToolRegistry::new();
        for contribution in todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id) {
            print_registry
                .register_contribution(contribution)
                .unwrap();
        }
'''
if old_print_test not in text:
    raise SystemExit("print todo test loop not found")
text = text.replace(old_print_test, new_print_test, 1)

old_tui_test = '''        let mut tui_registry = ToolRegistry::new();
        for tool in todo_tools_for_sessions_dir(&sessions_dir, session_id) {
            tui_registry.register(Arc::new(TuiPermissionAwareTool {
                inner: tool,
                approval: tui_approval.clone(),
            }));
        }
'''
new_tui_test = '''        let mut tui_registry = ToolRegistry::new();
        for contribution in todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id) {
            let contribution = contribution.map_tool(|tool| {
                Arc::new(TuiPermissionAwareTool {
                    inner: tool,
                    approval: tui_approval.clone(),
                })
            });
            tui_registry
                .register_contribution(contribution)
                .unwrap();
        }
'''
if old_tui_test not in text:
    raise SystemExit("TUI todo test loop not found")
text = text.replace(old_tui_test, new_tui_test, 1)

if text.count("todo_tools_for_sessions_dir") != 2:
    raise SystemExit("unexpected todo factory call count before persistence-test migration")
text = text.replace(
    "let before_tools = todo_tools_for_sessions_dir(&sessions_dir, session_id);",
    "let before_tools = todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id);",
    1,
)
text = text.replace(
    '''        let create_tool = before_tools
            .iter()
            .find(|t| t.name() == "todo_create")
            .unwrap();
''',
    '''        let create_tool = before_tools
            .iter()
            .find(|contribution| contribution.name() == "todo_create")
            .unwrap()
            .tool();
''',
    1,
)
text = text.replace(
    "let after_tools = todo_tools_for_sessions_dir(&sessions_dir, session_id);",
    "let after_tools = todo_tool_contributions_for_sessions_dir(&sessions_dir, session_id);",
    1,
)
text = text.replace(
    '''        let query_tool = after_tools
            .iter()
            .find(|t| t.name() == "todo_query")
            .unwrap();
''',
    '''        let query_tool = after_tools
            .iter()
            .find(|contribution| contribution.name() == "todo_query")
            .unwrap()
            .tool();
''',
    1,
)

if "todo_tools_for_sessions_dir" in text or "default_todo_tools" in text:
    raise SystemExit("stale CLI-owned todo factory remains")

path.write_text(text)
