# CODE-003-A: Code-Intelligence Contract Characterization

| Field | Value |
|---|---|
| Story ID | CODE-003-A |
| Type | Architecture Characterization Story |
| Priority | P1 |
| Status | Ready / Unclaimed |
| Parent Epic | CODE-003 |
| Source | [GitHub Issue #317](https://github.com/wjhuang88/talos/issues/317) |
| Selected Iteration | None |
| Depends On | CODE-001 and CODE-002 Complete; ADR-020 Accepted; current symbol-tool implementation |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #317 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Select a bounded characterization iteration and establish an effective claim before editing code, tool contracts or user documentation. |

## Identity / Goal / Value

Give maintainers and agents an evidence-backed description of what the current symbol tools can
and cannot prove before stronger query, indexing or resolution layers are introduced.

## Scope

- Characterize current `find_symbol`, `find_references`, `list_symbols` and `list_imports`
  behavior with shadowing, same-name, alias, method/function collision and malformed-code fixtures.
- Correct documentation or model-facing claims that exceed current syntax/identifier semantics.
- Inventory and minimally centralize language availability facts needed to identify obvious
  registry drift; do not introduce query packs or a persistent index.
- Produce one reference matrix distinguishing syntax facts, heuristic candidates and unavailable
  authoritative resolution.

## Exclusions

- No persistent SQLite index, cache schema, heuristic cross-file resolver or expansion planner.
- No new dependency, language query pack, LSP/SCIP/compiler integration or parser-safety weakening.
- No claim that Tree-sitter alone provides compiler-authoritative semantics.

## Acceptance

- Fixtures expose shadowing, same-name cross-scope, alias, method/function collision and malformed
  syntax behavior for every affected public tool.
- Tool and user documentation use syntax/candidate terminology matching observed behavior.
- The language-support matrix separates highlighting, symbol, import, call, scope and resolution
  maturity without inferring support from grammar availability alone.
- Existing permission, traversal, byte/output and parser-containment boundaries remain unchanged.
- Focused tool tests, locked workspace validation and applicable native-boundary checks pass.

## State / Status Owners

- Child Story scope and status: this file.
- Parent architecture and child order: `CODE-003`.
- Remote architecture discussion: GitHub Issue #317.

## User-Facing Documentation

Update symbol-tool descriptions and code-intelligence capability documentation when the
characterization shows an existing claim is stronger than runtime behavior.

## Required Reads

- `docs/backlog/active/CODE-003-tree-sitter-usage-pattern-analysis.md`
- `docs/backlog/active/CODE-001-tree-sitter-code-analysis-research.md`
- `docs/backlog/active/CODE-002-symbol-tools.md`
- `docs/backlog/active/TOOL-008-tree-sitter-on-demand.md`
- `docs/backlog/active/ARCH-034-R04-AG5-parser-panic-deadline.md`
- `docs/decisions/020-tree-sitter-code-analysis.md`
- `crates/talos-tools/src/symbol.rs`
- `crates/talos-tools/Cargo.toml`

## Residual Destination

Query packs, ephemeral working sets, persistence, resolution, expansion, context integration and
authoritative-provider promotion remain in later CODE-003 children.
