use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;

use super::{DeleteError, FileToolError, resolve_workspace_path};

/// Input parameters for the [`DeleteTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteInput {
    /// Path to the file or directory to delete.
    pub path: String,
}

/// A tool that deletes files or directories within the workspace.
pub struct DeleteTool {
    workspace_root: PathBuf,
}

impl DeleteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let delete_input: DeleteInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &delete_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(delete_input.path));
        }

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .map_err(FileToolError::Io)?;

        if path == canonical_root {
            return Err(FileToolError::InvalidInput(
                DeleteError::WorkspaceRoot.to_string(),
            ));
        }

        if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await?;
        } else {
            tokio::fs::remove_file(&path).await?;
        }

        let display = path
            .strip_prefix(&canonical_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        Ok(format!("deleted {display}"))
    }
}

#[async_trait]
impl AgentTool for DeleteTool {
    fn name(&self) -> &str {
        "delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory (cannot delete workspace root)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(DeleteInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}
