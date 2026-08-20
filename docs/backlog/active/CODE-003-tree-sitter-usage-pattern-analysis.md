# CODE-003: Tree-sitter Usage Pattern Analysis

**Status**: Refinement / Unclaimed
**Priority**: P3
**Type**: Architecture Epic
**Source**: User request 2026-06-26; [GitHub Issue #317](https://github.com/wjhuang88/talos/issues/317)
**Iteration**: None yet

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned - Epic parents are not implementation units |
| Claimed At | Not applicable |
| Source Issue | #317 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None - implementation belongs to claimed child Stories |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Refine and claim one runnable child at a time; the Epic creates no code, schema, dependency or parser-policy authority. |

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

## 2026-08-20 Issue #317 Architecture Expansion

Issue #317 expands the original cache/performance research into progressive workspace intelligence.
The existing CODE-003 owner is retained because both records ask how Tree-sitter-derived facts are
reused across agent exploration; creating a second overlapping Epic would split ownership.

The target direction keeps source files authoritative and treats every index as deletable derived
state. Tree-sitter queries produce language-specific syntax facts; deterministic resolvers may add
explainable candidates; later authoritative providers may promote those candidates without
replacing the occurrence/symbol model. Code-index lifecycle remains separate from durable Talos
Memory.

### Stable Child Map

| Child | Deliverable | Depends On | State |
|---|---|---|---|
| CODE-003-A | Current tool contract and characterization fixtures | CODE-001/002; ADR-020 | Ready / Unclaimed; separate owner exists |
| CODE-003-B | Language registry and query-based normalized syntax facts | A | Refinement; owner not yet materialized |
| CODE-003-C | Ephemeral semantic working set and coverage states | B | Refinement; owner not yet materialized |
| CODE-003-D | Persistent progressive workspace index | C; storage/SQLite decision | Refinement; owner not yet materialized |
| CODE-003-E | Deterministic candidate resolver with provenance | B/C; D only if persistence is justified | Refinement; owner not yet materialized |
| CODE-003-F | Query-driven bounded expansion planner | C/E | Refinement; owner not yet materialized |
| CODE-003-G | Context/memory consumption without lifecycle merging | C/F | Refinement; owner not yet materialized |
| CODE-003-H | Optional authoritative LSP/SCIP/compiler promotion | Stable base candidate contract | Future / Unclaimed |

Only CODE-003-A is independently runnable. Later rows are decomposition identities, not Ready
Stories, and must receive their own owner, acceptance, iteration and effective claim before work.

## Architecture Boundaries Added By Issue #317

- Do not persist raw Tree-sitter Trees as authoritative data or duplicate source text by default.
- Persistent materialization, if selected, is atomic per file version, version-aware and safe to
  delete/rebuild; another production SQLite consumer requires a storage decision review.
- Syntax facts and heuristic candidates are distinct. Multiple candidates, coverage and evidence
  remain visible; no renderer or tool may label a heuristic as compiler-authoritative.
- Whole-workspace startup indexing, graph/vector databases, ast-grep, LSP and compiler toolchains
  are not prerequisites.
- Existing workspace/read-only permissions, traversal limits and ARCH-034-R04 parser containment
  remain mandatory and cannot be weakened by a child claim.
- Dirty editor buffers may later overlay persistent state in memory; per-keystroke SQLite writes
  are outside the target architecture.

## State / Status Owners

- Epic architecture, child order and completion: this file.
- First runnable child: `CODE-003-A-code-intelligence-contract-characterization.md`.
- Remote discussion and proposed design: GitHub Issue #317.
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`.

## User-Facing Documentation

Each behavior-changing child must update tool/capability documentation. This intake reconciliation
changes no runtime behavior and makes no shipped-feature claim.

## Epic Completion Condition

CODE-003 remains non-terminal until independently governed child evidence proves accepted query,
semantic-candidate, progressive-index, storage, safety and future-promotion boundaries. A child
completion cannot self-certify the Epic.

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
