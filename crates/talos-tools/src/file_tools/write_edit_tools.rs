use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;

use super::{FileToolError, resolve_workspace_path};

/// Input parameters for the [`WriteTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteInput {
    pub path: String,
    pub content: String,
}

/// A tool that creates a new file with the given content.
/// Does not overwrite existing files — use the edit tool for modifications.
pub struct WriteTool {
    workspace_root: PathBuf,
}

impl WriteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let write_input: WriteInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &write_input.path)?;

        if path.exists() {
            return Err(FileToolError::FileExists(write_input.path));
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, &write_input.content).await?;

        Ok(format!(
            "wrote {} bytes to {}",
            write_input.content.len(),
            write_input.path
        ))
    }
}

#[async_trait]
impl AgentTool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Create a new file (does not overwrite existing files — use edit instead)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(WriteInput)
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

/// Input parameters for the [`EditTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EditInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

/// A tool that applies a string replacement in a file.
pub struct EditTool {
    workspace_root: PathBuf,
}

impl EditTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let edit_input: EditInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &edit_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(edit_input.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;

        let Some(first_match_start) = content.find(&edit_input.old_string) else {
            return Err(FileToolError::StringNotFound(edit_input.old_string));
        };

        let first_match_end = first_match_start + edit_input.old_string.len();
        let mut new_content = String::with_capacity(
            content.len() + edit_input.new_string.len() - edit_input.old_string.len(),
        );
        new_content.push_str(&content[..first_match_start]);
        new_content.push_str(&edit_input.new_string);
        new_content.push_str(&content[first_match_end..]);

        tokio::fs::write(&path, &new_content).await?;

        Ok(format!("edited {}", edit_input.path))
    }
}

#[async_trait]
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Apply a string replacement in a file"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(EditInput)
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
