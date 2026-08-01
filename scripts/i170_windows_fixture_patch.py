from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{label}: expected one occurrence, found {count}")
        return text.replace(old, new, 1)
    if new not in text:
        raise SystemExit(f"{label}: neither old nor new form found")
    return text


# Current I158 interactive profile: compare two independently sorted inventories.
path = Path("crates/talos-cli/src/mode_interactive.rs")
text = path.read_text(encoding="utf-8")
start_marker = "        assert_eq!(\n            names,\n            [\n"
end_marker = "        );\n        assert!(registry.get(\"exec\").is_none());"
if "let mut expected = vec![" not in text:
    if text.count(start_marker) != 1 or text.count(end_marker) != 1:
        raise SystemExit("mode_interactive inventory assertion markers changed")
    start = text.index(start_marker)
    end = text.index(end_marker, start)
    replacement = '''        let mut expected = vec![
            SHELL_TOOL_NAME,
            "delete",
            "diff",
            "edit",
            "git_add",
            "git_branch_list",
            "git_checkout",
            "git_commit",
            "git_diff",
            "git_log",
            "git_pull",
            "git_push",
            "git_show",
            "git_status",
            "glob",
            "grep",
            "ls",
            "read",
            "stat",
            "tree",
            "write",
        ];
        expected.sort();
        assert_eq!(names, expected);
'''
    text = text[:start] + replacement + text[end + len("        );\n"):]
path.write_text(text, encoding="utf-8")

# Current product profile inventories are returned sorted; sort the platform-aware expected vectors.
path = Path("crates/talos-cli/src/registry.rs")
text = path.read_text(encoding="utf-8")
text = replace_once(
    text,
    "        let print_tui_inventory = vec![\n",
    "        let mut print_tui_inventory = vec![\n",
    "print/TUI inventory mutability",
)
text = replace_once(
    text,
    "        let mcp_inventory = vec![\n",
    "        let mut mcp_inventory = vec![\n",
    "MCP inventory mutability",
)
text = replace_once(
    text,
    "\n        assert_eq!(sorted_registry_names(&print_registry), print_tui_inventory);\n",
    "\n        print_tui_inventory.sort();\n        mcp_inventory.sort();\n\n        assert_eq!(sorted_registry_names(&print_registry), print_tui_inventory);\n",
    "registry expected inventory sort",
)
path.write_text(text, encoding="utf-8")

# Unit document fixtures must execute inside the fixture's own explicit workspace.
path = Path("crates/talos-tools/src/document_extract.rs")
text = path.read_text(encoding="utf-8")
old = '''    fn run_extract(path: &Path, format_hint: Option<&str>, max_bytes: Option<usize>) -> String {
        let tool = DocumentExtractTool::new(PathBuf::from("/"));
        let input = if let Some(mb) = max_bytes {
            serde_json::json!({
                "path": path.to_string_lossy(),
                "format": format_hint.unwrap_or("auto"),
                "max_bytes": mb,
            })
        } else {
            serde_json::json!({
                "path": path.to_string_lossy(),
                "format": format_hint.unwrap_or("auto"),
            })
        };
'''
new = '''    fn run_extract(path: &Path, format_hint: Option<&str>, max_bytes: Option<usize>) -> String {
        let workspace_root = path.parent().expect("fixture file has a parent directory");
        let relative_path = path
            .file_name()
            .expect("fixture file has a name")
            .to_string_lossy();
        let tool = DocumentExtractTool::new(workspace_root.to_path_buf());
        let input = if let Some(mb) = max_bytes {
            serde_json::json!({
                "path": relative_path,
                "format": format_hint.unwrap_or("auto"),
                "max_bytes": mb,
            })
        } else {
            serde_json::json!({
                "path": relative_path,
                "format": format_hint.unwrap_or("auto"),
            })
        };
'''
text = replace_once(text, old, new, "document_extract unit fixture workspace")
path.write_text(text, encoding="utf-8")

# Integration success fixtures must also remain inside the tool's explicit workspace.
path = Path("crates/talos-tools/tests/document_boundaries.rs")
text = path.read_text(encoding="utf-8")
old = '''fn run_extract(tool: &DocumentExtractTool, path: &str) -> String {
    let input = serde_json::json!({
        "path": path,
        "format": "auto",
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(tool.execute(input));
'''
new = '''fn run_extract(path: &PathBuf) -> String {
    let workspace_root = path.parent().expect("fixture file has a parent directory");
    let relative_path = path
        .file_name()
        .expect("fixture file has a name")
        .to_string_lossy();
    let tool = DocumentExtractTool::new(workspace_root.to_path_buf());
    let input = serde_json::json!({
        "path": relative_path,
        "format": "auto",
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(tool.execute(input));
'''
text = replace_once(text, old, new, "document boundary helper workspace")

manual_old = '''    let saved_path = create_temp_file("composed.json", content);
    let path_str = saved_path.to_string_lossy().to_string();

    // Step 2: Use document_extract on the saved file.
    let extract_tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&extract_tool, &path_str);
'''
manual_new = '''    let saved_path = create_temp_file("composed.json", content);

    // Step 2: Use document_extract on the saved file within its fixture workspace.
    let output = run_extract(&saved_path);
'''
text = replace_once(text, manual_old, manual_new, "manual composition fixture workspace")

common_old = '''    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&tool, &path_str);
'''
common_new = '''    let output = run_extract(&path);
'''
count = text.count(common_old)
if count == 3:
    text = text.replace(common_old, common_new)
elif count != 0 or text.count(common_new) < 3:
    raise SystemExit(f"document boundary successful fixture count changed: old={count}, new={text.count(common_new)}")

trunc_old = '''    let path = create_temp_file("large.txt", &content);
    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let input = serde_json::json!({
        "path": path_str,
        "format": "auto",
        "max_bytes": 100,
    });
'''
trunc_new = '''    let path = create_temp_file("large.txt", &content);
    let workspace_root = path.parent().expect("fixture file has a parent directory");
    let relative_path = path
        .file_name()
        .expect("fixture file has a name")
        .to_string_lossy();

    let tool = DocumentExtractTool::new(workspace_root.to_path_buf());
    let input = serde_json::json!({
        "path": relative_path,
        "format": "auto",
        "max_bytes": 100,
    });
'''
text = replace_once(text, trunc_old, trunc_new, "truncation fixture workspace")
path.write_text(text, encoding="utf-8")

# User-visible diagnostics use platform-neutral language.
path = Path("crates/talos-cli/src/permissions.rs")
text = path.read_text(encoding="utf-8")
text = replace_once(
    text,
    '                    "  Bash/exec: still per-command unless access evidence proves repo-local read (ADR-040)."',
    '                    "  Shell/exec: still per-command unless access evidence proves repo-local read (ADR-040)."',
    "permission trust text",
)
path.write_text(text, encoding="utf-8")

path = Path("crates/talos-cli/src/diagnostics.rs")
text = path.read_text(encoding="utf-8")
text = replace_once(
    text,
    '            summary: "bash/exec remains per-command Ask/Deny (evidence is diagnostic-only)",',
    '            summary: "shell/exec remains per-command Ask/Deny (evidence is diagnostic-only)",',
    "diagnostics residual text",
)
path.write_text(text, encoding="utf-8")
