use crate::SourceLocation;

/// Find a symbol in caller-supplied source, safely degrading to no result on parser failure.
pub fn find_symbol(
    language: &str,
    code: &str,
    root: &Path,
    path: &Path,
    name: &str,
) -> Option<SymbolResult> {
    crate::guarded(|| Ok(find_symbol_inner(language, code, root, path, name)))
        .ok()
        .flatten()
}

/// Find references without reading the caller-supplied path label.
pub fn find_references(
    language: &str,
    code: &str,
    path: &Path,
    name: &str,
) -> Result<Vec<SourceLocation>, String> {
    crate::guarded(|| find_references_inner(language, code, path, name))
}

/// List imports without reading the caller-supplied path label.
pub fn list_imports(language: &str, code: &str, path: &Path) -> Result<Vec<ImportInfo>, String> {
    crate::guarded(|| list_imports_inner(language, code, path))
}
use arborium::tree_sitter;
use serde::Serialize;
use std::path::Path;

/// Definition lookup result in the existing symbol-tool JSON shape.
#[derive(Debug, Serialize, Clone)]
pub struct SymbolResult {
    /// Requested name.
    pub name: String,
    /// Definition syntax kind.
    pub kind: String,
    /// First matching definition.
    pub definition: Option<SourceLocation>,
    /// Subsequent matching declarations.
    pub references: Vec<SourceLocation>,
}

/// Existing import extraction result.
#[derive(Debug, Serialize, Clone)]
pub struct ImportInfo {
    /// Module token extracted from the statement.
    pub module: String,
    /// Statement components retained by the compatibility parser.
    pub symbols: Vec<String>,
    /// Caller-supplied file label.
    pub file: String,
    /// One-based source line.
    pub line: usize,
}

/// Find a definition and declaration references in caller-supplied source.
fn find_symbol_inner(
    language: &str,
    code: &str,
    root: &Path,
    path: &Path,
    name: &str,
) -> Option<SymbolResult> {
    let tree = crate::parse_builtin(language, code).ok()?;

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

/// Find identifier references without reading files.
fn find_references_inner(
    lang: &str,
    code: &str,
    path: &Path,
    name: &str,
) -> Result<Vec<SourceLocation>, String> {
    let tree = crate::parse_builtin(lang, code)?;

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

/// Extract imports from caller-supplied source without reading files.
fn list_imports_inner(lang: &str, code: &str, path: &Path) -> Result<Vec<ImportInfo>, String> {
    let tree = crate::parse_builtin(lang, code)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_only_operations_preserve_locations_and_import_shape() {
        let source = "use std::fmt;\nfn greet() {}\nfn main() { greet(); }\n";
        let path = Path::new("src/lib.rs");
        let definition =
            find_symbol("rust", source, Path::new(""), path, "greet").expect("definition");
        let location = definition.definition.expect("location");
        assert_eq!(
            (location.file.as_str(), location.line, location.column),
            ("src/lib.rs", 2, 0)
        );
        let references = find_references("rust", source, path, "greet").expect("references");
        assert_eq!(
            references
                .iter()
                .map(|r| (r.line, r.column))
                .collect::<Vec<_>>(),
            vec![(2, 4), (3, 13)]
        );
        let imports = list_imports("rust", source, path).expect("imports");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module, "std::fmt;");
        assert_eq!(imports[0].symbols, vec!["std::fmt"]);
        let symbols = crate::symbol::list_symbols("rust", source, "src/lib.rs", Some("function"))
            .expect("symbols");
        assert_eq!(
            symbols.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["greet", "main"]
        );
    }

    #[test]
    fn unsupported_language_fails_without_reading_a_path() {
        let path = Path::new("/nonexistent/never-read.rs");
        assert!(find_symbol("unknown-language", "x", Path::new(""), path, "x").is_none());
        assert_eq!(
            find_references("unknown-language", "x", path, "x").expect_err("unsupported"),
            "language not loaded"
        );
        assert!(list_imports("unknown-language", "x", path).is_err());
    }

    #[test]
    fn parser_panic_is_returned_as_error() {
        assert_eq!(
            crate::guarded::<()>(|| panic!("fixture")),
            Err("parse failed".into())
        );
    }

    #[test]
    fn deeply_nested_source_is_rejected_before_recursive_visitors() {
        let source = format!("fn f() {{ {} 0; {} }}", "{".repeat(150), "}".repeat(150));
        assert_eq!(
            list_imports("rust", &source, Path::new("fixture.rs")).expect_err("depth budget"),
            "parse budget exceeded"
        );
        assert!(
            find_symbol("rust", &source, Path::new(""), Path::new("fixture.rs"), "f").is_none()
        );
    }
}
