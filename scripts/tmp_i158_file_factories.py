from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, got {count}")
    return text.replace(old, new, 1)


path = Path("crates/talos-tools/src/contributions.rs")
text = path.read_text()
text = replace_once(
    text,
    "use crate::ReadImageTool;\n",
    '''use crate::ReadImageTool;
use crate::file_tools::{
    DeleteTool, EditTool, LsTool, ReadTool, WriteTool, snapshot_aware_file_tools,
};
''',
    "file imports",
)
text = replace_once(
    text,
    '''const GIT_CONTRIBUTION_SOURCE: &str = "talos-tools:git";
''',
    '''const FILE_CONTRIBUTION_SOURCE: &str = "talos-tools:file";
const GIT_CONTRIBUTION_SOURCE: &str = "talos-tools:git";
''',
    "file source",
)
text = replace_once(
    text,
    '''fn git_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(GIT_CONTRIBUTION_SOURCE, tool)
}

''',
    '''fn file_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(FILE_CONTRIBUTION_SOURCE, tool)
}

fn git_contribution(tool: Arc<dyn AgentTool>) -> ToolContribution {
    contribution(GIT_CONTRIBUTION_SOURCE, tool)
}

/// Builds the core file tool group with one shared model-private snapshot registry.
///
/// Print and TUI composition use this variant to preserve snapshot-bound write,
/// edit, and delete behavior. Permission wrappers remain an outer-product policy.
#[must_use]
pub fn snapshot_aware_file_tool_contributions(
    workspace_root: PathBuf,
) -> Vec<ToolContribution> {
    let (read, write, edit, delete) = snapshot_aware_file_tools(workspace_root.clone());
    vec![
        file_contribution(Arc::new(read)),
        file_contribution(Arc::new(write)),
        file_contribution(Arc::new(edit)),
        file_contribution(Arc::new(LsTool::new(workspace_root))),
        file_contribution(Arc::new(delete)),
    ]
}

/// Builds the core file tool group with ordinary independent constructors.
///
/// MCP composition uses this variant to preserve its current registry behavior.
#[must_use]
pub fn ordinary_file_tool_contributions(workspace_root: PathBuf) -> Vec<ToolContribution> {
    vec![
        file_contribution(Arc::new(ReadTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(WriteTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(EditTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(LsTool::new(workspace_root.clone()))),
        file_contribution(Arc::new(DeleteTool::new(workspace_root))),
    ]
}

''',
    "file factories",
)
text = replace_once(
    text,
    '''    #[test]
    fn git_read_group_has_stable_inventory_and_source() {
''',
    '''    #[test]
    fn file_groups_have_stable_equivalent_inventory_and_source() {
        let snapshot = snapshot_aware_file_tool_contributions(PathBuf::from("workspace"));
        let ordinary = ordinary_file_tool_contributions(PathBuf::from("workspace"));
        let expected = ["read", "write", "edit", "ls", "delete"];

        assert_eq!(names(&snapshot), expected);
        assert_eq!(names(&ordinary), expected);
        assert_source(&snapshot, FILE_CONTRIBUTION_SOURCE);
        assert_source(&ordinary, FILE_CONTRIBUTION_SOURCE);
    }

    #[test]
    fn git_read_group_has_stable_inventory_and_source() {
''',
    "file tests",
)
path.write_text(text)

lib_path = Path("crates/talos-tools/src/lib.rs")
text = lib_path.read_text()
text = replace_once(
    text,
    '''pub use contributions::{
    git_mutation_tool_contributions, git_read_tool_contributions, read_image_tool_contribution,
    symbol_tool_contributions,
};
''',
    '''pub use contributions::{
    git_mutation_tool_contributions, git_read_tool_contributions,
    ordinary_file_tool_contributions, read_image_tool_contribution,
    snapshot_aware_file_tool_contributions, symbol_tool_contributions,
};
''',
    "file factory exports",
)
lib_path.write_text(text)
