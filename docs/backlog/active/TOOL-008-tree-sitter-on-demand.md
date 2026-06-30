# TOOL-008: Tree-sitter Parser On-Demand Loading

**Status**: Planned
**Priority**: P3 (Medium-term)
**Source**: Binary size analysis 2026-06-20
**Depends on**: CODE-001, CODE-002 (tree-sitter infrastructure); TOOL-007 (tool set audit)

## Problem

Talos embeds 23 tree-sitter language parsers (via arborium) as compile-time static
constants. Each parser's compiled grammar is a large data structure (1-9 MB per language),
contributing ~40 MB to the binary:

| Parser | Size |
|---|---|
| SQL | 9.3 MB |
| Kotlin | 5.3 MB |
| C++ | 5.2 MB |
| C# | 4.9 MB |
| Swift | 3.3 MB |
| Ruby | 2.1 MB |
| Rust, TypeScript, Bash, PHP, etc. | 1-2 MB each |
| 13 smaller parsers | <1 MB each |

The binary (64 MB release) spends ~60% of its size on parsers the user may never
use in a given session. A session working on a Rust project doesn't need the Kotlin,
SQL, or Swift parsers.

## Scope

Replace compile-time embedding of tree-sitter parsers with on-demand loading:

### Phase 1: Lazy static loading

- Keep parsers embedded but gate them behind `LazyLock` so they only
  materialize when first used.
- **Benefit**: zero code change to consumers; parsers still embedded but
  not all loaded at startup.
- **Binary size**: unchanged (parsers still in binary), but RSS/memory
  footprint reduced.

### Phase 2: Feature-gated parser selection

- Gate each language parser behind a Cargo feature flag.
- Default: only include the top 5-8 most common languages (Rust, Python,
  TypeScript, Go, C, C++, JavaScript, Bash).
- `--all-languages` feature flag for full set.
- **Binary size**: 64 MB → ~25-30 MB (default), ~40 MB (--all-languages).

### Phase 3: Runtime parser loading (long-term)

- Compile parsers to WASM modules using `tree-sitter` WASM compilation target.
- Load via PLUGIN-001 WASM runtime infrastructure (sandbox, lifecycle, security).
- Download and cache via DIST-001 optional asset distribution.
- **Binary size**: 64 MB → ~20-25 MB (core binary only, parsers as WASM modules).

> **2026-06-30 note.** ADR-027/028/029/030 unblocked plugin architecture decisions. Phase 3 is no
> longer blocked on missing architecture decisions, but it still depends on PLUGIN-001's local WASM
> runtime adapter existing and passing its dependency/security review before parser modules can be
> loaded at runtime.

## Non-Goals

- Do not remove tree-sitter as the code analysis engine.
- Do not change the `AgentTool` API for symbol tools.
- Do not add network dependencies at startup (Phase 2 is a compile-time fix).
- Phase 3 is gated on DIST-001 and a security review.

## Acceptance Criteria (Phase 2)

- [ ] Each language parser is gated behind a Cargo feature flag
      (`lang-rust`, `lang-python`, etc.).
- [ ] Default feature set includes only commonly-used languages
      (Rust, Python, TypeScript, Go, C, C++, JavaScript, Bash).
- [ ] `cargo build --release -p talos-cli` binary size ≤ 35 MB.
- [ ] `cargo test -p talos-tools` (symbol tools) pass with default features.
- [ ] `cargo test -p talos-tools --all-features` pass with full parser set.
- [ ] TUI syntax highlighting degrades gracefully to plain code blocks
      when a language parser is unavailable.

## Relationship To Other Requirements

| Requirement | Relationship |
|---|---|
| CODE-001 | Tree-sitter research: parser loading architecture |
| CODE-002 | Symbol tools: must work with feature-gated parsers |
| DIST-001 | Phase 3 requires optional asset distribution |
| PLUGIN-001 | Phase 3 reuses WASM runtime for parser module loading |
| TOOL-007 | Tool set audit: code intelligence tool relevance |

## Required Reads

- `crates/talos-tools/Cargo.toml` — arborium feature flags
- `crates/talos-tools/src/symbol.rs` — parser usage
- `crates/talos-tui/src/highlight.rs` — TUI syntax highlighting
- `docs/decisions/020-tree-sitter-code-analysis.md`
