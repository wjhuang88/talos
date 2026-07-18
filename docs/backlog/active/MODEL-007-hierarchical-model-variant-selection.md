# MODEL-007: Hierarchical Runtime Model And Variant Selection

**Status**: Complete (I141, 2026-07-18)
**Priority**: P1
**Source**: Maintainer request 2026-07-17
**Parent / Relates To**: MODEL-005, MODEL-003, TUI-010

## Change Control (2026-07-18)

Two acceptance requirements are amended by the maintainer before I141 implementation
landed, recorded under `docs/sop/CHANGE-CONTROL.md`:

1. **Conditional variant stage.** The original "A provider/model with no declared
   variant must expose one explicit `Default` option rather than silently skipping
   stage 3" requirement is reversed. A model with no declared variants skips the
   variant picker stage entirely; selecting such a model switches the runtime
   immediately on Enter.
2. **Mirror `/connect` UX.** The original "Keep `Esc`/Back deterministic: variant ->
   model -> provider -> close" requirement is replaced by the existing `/connect`
   pattern. `/model` stays a single provider-grouped picker screen; a Variant picker
   screen replaces the panel content conditionally when the selected model has
   declared variants. `Esc` closes the panel entirely (no stage-by-stage Back).
   ADR-048 is amended identically.

The Compatibility Gate (Architecture / Compatibility Gate section below) remains
satisfied by ADR-048 (amended).

## Identity / Goal / Value

Replace the current flat `/model` result list with an explicit, keyboard-driven
three-stage selection flow:

```text
Provider -> Model -> Variant -> switch on the next turn
```

This keeps a large authenticated catalog navigable and makes provider-specific
model modes visible before a runtime rebuild occurs. A user should not need to
encode a provider qualifier or an opaque mode string manually in the composer.

For this story, a **variant** is a named, non-secret invocation preset for one
`(provider, model)` pair. It may select an existing configuration such as a
reasoning effort or thinking budget, but it must never contain an API key,
Authorization header, raw provider response, or arbitrary provider request
JSON.

## Current Capability And Gap

- `/connect` already uses a provider-first registration flow.
- `/model` currently renders a flat list with non-selectable provider headers;
  `ModelPickerItem` has provider and model data but no variant identity.
- `ModelConfig` has limited reasoning options, but there is no catalog/config
  contract for named variants or a public picker payload for them.

## Scope

- Add staged TUI navigation: Provider list, model list scoped to that provider,
  then variant list scoped to the exact `(provider, model)` pair.
- Keep `Esc`/Back deterministic: variant -> model -> provider -> close, without
  changing the active runtime configuration until final confirmation.
- Define the minimum backwards-compatible variant representation and resolution
  order before code is written. A provider/model with no declared variant must
  expose one explicit `Default` option rather than silently skipping stage 3.
- Preserve the existing next-turn session rebuild and approval/tool semantics.
- Make current provider, model, and variant identifiable in the picker and the
  status snapshot, without displaying credentials.
- Ensure filtering applies only to the visible stage and never loses the
  selected provider/model identity.

## Non-Goals

- No provider marketplace, remote catalog refresh, credential verification
  network call, automatic model switching mid-turn, multi-provider failover,
  model training, or arbitrary request-body editor.
- No credential migration, change to provider authentication semantics, or
  alteration of provider streaming/tool-call behavior.
- No new dependency unless an ADR and explicit approval justify it.

## Architecture / Compatibility Gate

`ModelConfig`, `ModelPickerData`, `ModelPickerItem`, and `StatusSnapshot` are
public or cross-crate contracts. Before activation, record an ADR or a bounded
compatibility note that answers all of the following:

1. Whether variants are catalog metadata, user configuration, or a derived
   projection of existing `ReasoningOptions`.
2. The stable persisted identity of a selected variant and migration behavior
   for existing configs that only contain provider/model.
3. Provider capability validation and the safe fallback for an unsupported or
   deleted variant.
4. Why the representation does not introduce a `talos-config` /
   `talos-conversation` dependency cycle.

If this cannot be done additively, stop and create a separate migration story;
do not change public APIs or config meaning under this owner.

## Acceptance For Behavior

- Given multiple authenticated providers, when `/model` opens, then the first
  selectable level contains one entry per provider and does not expose a flat
  cross-provider model list.
- Given a selected provider, when Enter is pressed, then only that provider's
  models are shown; Back returns to the provider level while preserving the
  current filter and selection where still valid.
- Given a selected model, when Enter is pressed, then only its declared
  variants plus an explicit `Default` fallback are shown.
- Given a selected variant, when Enter is pressed between turns, then the
  Runtime rebuild uses exactly that provider/model/variant and the next turn
  observes it; the in-flight turn, transcript, permissions, and tools remain
  unchanged.
- Given an old config without variant data, when `/model` is opened, then it
  remains usable and resolves to `Default` without a config rewrite.
- Given an unknown, unsupported, or unauthenticated selection, when confirmed,
  then Talos returns a bounded structured error and retains the active model.
- Given a picker at any stage, when `Esc` is pressed, then it moves back one
  stage or closes at the root without changing active configuration.
- Picker labels, tips, diagnostics, and status display never reveal API keys,
  Authorization values, cookies, raw provider responses, or hidden reasoning.

## Validation

- Focused `talos-config`, `talos-conversation`, `talos-cli`, and `talos-tui`
  tests for stage transitions, filtering, old-config fallback, invalid variants,
  next-turn rebuild, and no-secret projection.
- At least one mock-provider runtime integration test proving the chosen variant
  reaches the intended request configuration without changing stream ordering.
- A real terminal walkthrough covering all three stages, Back/Esc behavior,
  narrow-width rendering, and an old config fixture.
- Locked workspace fmt/check/clippy/test, release preflight, governance
  validation, and `git diff --check`.

## State / Documentation Owners

- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- README `/model` and configuration documentation
- model/variant ADR or compatibility note
- iteration owner, iteration index, Board, and any source issue at activation
  and closeout

## Required Reads

- `AGENTS.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/NEW-FEATURE.md`
- `docs/sop/CHANGE-CONTROL.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/backlog/active/MODEL-003-reasoning-thinking-support.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-tui/src/panel_state.rs`

## Residual Destination

- New provider-native variant fields or incompatible config schema: dedicated
  ADR plus migration story.
- Runtime provider capability expansion: MODEL-003 / provider-specific owner.
- Status-bar presentation of the selected variant: TUI-031.
