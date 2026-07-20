# Iteration I146: TUI-033 Parameterless Model And Provider Commands

> Document status: Planned
> Published plan date: 2026-07-20
> Planned objective: make TUI `/model` and `/connect` strict no-argument menu commands so all
> provider/model selection and search happens inside their existing panels, not through
> parameterized command text.
> Baseline rule: preserve this objective; changed targets use a new iteration ID.
> MVP deliverable: a runnable TUI where `/model` and `/connect` open their pickers without
> arguments, parameterized TUI input is side-effect-free and redirects to the picker, and slash
> completion opens menus directly.

## Published Baseline

- Selected Ready story: TUI-033, under ADR-013 (provider config schema) and ADR-048 (model
  variant representation).
- Dependencies satisfied: TUI-010 slash panel, MODEL-007 hierarchical picker, TUI-032 multiline
  composer, TUI-026 queued steering display (Review, not a blocker).
- `/model` is a strict no-argument command that opens the existing Provider → Model → conditional
  Variant picker. Typing after the command filters the current panel level only.
- `/connect` is a strict no-argument command that opens the provider picker. Typing after the
  command filters that picker; selection opens the existing credential or custom-provider setup
  flow (the wizard itself is MODEL-008-A/I147, not this iteration).
- Parameterized TUI input (`/model gpt-4o`, `/model provider/model`, `/model foo@variant`,
  `/connect openai`, whitespace-only forms) must NOT:
  - directly switch a model,
  - rebuild a session,
  - write configuration,
  - enter a credential flow,
  - or contact a provider.
- Instead, parameterized TUI input must show one bounded corrective message and open the relevant
  picker with the supplied text as the search query where feasible.
- Slash-panel completion for `/model` and `/connect` must execute/open the menu directly and must
  not insert a trailing parameter space.
- Selection actions must carry structured `(provider, model, variant_id)` identity internally;
  they must not reserialize to `/model provider/model`, `/model model@variant`, or
  `/connect name` and route it back through command parsing.
- Existing keyboard navigation, Escape/cancel behavior, credential masking, session-rebuild
  timing, and custom-provider registration work (MODEL-008-A/B) remain unchanged.
- README EN/zh-CN and the public documentation site must be updated to describe the no-argument
  command interaction and must not contain parameterized TUI examples for `/model` or `/connect`.

## Scope

1. TUI state/app changes so bare `/model` and `/connect` open their pickers with an empty search
   query.
2. TUI state/app changes so parameterized `/model <text>` and `/connect <text>` (including
   whitespace-only) are side-effect-free: one bounded correction + open the relevant picker with
   the text as the search query.
3. Slash-panel completion behavior for `/model` and `/connect`: Enter opens the menu directly, no
   trailing parameter space.
4. In-panel search filters only the current panel level.
5. Structured `(provider, model, variant_id)` identity propagation from panel selection to the
   existing switch/lifecycle path — no command-string reserialization.
6. Compatibility inventory of every `UserInput`/`UiOutput` consumer proving parameter rejection
   is limited to interactive TUI mode and cannot break headless, inline, print, RPC, or
   stored-session paths.
7. Regression tests for custom provider, model IDs containing `/` or `@`, Escape, credential
   input, and approval priority.
8. README EN/zh-CN and `site/` command documentation updates.

## Explicit Non-Goals

- No change to non-TUI CLI flags, machine-readable commands, config editing commands, or external
  API contracts without a separate compatibility decision and ADR.
- No redesign of the provider/model picker hierarchy, variants, credential storage, or custom
  provider protocol selection — those remain MODEL-007/MODEL-008 scope.
- No fuzzy matching, remote model refresh, arbitrary command aliases, or command-registry-wide
  argument syntax change.
- No release tag in this iteration; release selection remains a follow-up after acceptance.

## Compatibility And State Constraints

- This is a TUI interaction policy. The compatibility inventory must prove that parameter
  rejection is limited to interactive TUI mode.
- Public request types are semver-bound. Prefer a private TUI action or additive structured
  request over changing a public command payload; create an ADR and migration plan before any
  breaking public API change.
- Parameter rejection must be side-effect-free: do not create sessions, write config, contact a
  provider, or prompt for credentials before a panel selection is confirmed.
- Preserve exact model identifiers containing `/` or `@`; they are panel data, never command
  syntax.

## Acceptance

- Given the TUI composer, when the user submits `/model`, then the provider-first model picker
  opens with an empty search query and no active model changes.
- Given an open model picker, when the user types a query, then only the current panel level is
  filtered; selecting a result carries its exact structured identity into the existing switch
  flow.
- Given `/model gpt-4o`, `/model provider/model`, or `/model model@variant`, when submitted in
  the TUI, then no direct switch/rebuild/config write occurs; the user receives one correction
  and the picker opens with a useful search query.
- Given the TUI composer, when the user submits `/connect`, then the provider picker opens with
  no credential/config mutation.
- Given `/connect openai` or another argument-bearing form in the TUI, when submitted, then no
  credential prompt or provider mutation begins until the user selects a provider from the
  picker.
- Given slash-menu completion for `/model` or `/connect`, when Enter is pressed, then it opens
  the corresponding menu directly and does not insert a trailing parameter space.
- Given keyboard search, Escape, credential entry, approval UI, a custom provider, or a model ID
  containing `/` or `@`, when the user navigates the menus, then existing behavior remains
  usable and no secret is displayed.
- Given README EN/zh-CN and site command documentation, when users read model/provider setup
  instructions, then they describe no-argument commands and menu search only.

## Planned Validation

- TUI state/app tests covering: bare `/model` and `/connect`, whitespace-only forms,
  parameterized rejection, slash-menu completion, current-level filtering, exact `/` and `@`
  identities, Escape, and no unintended request/config mutation.
- Bridge/lifecycle integration tests proving panel selections still trigger exactly one intended
  model switch or provider setup, while parameterized TUI text triggers none.
- Regression tests for every non-TUI command consumer identified by the compatibility inventory.
- Real-terminal walkthrough checklist for `/model`, `/connect`, search, cancel, standard
  credential entry, and custom provider selection. If no human verifier is available, record the
  walkthrough as pending maintainer acceptance and do not mark the iteration Complete.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Required Reads

- `docs/backlog/active/TUI-033-parameterless-model-connect-commands.md`
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/MODEL-007-hierarchical-model-variant-selection.md`
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/048-model-variant-representation.md`
- `crates/talos-conversation/src/command.rs`
- `crates/talos-conversation/src/command_registry.rs`
- `crates/talos-conversation/src/types.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/app.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/session_handlers.rs`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published as part of the P0 governance commit. No implementation, release, tag, or production-code change has started. Activation requires the P0 governance commit to be pushed to `origin/main` first. |
