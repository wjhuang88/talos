# CODE-001: Tree-sitter Code Analysis Research

| Field | Value |
|-------|-------|
| Story ID | CODE-001 |
| Priority | P2 |
| Status | Research |
| Depends On | ADR-010 dependency boundary; Rust-first hard constraint |
| Blocks | TUI-006 syntax highlighting sub-slice; future AST-aware file/search tools |
| Origin | User request on 2026-06-12 to research `https://github.com/tree-sitter/tree-sitter` for code analysis |

## Problem

Talos needs a researched path for structured code analysis. `tree-sitter` is a strong candidate
because it provides incremental parsing and concrete syntax trees for many languages, but adopting
it changes the dependency boundary: the upstream runtime is a C library with Rust bindings and
language grammars may introduce generated C/C++ parser code.

This is not ready for direct implementation. The first slice must answer whether tree-sitter fits
Talos' Rust-first, safety-first constraints and which limited product use case should land first.

## Research Questions

1. Which Rust crates should be used: `tree-sitter`, grammar crates, `tree-sitter-highlight`, or a
   smaller wrapper owned by Talos?
2. Which languages should be supported in the first implementation slice, and how are grammar
   versions pinned?
3. Does adding tree-sitter require a new ADR because it introduces C runtime or generated parser
   code beyond the current `libc` and bundled SQLite exceptions?
4. Should the first use case be TUI code block syntax highlighting, AST-aware workspace search, or
   code structure extraction for tools?
5. How should parsing failures, unknown languages, very large files, and invalid syntax be handled
   without blocking agent turns?
6. What cache shape is acceptable: no cache, per-file parse cache, or future workspace index?
7. How does this interact with permissions and workspace boundaries for read-only code analysis?

## Candidate First Slices

### Option A: TUI syntax highlighting only

- Parse completed Markdown code blocks by language tag.
- No workspace file reads.
- Fails closed to plain code rendering when language or parse fails.
- Lowest permission and indexing risk, but introduces dependency mainly for UI polish.

### Option B: AST-aware workspace symbol extraction

- Read workspace files through existing read-only path rules.
- Extract functions/classes/modules for supported languages.
- More useful for agent code analysis, but higher performance and cache risk.

### Option C: Shared parser service

- Add a small internal crate that owns tree-sitter parser setup and exposes stable analysis APIs.
- TUI highlighting and future tools consume the same boundary.
- Highest design value, but should wait until Option A/B evidence proves enough shared behavior.

## Acceptance Criteria

- Research note compares at least two implementation options and recommends one first slice.
- Dependency/ADR decision is explicit before any tree-sitter crate is added.
- Supported-language strategy is bounded and version-pinned.
- Failure behavior is specified for unsupported language, parser error, large input, and timeout.
- Permission boundary is documented: code analysis remains read-only unless a later story adds
  write-capable refactoring.
- If implementation is approved, a follow-up executable story is created with tests and runtime
  verification.

## Non-Goals

- No tree-sitter dependency in this research story.
- No syntax highlighting implementation.
- No persistent workspace code index.
- No write-capable refactoring tools.
- No broad language support promise before grammar/version research is complete.

## Required Reads

- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/backlog/active/TUI-006-code-block-rendering.md`
- `docs/proposals/tree-sitter-code-analysis.md`
- `docs/proposals/builtin-workspace-search-tools.md`
- `crates/talos-tools/src/`
- `crates/talos-tui/src/app.rs`
