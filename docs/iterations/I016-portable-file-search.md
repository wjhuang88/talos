# I016: Portable File And Search Tools

**User can**: Perform common file, listing, and search operations through native structured tools on
minimal machines without relying on host POSIX utilities.

## Status: SUPERSEDED — implementation delivered through I025/TOOL-003

## Decision Gate

Follow ADR-010 for search dependency boundaries. Create a follow-up ADR before public
`ToolPack`, `ToolProvenance`, config, MCP listing, or RPC listing contracts change.

## Selected Stories

- [ ] #I012-S1: Built-in POSIX basic tools subset
- [ ] #I012-S2: Embeddable tool pack interface
- [ ] #I012-S3: Built-in workspace search tools

## Scope

- Implement a small native POSIX-style subset as structured tools.
- Introduce the smallest tool-pack registration shape needed by the native pack.
- Implement stateless `find_files` and `grep` with workspace boundaries, ignore rules, binary
  skip, and output budgets.

## Non-Goals

- No shell parser.
- No filesystem watcher.
- No persistent search database or frecency store.
- No `fff-search`, `git2`, LMDB/heed, Python, Node, or arbitrary native bindings.

## Acceptance Criteria

- [ ] Native file/list/search tools work on a deliberately minimal `PATH`.
- [ ] Write-capable tools remain permission-gated.
- [ ] Search rejects path escape and does not follow symlinks by default.
- [ ] Tool-pack provenance is visible to list-tools consumers if public listing changes land.
- [ ] `cargo test -p talos-tools -p talos-core -p talos-cli` passes.

## 2026-06-19 Supersession Record

I016 was never activated as its own execution batch. The runnable native file/search tool outcome
was delivered later through I025 and TOOL-003, including grep, glob, ls, delete, diff, stat, tree,
read limits, schemas, permission classification, and display integration.

This record does not claim every original I016 acceptance item passed under this iteration. Residual
persistent indexes, extra native dependencies, and broader tool-pack portability remain under
TOOL-001.

Disposition: Superseded by I025 for delivered scope; residuals remain explicitly owned.
