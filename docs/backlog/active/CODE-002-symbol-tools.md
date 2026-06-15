# CODE-002: Tree-sitter Symbol Tools

| Field | Value |
|-------|-------|
| Story ID | CODE-002 |
| Priority | P1 |
| Status | Planned |
| Depends On | CODE-001 (Complete), ADR-020 (Accepted), arborium integration (Complete) |
| Blocks | Project structure snapshot; context compression; semantic diff analysis |
| Origin | CODE-001 research conclusions (2026-06-15) |

## Outcome

Talos provides a set of AST-aware code exploration tools that let agents understand
workspace structure with structural precision instead of regex-level text matching.
These tools operate on workspace files through the existing read-only permission
pipeline, using arborium's tree-sitter parsers already available in the dependency
graph.

## Motivation

Current agent code exploration relies on grep, find, and raw file reading. This works
but is fragile:

- `grep AgentRuntime` matches comments, strings, variable names — not just the struct definition
- Finding "all functions in this file" requires LLM to parse the code itself
- Understanding imports/dependencies requires manual file tracing

Tree-sitter gives the agent AST-level precision at CPU speeds — sub-100ms per query,
zero hallucination, no token cost.

## Symbol Tools

### T1: `find_symbol`

```
find_symbol(name: "AgentRuntime") → {
    definition: { file: "src/runtime.rs", line: 42, kind: "struct" },
    references: [{ file: "src/main.rs", line: 15 }, ...]
}
```

- Searches all workspace files for the named symbol
- Returns definition location + all reference locations
- Supports: function, struct, enum, trait, impl, type alias, module
- Falls back to text search for unsupported languages

### T2: `find_references`

```
find_references(symbol: "execute", file: "src/agent.rs") → [
    { file: "src/main.rs", line: 88 },
    { file: "src/cli.rs", line: 156 },
    ...
]
```

- Given a symbol name and definition file, find all usage sites
- Language-aware (respects scope, ignores same-name symbols in different modules)
- Falls back to text grep for unsupported languages

### T3: `list_symbols`

```
list_symbols(path: "src/", kind: "function") → [
    { name: "main", file: "src/main.rs", line: 5, params: [...], return_type: "()" },
    ...
]
```

- Lists all symbols of a given kind in a directory/file
- Kinds: function, struct, enum, trait, impl, module, import, type_alias
- Supports glob patterns for path filtering
- Falls back to empty list for unsupported languages

### T4: `list_imports`

```
list_imports(path: "src/agent.rs") → [
    { module: "std::sync::Arc", symbols: ["Arc"] },
    { module: "crate::runtime", symbols: ["Runtime"] },
    ...
]
```

- Extracts all import/use statements from a file
- Returns module path + imported symbols
- Language-specific parsing (Rust `use`, Python `import`, JS `require/import`, etc.)

## Design

### API Boundary

All tools operate through a single `SymbolQueryEngine` struct wrapping arborium:

```rust
pub struct SymbolQueryEngine {
    // Holds per-language tree-sitter Language references (from arborium)
}

impl SymbolQueryEngine {
    pub fn find_symbol(&self, workspace: &Path, name: &str) -> Result<SymbolResult>;
    pub fn find_references(&self, workspace: &Path, symbol: &str, file: &Path) -> Result<Vec<Location>>;
    pub fn list_symbols(&self, path: &Path, kind: SymbolKind) -> Result<Vec<SymbolInfo>>;
    pub fn list_imports(&self, path: &Path) -> Result<Vec<ImportInfo>>;
}
```

- Language auto-detection via file extension → arborium `get_language()`
- Unsupported languages → fall back to text-based grep for `find_symbol`/`find_references`, empty results for `list_symbols`/`list_imports`
- All file access through workspace root boundary (existing permission pipeline)

### Integration Path

These tools become **Agent Tools** registered in `talos-tools/src/lib.rs`, available to
the agent via the standard tool call interface. The LLM decides when to use them — no
mandatory pre-processing.

```
User: "Find all usages of AgentRuntime"
  → Agent calls find_references("AgentRuntime", "src/agent.rs")
  → Returns structured results
  → Agent presents findings to user
```

## Acceptance Criteria

- [ ] `find_symbol` returns definition + references for Rust, Python, TypeScript, Go
- [ ] `find_references` returns all call sites across workspace files
- [ ] `list_symbols` returns structured symbol list for a given path and kind
- [ ] `list_imports` returns import statements for Rust/Python/TS files
- [ ] Unsupported languages fall back to text grep (find_symbol/find_references) or empty (list_symbols/list_imports)
- [ ] All file access is read-only and workspace-root-bounded
- [ ] Per-file parse results cached for the duration of a single agent turn
- [ ] `cargo test --workspace` passes
- [ ] New `cargo clippy --workspace -- -D warnings` passes

## Non-Goals

- No persistent workspace index (deferred to MEM-001)
- No cross-file call graph or dependency graph (deferred to later iteration)
- No write-capable refactoring tools
- No language server protocol (LSP) integration
- No auto-detection of "project type" (cargo/poetry/npm/etc.)

## Required Reads

- `docs/backlog/active/CODE-001-tree-sitter-code-analysis-research.md`
- `docs/decisions/020-tree-sitter-code-analysis.md`
- `crates/talos-tui/src/highlight.rs` (existing arborium integration pattern)
- `crates/talos-tools/src/lib.rs` (current tool registry)
- `docs/decisions/010-git-search-tool-dependency-boundary.md`
