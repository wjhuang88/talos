//! File operation tools: read, write, edit, delete, ls.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;
use thiserror::Error;

/// Size threshold for binary file detection (8KB).
const BINARY_CHECK_SIZE: usize = 8 * 1024;

/// Errors that can occur during file tool operations.
#[derive(Debug, Error)]
pub enum FileToolError {
    #[error("path escapes workspace root: {0}")]
    PathEscape(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("file appears to be binary: {0}")]
    BinaryFile(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("string not found in file: {0}")]
    StringNotFound(String),

    #[error("invalid line range: start_line ({start}) > end_line ({end})")]
    InvalidLineRange { start: u32, end: u32 },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("file already exists: {0}. Use the edit tool to modify existing files")]
    FileExists(String),
}

/// Errors specific to the delete tool.
#[derive(Debug, Error)]
pub enum DeleteError {
    #[error("refusing to delete workspace root")]
    WorkspaceRoot,
}

/// Resolves a relative path against the workspace root and validates
/// that the resulting path stays within the workspace.
pub(crate) fn resolve_workspace_path(
    workspace_root: &Path,
    relative: &str,
) -> Result<PathBuf, FileToolError> {
    let canon_root = workspace_root.canonicalize()?;

    let joined = if relative.starts_with('/') {
        PathBuf::from(relative)
    } else {
        workspace_root.join(relative)
    };

    let canonical = if joined.exists() {
        joined.canonicalize()?
    } else if let Some(parent) = joined.parent() {
        if parent.exists() {
            let canon_parent = parent.canonicalize()?;
            let file_name = joined
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            canon_parent.join(file_name)
        } else {
            canon_root.join(relative)
        }
    } else {
        canon_root.join(relative)
    };

    if !canonical.starts_with(&canon_root) {
        return Err(FileToolError::PathEscape(relative.to_owned()));
    }

    Ok(canonical)
}

/// Checks if a file appears to be binary by looking for null bytes
/// in the first 8KB of content.
pub(crate) fn is_binary_file(path: &Path) -> Result<bool, FileToolError> {
    let bytes = std::fs::read(path)?;
    let check_bytes = &bytes[..bytes.len().min(BINARY_CHECK_SIZE)];
    Ok(check_bytes.contains(&0u8))
}

/// Directories that are skipped during recursive search.
pub fn is_skip_dir(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

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
}

impl ReadTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
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

        let mut output = String::new();
        for (i, line) in selected.iter().enumerate() {
            if show_line_numbers {
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
        "Read file content with optional line range or offset/limit pagination"
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
    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}

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
}

/// Input parameters for the [`LsTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LsInput {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub all: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub long: bool,
}

/// A tool that lists directory contents with file metadata.
pub struct LsTool {
    workspace_root: PathBuf,
}

impl LsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let ls_input: LsInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let target = match &ls_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        if !target.exists() {
            return Err(FileToolError::FileNotFound(
                ls_input.path.unwrap_or_default(),
            ));
        }

        if target.is_file() {
            return Ok(format_entry(&target, &canonical_root, ls_input.long));
        }

        if ls_input.recursive {
            let mut output = String::new();
            for entry in walkdir::WalkDir::new(&target)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    if !ls_input.all && is_hidden_entry(e.file_name()) {
                        return false;
                    }
                    !(e.file_type().is_dir() && is_skip_dir(&e.file_name().to_string_lossy()))
                })
                .filter_map(Result::ok)
                .filter(|e| e.depth() > 0)
            {
                output.push_str(&format_entry(entry.path(), &canonical_root, ls_input.long));
                output.push('\n');
            }
            return Ok(output.trim_end().to_string());
        }

        let mut entries: Vec<_> = std::fs::read_dir(&target)?
            .filter_map(Result::ok)
            .filter(|e| ls_input.all || !is_hidden_entry(&e.file_name()))
            .collect();

        entries.sort_by_key(|e| {
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            (!is_dir, e.file_name())
        });

        let mut output = String::new();
        for entry in &entries {
            output.push_str(&format_entry(&entry.path(), &canonical_root, ls_input.long));
            output.push('\n');
        }

        Ok(output.trim_end().to_string())
    }
}

#[async_trait]
impl AgentTool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List directory contents (supports long format, hidden files, recursive)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(LsInput)
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
    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}

fn is_hidden_entry(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn format_entry(path: &Path, root: &Path, long: bool) -> String {
    let display = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    if long {
        return format_entry_long(path, &display);
    }

    let meta = std::fs::symlink_metadata(path).ok();
    match &meta {
        Some(m) => {
            let ft = m.file_type();
            if ft.is_dir() {
                format!("{display}/")
            } else if ft.is_symlink() {
                format!("{display}@")
            } else if is_executable(m) {
                format!("{display}* {}", m.len())
            } else {
                format!("{display} {}", m.len())
            }
        }
        None => display,
    }
}

#[cfg(unix)]
fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_meta: &std::fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn format_entry_long(path: &Path, display: &str) -> String {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => {
            return format!("?---------    -      -      -        -  ????-??-?? ??:??  {display}");
        }
    };

    let ft = meta.file_type();
    let type_char = if ft.is_dir() {
        'd'
    } else if ft.is_symlink() {
        'l'
    } else {
        '-'
    };

    let perms = format_permissions(meta.mode());
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();
    let mtime = format_mtime(meta.mtime());

    format!("{type_char}{perms}  {nlink:>3}  {uid:>5}  {gid:>5}  {size:>8}  {mtime}  {display}")
}

#[cfg(not(unix))]
fn format_entry_long(path: &Path, display: &str) -> String {
    let meta = std::fs::symlink_metadata(path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    format!("-         {size:>8}  {display}")
}

#[cfg(unix)]
fn format_permissions(mode: u32) -> String {
    let bits: [(u32, char); 9] = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    bits.iter()
        .map(|(bit, c)| if mode & bit != 0 { *c } else { '-' })
        .collect()
}

#[cfg(unix)]
fn format_mtime(mtime: i64) -> String {
    let secs = mtime.max(0) as u64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;

    let z = days as i64 + 719468;
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02} {hour:02}:{min:02}")
}

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
}

#[cfg(test)]
#[allow(warnings)]
mod file_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_read_tool_read_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "line1\nline2\nline3\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("line1"));
        assert!(result.content.contains("line2"));
        assert!(result.content.contains("line3"));
    }

    #[tokio::test]
    async fn test_read_tool_line_range() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(
            temp_dir.path().join("test.txt"),
            "line1\nline2\nline3\nline4\n",
        )
        .unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "start_line": 2, "end_line": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_limit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 1, "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("3: line3"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("4: line4"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_zero() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 1 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: a"));
        assert!(!result.content.contains("2: b"));
    }

    #[tokio::test]
    async fn test_read_tool_limit_only() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "limit": 2 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("1: line1"));
        assert!(result.content.contains("2: line2"));
        assert!(result.content.contains("more lines"));
        assert!(result.content.contains("offset=2"));
    }

    #[tokio::test]
    async fn test_read_tool_pagination_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=10).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 3 }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("7 more lines"));
        assert!(result.content.contains("offset=3"));
    }

    #[tokio::test]
    async fn test_read_tool_no_truncation_no_hint() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "test.txt", "offset": 0, "limit": 100 }))
            .await;

        assert!(!result.is_error);
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_offset_takes_precedence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let content: String = (1..=5).map(|i| format!("line{i}\n")).collect();
        fs::write(temp_dir.path().join("test.txt"), content).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "test.txt",
                "start_line": 1,
                "end_line": 2,
                "offset": 2,
                "limit": 2
            }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("3: line3"));
        assert!(result.content.contains("4: line4"));
        assert!(!result.content.contains("1: line1"));
        assert!(!result.content.contains("2: line2"));
    }

    #[tokio::test]
    async fn test_read_tool_backward_compat_no_params() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "a\nb\nc\n").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "test.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a"));
        assert!(result.content.contains("b"));
        assert!(result.content.contains("c"));
        assert!(!result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_read_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "../outside.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_read_tool_binary_detection() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::write(temp_dir.path().join("binary.bin"), &[0u8, 1, 2, 3]).unwrap();

        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "binary.bin" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("binary"));
    }

    #[tokio::test]
    async fn test_read_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = ReadTool::new(temp_dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[tokio::test]
    async fn test_write_tool_new_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({ "path": "new.txt", "content": "hello world" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("wrote"));

        let content = fs::read_to_string(temp_dir.path().join("new.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_write_tool_refuses_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("existing.txt");
        fs::write(&file, "old content").unwrap();

        let tool = WriteTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "existing.txt", "content": "new content" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("already exists"));
        assert!(result.content.contains("edit tool"));
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "old content");
    }

    #[tokio::test]
    async fn test_write_tool_create_parent_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = WriteTool::new(temp_dir.path().to_path_buf());

        let result = tool
            .execute(json!({
                "path": "a/b/c/deep.txt",
                "content": "deep content"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(temp_dir.path().join("a/b/c/deep.txt")).unwrap();
        assert_eq!(content, "deep content");
    }

    #[tokio::test]
    async fn test_edit_tool_replace_first_occurrence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "foo bar foo baz").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "foo",
                "new_string": "qux"
            }))
            .await;

        assert!(!result.is_error);
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "qux bar foo baz");
    }

    #[tokio::test]
    async fn test_edit_tool_no_match() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("edit.txt");
        fs::write(&file, "hello world").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "edit.txt",
                "old_string": "notfound",
                "new_string": "replacement"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("string not found"));
    }

    #[tokio::test]
    async fn test_edit_tool_path_escape() {
        let temp_dir = tempfile::tempdir().unwrap();
        let outside = temp_dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "../outside.txt",
                "old_string": "secret",
                "new_string": "exposed"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
    }

    #[tokio::test]
    async fn test_edit_tool_file_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tool = EditTool::new(temp_dir.path().to_path_buf());
        let result = tool
            .execute(json!({
                "path": "nonexistent.txt",
                "old_string": "foo",
                "new_string": "bar"
            }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[test]
    fn test_resolve_workspace_path_within_root() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("subdir/file.txt");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "content").unwrap();

        let resolved = resolve_workspace_path(temp_dir.path(), "subdir/file.txt").unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn test_resolve_workspace_path_escape_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = resolve_workspace_path(temp_dir.path(), "../outside.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileToolError::PathEscape(_)));
    }
}

#[cfg(test)]
#[allow(warnings)]
mod ls_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;\n").unwrap();
        fs::write(dir.path().join(".hidden"), "secret\n").unwrap();
        dir
    }

    #[tokio::test]
    async fn test_ls_flat() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("Cargo.toml"));
        assert!(result.content.contains("src"));
        assert!(!result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_show_hidden() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "all": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_ls_recursive() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "recursive": true })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("src/mod.rs"));
    }

    #[tokio::test]
    async fn test_ls_specific_dir() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "src" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_dir_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let src_line = result.content.lines().find(|l| l.contains("src")).unwrap();
        assert!(src_line.ends_with('/'));
        assert!(!src_line.contains(' '));
    }

    #[tokio::test]
    async fn test_ls_file_type_indicator() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let toml_line = result
            .content
            .lines()
            .find(|l| l.contains("Cargo.toml"))
            .unwrap();
        assert!(!toml_line.ends_with('/'));
    }

    #[tokio::test]
    async fn test_ls_file_shows_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({})).await;

        assert!(!result.is_error);
        let line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap();
        assert!(line.ends_with(" 11"));
    }

    #[tokio::test]
    async fn test_ls_not_found() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent" })).await;

        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_ls_single_file() {
        let dir = make_workspace();
        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "main.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_ls_long_format() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let txt_line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap_or_else(|| panic!("no test.txt line in: {}", result.content));
        assert!(txt_line.starts_with('-'));
        assert!(txt_line.contains("rw"));
    }

    #[tokio::test]
    async fn test_ls_long_format_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let src_line = result
            .content
            .lines()
            .find(|l| l.contains("src"))
            .unwrap_or_else(|| panic!("no src line in: {}", result.content));
        assert!(src_line.starts_with('d'));
    }

    #[tokio::test]
    async fn test_ls_long_shows_permissions() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "content").unwrap();

        let tool = LsTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "long": true })).await;

        assert!(!result.is_error);
        let line = result
            .content
            .lines()
            .find(|l| l.contains("test.txt"))
            .unwrap();
        let perms_field = line.split_whitespace().nth(0).unwrap_or("");
        assert!(perms_field.starts_with('-'));
        assert!(perms_field.len() == 10);
    }

    #[test]
    fn test_ls_tool_is_read_only() {
        let tool = LsTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
#[allow(warnings)]
mod delete_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[tokio::test]
    async fn test_delete_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("temp.txt"), "content").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "temp.txt" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("deleted"));
        assert!(!dir.path().join("temp.txt").exists());
    }

    #[tokio::test]
    async fn test_delete_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("subdir/nested")).unwrap();
        fs::write(dir.path().join("subdir/nested/file.txt"), "data").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "subdir" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("deleted"));
        assert!(!dir.path().join("subdir").exists());
    }

    #[tokio::test]
    async fn test_delete_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("empty")).unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "empty" })).await;

        assert!(!result.is_error);
        assert!(!dir.path().join("empty").exists());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "nonexistent.txt" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[tokio::test]
    async fn test_delete_path_escape() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().parent().unwrap().join("outside_target.txt");
        fs::write(&outside, "secret").unwrap();

        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "path": "../outside_target.txt" }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("path escapes workspace root"));
        assert!(outside.exists());
        let _ = fs::remove_file(&outside);
    }

    #[tokio::test]
    async fn test_delete_refuses_workspace_root() {
        let dir = tempfile::tempdir().unwrap();
        let tool = DeleteTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "path": "." })).await;

        assert!(result.is_error);
        assert!(result.content.contains("workspace root"));
        assert!(dir.path().exists());
    }

    #[tokio::test]
    async fn test_delete_not_read_only() {
        let tool = DeleteTool::new(PathBuf::from("."));
        assert!(!tool.is_read_only());
    }
}
