# ADR-020: Tree-sitter for Code Analysis and Syntax Highlighting

- **Status**: Accepted
- **Date**: 2026-06-15
- **Story**: CODE-001 / TUI-006

## Context

Talos needs structured code analysis capabilities for two use cases:
1. **Syntax highlighting** in the TUI for code blocks in LLM responses (TUI-006)
2. **Future AST-aware tools** for workspace file analysis and symbol extraction

`tree-sitter` is the dominant Rust-ecosystem parser library for this purpose. It provides
incremental parsing, concrete syntax trees, and pre-built grammar crates for 100+ languages.
However, it introduces C runtime code (the tree-sitter core parser engine) and generated C parser
code (per-language grammar crates). AGENTS.md Hard Constraint #1 requires that any C/C++ bindings
be approved by ADR.

### Dependency Analysis

The tree-sitter Rust ecosystem consists of:

| Crate | Purpose | C Code? |
|-------|---------|---------|
| `tree-sitter` (core binding) | Rust wrapper around tree-sitter C runtime | Yes — links to C parser runtime |
| `tree-sitter-highlight` | Syntax highlighting query engine | No (pure Rust) |
| Per-language grammar crates | Generated parser code for specific languages | Yes — contains generated C code |
| `arborium` | Bundles ~100 grammar crates behind feature flags | Yes — includes all selected grammar crates |

This pattern is analogous to ADR-008 (bundled SQLite): a proven C library providing unique
capability, statically linked, with no host runtime dependency.

The CODE-001 research recommended:
- `arborium` as the single grammar dependency (replaces 23 individual crates)
- TUI syntax highlighting as the first use case (lowest risk, fails closed)
- 23 languages in the initial set: Rust, Python, TypeScript, JavaScript, Go, Java, C, C++, C#,
  Bash, SQL, PowerShell, Lua, Dart, HTML, CSS, JSON, TOML, Markdown, Ruby, PHP, Kotlin, Swift

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Rust-first; no arbitrary C/C++ bindings | Hard | AGENTS.md #1 | Only by ADR |
| No `unsafe` without ADR | Hard | AGENTS.md #2 | Only by ADR |
| TUI code blocks need syntax highlighting | Soft | TUI-006 / backlog priority | Yes |
| Binary size should remain reasonable | Soft | Distribution goal | Yes, mitigated by feature flags |
| Parsing must not block agent turns | Hard | Agent UX | No; requires timeout + fallback |
| All write-capable tools gated by permissions | Hard | AGENTS.md #4 | No |

## Reasoning

Tree-sitter is the only production-grade incremental parser library in the Rust ecosystem with:
- 100+ language grammars, 24M+ crate downloads
- Pre-built syntax highlighting queries (`highlights.scm`) for every language
- MIT license on both the core binding and all grammar crates
- Active maintenance (core v0.26.9, May 2026)

The C dependency is the same class as ADR-008 (bundled SQLite): statically linked, no host
installation required, narrow scope (parsing only), proven safety record.

The first use case (TUI highlighting) operates entirely on LLM-generated text, with zero
filesystem access. This isolates the new dependency from the permission pipeline and keeps
the blast radius minimal.

### Risk Mitigation

| Risk | Mitigation |
|------|------------|
| C runtime in binary | Static linking, same as SQLite (ADR-008) |
| C runtime panic crashes process | `catch_unwind` at integration boundary, degrade to plain text; enforced by AGENTS.md HC #9 |
| Parse failures block turns | Timeout (500ms), fallback to plain text |
| Binary bloat from 23 grammars | arborium feature flags — only selected languages compiled in |
| Unsupported language in code block | Fails closed to plain text rendering |
| Generated C code in grammar crates | No unsafe usage in tree-sitter bindings (verified in crate sources) |

## Decision

1. **Approve** `tree-sitter` and `tree-sitter-highlight` as dependencies under the same
   C-binding exception as ADR-008 (bundled SQLite).
2. **Approve** `arborium` as the single grammar dependency, with language selection via
   feature flags (`lang-rust`, `lang-python`, etc.).
3. **First use case**: TUI code block syntax highlighting (TUI-006 Sub-slice B). Operates
   on LLM output only; no filesystem access.
4. **Future use cases** (symbol extraction, workspace search) require separate ADR
   evaluation for their filesystem and permission implications.
5. The dependency is limited to `talos-tui` crate initially. Expansion to `talos-tools`
   requires a follow-up story.

## Reversal Trigger

Re-evaluate if:
- tree-sitter maintenance stalls for >12 months with unfixed security issues
- Binary size with selected grammars exceeds 5MB beyond current baseline
- A pure-Rust incremental parser with comparable language coverage emerges
- Parse time for typical code blocks consistently exceeds 50ms

## References

- [CODE-001 Research](../backlog/active/CODE-001-tree-sitter-code-analysis-research.md)
- [TUI-006 Code Block Rendering](../backlog/active/TUI-006-code-block-rendering.md)
- [ADR-008 Bundled SQLite](008-sqlite-bundled-storage.md)
- [tree-sitter GitHub](https://github.com/tree-sitter/tree-sitter)
- [arborium crates.io](https://crates.io/crates/arborium)
