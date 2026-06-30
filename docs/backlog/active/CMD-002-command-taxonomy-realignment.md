# CMD-002: Command Taxonomy Realignment (/skills /mcp /plugins /hooks)

| Field | Value |
| --- | --- |
| Story ID | CMD-002 |
| Status | **Planned — architecture unblocked 2026-06-30**. ADR-030 accepted. First slice can move MCP status to `/mcp` and make `/plugins` a transition notice before plugin packages ship. |
| Priority | P2 |
| Source | Owner architecture declaration, 2026-06-30 |
| Relates To | CMD-001, PLUGIN-001, EXT-001, HOOK-001, ADR-009 |

## Requirement

Realign the interactive slash-command vocabulary so each extensibility entity has a peer command,
and the `/plugins` name stops being occupied by MCP status display.

## Problem

Today `/plugins` does not list plugins (plugins do not exist). It renders MCP server startup status
plus per-provenance tool call counts (`engine.rs` `handle_plugins_command`). This collides with the
planned plugin entity and misleads users about the extensibility surface. Meanwhile `/skills` exists
but MCP and hooks have no dedicated command.

## Scope

ADR-030 accepted the taxonomy. Implement in two slices:

Slice 1, before plugin packages ship:

- Add `/mcp` to list MCP server status + observed MCP tool provenance (the current `/plugins` body,
  renamed).
- Repurpose `/plugins` to show a short transition notice: plugin packages are not available yet;
  use `/mcp` for MCP server status. Do not alias `/plugins` to `/mcp`.

Slice 2, after PLUGIN-001 ships:

- `/plugins` lists loaded plugin packages and their declared capabilities (skills/mcp/hooks/tools).
- Add `/hooks` to list registered hooks (builtin + config-introduced) once HOOK-001 ships.
- Keep `/skills` as-is.
- Update README, TUI command menu metadata (CMD-001 registry), help text, and provenance markers.

## Non-Goals

- No change to command *execution* semantics — this is a taxonomy/naming realignment riding on
  CMD-001 infrastructure.
- No `/plugins` content until the plugin entity exists; `/plugins` may be reserved or alias `/mcp`
  during the transition.

## Acceptance Criteria

- [ ] `/mcp` shows MCP server status and observed MCP tool provenance.
- [ ] `/plugins` is either reserved, aliased to `/mcp` during transition, or lists loaded plugins
      (depending on whether PLUGIN-001 has shipped).
- [ ] `/hooks` lists registered hooks once HOOK-001 ships.
- [ ] `/skills` unchanged.
- [ ] README, slash-command menu, and help text updated.
- [ ] No command-name collision or silent override.

## Required Reads

- `docs/proposals/plugin-encapsulation-format.md`
- `docs/decisions/030-extensibility-command-taxonomy.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/HOOK-001-config-introduced-hooks.md`
- `docs/decisions/009-tool-provenance.md`
- `crates/talos-conversation/src/command_registry.rs`
- `crates/talos-conversation/src/engine.rs`

## Open Questions

1. Transition policy: reserve `/plugins` (no-op with notice), or alias it to `/mcp` until plugins
   ship?
2. Should `/hooks` appear before HOOK-001 lands (listing builtin hooks only)?
