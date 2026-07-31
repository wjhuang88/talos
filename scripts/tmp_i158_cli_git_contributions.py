from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, got {count}")
    return text.replace(old, new, 1)


registry_path = Path("crates/talos-cli/src/registry.rs")
text = registry_path.read_text()

text = replace_once(
    text,
    '''use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
''',
    "",
    "registry direct Git imports",
)
text = replace_once(
    text,
    '''use talos_tools::{
    BashTool, DeleteTool, DiffTool, DocumentExtractTool, EditTool, ExecTool, FetchUrlTool,
    GlobTool, GrepTool, HttpRequestTool, LsTool, ReadImageTool, ReadTool, SaveUrlTool, StatTool,
    TreeTool, WebSearchTool, WriteTool, snapshot_aware_file_tools,
};
''',
    '''use talos_tools::{
    BashTool, DeleteTool, DiffTool, DocumentExtractTool, EditTool, ExecTool, FetchUrlTool,
    GlobTool, GrepTool, HttpRequestTool, LsTool, ReadImageTool, ReadTool, SaveUrlTool, StatTool,
    TreeTool, WebSearchTool, WriteTool, git_mutation_tool_contributions,
    git_read_tool_contributions, snapshot_aware_file_tools,
};
''',
    "registry talos-tools imports",
)

print_git = '''    registry.register(Arc::new(GitStatusTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitDiffTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitLogTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitShowTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitBranchListTool::new(PathBuf::from("."))));
    registry.register(Arc::new(TreeTool::new(PathBuf::from("."))));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitAddTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPushTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPullTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(PathBuf::from("."))),
        approval: approval.clone(),
        print_mode: true,
    }));
'''
print_contributions = '''    for contribution in git_read_tool_contributions(PathBuf::from(".")) {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    registry.register(Arc::new(TreeTool::new(PathBuf::from("."))));
    for contribution in git_mutation_tool_contributions(PathBuf::from(".")) {
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
text = replace_once(text, print_git, print_contributions, "print Git composition")

tui_git = '''    registry.register(Arc::new(GitStatusTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitDiffTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitLogTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitShowTool::new(workspace_root.clone())));
    registry.register(Arc::new(GitBranchListTool::new(workspace_root.clone())));
    registry.register(Arc::new(TreeTool::new(workspace_root.clone())));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitAddTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitPushTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitPullTool::new(workspace_root.clone())),
        approval: approval_handler.clone(),
    }));
    registry.register(Arc::new(TuiPermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(workspace_root)),
        approval: approval_handler.clone(),
    }));
'''
tui_contributions = '''    for contribution in git_read_tool_contributions(workspace_root.clone()) {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    registry.register(Arc::new(TreeTool::new(workspace_root.clone())));
    for contribution in git_mutation_tool_contributions(workspace_root) {
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
text = replace_once(text, tui_git, tui_contributions, "TUI Git composition")

mcp_read_head = '''    registry.register(Arc::new(GitStatusTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitDiffTool::new(PathBuf::from("."))));
'''
mcp_read_group = '''    for contribution in git_read_tool_contributions(PathBuf::from(".")) {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
'''
text = replace_once(text, mcp_read_head, mcp_read_group, "MCP Git read group")
text = replace_once(
    text,
    '''    registry.register(Arc::new(GitLogTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitShowTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitBranchListTool::new(PathBuf::from("."))));
''',
    "",
    "MCP later Git read registrations",
)
text = replace_once(
    text,
    '''    registry.register(Arc::new(GitAddTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitCommitTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitPushTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitPullTool::new(PathBuf::from("."))));
    registry.register(Arc::new(GitCheckoutTool::new(PathBuf::from("."))));
''',
    '''    for contribution in git_mutation_tool_contributions(PathBuf::from(".")) {
        registry
            .register_contribution(contribution)
            .unwrap_or_else(|error| panic!("{error}"));
    }
''',
    "MCP Git mutation group",
)

# Add one product-level inventory regression test near the existing Todo inventory test.
marker = '''    #[test]
    fn print_and_tui_registries_include_todo_tools() {
'''
git_test = '''    #[test]
    fn print_tui_and_mcp_registries_preserve_git_inventory() {
        let print_registry = build_print_tool_registry(Vec::new());
        let (tx, _rx) = mpsc::unbounded_channel();
        let tui_registry = build_tui_tool_registry(
            Arc::new(TuiApprovalHandler::new(tx, PathBuf::from("."))),
            PathBuf::from("."),
            Uuid::new_v4(),
            Vec::new(),
        );
        let mcp_registry = build_mcp_tool_registry();
        let names = [
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

        for name in names {
            assert!(print_registry.get(name).is_some(), "print missing {name}");
            assert!(tui_registry.get(name).is_some(), "TUI missing {name}");
            assert!(mcp_registry.get(name).is_some(), "MCP missing {name}");
        }
    }

'''
text = replace_once(text, marker, git_test + marker, "Git inventory regression test")

if "GitStatusTool" in text or "GitCheckoutTool" in text:
    raise SystemExit("stale direct Git construction remains in registry.rs")
registry_path.write_text(text)

mode_runners_path = Path("crates/talos-cli/src/mode_runners.rs")
text = mode_runners_path.read_text()
text = replace_once(
    text,
    '''use talos_tools::git::{
    GitAddTool, GitBranchListTool, GitCheckoutTool, GitCommitTool, GitDiffTool, GitLogTool,
    GitPullTool, GitPushTool, GitShowTool, GitStatusTool,
};
use talos_tools::{BashTool, DiffTool, GlobTool, GrepTool, LsTool, StatTool, TreeTool};
''',
    '''use talos_tools::{
    BashTool, DiffTool, GlobTool, GrepTool, LsTool, StatTool, TreeTool,
    git_mutation_tool_contributions, git_read_tool_contributions,
};
''',
    "mode runner Git imports",
)
mode_runners_path.write_text(text)

interactive_path = Path("crates/talos-cli/src/mode_interactive.rs")
text = interactive_path.read_text()
interactive_git = '''    registry.register(Arc::new(GitStatusTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitDiffTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitLogTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitShowTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(GitBranchListTool::new(
        workspace_root.to_path_buf(),
    )));
    registry.register(Arc::new(TreeTool::new(workspace_root.to_path_buf())));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitAddTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCommitTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPushTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitPullTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
    registry.register(Arc::new(PermissionAwareTool {
        inner: Arc::new(GitCheckoutTool::new(workspace_root.to_path_buf())),
        approval: approval.clone(),
        print_mode: false,
    }));
'''
interactive_contributions = '''    for contribution in git_read_tool_contributions(workspace_root.to_path_buf()) {
        registry.register_contribution(contribution)?;
    }
    registry.register(Arc::new(TreeTool::new(workspace_root.to_path_buf())));
    for contribution in git_mutation_tool_contributions(workspace_root.to_path_buf()) {
        let contribution = contribution.map_tool(|tool| {
            Arc::new(PermissionAwareTool {
                inner: tool,
                approval: approval.clone(),
                print_mode: false,
            })
        });
        registry.register_contribution(contribution)?;
    }
'''
text = replace_once(text, interactive_git, interactive_contributions, "interactive Git composition")
if "GitStatusTool" in text or "GitCheckoutTool" in text:
    raise SystemExit("stale direct Git construction remains in mode_interactive.rs")
interactive_path.write_text(text)
