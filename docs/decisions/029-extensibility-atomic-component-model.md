# 029: Extensibility Atomic Component Model

## Status

Accepted

## Context

Talos currently has skills, MCP servers, hooks, and a planned plugin concept, but they are not
modeled consistently:

- skills are config/discovery introduced and visible via `/skills`;
- MCP servers are config introduced but shown under the misleading `/plugins` command;
- hooks exist only as code-registered handlers;
- plugins do not exist yet and need a packaging definition.

This drift blocks PLUGIN-001, HOOK-001, and command taxonomy work.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Existing skill and MCP behavior must remain compatible. | Hard | User-facing shipped behavior | No |
| Hooks must not bypass lifecycle failure policy. | Hard | `talos-plugin` safety behavior | No |
| Plugins must not become a hidden global event bus. | Hard | ADR-006 | No |
| Plugin packages should bundle multiple capability types. | Soft | Owner architecture direction | Yes |

## Reasoning

The simplest durable model is two layers:

1. **Atomic components** are capability types that can be configured, diagnosed, and surfaced
   independently.
2. **Plugin packages** are distribution/encapsulation units that can bundle those components plus
   plugin-provided tools.

This prevents "plugin" from becoming a vague synonym for every extension and allows each component
to keep its own runtime contract.

## Decision

1. **The three config-introduced atomic component types are Skill, MCP, and Hook.**
   - Skill: prompt-level capability bundle with Level 0/1/2 progressive disclosure.
   - MCP: external process/server that exposes tools through the MCP client path.
   - Hook: lifecycle observer/modifier registered into `talos-plugin` hook chains.

2. **Plugin is a package format, not a fourth atomic component.**
   - A plugin package may declare any subset of `skills`, `mcp`, `hooks`, and `tools`.
   - Plugin-provided tools are registered through the tool registry via the runtime adapter.

3. **Hooks become config-introduced through HOOK-001.**
   - Built-in code-registered hooks remain valid.
   - Config-introduced hooks must carry provenance, validated event kind, ordering/priority, and
     failure behavior.
   - Whether standalone config hooks are allowed outside plugin packages is a HOOK-001 design
     detail, but the schema must not require native code loading.

4. **Plugin declarations cannot silently override atomic components.**
   - Name conflicts, precedence, and shadowing must be explicit in the plugin manifest and loader
     diagnostics.

## Rejected Alternatives

- **Make plugin a fourth peer component.** Rejected because plugin is the container/distribution
  layer, not a capability type like skill/MCP/hook.
- **Keep hooks code-only.** Rejected because it keeps hooks below the other extensibility axes and
  blocks `/hooks` diagnostics.
- **Fold hooks into skills.** Rejected because hooks execute lifecycle code, while skills are
  prompt/context assets.

## Reversal Trigger

Revisit if real plugin packages cannot be expressed as bundles of skills, MCP declarations, hooks,
and tools without a fourth runtime capability type.

## Related

- [PLUGIN-001](../backlog/active/PLUGIN-001-wasm-runtime-plugins.md)
- [HOOK-001](../backlog/active/HOOK-001-config-introduced-hooks.md)
- [CMD-002](../backlog/active/CMD-002-command-taxonomy-realignment.md)
