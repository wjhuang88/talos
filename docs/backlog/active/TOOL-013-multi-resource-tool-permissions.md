# TOOL-013: Multi-Resource Tool Permission Classification

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Planned |
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

- [ ] A tool can expose multiple risk facets or resources for one invocation.
- [ ] Permission evaluation can require approval for each relevant facet without bypassing the
      existing deny/ask/allow flow.
- [ ] `save_url` checks both URL/domain and destination path.
- [ ] `git_push` and `git_pull` model remote/network and workspace/execute effects explicitly.
- [ ] Directory deletion can be distinguished from file deletion in approval text or risk metadata.
- [ ] TUI, print, MCP, and embeddable runtime paths share the same classification.
- [ ] Regression tests prove no write/network hybrid tool can execute when either facet is denied.

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
