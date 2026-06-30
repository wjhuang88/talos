//! Search tools: grep and glob.

use std::path::PathBuf;

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use crate::file_tools::{FileToolError, resolve_workspace_path};
use crate::search_engine::{RipgrepSearchEngine, SearchEngine, SearchError as EngineError};

/// Input parameters for the [`GrepTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GrepInput {
    /// Regular expression pattern to search for.
    pub pattern: String,
    /// File or directory to search in. Defaults to workspace root.
    #[serde(default)]
    pub path: Option<String>,
    /// Glob pattern to filter files (e.g. `*.rs`). Only matching files are searched.
    #[serde(default)]
    pub include: Option<String>,
    /// Maximum number of matches to return. Default 50.
    #[serde(default)]
    #[schemars(range(min = 1))]
    pub max_results: Option<u32>,
}

/// A tool that searches file contents by regex across the workspace.
pub struct GrepTool {
    workspace_root: PathBuf,
}

impl GrepTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let grep_input: GrepInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let search_path = match &grep_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        if !search_path.exists() {
            return Err(FileToolError::FileNotFound(
                grep_input.path.unwrap_or_default(),
            ));
        }

        let include_pattern = grep_input
            .include
            .as_deref()
            .map(glob::Pattern::new)
            .transpose()
            .map_err(|e| FileToolError::InvalidInput(format!("invalid include glob: {e}")))?;

        let max_results = grep_input.max_results.unwrap_or(50) as usize;

        let engine = RipgrepSearchEngine;
        let output = engine
            .search(
                &grep_input.pattern,
                &search_path,
                include_pattern.as_ref(),
                max_results,
            )
            .map_err(|e| match e {
                EngineError::InvalidRegex(msg) => {
                    FileToolError::InvalidInput(format!("invalid regex: {msg}"))
                }
                EngineError::Io(e) => FileToolError::Io(e),
                EngineError::SearchPanic(msg) => FileToolError::InvalidInput(msg),
            })?;

        format_output(&grep_input.pattern, &output.matches, max_results)
    }
}

#[async_trait]
impl AgentTool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents by regex across the workspace"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(GrepInput)
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
        ToolFamily::Search
    }
    fn is_always_on(&self) -> bool {
        true
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["pattern", "path", "include"]
    }
}

fn format_output(
    pattern: &str,
    matches: &[crate::search_engine::FileMatches],
    max_results: usize,
) -> Result<String, FileToolError> {
    let total: usize = matches.iter().map(|m| m.lines.len()).sum();

    if total == 0 {
        return Ok(format!("no matches found for pattern '{pattern}'"));
    }

    let mut output = String::new();
    for fm in matches {
        if fm.lines.is_empty() {
            continue;
        }
        output.push_str(&format!("{}:\n", fm.path));
        for (line_num, line) in &fm.lines {
            output.push_str(&format!("  {line_num}: {line}\n"));
        }
    }

    if total >= max_results {
        output.push_str(&format!(
            "\n... (showing first {max_results} matches, refine pattern for more)"
        ));
    }

    Ok(output)
}

/// Input parameters for the [`GlobTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlobInput {
    /// Glob pattern (e.g. `**/*.rs`, `src/**/*.ts`, `*.toml`).
    pub pattern: String,
    /// Base directory for the search. Defaults to workspace root.
    #[serde(default)]
    pub path: Option<String>,
}

/// A tool that finds files by name pattern using glob matching.
pub struct GlobTool {
    workspace_root: PathBuf,
}

impl GlobTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let glob_input: GlobInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let base_path = match &glob_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        let full_pattern = base_path.join(&glob_input.pattern);
        let pattern_str = full_pattern.to_string_lossy().to_string();

        let opts = glob::MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut paths: Vec<String> = Vec::new();
        for entry in glob::glob_with(&pattern_str, opts)
            .map_err(|e| FileToolError::InvalidInput(format!("invalid glob pattern: {e}")))?
        {
            let path =
                entry.map_err(|e| FileToolError::InvalidInput(format!("glob error: {e}")))?;
            let display_path = path
                .strip_prefix(&canonical_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            paths.push(display_path);
        }

        if paths.is_empty() {
            return Ok(format!("no files matched pattern '{}'", glob_input.pattern));
        }

        paths.sort();

        let mut output = String::new();
        for p in &paths {
            output.push_str(p);
            output.push('\n');
        }
        output.push_str(&format!("\n{} file(s) matched", paths.len()));

        Ok(output)
    }
}

#[async_trait]
impl AgentTool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files by glob pattern (e.g. **/*.rs)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(GlobInput)
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
        ToolFamily::Search
    }
    fn is_always_on(&self) -> bool {
        true
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["pattern", "path"]
    }
}

#[cfg(test)]
#[allow(warnings)]
mod grep_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn hello() {}\nfn world() {}\n").unwrap();
        fs::write(dir.path().join("b.txt"), "hello world\nfoo bar\n").unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(
            dir.path().join("sub/c.rs"),
            "hello from sub\nanother line\n",
        )
        .unwrap();
        dir
    }

    #[tokio::test]
    async fn test_grep_basic_match() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "hello" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:"));
        assert!(result.content.contains("b.txt:"));
        assert!(result.content.contains("sub/c.rs:"));
        assert!(result.content.contains("  1:"));
    }

    #[tokio::test]
    async fn test_grep_regex_pattern() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "fn \\w+\\(\\)" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:"));
        assert!(result.content.contains("  1: fn hello()"));
        assert!(result.content.contains("  2: fn world()"));
        assert!(!result.content.contains("b.txt"));
    }

    #[tokio::test]
    async fn test_grep_include_filter() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "include": "*.rs" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("a.rs:"));
        assert!(result.content.contains("sub/c.rs:"));
        assert!(!result.content.contains("b.txt"));
    }

    #[tokio::test]
    async fn test_grep_single_file() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "foo", "path": "b.txt" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("b.txt:"));
        assert!(result.content.contains("2: foo bar"));
    }

    #[tokio::test]
    async fn test_grep_no_match() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "nonexistent_pattern_xyz" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("no matches"));
    }

    #[tokio::test]
    async fn test_grep_max_results() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "max_results": 1 }))
            .await;

        assert!(!result.is_error);
        let match_count = result
            .content
            .lines()
            .filter(|l| l.contains(": ") && !l.starts_with("..."))
            .count();
        assert_eq!(match_count, 1);
        assert!(result.content.contains("showing first 1"));
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "[invalid" })).await;

        assert!(result.is_error);
        assert!(result.content.contains("invalid regex"));
    }

    #[tokio::test]
    async fn test_grep_skips_hidden_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("visible.txt"), "target_text\n").unwrap();
        fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        fs::write(dir.path().join(".hidden/secret.txt"), "target_text\n").unwrap();

        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "target_text" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("visible.txt"));
        assert!(!result.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_grep_path_not_found() {
        let dir = make_workspace();
        let tool = GrepTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "hello", "path": "nonexistent_dir" }))
            .await;

        assert!(result.is_error);
    }

    #[test]
    fn test_grep_tool_is_read_only() {
        let tool = GrepTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}

#[cfg(test)]
#[allow(warnings)]
mod glob_tool_tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn make_workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn lib() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/mod.rs"), "pub mod sub;\n").unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(
            dir.path().join("tests/integration.rs"),
            "#[test]\nfn test() {}\n",
        )
        .unwrap();
        dir
    }

    #[tokio::test]
    async fn test_glob_recursive_rs() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("lib.rs"));
        assert!(result.content.contains("src/mod.rs"));
        assert!(result.content.contains("tests/integration.rs"));
        assert!(!result.content.contains("Cargo.toml"));
    }

    #[tokio::test]
    async fn test_glob_top_level_pattern() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("lib.rs"));
        assert!(!result.content.contains("src/mod.rs"));
    }

    #[tokio::test]
    async fn test_glob_toml_files() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "*.toml" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("Cargo.toml"));
        assert!(!result.content.contains(".rs"));
    }

    #[tokio::test]
    async fn test_glob_specific_dir() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "src/**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("src/mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_glob_no_match() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.py" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("no files matched"));
    }

    #[tokio::test]
    async fn test_glob_with_path_param() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool
            .execute(json!({ "pattern": "*.rs", "path": "src" }))
            .await;

        assert!(!result.is_error);
        assert!(result.content.contains("mod.rs"));
        assert!(!result.content.contains("main.rs"));
    }

    #[tokio::test]
    async fn test_glob_file_count() {
        let dir = make_workspace();
        let tool = GlobTool::new(dir.path().to_path_buf());
        let result = tool.execute(json!({ "pattern": "**/*.rs" })).await;

        assert!(!result.is_error);
        assert!(result.content.contains("4 file(s) matched"));
    }

    #[test]
    fn test_glob_tool_is_read_only() {
        let tool = GlobTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }
}
