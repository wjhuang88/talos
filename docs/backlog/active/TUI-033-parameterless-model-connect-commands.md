# TUI-033: Parameterless Model And Provider Commands

| Field | Value |
| --- | --- |
| Story ID | TUI-033 |
| Type | Product / Interaction Story |
| Priority | P2 |
| Status | Ready — selected into I146 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20 |
| Depends on | MODEL-007 picker; MODEL-008 custom-provider flow; TUI-010 slash panel |
| Blocks | — |
| Selected Into | [I146](../../iterations/I146-tui-parameterless-model-connect-commands.md) |

## Problem

The TUI already has provider/model picker panels with filtering and staged navigation, but it also
accepts parameterized text forms such as `/model <value>` and `/connect <provider>`. This creates
two selection paths with different parsing, escaping, validation, and discoverability properties.
It is especially fragile for provider-qualified model IDs and custom provider names.

## Goal / Value

Make model and provider setup selection unambiguous and discoverable in the TUI: `/model` and
`/connect` open their respective menus without arguments; users search, navigate, and confirm a
structured panel item rather than encoding identities in command text.

## Scope

- In TUI interactive mode, `/model` is a strict no-argument command that opens the existing
  provider → model → conditional variant picker. Typing after the command is used only as the
  panel's search/filter query, never as a direct model-switch request.
- In TUI interactive mode, `/connect` is a strict no-argument command that opens the provider
  picker. Typing after the command filters that picker; selection opens the existing credential or
  custom-provider setup flow.
- Slash-panel completion for these commands must execute/open the menu directly; it must not leave
  a misleading mandatory trailing space that suggests a parameter is expected.
- A manually submitted parameterized form, including whitespace-only variations, must not switch a
  model, start provider setup, rebuild a session, or persist configuration. It should show one
  bounded corrective message and open the relevant picker with the supplied text as its search
  query where feasible.
- Selection actions must carry structured provider/model/variant identity internally. They must not
  reserialize a selection into `/model provider/model`, `/model model@variant`, or `/connect name`
  and route it back through command parsing.
- Preserve menu search behavior at each visible level, keyboard navigation, Escape/cancel behavior,
  credential masking, session-rebuild timing, and custom-provider registration work owned by
  MODEL-008.
- Update TUI command documentation in both READMEs and the public documentation site to describe
  the menu/search interaction and remove parameterized TUI examples.

## Explicit Exclusions

- Do not remove or change non-TUI CLI flags, machine-readable commands, config editing commands,
  or external API contracts without a separate compatibility decision.
- Do not redesign the provider/model picker hierarchy, variants, credential storage, custom
  provider protocol selection, or `/models` discovery; those remain MODEL-007/MODEL-008 scope.
- No fuzzy matching, remote model refresh, arbitrary command aliases, or command-registry-wide
  argument syntax change.

## Compatibility And State Constraints

- This is a TUI interaction policy. Before implementation, inventory every `UserInput` and
  `UiOutput` consumer to prove that parameter rejection is limited to interactive TUI mode and
  cannot break headless, inline, print, RPC, or stored-session paths.
- Public request types are semver-bound. Prefer a private TUI action or additive structured request
  over changing a public command payload; create an ADR and migration plan before any breaking
  public API change.
- Parameter rejection must be side-effect-free: do not create sessions, write config, contact a
  provider, or prompt for credentials before a panel selection is confirmed.
- Preserve exact model identifiers containing `/` or `@`; they are panel data, never command
  syntax.

## Acceptance

- Given the TUI composer, when the user submits `/model`, then the provider-first model picker
  opens with an empty search query and no active model changes.
- Given an open model picker, when the user types a query, then only the current panel level is
  filtered; selecting a result carries its exact structured identity into the existing switch flow.
- Given `/model gpt-4o`, `/model provider/model`, or `/model model@variant`, when submitted in the
  TUI, then no direct switch/rebuild/config write occurs; the user receives one correction and the
  picker opens with a useful search query.
- Given the TUI composer, when the user submits `/connect`, then the provider picker opens with no
  credential/config mutation.
- Given `/connect openai` or another argument-bearing form in the TUI, when submitted, then no
  credential prompt or provider mutation begins until the user selects a provider from the picker.
- Given slash-menu completion for `/model` or `/connect`, when Enter is pressed, then it opens the
  corresponding menu directly and does not insert a trailing parameter space.
- Given keyboard search, Escape, credential entry, approval UI, a custom provider, or a model ID
  containing `/` or `@`, when the user navigates the menus, then existing behavior remains usable
  and no secret is displayed.
- Given README and site command documentation, when users read model/provider setup instructions,
  then they describe no-argument commands and menu search only.

## Required Reads

- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/MODEL-007-hierarchical-model-variant-selection.md`
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/048-model-variant-representation.md`
- `crates/talos-conversation/src/command.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/session_handlers.rs`

## Minimum Validation

- TUI state/app tests for bare commands, whitespace-only forms, parameterized rejection, slash-menu
  completion, current-level filtering, exact `/` and `@` identities, Escape, and no unintended
  request/config mutation.
- Bridge/lifecycle integration tests proving panel selections still trigger exactly one intended
  model switch or provider setup, while parameterized TUI text triggers none.
- Regression tests for non-TUI command consumers identified by the compatibility inventory.
- Real-terminal walkthrough for `/model`, `/connect`, search, cancel, standard credential entry,
  and custom provider selection.
- README EN/zh-CN and public-site documentation checks, then locked fmt/check/clippy/test and
  governance validation.
