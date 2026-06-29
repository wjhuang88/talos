# TOOL-012: Tool Family Metadata And Progressive Loading

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Complete (2026-06-29) |
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

- [x] Tool families are explicit data, not inferred from string prefixes alone.
- [x] Always-on tools are documented and tested.
- [x] Git, code intelligence, network/web, and shell families can be loaded independently.
- [x] Provider tool definitions and prompt tool descriptions stay in sync.
- [x] Fallback behavior is defined when the model requests a tool from an unloaded family.
- [x] Stable-prefix cache invalidation is tested for unchanged families.
- [x] The implementation degrades gracefully for providers that require a full tool list per
      request.
- [x] `TOOL-007` audit recommendations are reflected in README only if user-facing behavior changes.

## Implementation Notes

- Added `ToolFamily` and `ToolPresentationPolicy` in `talos-core`; `ToolRegistry` remains the
  executable source of truth.
- Built-in tools now expose explicit family metadata. The always-on set is `read`, `write`,
  `edit`, `ls`, `grep`, and `glob`; shell, Git, network/web, and code-intelligence tools are
  independently selectable families.
- `talos-agent` derives prompt tool descriptions and provider `ToolDefinition`s from the same
  policy. Calls to registered but unpresented tools return a recoverable tool error and do not
  execute.
- Prompt rendering groups tools into stable family sections so adding a family does not rewrite an
  unchanged family block.
- Default policy is `ToolPresentationPolicy::full()`, so existing user-facing behavior and README
  usage remain unchanged.

## Validation

- `cargo check --workspace`
- `cargo test -p talos-core tool_presentation_policy`
- `cargo test -p talos-agent prompt::tests`
- `cargo test -p talos-agent tool_presentation`
- `cargo test -p talos-agent unpresented_registered_tool`
- `cargo test --workspace`
- `sh scripts/validate_project_governance.sh .`

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
