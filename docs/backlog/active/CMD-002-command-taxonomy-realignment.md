# CMD-002: Command Taxonomy Realignment (/skills /mcp /plugins /hooks)

| Field | Value |
| --- | --- |
| Story ID | CMD-002 |
| Status | **Blocked** — pending `docs/proposals/plugin-encapsulation-format.md` ADR #4 (command taxonomy) and the plugin entity existing. |
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

After the plugin encapsulation ADRs are accepted:

- Add `/mcp` to list MCP server status + observed MCP tool provenance (the current `/plugins` body,
  renamed).
- Repurpose `/plugins` to list loaded plugin packages and their declared capabilities
  (skills/mcp/hooks/tools) once PLUGIN-001 ships.
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
