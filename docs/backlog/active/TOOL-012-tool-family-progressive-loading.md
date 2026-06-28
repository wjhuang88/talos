# TOOL-012: Tool Family Metadata And Progressive Loading

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Planned |
| Depends On | `TOOL-007`; ADR-025 search direction |
| Owner Boundary | H2 architect-owned tool-family work |

## Outcome

Talos can present built-in tools to the model by stable tool families instead of injecting every
tool description and schema as one flat prompt block on every turn.

## Scope

- Add tool-family metadata for native and MCP tools.
- Keep `ToolRegistry` as the executable source of truth.
- Add a presentation policy that selects tool descriptions/provider tool definitions by context.
- Preserve a safe always-on set for common file/search/edit operations.
- Design cache-friendly family blocks so adding one family does not invalidate unrelated prompt
  prefixes unnecessarily.

## Acceptance Criteria

- [ ] Tool families are explicit data, not inferred from string prefixes alone.
- [ ] Always-on tools are documented and tested.
- [ ] Git, code intelligence, network/web, and shell families can be loaded independently.
- [ ] Provider tool definitions and prompt tool descriptions stay in sync.
- [ ] Fallback behavior is defined when the model requests a tool from an unloaded family.
- [ ] Stable-prefix cache invalidation is tested for unchanged families.
- [ ] The implementation degrades gracefully for providers that require a full tool list per
      request.
- [ ] `TOOL-007` audit recommendations are reflected in README only if user-facing behavior changes.

## Non-Goals

- No tool removal or rename.
- No permission model rewrite; `TOOL-013` owns hybrid risk classification.
- No document extraction implementation.

## Required Reads

- `docs/proposals/builtin-tool-family-design.md`
- `docs/backlog/active/TOOL-007-tool-set-design-audit.md`
- `crates/talos-core/src/tool.rs`
- `crates/talos-agent/src/prompt/builder.rs`
- `crates/talos-agent/src/configuration.rs`
- `crates/talos-cli/src/registry.rs`
