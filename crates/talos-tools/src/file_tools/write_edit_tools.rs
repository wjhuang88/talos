use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use super::{FileToolError, resolve_workspace_path};

const MAX_PREVIEW_LINES: usize = 20;
const MAX_PREVIEW_CHARS: usize = 2_000;

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
            "wrote {} bytes to {}\npreview:\n{}",
            write_input.content.len(),
            write_input.path,
            bounded_preview(&write_input.content)
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

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    fn is_always_on(&self) -> bool {
        true
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

        Ok(format!(
            "edited {}\ndiff:\n{}",
            edit_input.path,
            bounded_replacement_diff(&edit_input.old_string, &edit_input.new_string)
        ))
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

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    fn is_always_on(&self) -> bool {
        true
    }
}

fn bounded_preview(content: &str) -> String {
    if content.is_empty() {
        return "(empty)".to_string();
    }

    let total_lines = content.lines().count();
    let mut rendered = String::new();
    let mut used_chars = 0usize;
    let mut truncated = false;

    for (idx, line) in content.lines().enumerate() {
        if idx >= MAX_PREVIEW_LINES {
            truncated = true;
            break;
        }
        if used_chars >= MAX_PREVIEW_CHARS {
            truncated = true;
            break;
        }

        let remaining = MAX_PREVIEW_CHARS - used_chars;
        let line_chars = line.chars().count();
        if line_chars > remaining {
            rendered.push_str(&line.chars().take(remaining).collect::<String>());
            used_chars = MAX_PREVIEW_CHARS;
            truncated = true;
            break;
        }

        if idx > 0 {
            rendered.push('\n');
            used_chars += 1;
        }
        rendered.push_str(line);
        used_chars += line_chars;
    }

    if content.chars().count() > used_chars {
        truncated = true;
    }

    if truncated {
        if !rendered.is_empty() {
            rendered.push('\n');
        }
        rendered.push_str(&format!(
            "... preview truncated ({total_lines} lines, {} bytes total)",
            content.len()
        ));
    }

    rendered
}

fn bounded_replacement_diff(old: &str, new: &str) -> String {
    let old_preview = prefixed_block("-", old);
    let new_preview = prefixed_block("+", new);
    format!("{old_preview}\n{new_preview}")
}

fn prefixed_block(prefix: &str, content: &str) -> String {
    bounded_preview(content)
        .lines()
        .map(|line| format!("{prefix} {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
