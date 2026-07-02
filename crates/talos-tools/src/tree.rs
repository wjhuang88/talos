//! Tree visualization tool for directory structure rendering.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use crate::is_skip_dir;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TreeInput {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub max_depth: Option<u32>,
}

pub struct TreeTool {
    workspace_root: PathBuf,
}

impl TreeTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[async_trait]
impl AgentTool for TreeTool {
    fn name(&self) -> &str {
        "tree"
    }

    fn description(&self) -> &str {
        "Show directory structure as ASCII tree"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(TreeInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path", "max_depth"]
    }
}

impl TreeTool {
    async fn execute_inner(&self, input: Value) -> Result<String, String> {
        let tree_input: TreeInput = serde_json::from_value(input).map_err(|e| e.to_string())?;

        let max_depth = tree_input.max_depth.unwrap_or(3) as usize;

        let root = match tree_input.path {
            Some(ref p) => self.workspace_root.join(p),
            None => self.workspace_root.clone(),
        };

        if !root.exists() {
            return Err(format!("path not found: {}", root.display()));
        }

        let mut output = String::new();
        output.push_str(&format!(
            "{}\n",
            display_root_name(&root, &self.workspace_root)
        ));

        build_tree(&root, "", 0, max_depth, &mut output);

        Ok(output.trim_end().to_string())
    }
}

fn display_root_name(root: &Path, workspace_root: &Path) -> String {
    let canonical_workspace = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    if let Ok(relative) = canonical_root.strip_prefix(&canonical_workspace) {
        if relative.as_os_str().is_empty() {
            return format_path_name(&canonical_workspace);
        }
        return format!("{}{}", relative.display(), trailing_slash(&canonical_root));
    }

    format_path_name(&canonical_root)
}

fn format_path_name(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string());
    format!("{name}{}", trailing_slash(path))
}

fn trailing_slash(path: &Path) -> &'static str {
    if path.is_dir() { "/" } else { "" }
}

fn build_tree(
    dir: &std::path::Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    output: &mut String,
) {
    if depth >= max_depth {
        return;
    }

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(Result::ok).collect(),
        Err(_) => return,
    };

    entries.retain(|e| {
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        !is_skip_dir(&name_str)
    });

    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        (!is_dir, e.file_name())
    });

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let branch = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        let display_name = if is_dir { format!("{name}/") } else { name };

        output.push_str(&format!("{prefix}{branch}{display_name}\n"));

        if is_dir {
            build_tree(
                &path,
                &format!("{prefix}{child_prefix}"),
                depth + 1,
                max_depth,
                output,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tree_root_first_line_uses_workspace_directory_name() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        let tool = TreeTool::new(tmp.path().to_path_buf());

        let result = tool
            .execute_inner(serde_json::json!({ "max_depth": 1 }))
            .await
            .unwrap();

        let first_line = result.lines().next().unwrap();
        assert!(!first_line.trim().is_empty());
        assert_eq!(
            first_line,
            format!("{}/", tmp.path().file_name().unwrap().to_string_lossy())
        );
    }

    #[tokio::test]
    async fn tree_subdir_first_line_uses_target_relative_name() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        let tool = TreeTool::new(tmp.path().to_path_buf());

        let result = tool
            .execute_inner(serde_json::json!({ "path": "src", "max_depth": 1 }))
            .await
            .unwrap();

        assert_eq!(result.lines().next().unwrap(), "src/");
    }
}
