use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use super::{FileSnapshotRegistry, FileToolError, is_binary_file, resolve_workspace_path};
use crate::file_tools::snapshot::line_spans;

/// Input parameters for the [`ReadTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadInput {
    pub path: String,
    /// Starting line number (1-based). Prefer `offset` for new code.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub start_line: Option<u32>,
    /// Ending line number (1-based, inclusive). Prefer `limit` for new code.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub end_line: Option<u32>,
    /// 0-based line offset for pagination. `offset=0` starts at line 1.
    /// Takes precedence over `start_line`/`end_line` when specified.
    #[serde(default)]
    #[schemars(range(min = 0))]
    pub offset: Option<u32>,
    /// Maximum number of lines to return. Defaults to 2000 when `offset` is set.
    /// Takes precedence over `start_line`/`end_line` when specified.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub limit: Option<u32>,
}

/// A tool that reads file content with optional line range support.
pub struct ReadTool {
    workspace_root: PathBuf,
    snapshots: Option<FileSnapshotRegistry>,
}

impl ReadTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            snapshots: None,
        }
    }

    /// Creates a read tool that emits compact model-private line anchors.
    #[must_use]
    pub fn with_snapshot_registry(
        workspace_root: PathBuf,
        snapshots: FileSnapshotRegistry,
    ) -> Self {
        Self {
            workspace_root,
            snapshots: Some(snapshots),
        }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let read_input: ReadInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &read_input.path)?;

        if !path.exists() {
            return Err(FileToolError::FileNotFound(read_input.path));
        }

        if is_binary_file(&path)? {
            return Err(FileToolError::BinaryFile(read_input.path));
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let (start, max_lines) = if read_input.offset.is_some() || read_input.limit.is_some() {
            let offset = read_input.offset.unwrap_or(0) as usize;
            let limit = read_input.limit.unwrap_or(2000) as usize;
            (offset, limit)
        } else if read_input.start_line.is_some() || read_input.end_line.is_some() {
            let start = read_input.start_line.unwrap_or(1).saturating_sub(1) as usize;
            let end = match read_input.end_line {
                Some(e) => e as usize,
                None => lines.len(),
            };
            if start > end {
                return Err(FileToolError::InvalidLineRange {
                    start: read_input.start_line.unwrap_or(1),
                    end: read_input.end_line.unwrap_or(lines.len() as u32),
                });
            }
            (start, end.saturating_sub(start))
        } else {
            (0, lines.len())
        };

        let start = start.min(total_lines);
        let end = (start + max_lines).min(total_lines);
        let selected = &lines[start..end];

        let show_line_numbers = total_lines > 50
            || read_input.offset.is_some()
            || read_input.limit.is_some()
            || read_input.start_line.is_some()
            || read_input.end_line.is_some();

        let snapshot = if let Some(registry) = &self.snapshots {
            let bytes = content.as_bytes();
            match registry.capture(&path, bytes) {
                Ok((id, codes)) => Some((id, codes, line_spans(bytes))),
                Err(FileToolError::SnapshotLimit(_)) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };

        let mut output = String::new();
        if let Some((id, _, _)) = &snapshot {
            output.push_str(&format!("[snapshot:{id}]\n"));
        }
        for (i, line) in selected.iter().enumerate() {
            if let Some((_, codes, spans)) = &snapshot {
                let line_index = start + i;
                let rendered = spans
                    .get(line_index)
                    .and_then(|span| content.get(span.content_start..span.content_end))
                    .unwrap_or(line);
                output.push_str(&format!(
                    "{}:{}|{}\n",
                    line_index + 1,
                    codes.get(line_index).map(String::as_str).unwrap_or("00"),
                    rendered
                ));
            } else if show_line_numbers {
                let line_num = start + i + 1;
                output.push_str(&format!("{line_num}: {line}\n"));
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        let remaining = total_lines.saturating_sub(end);
        if remaining > 0 {
            let next_offset = end;
            output.push_str(&format!(
                "\n... ({remaining} more lines, use offset={next_offset} to continue)"
            ));
        }

        Ok(output)
    }
}

#[async_trait]
impl AgentTool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        if self.snapshots.is_some() {
            "Read file content with compact line:hh anchors and a transient snapshot id for precise edit calls"
        } else {
            "Read file content with optional line range or offset/limit pagination"
        }
    }

    fn parameters(&self) -> Value {
        tool_parameters!(ReadInput)
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
    fn is_always_on(&self) -> bool {
        true
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }

    fn project_result(
        &self,
        result: &talos_core::tool::ToolResult,
    ) -> talos_core::tool::ToolResultProjection {
        if !result.is_error && result.content.starts_with("[snapshot:") {
            let mut sanitized = String::new();
            for line in result.content.lines().skip(1) {
                let ordinary = line
                    .split_once('|')
                    .and_then(|(reference, content)| {
                        reference.split_once(':').and_then(|(number, code)| {
                            (number.bytes().all(|byte| byte.is_ascii_digit())
                                && code.len() == 2
                                && code.bytes().all(|byte| byte.is_ascii_hexdigit()))
                            .then(|| format!("{number}: {content}"))
                        })
                    })
                    .unwrap_or_else(|| line.to_string());
                sanitized.push_str(&ordinary);
                sanitized.push('\n');
            }
            talos_core::tool::ToolResultProjection {
                model_content: result.content.clone(),
                display_content: sanitized.clone(),
                persistence_content: sanitized,
            }
        } else {
            talos_core::tool::ToolResultProjection::shared(result.content.clone())
        }
    }
}
