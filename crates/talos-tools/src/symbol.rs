//! AST-aware symbol query engine using arborium/tree-sitter.
//!
//! Provides structural code exploration at AST precision — find symbols,
//! locate references, list functions/classes, and extract imports.

use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;

use arborium::tree_sitter; // arborium re-exports tree-sitter

/// Maps file extensions to arborium language names.
fn detect_language(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Some("rust"),
        Some("py") => Some("python"),
        Some("ts") | Some("tsx") => Some("typescript"),
        Some("js") | Some("jsx") | Some("mjs") => Some("javascript"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("c") | Some("h") => Some("c"),
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => Some("cpp"),
        Some("cs") => Some("c-sharp"),
        Some("sh") | Some("bash") | Some("zsh") => Some("bash"),
        Some("sql") => Some("sql"),
        Some("ps1") => Some("powershell"),
        Some("lua") => Some("lua"),
        Some("dart") => Some("dart"),
        Some("html") => Some("html"),
        Some("css") => Some("css"),
        Some("json") => Some("json"),
        Some("yaml") | Some("yml") => Some("yaml"),
        Some("toml") => Some("toml"),
        Some("md") => Some("markdown"),
        Some("rb") => Some("ruby"),
        Some("php") => Some("php"),
        Some("kt") | Some("kts") => Some("kotlin"),
        Some("swift") => Some("swift"),
        _ => None,
    }
}

/// Result from a find_symbol query.
#[derive(Debug, Serialize, Clone)]
struct SymbolResult {
    name: String,
    kind: String,
    definition: Option<SourceLocation>,
    references: Vec<SourceLocation>,
}

#[derive(Debug, Serialize, Clone)]
struct SourceLocation {
    file: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Serialize, Clone)]
struct SymbolInfo {
    name: String,
    kind: String,
    file: String,
    line: usize,
}

#[derive(Debug, Serialize, Clone)]
struct ImportInfo {
    module: String,
    symbols: Vec<String>,
    file: String,
    line: usize,
}

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

fn scan_workspace(root: &Path, name: &str) -> Result<Vec<SymbolResult>, String> {
    let mut results = Vec::new();
    scan_dir(root, root, name, &mut results)?;
    Ok(results)
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    name: &str,
    results: &mut Vec<SymbolResult>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            scan_dir(root, &path, name, results)?;
        } else if path.is_file()
            && let Some(result) = find_symbol_in_file(&path, root, name)
        {
            results.push(result);
        }
    }
    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name == "target" || name == "node_modules" || name == ".git" || name.starts_with('.')
}

fn find_symbol_in_file(path: &Path, root: &Path, name: &str) -> Option<SymbolResult> {
    let lang = detect_language(path)?;
    let code = fs::read_to_string(path).ok()?;
    let mut parser = arborium::tree_sitter::Parser::new();
    parser.set_language(&arborium::get_language(lang)?).ok()?;
    let tree = parser.parse(&code, None)?;

    let root_node = tree.root_node();
    let source = code.as_bytes();

    let kinds: &[&str] = &[
        "function_item",
        "function_definition",
        "struct_item",
        "class_definition",
        "enum_item",
        "trait_item",
        "impl_item",
        "type_alias",
        "variable_declaration",
        "method_definition",
        "function_declaration",
        "interface_declaration",
    ];

    let cursor = &mut root_node.walk();
    let mut found_def = None;
    let mut refs = Vec::new();

    for node in root_node.children(&mut cursor.clone()) {
        check_node(
            &node,
            source,
            name,
            kinds,
            root,
            path,
            &mut found_def,
            &mut refs,
        );
    }

    found_def.map(|def| SymbolResult {
        name: name.to_string(),
        kind: def.kind,
        definition: Some(SourceLocation {
            file: def.rel_path,
            line: def.line,
            column: 0,
        }),
        references: refs,
    })
}

struct DefInfo {
    kind: String,
    rel_path: String,
    line: usize,
}

#[allow(warnings)]
fn check_node(
    node: &tree_sitter::Node,
    source: &[u8],
    name: &str,
    kinds: &[&str],
    root: &Path,
    path: &Path,
    found_def: &mut Option<DefInfo>,
    refs: &mut Vec<SourceLocation>,
) {
    let kind = node.kind();
    let is_candidate =
        kinds.contains(&kind) || kind.contains("definition") || kind.contains("declaration");

    if is_candidate
        && let Some(ident) = find_identifier(node, source)
        && ident == name
    {
        let line = node.start_position().row + 1;
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let location = SourceLocation {
            file: rel_path.clone(),
            line,
            column: 0,
        };

        if found_def.is_none() {
            *found_def = Some(DefInfo {
                kind: kind.to_string(),
                rel_path,
                line,
            });
        } else {
            refs.push(location);
        }
    }

    if node.child_count() > 0 && !is_terminal_node(kind) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            check_node(&child, source, name, kinds, root, path, found_def, refs);
        }
    }
}

fn find_identifier(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == "identifier" || child.kind() == "name" {
            return child.utf8_text(source).ok().map(|s| s.to_string());
        }
    }
    None
}

fn is_terminal_node(kind: &str) -> bool {
    matches!(
        kind,
        "identifier" | "string" | "comment" | "number" | "boolean" | "null"
    )
}

fn find_refs_in_file(path: &Path, name: &str) -> Result<Vec<SourceLocation>, String> {
    let lang = detect_language(path).ok_or_else(|| "unsupported file type".to_string())?;
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut parser = arborium::tree_sitter::Parser::new();
    parser
        .set_language(
            &arborium::get_language(lang).ok_or_else(|| "language not loaded".to_string())?,
        )
        .map_err(|e| e.to_string())?;
    let tree = parser
        .parse(&code, None)
        .ok_or_else(|| "parse failed".to_string())?;

    let root_node = tree.root_node();
    let source = code.as_bytes();
    let mut cursor = root_node.walk();

    let mut locations = Vec::new();
    let mut visited = false;
    find_all_identifiers(
        &root_node,
        source,
        name,
        &mut cursor,
        &mut locations,
        &mut visited,
        path,
    );
    Ok(locations)
}

#[allow(warnings)]
fn find_all_identifiers(
    node: &tree_sitter::Node,
    source: &[u8],
    name: &str,
    cursor: &mut tree_sitter::TreeCursor,
    locations: &mut Vec<SourceLocation>,
    visited: &mut bool,
    path: &Path,
) {
    if (node.kind() == "identifier" || node.kind() == "name")
        && let Ok(text) = node.utf8_text(source)
        && text == name
    {
        locations.push(SourceLocation {
            file: path.to_string_lossy().to_string(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        });
    }

    if node.child_count() > 0 {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                find_all_identifiers(&child, source, name, cursor, locations, visited, path);
            }
        }
    }
}

fn list_symbols_in_path(path: &Path, kind_filter: Option<&str>) -> Result<Vec<SymbolInfo>, String> {
    let mut results = Vec::new();
    if path.is_dir() {
        list_dir_symbols(path, path, kind_filter, &mut results)?;
    } else if path.is_file() {
        list_file_symbols(path, path, kind_filter, &mut results)?;
    }
    Ok(results)
}

fn list_dir_symbols(
    root: &Path,
    dir: &Path,
    kind_filter: Option<&str>,
    results: &mut Vec<SymbolInfo>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            list_dir_symbols(root, &path, kind_filter, results)?;
        } else if path.is_file() {
            list_file_symbols(root, &path, kind_filter, results)?;
        }
    }
    Ok(())
}

fn list_file_symbols(
    root: &Path,
    path: &Path,
    kind_filter: Option<&str>,
    results: &mut Vec<SymbolInfo>,
) -> Result<(), String> {
    let lang = match detect_language(path) {
        Some(l) => l,
        None => return Ok(()),
    };
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut parser = arborium::tree_sitter::Parser::new();
    let lang_ref = arborium::get_language(lang).ok_or_else(|| "language not loaded".to_string())?;
    parser.set_language(&lang_ref).map_err(|e| e.to_string())?;
    let tree = parser
        .parse(&code, None)
        .ok_or_else(|| "parse failed".to_string())?;

    let root_node = tree.root_node();
    let source = code.as_bytes();
    let mut cursor = root_node.walk();

    let symbol_kinds: &[&str] = &[
        "function_item",
        "function_definition",
        "method_definition",
        "function_declaration",
        "struct_item",
        "class_definition",
        "enum_item",
        "trait_item",
        "impl_item",
        "type_alias",
        "variable_declaration",
        "module",
    ];

    collect_symbols(
        &root_node,
        source,
        symbol_kinds,
        kind_filter,
        root,
        path,
        &mut cursor,
        results,
    );
    Ok(())
}

#[allow(warnings)]
fn collect_symbols(
    node: &tree_sitter::Node,
    source: &[u8],
    symbol_kinds: &[&str],
    kind_filter: Option<&str>,
    root: &Path,
    path: &Path,
    cursor: &mut tree_sitter::TreeCursor,
    results: &mut Vec<SymbolInfo>,
) {
    let kind = node.kind();

    if symbol_kinds.contains(&kind) || kind.contains("definition") || kind.contains("declaration") {
        if let Some(filter) = kind_filter {
            let kind_lower = kind.to_lowercase();
            let filter_lower = filter.to_lowercase();
            let matches_filter = kind_lower.contains(&filter_lower)
                || (filter_lower == "function" && kind_lower.contains("function"))
                || (filter_lower == "struct" && kind_lower.contains("struct"))
                || (filter_lower == "class" && kind_lower.contains("class"))
                || (filter_lower == "enum" && kind_lower.contains("enum"))
                || (filter_lower == "trait" && kind_lower.contains("trait"))
                || (filter_lower == "interface" && kind_lower.contains("interface"));

            if !matches_filter {
                if node.child_count() > 0 {
                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i as u32) {
                            collect_symbols(
                                &child,
                                source,
                                symbol_kinds,
                                kind_filter,
                                root,
                                path,
                                cursor,
                                results,
                            );
                        }
                    }
                }
                return;
            }
        }

        if let Some(ident) = find_identifier(node, source) {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            results.push(SymbolInfo {
                name: ident,
                kind: kind.to_string(),
                file: rel_path,
                line: node.start_position().row + 1,
            });
        }
    }

    if node.child_count() > 0 {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                collect_symbols(
                    &child,
                    source,
                    symbol_kinds,
                    kind_filter,
                    root,
                    path,
                    cursor,
                    results,
                );
            }
        }
    }
}

fn list_imports_in_file(path: &Path) -> Result<Vec<ImportInfo>, String> {
    let lang = detect_language(path).ok_or_else(|| "unsupported file type".to_string())?;
    let code = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut parser = arborium::tree_sitter::Parser::new();
    parser
        .set_language(
            &arborium::get_language(lang).ok_or_else(|| "language not loaded".to_string())?,
        )
        .map_err(|e| e.to_string())?;
    let tree = parser
        .parse(&code, None)
        .ok_or_else(|| "parse failed".to_string())?;

    let root_node = tree.root_node();
    let source = code.as_bytes();

    let mut results = Vec::new();
    collect_imports(&root_node, source, path, &mut results);
    Ok(results)
}

fn collect_imports(
    node: &tree_sitter::Node,
    source: &[u8],
    path: &Path,
    results: &mut Vec<ImportInfo>,
) {
    let kind = node.kind();

    let is_import = kind.contains("use_declaration")
        || kind.contains("import")
        || kind == "import_statement"
        || kind == "import_from_statement"
        || kind == "require_call"
        || kind == "lexical_declaration";

    if is_import {
        let import_text = node.utf8_text(source).unwrap_or("").to_string();
        let symbols: Vec<String> = import_text
            .lines()
            .flat_map(|line| {
                line.trim()
                    .trim_start_matches("use ")
                    .trim_start_matches("import ")
                    .trim_start_matches("from ")
                    .trim_start_matches("require(")
                    .trim_start_matches("const ")
                    .trim_start_matches("let ")
                    .trim_start_matches("var ")
                    .split([',', ';', '{', '}'])
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty() && s != "from" && s != "import" && s != "require")
                    .collect::<Vec<_>>()
            })
            .collect();

        if !symbols.is_empty() {
            let module = import_text
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_string();
            results.push(ImportInfo {
                module,
                symbols,
                file: path.to_string_lossy().to_string(),
                line: node.start_position().row + 1,
            });
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            collect_imports(&child, source, path, results);
        }
    }
}
