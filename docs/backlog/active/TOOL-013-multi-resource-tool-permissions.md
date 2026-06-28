# TOOL-013: Multi-Resource Tool Permission Classification

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Complete (2026-06-28) |
| Depends On | `TOOL-007`; current `ToolNature` permission engine |
| Owner Boundary | H2/H3 architect-owned permission boundary work |

## Outcome

Talos can classify and authorize tools that touch more than one risk surface, such as network plus
file write, shell execution plus network, or local write plus remote mutation.

## Problem

`ToolNature` is currently a single enum: `Read`, `Write`, `Execute`, or `Network`. This is too
coarse for hybrid tools:

- `save_url` downloads from the network and writes a local file, but currently reports `Write`.
- `git_push` shells out and mutates a remote, but currently reports `Execute`.
- `git_pull` shells out, talks to a remote, and may mutate the workspace, but currently reports
  `Execute`.
- `delete` covers file and directory deletion under one Write classification.

Before `WEBFETCH-001` grows PDF/Office/document extraction and save/download workflows, Talos needs
a permission model that can evaluate all relevant resources for one tool call.

## Acceptance Criteria

- [x] A tool can expose multiple risk facets or resources for one invocation.
- [x] Permission evaluation can require approval for each relevant facet without bypassing the
      existing deny/ask/allow flow.
- [x] `save_url` checks both URL/domain and destination path.
- [x] `git_push` and `git_pull` model remote/network and workspace/execute effects explicitly.
- [x] Directory deletion can be distinguished from file deletion in approval text or risk metadata.
- [x] TUI, print, MCP, and embeddable runtime paths share the same classification.
- [x] Regression tests prove no write/network hybrid tool can execute when either facet is denied.

## Execution Notes

- 2026-06-28: Added `ToolPermissionFacet` and `ToolResourceKind` to `talos-core`.
- 2026-06-28: Added `PermissionEngine::evaluate_profile()` with conservative aggregation:
  denied facet wins, otherwise ask wins, otherwise allow.
- 2026-06-28: Updated `save_url`, `git_push`, `git_pull`, and `delete` to expose invocation-specific
  permission profiles.
- 2026-06-28: Updated Agent, CLI print, TUI, MCP, and `talos-runtime` permission paths to evaluate
  the same profile.
- 2026-06-28: Recorded ADR-026 for the multi-resource permission boundary.

## Validation Notes

- `cargo test -p talos-permission -p talos-tools -p talos-runtime`
- `cargo test -p talos-agent -p talos-mcp -p talos-cli registry`
- `cargo check --workspace`
- `cargo fmt --all -- --check`

## Non-Goals

- No Guardian AI approval.
- No scheduled/autonomous execution changes.
- No new network/document tools.

## Required Reads

- `docs/proposals/builtin-tool-family-design.md`
- `docs/backlog/active/TOOL-007-tool-set-design-audit.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `crates/talos-core/src/tool.rs`
- `crates/talos-permission/src/lib.rs`
- `crates/talos-cli/src/registry.rs`
- `crates/talos-tools/src/save_url.rs`
- `crates/talos-tools/src/git.rs`
