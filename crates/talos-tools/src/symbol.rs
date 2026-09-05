//! AST-aware symbol query engine using arborium/tree-sitter.
//!
//! Provides structural code exploration at AST precision — find symbols,
//! locate references, list functions/classes, and extract imports.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolFamily, ToolResult};
use talos_core::tool_parameters;

use talos_text::symbol_queries::{ImportInfo, SymbolResult};

const MAX_DEPTH: usize = 64;
const MAX_FILES: usize = 10_000;
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug, Serialize)]
struct TraversalNotice {
    talos_notice: &'static str,
    reasons: Vec<&'static str>,
    counts: TraversalNoticeCounts,
    admitted_files: usize,
    admitted_bytes: usize,
}

#[derive(Debug, Serialize)]
struct TraversalNoticeCounts {
    symlink_skipped: usize,
    oversized_file: usize,
    depth_limit: usize,
    file_limit: usize,
    aggregate_byte_limit: usize,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum TraversalOutput<T> {
    Result(T),
    Notice(TraversalNotice),
}

#[derive(Default)]
struct TraversalState {
    depth_limit: usize,
    file_limit: usize,
    aggregate_byte_limit: usize,
    symlink_skipped: usize,
    oversized_file: usize,
    depth_limit_hits: usize,
    file_limit_hits: usize,
    aggregate_byte_limit_hits: usize,
    admitted_files: usize,
    admitted_bytes: usize,
    exhausted: bool,
}

impl TraversalState {
    fn new() -> Self {
        Self {
            depth_limit: MAX_DEPTH,
            file_limit: MAX_FILES,
            aggregate_byte_limit: MAX_TOTAL_BYTES,
            ..Self::default()
        }
    }

    fn notice(&self) -> Option<TraversalNotice> {
        let mut reasons = Vec::new();
        for (key, value) in [
            ("symlink_skipped", self.symlink_skipped),
            ("oversized_file", self.oversized_file),
            ("depth_limit", self.depth_limit_hits),
            ("file_limit", self.file_limit_hits),
            ("aggregate_byte_limit", self.aggregate_byte_limit_hits),
        ] {
            if value != 0 {
                reasons.push(key);
            }
        }
        (!reasons.is_empty()).then_some(TraversalNotice {
            talos_notice: "bounded_traversal",
            reasons,
            counts: TraversalNoticeCounts {
                symlink_skipped: self.symlink_skipped,
                oversized_file: self.oversized_file,
                depth_limit: self.depth_limit_hits,
                file_limit: self.file_limit_hits,
                aggregate_byte_limit: self.aggregate_byte_limit_hits,
            },
            admitted_files: self.admitted_files,
            admitted_bytes: self.admitted_bytes,
        })
    }
}

enum FileAdmission {
    Admitted {
        language: &'static str,
        code: String,
    },
    Unsupported,
    Omitted,
}

fn admit_file(path: &Path, state: &mut TraversalState) -> Result<FileAdmission, String> {
    let Some(language) = detect_language(path) else {
        return Ok(FileAdmission::Unsupported);
    };
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take((MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FILE_BYTES {
        state.oversized_file += 1;
        return Ok(FileAdmission::Omitted);
    }
    let code = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    if state.admitted_files >= state.file_limit {
        state.file_limit_hits += 1;
        state.exhausted = true;
        return Ok(FileAdmission::Omitted);
    }
    if state.admitted_bytes.saturating_add(code.len()) > state.aggregate_byte_limit {
        state.aggregate_byte_limit_hits += 1;
        state.exhausted = true;
        return Ok(FileAdmission::Omitted);
    }
    state.admitted_files += 1;
    state.admitted_bytes += code.len();
    Ok(FileAdmission::Admitted { language, code })
}

fn detect_language(path: &Path) -> Option<&'static str> {
    talos_text::language_for_extension(path.extension()?.to_str()?)
}

type SourceLocation = talos_text::SourceLocation;
type SymbolInfo = talos_text::SymbolInfo;

/// Input for find_symbol tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindSymbolInput {
    pub name: String,
    #[serde(default)]
    pub path: Option<String>,
}

/// Input for find_references tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindReferencesInput {
    pub name: String,
    pub file: String,
}

/// Input for list_symbols tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListSymbolsInput {
    pub path: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// Input for list_imports tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListImportsInput {
    pub file: String,
}

pub struct FindSymbolTool {
    workspace_root: PathBuf,
}

pub struct FindReferencesTool {
    workspace_root: PathBuf,
}

pub struct ListSymbolsTool {
    workspace_root: PathBuf,
}

pub struct ListImportsTool {
    workspace_root: PathBuf,
}

impl FindSymbolTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl FindReferencesTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl ListSymbolsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

impl ListImportsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

macro_rules! impl_read_only_tool {
    ($name:expr, $desc:expr, $struct:ty, $input:ty, $execute:expr, $summary:expr) => {
        #[async_trait]
        impl AgentTool for $struct {
            fn name(&self) -> &str {
                $name
            }
            fn description(&self) -> &str {
                $desc
            }
            fn parameters(&self) -> Value {
                tool_parameters!($input)
            }
            fn is_read_only(&self) -> bool {
                true
            }
            fn family(&self) -> ToolFamily {
                ToolFamily::CodeIntelligence
            }
            fn summary_fields(&self) -> &'static [&'static str] {
                $summary
            }
            async fn execute(&self, input: Value) -> ToolResult {
                $execute(self, input).await
            }
        }
    };
}

impl_read_only_tool!(
    "find_symbol",
    "Find a symbol (function, struct, class, etc.) by name across workspace files",
    FindSymbolTool,
    FindSymbolInput,
    execute_find_symbol,
    &["name", "path"]
);

impl_read_only_tool!(
    "find_references",
    "Find all usages of a named symbol within a specific file",
    FindReferencesTool,
    FindReferencesInput,
    execute_find_references,
    &["name", "file"]
);

impl_read_only_tool!(
    "list_symbols",
    "List symbols of a given kind (function, struct, class) in a directory or file",
    ListSymbolsTool,
    ListSymbolsInput,
    execute_list_symbols,
    &["path", "kind"]
);

impl_read_only_tool!(
    "list_imports",
    "List all imports/exports in a file",
    ListImportsTool,
    ListImportsInput,
    execute_list_imports,
    &["file"]
);

async fn execute_find_symbol(tool: &FindSymbolTool, input: Value) -> ToolResult {
    let params: FindSymbolInput = match serde_json::from_value(input) {
        Ok(p) => p,
        Err(e) => return ToolResult::error(format!("invalid input: {e}")),
    };

    let search_path = params
        .path
        .map(|p| tool.workspace_root.join(p))
        .unwrap_or_else(|| tool.workspace_root.clone());

    match scan_workspace(&search_path, &params.name) {
        Ok(results) => {
            ToolResult::success(serde_json::to_string_pretty(&results).unwrap_or_default())
        }
        Err(e) => ToolResult::error(e),
    }
}

async fn execute_find_references(tool: &FindReferencesTool, input: Value) -> ToolResult {
    let params: FindReferencesInput = match serde_json::from_value(input) {
        Ok(p) => p,
        Err(e) => return ToolResult::error(format!("invalid input: {e}")),
    };

    let file_path = tool.workspace_root.join(&params.file);
    match find_refs_in_file(&file_path, &params.name) {
        Ok(refs) => ToolResult::success(serde_json::to_string_pretty(&refs).unwrap_or_default()),
        Err(e) => ToolResult::error(e),
    }
}

async fn execute_list_symbols(tool: &ListSymbolsTool, input: Value) -> ToolResult {
    let params: ListSymbolsInput = match serde_json::from_value(input) {
        Ok(p) => p,
        Err(e) => return ToolResult::error(format!("invalid input: {e}")),
    };

    let path = tool.workspace_root.join(&params.path);
    match list_symbols_in_path(&path, params.kind.as_deref()) {
        Ok(symbols) => {
            ToolResult::success(serde_json::to_string_pretty(&symbols).unwrap_or_default())
        }
        Err(e) => ToolResult::error(e),
    }
}

async fn execute_list_imports(tool: &ListImportsTool, input: Value) -> ToolResult {
    let params: ListImportsInput = match serde_json::from_value(input) {
        Ok(p) => p,
        Err(e) => return ToolResult::error(format!("invalid input: {e}")),
    };

    let file_path = tool.workspace_root.join(&params.file);
    match list_imports_in_file(&file_path) {
        Ok(imports) => {
            ToolResult::success(serde_json::to_string_pretty(&imports).unwrap_or_default())
        }
        Err(e) => ToolResult::error(e),
    }
}

fn scan_workspace(root: &Path, name: &str) -> Result<Vec<TraversalOutput<SymbolResult>>, String> {
    let mut results = Vec::new();
    let mut state = TraversalState::new();
    scan_dir(root, root, name, &mut results, &mut state, 0)?;
    if let Some(notice) = state.notice() {
        results.push(TraversalOutput::Notice(notice));
    }
    Ok(results)
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    name: &str,
    results: &mut Vec<TraversalOutput<SymbolResult>>,
    state: &mut TraversalState,
    depth: usize,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if state.exhausted {
            break;
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            state.symlink_skipped += 1;
            continue;
        }
        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            if depth >= state.depth_limit {
                state.depth_limit_hits += 1;
                continue;
            }
            scan_dir(root, &path, name, results, state, depth + 1)?;
        } else if file_type.is_file()
            && let Some(result) = find_symbol_in_file(&path, root, name, state)
        {
            results.push(TraversalOutput::Result(result));
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name == "target" || name == "node_modules" || name == ".git" || name.starts_with('.')
}

fn find_symbol_in_file(
    path: &Path,
    root: &Path,
    name: &str,
    state: &mut TraversalState,
) -> Option<SymbolResult> {
    let FileAdmission::Admitted { language, code } = admit_file(path, state).ok()? else {
        return None;
    };
    talos_text::symbol_queries::find_symbol(language, &code, root, path, name)
}

fn find_refs_in_file(path: &Path, name: &str) -> Result<Vec<SourceLocation>, String> {
    let lang = detect_language(path).ok_or_else(|| "unsupported file type".to_string())?;
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    talos_text::symbol_queries::find_references(lang, &code, path, name)
}

fn list_symbols_in_path(
    path: &Path,
    kind_filter: Option<&str>,
) -> Result<Vec<TraversalOutput<SymbolInfo>>, String> {
    let mut results = Vec::new();
    if path.is_dir() {
        let mut state = TraversalState::new();
        list_dir_symbols(path, path, kind_filter, &mut results, &mut state, 0)?;
        if let Some(notice) = state.notice() {
            results.push(TraversalOutput::Notice(notice));
        }
    } else if path.is_file() {
        let mut direct_results = Vec::new();
        list_file_symbols(path, path, kind_filter, &mut direct_results)?;
        results.extend(direct_results.into_iter().map(TraversalOutput::Result));
    }
    Ok(results)
}

fn list_dir_symbols(
    root: &Path,
    dir: &Path,
    kind_filter: Option<&str>,
    results: &mut Vec<TraversalOutput<SymbolInfo>>,
    state: &mut TraversalState,
    depth: usize,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        if state.exhausted {
            break;
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            state.symlink_skipped += 1;
            continue;
        }
        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            if depth >= state.depth_limit {
                state.depth_limit_hits += 1;
                continue;
            }
            list_dir_symbols(root, &path, kind_filter, results, state, depth + 1)?;
        } else if file_type.is_file() {
            list_file_symbols_bounded(root, &path, kind_filter, results, state)?;
        }
    }
    Ok(())
}

fn list_file_symbols_bounded(
    root: &Path,
    path: &Path,
    kind_filter: Option<&str>,
    results: &mut Vec<TraversalOutput<SymbolInfo>>,
    state: &mut TraversalState,
) -> Result<(), String> {
    let FileAdmission::Admitted { language, code } = admit_file(path, state)? else {
        return Ok(());
    };
    let mut file_results = Vec::new();
    collect_file_symbols(root, path, kind_filter, language, &code, &mut file_results)?;
    results.extend(file_results.into_iter().map(TraversalOutput::Result));
    Ok(())
}

fn list_file_symbols(
    root: &Path,
    path: &Path,
    kind_filter: Option<&str>,
    results: &mut Vec<SymbolInfo>,
) -> Result<(), String> {
    let language = match detect_language(path) {
        Some(l) => l,
        None => return Ok(()),
    };
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    collect_file_symbols(root, path, kind_filter, language, &code, results)
}

fn collect_file_symbols(
    root: &Path,
    path: &Path,
    kind_filter: Option<&str>,
    language: &str,
    code: &str,
    results: &mut Vec<SymbolInfo>,
) -> Result<(), String> {
    let file = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    results.extend(talos_text::symbol::list_symbols(
        language,
        code,
        &file,
        kind_filter,
    )?);
    Ok(())
}

fn list_imports_in_file(path: &Path) -> Result<Vec<ImportInfo>, String> {
    let lang = detect_language(path).ok_or_else(|| "unsupported file type".to_string())?;
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    talos_text::symbol_queries::list_imports(lang, &code, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_rust(path: &Path, source: &str) {
        fs::write(path, source).expect("Rust fixture should be writable");
    }

    fn notice_value<T: Serialize>(results: &[TraversalOutput<T>]) -> Value {
        serde_json::to_value(results.last().expect("notice should be present"))
            .expect("notice should serialize")
    }

    #[test]
    fn symbol_traversal_production_limits_match_reviewed_contract() {
        assert_eq!(MAX_DEPTH, 64);
        assert_eq!(MAX_FILES, 10_000);
        assert_eq!(MAX_FILE_BYTES, 2 * 1024 * 1024);
        assert_eq!(MAX_TOTAL_BYTES, 50 * 1024 * 1024);
    }

    #[test]
    fn symbol_traversal_preserves_normal_tree_json() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        write_rust(&workspace.path().join("sample.rs"), "fn needle() {}\n");

        let found = scan_workspace(workspace.path(), "needle").expect("scan should succeed");
        let found_json = serde_json::to_string_pretty(&found).expect("result should serialize");
        assert_eq!(
            found_json,
            r#"[
  {
    "name": "needle",
    "kind": "function_item",
    "definition": {
      "file": "sample.rs",
      "line": 1,
      "column": 0
    },
    "references": []
  }
]"#
        );

        let listed = list_symbols_in_path(workspace.path(), None).expect("listing should succeed");
        let listed_json = serde_json::to_string_pretty(&listed).expect("result should serialize");
        assert_eq!(
            listed_json,
            r#"[
  {
    "name": "needle",
    "kind": "function_item",
    "file": "sample.rs",
    "line": 1
  }
]"#
        );
        assert!(!found_json.contains("talos_notice"));
        assert!(!listed_json.contains("talos_notice"));
    }

    #[test]
    fn symbol_traversal_reports_oversized_file_as_notice_only() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        fs::write(
            workspace.path().join("oversized.rs"),
            vec![b' '; MAX_FILE_BYTES + 1],
        )
        .expect("oversized fixture should be writable");

        let listed = list_symbols_in_path(workspace.path(), None).expect("listing should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(
            notice_value(&listed),
            serde_json::json!({
                "talos_notice": "bounded_traversal",
                "reasons": ["oversized_file"],
                "counts": {
                    "symlink_skipped": 0,
                    "oversized_file": 1,
                    "depth_limit": 0,
                    "file_limit": 0,
                    "aggregate_byte_limit": 0
                },
                "admitted_files": 0,
                "admitted_bytes": 0
            })
        );

        let found = scan_workspace(workspace.path(), "needle").expect("scan should succeed");
        assert_eq!(found.len(), 1);
        assert_eq!(
            notice_value(&found)["counts"]["oversized_file"],
            serde_json::json!(1)
        );
    }

    #[test]
    fn symbol_traversal_stops_at_file_limit_after_bounded_read() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        write_rust(&workspace.path().join("sample.rs"), "fn needle() {}\n");
        write_rust(&workspace.path().join("second.rs"), "fn needle() {}\n");
        let mut state = TraversalState {
            file_limit: 1,
            ..TraversalState::new()
        };
        let mut results = Vec::new();

        list_dir_symbols(
            workspace.path(),
            workspace.path(),
            None,
            &mut results,
            &mut state,
            0,
        )
        .expect("listing should succeed");

        assert_eq!(results.len(), 1);
        assert!(state.exhausted);
        assert_eq!(state.file_limit_hits, 1);
        assert_eq!(state.aggregate_byte_limit_hits, 0);
        assert_eq!(state.admitted_files, 1);
        assert_eq!(state.admitted_bytes, "fn needle() {}\n".len());

        let mut scan_state = TraversalState {
            file_limit: 1,
            ..TraversalState::new()
        };
        let mut scan_results = Vec::new();
        scan_dir(
            workspace.path(),
            workspace.path(),
            "needle",
            &mut scan_results,
            &mut scan_state,
            0,
        )
        .expect("scan should succeed");
        assert_eq!(scan_results.len(), 1);
        assert!(scan_state.exhausted);
        assert_eq!(scan_state.file_limit_hits, 1);
        assert_eq!(scan_state.admitted_files, 1);
    }

    #[test]
    fn symbol_traversal_stops_at_aggregate_byte_limit() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        let source = "fn needle() {}\n";
        write_rust(&workspace.path().join("sample.rs"), source);
        write_rust(&workspace.path().join("second.rs"), source);
        let mut state = TraversalState {
            aggregate_byte_limit: source.len(),
            ..TraversalState::new()
        };
        let mut results = Vec::new();

        list_dir_symbols(
            workspace.path(),
            workspace.path(),
            None,
            &mut results,
            &mut state,
            0,
        )
        .expect("listing should succeed");

        assert_eq!(results.len(), 1);
        assert!(state.exhausted);
        assert_eq!(state.file_limit_hits, 0);
        assert_eq!(state.aggregate_byte_limit_hits, 1);
        assert_eq!(state.admitted_files, 1);
        assert_eq!(state.admitted_bytes, source.len());

        let mut scan_state = TraversalState {
            aggregate_byte_limit: source.len(),
            ..TraversalState::new()
        };
        let mut scan_results = Vec::new();
        scan_dir(
            workspace.path(),
            workspace.path(),
            "needle",
            &mut scan_results,
            &mut scan_state,
            0,
        )
        .expect("scan should succeed");
        assert_eq!(scan_results.len(), 1);
        assert!(scan_state.exhausted);
        assert_eq!(scan_state.aggregate_byte_limit_hits, 1);
        assert_eq!(scan_state.admitted_bytes, source.len());
    }

    #[test]
    fn symbol_traversal_counts_refused_depth() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        fs::create_dir(workspace.path().join("child")).expect("child directory should be created");
        let mut state = TraversalState {
            depth_limit: 0,
            file_limit: MAX_FILES,
            aggregate_byte_limit: MAX_TOTAL_BYTES,
            ..TraversalState::default()
        };
        let mut results = Vec::new();

        list_dir_symbols(
            workspace.path(),
            workspace.path(),
            None,
            &mut results,
            &mut state,
            0,
        )
        .expect("listing should succeed");

        assert!(results.is_empty());
        assert_eq!(state.depth_limit_hits, 1);
        assert!(!state.exhausted);

        let mut scan_state = TraversalState {
            depth_limit: 0,
            file_limit: MAX_FILES,
            aggregate_byte_limit: MAX_TOTAL_BYTES,
            ..TraversalState::default()
        };
        let mut scan_results = Vec::new();
        scan_dir(
            workspace.path(),
            workspace.path(),
            "needle",
            &mut scan_results,
            &mut scan_state,
            0,
        )
        .expect("scan should succeed");
        assert!(scan_results.is_empty());
        assert_eq!(scan_state.depth_limit_hits, 1);
    }

    #[test]
    fn symbol_traversal_refuses_production_depth_65() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        let mut deepest = workspace.path().to_path_buf();
        for _ in 0..=MAX_DEPTH {
            deepest.push("d");
            fs::create_dir(&deepest).expect("nested directory should be created");
        }

        let listed = list_symbols_in_path(workspace.path(), None).expect("listing should succeed");
        assert_eq!(listed.len(), 1);
        assert_eq!(
            notice_value(&listed)["counts"]["depth_limit"],
            serde_json::json!(1)
        );

        let found = scan_workspace(workspace.path(), "needle").expect("scan should succeed");
        assert_eq!(found.len(), 1);
        assert_eq!(
            notice_value(&found)["counts"]["depth_limit"],
            serde_json::json!(1)
        );
    }

    #[test]
    fn symbol_traversal_reason_order_is_stable() {
        let state = TraversalState {
            symlink_skipped: 2,
            oversized_file: 3,
            depth_limit_hits: 4,
            file_limit_hits: 1,
            aggregate_byte_limit_hits: 1,
            admitted_files: 7,
            admitted_bytes: 2048,
            ..TraversalState::new()
        };
        let notice = state.notice().expect("notice should be present");

        assert_eq!(
            notice.reasons,
            [
                "symlink_skipped",
                "oversized_file",
                "depth_limit",
                "file_limit",
                "aggregate_byte_limit"
            ]
        );
        assert_eq!(notice.admitted_files, 7);
        assert_eq!(notice.admitted_bytes, 2048);
    }

    #[cfg(unix)]
    #[test]
    fn symbol_traversal_follows_root_symlink_but_skips_descendant_symlinks() {
        use std::os::unix::fs::symlink;

        let workspace = tempfile::tempdir().expect("tempdir should be created");
        let outside = tempfile::tempdir().expect("outside tempdir should be created");
        let real_root = workspace.path().join("real");
        fs::create_dir(&real_root).expect("real root should be created");
        write_rust(&real_root.join("sample.rs"), "fn needle() {}\n");
        symlink(&real_root, real_root.join("cycle")).expect("cycle symlink should be created");
        let oversized_target = outside.path().join("oversized.rs");
        fs::write(&oversized_target, vec![b' '; MAX_FILE_BYTES + 1])
            .expect("oversized target should be writable");
        symlink(&oversized_target, real_root.join("sample-link.rs"))
            .expect("file symlink should be created");
        let root_link = workspace.path().join("root-link");
        symlink(&real_root, &root_link).expect("root symlink should be created");

        let listed = list_symbols_in_path(&root_link, None).expect("listing should succeed");
        assert_eq!(listed.len(), 2);
        assert!(matches!(listed[0], TraversalOutput::Result(_)));
        assert_eq!(
            notice_value(&listed)["counts"]["symlink_skipped"],
            serde_json::json!(2)
        );

        let found = scan_workspace(&root_link, "needle").expect("scan should succeed");
        assert_eq!(found.len(), 2);
        assert!(matches!(found[0], TraversalOutput::Result(_)));
        assert_eq!(
            notice_value(&found)["counts"]["symlink_skipped"],
            serde_json::json!(2)
        );
    }

    #[test]
    fn direct_file_symbol_listing_retains_unbounded_read_contract() {
        let workspace = tempfile::tempdir().expect("tempdir should be created");
        let path = workspace.path().join("large.rs");
        let mut source = String::from("fn needle() {}\n");
        source.push_str(&" ".repeat(MAX_FILE_BYTES));
        fs::write(&path, source).expect("large direct-file fixture should be writable");

        let listed = list_symbols_in_path(&path, None).expect("direct listing should succeed");
        assert!(!listed.is_empty());
        assert!(
            listed
                .iter()
                .all(|item| matches!(item, TraversalOutput::Result(_)))
        );
    }
}
