# CODE-003: Tree-sitter Usage Pattern Analysis

**Status**: Research
**Priority**: P3
**Source**: User request 2026-06-26
**Iteration**: None yet

## Problem

CODE-002 delivered 4 symbol tools (`find_symbol`, `find_references`, `list_symbols`, `list_imports`) that parse files with tree-sitter on every call. CODE-001 concluded "no cache needed" for TUI highlighting (<500 line code blocks, O(ms) parse). But **agent tool usage is different**: an agent calling `find_symbol` across a 500-file workspace re-parses every file every turn. No research has been done on whether this scales, or whether a workspace-level parse index would be better.

## Scope

Benchmark and decide the optimal tree-sitter usage pattern for agent tool workloads.

### Research questions

1. **Current performance**: What is the per-file parse latency for representative Rust/TypeScript/Python files at 100/500/2000/5000 lines? What is the total latency when `find_symbol` scans a 500-file workspace?

2. **Real-time vs per-session cache**: If we cache parse results for the duration of a session (invalidated on file modification via mtime/hash), what is the speedup? What is the memory cost of holding 500 parsed ASTs?

3. **Persistent workspace index**: Should Talos maintain a persistent tree-sitter index (like ctags/LSP) that incrementally updates on file changes? What are the storage, invalidation, and staleness tradeoffs? Does this overlap with the MEM-001 "Code entities" layer or STORE-001 (storage evaluation)?

4. **Decision boundary**: At what workspace size does real-time parsing become unacceptable? What heuristic should Talos use to switch strategies? (e.g., <100 files → real-time; 100-1000 → session cache; >1000 → persistent index?)

5. **Impact on TOOL-008**: If a persistent index is needed, does it change the tree-sitter parser loading strategy (LazyLock vs feature-gated vs WASM)?

### Non-goals

- No LSP server integration (separate concern).
- No semantic analysis beyond syntax (that's Layer 2 in CODE-001's capability matrix).
- No vector/embedding-based code search (deferred to STORE-001 Spike).

## Acceptance

- [ ] Benchmark report: per-file parse latency, workspace-scale scan latency, cache hit/miss ratios.
- [ ] Decision recorded: real-time / session-cache / persistent-index, with threshold heuristic.
- [ ] If persistent index is recommended: new backlog story created with schema and invalidation design.

## Dependencies

- CODE-001 (Tree-sitter research) — Complete.
- CODE-002 (Symbol tools) — Complete.
- ADR-020 (tree-sitter approval).

## Required Reads

- `docs/backlog/active/CODE-001-tree-sitter-code-analysis-research.md` (especially RQ#6 and capability matrix)
- `docs/backlog/active/CODE-002-symbol-tools.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/decisions/020-tree-sitter-code-analysis.md`
- `docs/backlog/active/TOOL-007-tool-set-design-audit.md` (orthogonality dimension)
