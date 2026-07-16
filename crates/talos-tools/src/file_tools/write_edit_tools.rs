use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use super::{FileSnapshotRegistry, FileToolError, resolve_workspace_path};
use crate::file_tools::snapshot::{atomic_replace, digest, line_spans};

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
    snapshots: Option<FileSnapshotRegistry>,
}

impl WriteTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            snapshots: None,
        }
    }

    /// Creates a write tool that invalidates shared file snapshots.
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
        if let Some(registry) = &self.snapshots {
            registry.invalidate_path(&path)?;
        }

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
    snapshots: Option<FileSnapshotRegistry>,
}

impl EditTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            snapshots: None,
        }
    }

    /// Creates an edit tool with legacy string replacement plus snapshot-anchored edits.
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
        if input.get("snapshot_id").is_some() {
            return self.execute_anchored(input).await;
        }
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
        if let Some(registry) = &self.snapshots {
            registry.invalidate_path(&path)?;
        }

        Ok(format!(
            "edited {}\ndiff:\n{}",
            edit_input.path,
            bounded_replacement_diff(&edit_input.old_string, &edit_input.new_string)
        ))
    }

    async fn execute_anchored(&self, input: Value) -> Result<String, FileToolError> {
        let anchored: AnchoredEditInput = serde_json::from_value(input)
            .map_err(|error| FileToolError::InvalidInput(error.to_string()))?;
        let registry = self
            .snapshots
            .as_ref()
            .ok_or(FileToolError::SnapshotNotFound)?;
        if anchored.operations.is_empty() {
            return Err(FileToolError::InvalidEditRange(
                "at least one operation is required".into(),
            ));
        }
        let path = resolve_workspace_path(&self.workspace_root, &anchored.path)?;
        if !path.exists() {
            return Err(FileToolError::FileNotFound(anchored.path));
        }
        let snapshot = registry.get(&anchored.snapshot_id, &path)?;
        let current = tokio::fs::read(&path).await?;
        if digest(&current) != snapshot.file_revision {
            return Err(FileToolError::FileRevisionMismatch);
        }
        let spans = line_spans(&current);
        let mut mutations = Vec::with_capacity(anchored.operations.len());
        for operation in &anchored.operations {
            mutations.push(build_mutation(
                operation,
                &current,
                &spans,
                &snapshot.line_digests,
            )?);
        }
        validate_mutations(&mut mutations)?;
        let old_preview = mutations
            .iter()
            .map(|mutation| String::from_utf8_lossy(&current[mutation.start..mutation.end]))
            .collect::<Vec<_>>()
            .join("\n");
        let new_preview = mutations
            .iter()
            .map(|mutation| String::from_utf8_lossy(&mutation.replacement))
            .collect::<Vec<_>>()
            .join("\n");
        let diff = bounded_replacement_diff(&old_preview, &new_preview);
        let mut updated = current.clone();
        for mutation in mutations.iter().rev() {
            updated.splice(
                mutation.start..mutation.end,
                mutation.replacement.iter().copied(),
            );
        }
        let path_lock = registry.path_lock(&path)?;
        atomic_replace(&path, &updated, snapshot.file_revision, &path_lock)?;
        registry.invalidate_path(&path)?;
        Ok(format!("edited {}\ndiff:\n{diff}", anchored.path))
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct AnchoredEditInput {
    path: String,
    snapshot_id: String,
    operations: Vec<AnchoredOperation>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
enum AnchoredOperation {
    Replace {
        target: String,
        content: String,
    },
    ReplaceRange {
        start: String,
        end: String,
        content: String,
    },
    InsertBefore {
        target: String,
        content: String,
    },
    InsertAfter {
        target: String,
        content: String,
    },
    Delete {
        start: String,
        #[serde(default)]
        end: Option<String>,
    },
}

struct Mutation {
    start: usize,
    end: usize,
    replacement: Vec<u8>,
}

#[async_trait]
impl AgentTool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        if self.snapshots.is_some() {
            "Apply a legacy string replacement or a snapshot-anchored atomic line edit"
        } else {
            "Apply a string replacement in a file"
        }
    }

    fn parameters(&self) -> Value {
        if self.snapshots.is_some() {
            let legacy = tool_parameters!(EditInput);
            let mut anchored = tool_parameters!(AnchoredEditInput);
            let definitions = anchored
                .as_object_mut()
                .and_then(|object| object.remove("$defs"));
            let mut properties = serde_json::Map::new();
            for source in [&legacy, &anchored] {
                if let Some(source_properties) = source.get("properties").and_then(Value::as_object)
                {
                    properties.extend(source_properties.clone());
                }
            }
            let mut schema = serde_json::json!({
                "type": "object",
                "description": "Use old_string/new_string for legacy replacement, or snapshot_id/operations for anchored atomic edits.",
                "properties": properties,
                "required": ["path"],
                "anyOf": [
                    {"required": ["old_string", "new_string"]},
                    {"required": ["snapshot_id", "operations"]}
                ]
            });
            if let Some(definitions) = definitions
                && let Some(object) = schema.as_object_mut()
            {
                object.insert("$defs".into(), definitions);
            }
            schema
        } else {
            tool_parameters!(EditInput)
        }
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

    fn project_input(&self, input: &Value) -> Value {
        let mut projected = input.clone();
        if let Some(object) = projected.as_object_mut() {
            object.remove("snapshot_id");
            if let Some(operations) = object.get_mut("operations").and_then(Value::as_array_mut) {
                for operation in operations {
                    let Some(operation) = operation.as_object_mut() else {
                        continue;
                    };
                    for field in ["target", "start", "end"] {
                        if let Some(reference) = operation.get_mut(field)
                            && let Some(value) = reference.as_str()
                            && let Some((line, _)) = value.split_once(':')
                        {
                            *reference = Value::String(line.to_string());
                        }
                    }
                }
            }
        }
        projected
    }
}

fn build_mutation(
    operation: &AnchoredOperation,
    bytes: &[u8],
    spans: &[crate::file_tools::snapshot::LineSpan],
    line_digests: &[[u8; 32]],
) -> Result<Mutation, FileToolError> {
    match operation {
        AnchoredOperation::Replace { target, content } => {
            let line = verify_ref(target, spans, line_digests)?;
            let span = spans[line];
            let mut replacement = content.as_bytes().to_vec();
            append_original_terminator(&mut replacement, bytes, span);
            Ok(Mutation {
                start: span.content_start,
                end: span.full_end,
                replacement,
            })
        }
        AnchoredOperation::ReplaceRange {
            start,
            end,
            content,
        } => {
            let start = verify_ref(start, spans, line_digests)?;
            let end = verify_ref(end, spans, line_digests)?;
            if start > end {
                return Err(FileToolError::InvalidEditRange(
                    "range start is after range end".into(),
                ));
            }
            let mut replacement = content.as_bytes().to_vec();
            append_original_terminator(&mut replacement, bytes, spans[end]);
            Ok(Mutation {
                start: spans[start].content_start,
                end: spans[end].full_end,
                replacement,
            })
        }
        AnchoredOperation::InsertBefore { target, content } => {
            let line = verify_ref(target, spans, line_digests)?;
            let mut replacement = content.as_bytes().to_vec();
            append_default_terminator(&mut replacement, bytes, spans[line]);
            Ok(Mutation {
                start: spans[line].content_start,
                end: spans[line].content_start,
                replacement,
            })
        }
        AnchoredOperation::InsertAfter { target, content } => {
            let line = verify_ref(target, spans, line_digests)?;
            let span = spans[line];
            let mut replacement = Vec::new();
            if span.full_end == span.content_end && span.full_end == bytes.len() {
                replacement.extend(default_terminator(bytes, span));
            }
            replacement.extend_from_slice(content.as_bytes());
            if span.full_end < bytes.len() {
                append_default_terminator(&mut replacement, bytes, span);
            }
            Ok(Mutation {
                start: span.full_end,
                end: span.full_end,
                replacement,
            })
        }
        AnchoredOperation::Delete { start, end } => {
            let start = verify_ref(start, spans, line_digests)?;
            let end = end
                .as_deref()
                .map(|value| verify_ref(value, spans, line_digests))
                .transpose()?
                .unwrap_or(start);
            if start > end {
                return Err(FileToolError::InvalidEditRange(
                    "range start is after range end".into(),
                ));
            }
            Ok(Mutation {
                start: spans[start].content_start,
                end: spans[end].full_end,
                replacement: Vec::new(),
            })
        }
    }
}

fn verify_ref(
    reference: &str,
    spans: &[crate::file_tools::snapshot::LineSpan],
    line_digests: &[[u8; 32]],
) -> Result<usize, FileToolError> {
    let (line, check) = reference
        .split_once(':')
        .ok_or_else(|| FileToolError::InvalidRef("expected line:hh".into()))?;
    if check.len() != 2 || !check.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FileToolError::InvalidRef(
            "check code must be exactly two hexadecimal digits".into(),
        ));
    }
    let line = line
        .parse::<usize>()
        .map_err(|_| FileToolError::InvalidRef("line must be a positive integer".into()))?;
    if line == 0 || line > spans.len() || line > line_digests.len() {
        return Err(FileToolError::InvalidRef("line is out of range".into()));
    }
    if format!("{:02x}", line_digests[line - 1][0]) != check.to_ascii_lowercase() {
        return Err(FileToolError::HashMismatch);
    }
    Ok(line - 1)
}

fn validate_mutations(mutations: &mut [Mutation]) -> Result<(), FileToolError> {
    mutations.sort_by_key(|mutation| (mutation.start, mutation.end));
    for pair in mutations.windows(2) {
        if pair[1].start < pair[0].end
            || (pair[0].start == pair[1].start
                && pair[0].end == pair[0].start
                && pair[1].end == pair[1].start)
        {
            return Err(FileToolError::InvalidEditRange(
                "operations overlap or share an insertion point".into(),
            ));
        }
    }
    Ok(())
}

fn append_original_terminator(
    replacement: &mut Vec<u8>,
    bytes: &[u8],
    span: crate::file_tools::snapshot::LineSpan,
) {
    if !replacement.ends_with(b"\n") {
        replacement.extend_from_slice(&bytes[span.content_end..span.full_end]);
    }
}

fn append_default_terminator(
    replacement: &mut Vec<u8>,
    bytes: &[u8],
    span: crate::file_tools::snapshot::LineSpan,
) {
    if !replacement.ends_with(b"\n") {
        replacement.extend(default_terminator(bytes, span));
    }
}

fn default_terminator(bytes: &[u8], span: crate::file_tools::snapshot::LineSpan) -> &'static [u8] {
    if bytes.get(span.content_end..span.full_end) == Some(b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
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
