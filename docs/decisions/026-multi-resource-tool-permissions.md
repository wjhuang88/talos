# 026: Multi-Resource Tool Permissions

## Status

Accepted

## Context

Talos tools were previously classified by a single `ToolNature`: `Read`, `Write`, `Execute`, or
`Network`. That was sufficient for simple tools, but it made hybrid tools ambiguous:

- `save_url` performs a network download and writes to a local path.
- `git_push` executes host Git and mutates a remote.
- `git_pull` executes host Git, talks to a remote, and may mutate the workspace.
- `delete` can remove either a file or a directory under one write classification.

`WEBFETCH-001` Phase 2+ will add more document/fetch/save behavior. Expanding that surface while
permission evaluation can only see one risk facet would allow future tools to overstate one risk
and understate another.

## Decision

Talos tools now expose an invocation-specific permission profile:

- `ToolNature` remains as the backward-compatible primary classification.
- `ToolPermissionFacet` describes each risk facet touched by a call.
- `ToolResourceKind` describes whether a facet resource is a path, domain, command, or named
  remote.
- `AgentTool::permission_profile(input)` defaults to a single facet derived from `nature()`, so
  existing single-surface tools remain compatible.
- Hybrid tools override `permission_profile(input)` to expose every relevant facet.

`talos-permission` evaluates a full profile conservatively:

1. Any denied facet denies the whole tool call.
2. Otherwise, any ask facet requires approval.
3. Only when every facet is allowed does the whole call proceed.

The same profile evaluation is used by the Agent layer, CLI print wrapper, TUI wrapper, MCP server
permission gate, and `talos-runtime` facade.

## Consequences

- `save_url` can be governed by both URL/domain rules and destination path rules.
- `git_push` and `git_pull` expose host command and remote/network facets; `git_pull` also exposes
  workspace write impact.
- `delete` carries file-vs-directory risk detail in permission metadata when the path exists.
- Future web/document save or extraction tools must expose all network, write, execute, and
  optional extraction facets before execution.
- Existing tools that do not override `permission_profile` keep the previous single-nature
  behavior.

## Rejected Alternatives

- **Replace `ToolNature` entirely.** Rejected because it would create unnecessary API churn and
  force every tool to migrate at once.
- **Add tool-specific hardcoded checks only for `save_url`.** Rejected because Git, future document
  fetch, and plugin/asset installation have the same class of problem.
- **Create separate user prompts per facet in the first slice.** Rejected for now. The permission
  engine evaluates each facet, while UX still asks once for the tool call. Future UI can render the
  facet list without changing the engine contract.

## Validation

- `cargo test -p talos-permission -p talos-tools -p talos-runtime`
- `cargo test -p talos-agent -p talos-mcp -p talos-cli registry`
- `cargo check --workspace`
- `cargo fmt --all -- --check`

## Related

- [TOOL-013: Multi-Resource Tool Permission Classification](../backlog/active/TOOL-013-multi-resource-tool-permissions.md)
- [TOOL-007: Built-in Tool Set Design Audit](../backlog/active/TOOL-007-tool-set-design-audit.md)
- [WEBFETCH-001: Web And Document Fetch Tools](../backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md)
- [ADR-021: Tool Call Protocol Architecture](021-tool-call-protocol-architecture.md)
