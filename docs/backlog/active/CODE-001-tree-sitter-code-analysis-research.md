# CODE-001: Tree-sitter Code Analysis Research

| Field | Value |
|-------|-------|
| Story ID | CODE-001 |
| Priority | P2 |
| Status | Complete (2026-06-15) |
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

## Research Conclusions (2026-06-15)

### Decision Summary

| Question | Answer |
|----------|--------|
| Crate selection | `arborium` (bundles all grammars via feature flags, re-exports `tree-sitter`) |
| Language support | 23 languages via `arborium` features: Rust, Python, TS/JS, Go, Java, C/C++, C#, Bash, SQL, PowerShell, Lua, Dart, HTML, CSS, JSON, TOML, Markdown, Ruby, PHP, Kotlin, Swift |
| ADR required | **Yes** — ADR-020 created and accepted (2026-06-15). C runtime exception under same pattern as ADR-008 (bundled SQLite) |
| First use case | **TUI syntax highlighting** (Option A) — delivered 2026-06-15 in `crates/talos-tui/src/highlight.rs`. `catch_unwind` boundary per AGENTS.md HC #9 |
| Failure handling | Unsupported lang → plain text; parse error → plain text; timeout 500ms → plain text; C runtime panic → `catch_unwind` → plain text |
| Cache | No cache needed. LLM code blocks are <500 lines, parsing is O(ms) |
| Permission | TUI highlighting: none (operates on LLM output). Future workspace parsing: read-only via existing permission pipeline |

### Expanded Capability Value Matrix

Tree-sitter's value extends far beyond syntax highlighting. Talos should treat it as a **code understanding engine**, prioritized above embedding and local LLM models.

| Capability | Value | Effort | Priority | Depends On |
|------------|-------|--------|----------|------------|
| TUI Syntax Highlighting | UX polish for code blocks | ✅ Done | P2 | CODE-001, ADR-020 |
| **Symbol Tools** (FindSymbol, FindReferences, ListFunctions, ListImports) | Agent structural code understanding; replaces grep with AST-level precision | ~200 LOC | **P1** | arborium (ready) |
| **Project Structure Snapshot** | Replace "dump all files to LLM" with structured summary, 10-100x token savings | ~100 LOC (on top of Symbol Tools) | **P1** | Symbol Tools |
| Context Compression (Code Graph) | 1M-line repos → only relevant nodes to LLM | ~500 LOC | P2 | MEM-001, Symbol Tools |
| Semantic Diff Analysis | "Explain this PR" with fn/add/delete/param changes, not raw `+/-` text | ~300 LOC | P3 | GIT-001 |
| Call Graph / References Index | Cross-file symbol resolution, persistent index | ~1000 LOC | P4 | Layered Memory (MEM-001) |

### Recommended Roadmap

```
Completed:    TUI Syntax Highlighting (#TUI-006)
                ↓
Next:         Symbol Tools (#CODE-002) — FindSymbol, FindReferences, ListFunctions, ListImports
                ↓
Then:         Project Structure Snapshot (built on Symbol Tools)
                ↓
Later:        Context Compression → Semantic Diff → Call Graph Index
```

### Layered Capability Model (Talos-Specific)

```
Layer 0: Text
    Regex · Glob · Ripgrep
    (TOOL-001 planned)

Layer 1: Syntax
    Tree-sitter ← WE ARE HERE
    Symbol Tools, Structure Snapshot
    (#CODE-002 next)

Layer 2: Semantic
    Embedding · Reranker
    (MEM-001 planned)

Layer 3: Reasoning
    LLM (existing provider integration)
```

Tree-sitter is layer 1 — the bridge between raw text search and semantic understanding. It delivers zero-hallucination, CPU-level, sub-100ms responses for structural code queries, making it a foundational capability for any coding agent runtime.

### Next Executable Story

**CODE-002: Tree-sitter Symbol Tools** — implement `FindSymbol`, `FindReferences`, `ListFunctions`, `ListClasses`, `ListImports` using arborium's existing parser infrastructure. See `docs/backlog/active/CODE-002-symbol-tools.md`.
