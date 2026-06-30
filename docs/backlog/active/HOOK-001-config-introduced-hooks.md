# HOOK-001: Config-Introduced Hooks

| Field | Value |
| --- | --- |
| Story ID | HOOK-001 |
| Status | **Planned — architecture unblocked 2026-06-30**. ADR-029 accepted. First slice is schema and diagnostics for config-introduced hooks; executable hook carriers remain governed by ADR-027. |
| Priority | P3 |
| Source | Owner architecture declaration, 2026-06-30 |
| Relates To | PLUGIN-001, CMD-002, `talos-plugin`, I009 |

## Requirement

Promote **hook** from a code-only-registered capability to a first-class config-introduced atomic
component, on equal footing with skill and mcp.

## Problem

Talos has a hook system (`talos-plugin` crate, `HookHandler` trait) that observes/modifies lifecycle
events (`BeforeProviderCall`, `OnToolCall`, `TurnComplete`, etc.). But hooks are registered only in
code by builtins. Users cannot introduce hooks through configuration, so hook is not yet a true peer
of skill and mcp in the three-atomic-component model the owner declared on 2026-06-30.

## Scope

ADR-029 accepts hook as a config-introduced atomic component. Next slices:

- Define a user-facing config schema for declaring hooks (event kind, handler entry, provenance,
  ordering/priority).
- Distinguish builtin hooks from config-introduced hooks in diagnostics and `/hooks`.
- Route config-introduced hooks through the same `HookChain` execution path with provenance.
- Decide whether standalone config hooks are script-based (Lua) or require a plugin package (i.e.,
  config hooks only exist inside plugins). This is coupled to the carrier-strategy ADR.

First slice should prefer schema/diagnostics and builtin-hook listing. Do not add Lua or any
executable hook carrier without a separate ADR or the plugin runtime adapter from ADR-027.

## Non-Goals

- No new lifecycle events in v1 — reuse the existing hook event set.
- No hook marketplace.

## Acceptance Criteria

- [ ] A config schema for hook declaration exists and is validated on load.
- [ ] Builtin vs config-introduced hooks are distinguishable in diagnostics.
- [ ] `/hooks` lists both builtin and config-introduced hooks.
- [ ] Config-introduced hooks carry provenance and honor ordering/priority rules.
- [ ] Hook execution failure degrades gracefully per the existing hook policy.

## Required Reads

- `docs/proposals/plugin-encapsulation-format.md`
- `docs/decisions/027-plugin-runtime-boundary.md`
- `docs/decisions/029-extensibility-atomic-component-model.md`
- `docs/decisions/030-extensibility-command-taxonomy.md`
- `docs/backlog/active/PLUGIN-001-wasm-runtime-plugins.md`
- `docs/backlog/active/CMD-002-command-taxonomy-realignment.md`
- `crates/talos-plugin/src/`
- `docs/iterations/I009-extensible-agent.md`

## Open Questions

1. Can a user declare a standalone hook via config alone, or are config hooks only expressible
   inside a plugin package? (Tied to carrier strategy.)
2. What ordering/priority model is used when multiple hooks target the same event?
3. Do config hooks support mutation of lifecycle payloads, or observation-only in v1?
