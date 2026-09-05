use super::SymbolInfo;

/// Parse symbols from source using the reviewed built-in grammar boundary.
pub fn list_symbols(
    language: &str,
    source: &str,
    file: &str,
    kind_filter: Option<&str>,
) -> Result<Vec<SymbolInfo>, String> {
    crate::guarded(|| list_symbols_inner(language, source, file, kind_filter))
}

fn list_symbols_inner(
    language: &str,
    source: &str,
    file: &str,
    kind_filter: Option<&str>,
) -> Result<Vec<SymbolInfo>, String> {
    let tree = crate::parse_builtin(language, source)?;
    let mut output = Vec::new();
    let kinds = [
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
    visit(
        tree.root_node(),
        source.as_bytes(),
        file,
        kind_filter,
        &kinds,
        &mut output,
    );
    Ok(output)
}

fn visit(
    node: arborium::tree_sitter::Node<'_>,
    source: &[u8],
    file: &str,
    filter: Option<&str>,
    kinds: &[&str],
    output: &mut Vec<SymbolInfo>,
) {
    let kind = node.kind();
    let candidate =
        kinds.contains(&kind) || kind.contains("definition") || kind.contains("declaration");
    let matches =
        filter.is_none_or(|f| kind.to_ascii_lowercase().contains(&f.to_ascii_lowercase()));
    if candidate
        && matches
        && let Some(name) = identifier(node, source)
    {
        output.push(SymbolInfo {
            name,
            kind: kind.to_owned(),
            file: file.to_owned(),
            line: node.start_position().row + 1,
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit(child, source, file, filter, kinds, output);
    }
}

fn identifier(node: arborium::tree_sitter::Node<'_>, source: &[u8]) -> Option<String> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == "identifier" || child.kind() == "name")
        .and_then(|child| child.utf8_text(source).ok())
        .map(str::to_owned)
}
