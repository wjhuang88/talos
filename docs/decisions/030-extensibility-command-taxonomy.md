# 030: Extensibility Command Taxonomy

## Status

Accepted

## Context

The current `/plugins` command displays MCP server startup status and per-provenance tool counts.
This was useful before a real plugin package model existed, but it now blocks a coherent
extensibility vocabulary. Users need distinct commands for skills, MCP, hooks, and plugin packages.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Existing users may know `/plugins` as the MCP status surface. | Soft | Shipped TUI behavior | Yes, with transition |
| Command registry must stay typed and centralized. | Hard | CMD-001 | No |
| Plugin packages do not exist yet. | Known | PLUGIN-001 current state | Yes after implementation |
| Avoid misleading command names. | Hard | User-facing UX correctness | No |

## Reasoning

Aliasing `/plugins` to MCP status would preserve short-term behavior but teach the wrong mental
model. When plugin packages later ship, users would see the same command change meaning. A notice
is cleaner: move MCP status to `/mcp`, reserve `/plugins` until real plugin packages exist, and add
`/hooks` when hooks become user/config visible.

## Decision

1. **`/skills` remains the skill command.**

2. **Add `/mcp` for MCP server status and MCP tool provenance.**
   - The current `/plugins` body moves to `/mcp`.

3. **Reserve `/plugins` for plugin packages.**
   - Before PLUGIN-001 ships, `/plugins` should show a short notice that plugin packages are not
     installed/supported yet and point users to `/mcp` for MCP status.
   - Do not alias `/plugins` to `/mcp`.

4. **Add `/hooks` when HOOK-001 exposes hook diagnostics.**
   - It may initially list built-in hooks only if that is useful and truthful.
   - Config-introduced hooks appear after HOOK-001 lands.

5. **Update command metadata and docs together.**
   - Slash menu, `/help`, README, and TUI tests must move with the command change.

## Rejected Alternatives

- **Keep `/plugins` as MCP status.** Rejected as misleading once plugin packages are designed.
- **Alias `/plugins` to `/mcp`.** Rejected because it preserves the wrong habit and creates a later
  breaking semantic change.
- **Hide `/plugins` until plugins ship.** Rejected because users already have a command with that
  name; a clear notice is better than disappearing behavior.

## Reversal Trigger

Revisit if user testing shows the notice causes more confusion than an alias during the transition.

## Related

- [CMD-002](../backlog/active/CMD-002-command-taxonomy-realignment.md)
- [ADR-029](029-extensibility-atomic-component-model.md)
- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
