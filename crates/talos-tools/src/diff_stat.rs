//! Diff and stat tools.

use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolExecutionAuthorization, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use crate::file_tools::{FileToolError, resolve_authorized_path};

/// Input parameters for the [`DiffTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiffInput {
    pub old_path: String,
    pub new_path: String,
}

/// A tool that compares two files and shows a unified diff.
pub struct DiffTool {
    workspace_root: PathBuf,
}

impl DiffTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<String, FileToolError> {
        let diff_input: DiffInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let old_path = resolve_authorized_path(
            &self.workspace_root,
            &diff_input.old_path,
            "diff",
            talos_core::tool::ToolNature::Read,
            authorizations,
        )?;
        let new_path = resolve_authorized_path(
            &self.workspace_root,
            &diff_input.new_path,
            "diff",
            talos_core::tool::ToolNature::Read,
            authorizations,
        )?;

        if !old_path.exists() {
            return Err(FileToolError::FileNotFound(diff_input.old_path));
        }
        if !new_path.exists() {
            return Err(FileToolError::FileNotFound(diff_input.new_path));
        }

        let old_text = tokio::fs::read_to_string(&old_path).await?;
        let new_text = tokio::fs::read_to_string(&new_path).await?;

        let old_display = diff_input.old_path;
        let new_display = diff_input.new_path;

        let diff = similar::TextDiff::from_lines(&old_text, &new_text);
        let mut output = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                similar::ChangeTag::Delete => '-',
                similar::ChangeTag::Insert => '+',
                similar::ChangeTag::Equal => ' ',
            };
            output.push(sign);
            output.push_str(change.value());
        }

        if output.trim().is_empty() || output.lines().all(|l| l.starts_with(' ')) {
            return Ok(format!(
                "no differences between {old_display} and {new_display}"
            ));
        }

        let added = output.lines().filter(|l| l.starts_with('+')).count();
        let removed = output.lines().filter(|l| l.starts_with('-')).count();
        output.push_str(&format!(
            "\n{old_display} → {new_display}: +{added} -{removed} lines"
        ));

        Ok(output)
    }
}

#[async_trait]
impl AgentTool for DiffTool {
    fn name(&self) -> &str {
        "diff"
    }

    fn description(&self) -> &str {
        "Compare two files and show unified diff"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(DiffInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input, &[]).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    async fn execute_authorized(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> ToolResult {
        match self.execute_inner(input, authorizations).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }
    fn family(&self) -> ToolFamily {
        ToolFamily::Search
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["old_path", "new_path"]
    }
}

/// Input parameters for the [`StatTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatInput {
    pub path: String,
}

/// A tool that returns file metadata.
pub struct StatTool {
    workspace_root: PathBuf,
}

impl StatTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> Result<String, FileToolError> {
        let stat_input: StatInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_authorized_path(
            &self.workspace_root,
            &stat_input.path,
            "stat",
            talos_core::tool::ToolNature::Read,
            authorizations,
        )?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(stat_input.path));
        }

        let meta = tokio::fs::symlink_metadata(&path).await?;
        let ft = meta.file_type();

        let mut output = String::new();
        output.push_str(&format!("path: {}\n", stat_input.path));

        if ft.is_dir() {
            output.push_str("type: directory\n");
        } else if ft.is_symlink() {
            output.push_str("type: symlink\n");
        } else {
            output.push_str("type: file\n");
        }

        output.push_str(&format!("size: {}\n", meta.len()));

        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let mode = meta.permissions().mode();
            output.push_str(&format!("permissions: {:o}\n", mode & 0o777));
            output.push_str(&format!("uid: {}\n", meta.uid()));
            output.push_str(&format!("gid: {}\n", meta.gid()));
        }

        if let Ok(modified) = meta.modified()
            && let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
        {
            output.push_str(&format!("modified: {}\n", duration.as_secs()));
        }

        Ok(output.trim_end().to_string())
    }
}

#[async_trait]
impl AgentTool for StatTool {
    fn name(&self) -> &str {
        "stat"
    }

    fn description(&self) -> &str {
        "Get file metadata (size, type, permissions, modified time)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(StatInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input, &[]).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    async fn execute_authorized(
        &self,
        input: Value,
        authorizations: &[ToolExecutionAuthorization],
    ) -> ToolResult {
        match self.execute_inner(input, authorizations).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }
    fn family(&self) -> ToolFamily {
        ToolFamily::Search
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}

#[cfg(test)]
#[allow(warnings)]
mod diff_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_diff_identical_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello\n").unwrap();
        fs::write(dir.path().join("b.txt"), "hello\n").unwrap();

        let tool = DiffTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "old_path": "a.txt", "new_path": "b.txt" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("no differences"));
    }

    #[tokio::test]
    async fn test_diff_shows_changes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "line1\nline2\nline3\n").unwrap();
        fs::write(dir.path().join("b.txt"), "line1\nchanged\nline3\n").unwrap();

        let tool = DiffTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "old_path": "a.txt", "new_path": "b.txt" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("-line2"));
        assert!(result.content.contains("+changed"));
        assert!(result.content.contains("+1"));
        assert!(result.content.contains("-1"));
    }

    #[tokio::test]
    async fn test_diff_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello\n").unwrap();

        let tool = DiffTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "old_path": "a.txt", "new_path": "missing.txt" }))
            .await;

        assert!(result.is_error);
    }

    #[test]
    fn test_diff_is_read_only() {
        let tool = DiffTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
#[allow(warnings)]
mod stat_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_stat_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();

        let tool = StatTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("type: file"));
        assert!(result.content.contains("size: 11"));
    }

    #[tokio::test]
    async fn test_stat_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();

        let tool = StatTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "subdir" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("type: directory"));
    }

    #[tokio::test]
    async fn test_stat_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let tool = StatTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent" })).await;

        assert!(result.is_error);
    }

    #[test]
    fn test_stat_is_read_only() {
        let tool = StatTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}
