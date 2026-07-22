# Iteration I147: MODEL-008-A Custom Provider Wizard And Atomic Config

> Document status: Complete (maintainer terminal acceptance 2026-07-22)
> Published plan date: 2026-07-20
> Activated: 2026-07-20 (after I146 completion)
> Status changed to Review: 2026-07-20 (implementation + locked validation complete; real-terminal walkthrough pending)
> Completion Commit: `1c843b2` — provider-wizard rendering, cursor targeting, and visible protocol-choice acceptance repairs.
> Planned objective: let a user register an OpenAI-compatible or Anthropic-compatible
> custom provider entirely from `/connect` without editing TOML, through a cancel-safe
> five-step wizard with atomic config persistence.
> Baseline rule: preserve this objective; changed targets use a new iteration ID.
> MVP deliverable: a runnable TUI where `/connect` → Add custom provider opens a
> wizard that collects name, protocol, base URL, and API key, then persists them as
> one atomic config update with no partial write on any failure path.

## Published Baseline

- Selected Ready story: MODEL-008-A, under ADR-013 (provider config schema) and
  ADR-023 (inline api_key boundary).
- Dependencies satisfied: TUI-033 (I146, Review — parameterless `/connect` opens the
  provider picker), MODEL-005/MODEL-006 existing `/connect` flow.
- The wizard is a new panel kind (`ProviderWizard`) with a state machine that cycles
  through five steps: name → protocol → base URL → API key → confirm.
- Field validation:
  - Name: 1–64 char canonical slug (lowercase ASCII letter/digit start; then lowercase
    letters, digits, or `-`).
  - Protocol: closed set — `openai-chat` or `anthropic-messages`. No free-form strings.
  - Base URL: absolute `https://` required; `http://` only for loopback (`127.0.0.1`,
    `::1`, `localhost`). Reuse existing endpoint normalization.
  - API key: required, masked while entered and in all diagnostics. Saved through
    ADR-023 persistence/masking boundary.
  - Confirm: display name, protocol, normalized URL, and `***` key placeholder.
- On confirmation, write one coherent config update. Cancellation, field error, or
  confirmation failure leaves config exactly unchanged (no partial write).
- Duplicate name enters an explicit update flow that preserves unrelated providers,
  models, and secrets. No silent overwrite.
- API key never appears in logs, Debug output, panel labels, errors, or UI surfaces.
- The wizard carries `(name, protocol, base_url, api_key)` structurally to the bridge;
  no command-string reserialization.
- Public API semver impact: any new `UserInput` variant or `PanelKind` variant is a
  pre-1.0 semver break. Document in the iteration owner doc and release notes.

## Scope

1. New `PanelKind::ProviderWizard` panel state with a `WizardStep` enum and field
   buffers for name, protocol, base_url, and api_key.
2. Wizard step transitions: name → protocol → base_url → api_key → confirm → save (or
   cancel at any step).
3. Validation logic for each field (slug, closed protocol set, HTTPS/loopback, non-empty
   key).
4. Atomic config save: build the full `ProviderConfig` with all fields, then call
   `Config::save()` once. No intermediate saves.
5. Duplicate/update flow: if the name exists, show an explicit "update existing
   provider?" confirmation that preserves unrelated entries.
6. Key masking: API key is masked in `Debug` impls, logs, panel labels, errors, and
   `talos config list`/`get` output (ADR-023).
7. Structured identity: wizard sends `UserInput::RegisterCustomProvider { name,
   protocol, base_url, api_key }` to the bridge; bridge calls a new
   `handle_register_custom_provider` lifecycle handler.
8. TUI state/app changes: open the wizard from the `/connect` picker's "Add custom
   provider" entry.
9. Tests: wizard state-machine, every cancel point, name/protocol/url/key validation,
   config parse/save round trip, key masking, duplicate/update flow, no-partial-write.
10. README/site/config reference updates.

## Explicit Non-Goals

- Model discovery (MODEL-008-B/I148 owns discovery + manual fallback + activation).
- OAuth, device flow, token refresh, token cache (PROVIDER-003 remains separate).
- Arbitrary protocol plugins, custom request JSON, custom headers, new transport.
- New `unsafe` blocks or native dependencies.
- Relaxing ADR-023 credential masking/persistence boundary.
- Editing/deleting multiple custom models after registration.

## Compatibility And State Constraints

- The wizard is a TUI-only feature. The engine dispatch is unchanged — the bridge
  handles the structured `UserInput::RegisterCustomProvider`.
- Public `UserInput` enum gains a new variant. This is a pre-1.0 semver break for
  exhaustive matches. Release must be a minor bump.
- Config writes are atomic: either the full provider entry (with credential and
  base_url) is written, or nothing.
- The `ProviderConfig` type and `Config::save()` are reused; no second credential store.
- No partial write on any path (cancel, field error, confirmation failure, I/O
  failure).

## Acceptance

- Given `/connect`, when the user selects Add custom provider, then a five-step wizard
  opens with provider name as the first field and no mutation to config.
- Given an invalid name (empty, >64 chars, non-slug), when the user submits, then the UI
  identifies the field and config remains exactly unchanged.
- Given a name that already exists, when the user submits, then an explicit update flow
  is offered; unrelated providers and models are preserved; secrets remain masked.
- Given a non-`openai-chat` / non-`anthropic-messages` protocol input, when the user
  submits, then the field is rejected and config is unchanged.
- Given a non-HTTPS URL (except loopback), when the user submits, then the field is
  rejected and config is unchanged.
- Given valid fields up to confirm, when the user cancels at any step, then no partial
  provider/model/key is saved and the composer resumes.
- Given valid fields and confirmation, when the user confirms, then exactly one config
  update is written and `talos config list` masks the key as `***`.
- Given logs, panel labels, errors, or Debug output after save, when inspected, then the
  API key is absent or represented only as `***`.

## Planned Validation

- Wizard state-machine tests for every field, validation error, cancel point, update
  conflict, and the no-partial-write property.
- Name/protocol/URL validation unit tests (loopback HTTP, non-HTTPS rejection, non-slug
  rejection, length boundaries, closed protocol set).
- Config parse/save round trip for both protocols; existing providers preserved; key
  masking assertions.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`;
  `git diff --check`.
- Real-terminal walkthrough checklist (if no human verifier, record as pending
  maintainer acceptance and do not mark Complete).

## Required Reads

- `docs/backlog/active/MODEL-008-A-interactive-custom-provider-wizard.md`
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/config.rs`
- `crates/talos-config/src/endpoint.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/state.rs`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published. Activation follows I146 completion. |
| 2026-07-20 | Activation | I146 implementation pushed (`3e0e6b8`). I147 marked Active. |
| 2026-07-20 | Implementation (core logic) | 1. `talos-config::endpoint`: `validate_provider_name` (slug 1-64 chars), `validate_provider_protocol` (closed set `openai-chat`/`anthropic-messages`), `validate_provider_base_url` (HTTPS + loopback HTTP only). 2. `talos-conversation::types`: `UserInput::RegisterCustomProvider { name, protocol, base_url, api_key }`. 3. `session_handlers.rs`: `handle_register_custom_provider` — validates all fields, checks duplicate (update flow), builds `ProviderConfig`, one atomic `Config::save()`, key masked via ADR-023 `Debug` impl. 4. `tui_bridge.rs`: `SessionLifecycleRequest::RegisterCustomProvider` + bridge arm. 5. `mode_runners.rs`: handler dispatch. 6. Tests: validation (slug, protocol, URL, loopback, IPv6), handler (openai-chat, anthropic-messages, update, invalid name/protocol/URL/key, loopback HTTP, no-partial-write). |
| 2026-07-20 | Implementation (TUI wizard panel) | 1. `PanelKind::ProviderWizard` with `WizardStep` enum (Name, Protocol, BaseUrl, ApiKey, Confirm) and field buffers. 2. `PanelItemAction::OpenWizard` — "Add custom provider" entry at the top of the connect picker. 3. `PanelAction::RegisterCustomProvider` variant — dispatched via `UserInput::RegisterCustomProvider`. 4. `state.rs`: `wizard_append_char`, `wizard_backspace`, `wizard_cycle_protocol`, `wizard_advance` (step transitions + confirm), `wizard_cancel`. 5. `app.rs`: wizard input handling block (Enter/Esc/Backspace/Up/Down/Char) routed to wizard methods. 6. Tests: wizard opens at Name, name append/backspace, name advances to Protocol, empty name doesn't advance, protocol cycles between openai-chat and anthropic-messages, full flow emits RegisterCustomProvider with all fields, cancel at any step, empty base_url doesn't advance, empty api_key doesn't advance, protocol default is openai-chat. |
| 2026-07-20 | Validation | All locked validation passes (see below). Real-terminal walkthrough remains pending maintainer acceptance — **not Complete**. |
| 2026-07-22 | Acceptance repair | Real-terminal feedback found that `ProviderWizard` had a state machine and keyboard handler but no `BottomPanelComponent` rendering branch, producing `No matches`. The panel now renders its five named steps, masked API-key entry, and confirmation summary. A Buffer/InlineFrame regression proves the wizard never falls through to the generic empty-picker rendering. |
| 2026-07-22 | Acceptance repair 2 | The first renderer repair still left the terminal cursor in the composer and represented Protocol as one ambiguous value. The wizard now positions the cursor at its active entry field (or selected protocol row), and Protocol renders both `openai-chat` and `anthropic-messages` with a selection marker. Buffer and cursor-target regressions cover both protocol choices. |
| 2026-07-22 | Maintainer terminal acceptance | Maintainer retested the repaired wizard in a real terminal and confirmed all guided checks pass: the active field owns the cursor, both protocol choices are visibly selectable, API-key entry remains masked, and the full wizard flow behaves as specified. I147 is therefore Complete. |

## Actual Validation Results (2026-07-20)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | ✅ clean |
| `cargo check --workspace --locked` | ✅ exit 0 |
| `cargo clippy --workspace --locked -- -D warnings` | ✅ exit 0 |
| `cargo test --workspace --locked` | ✅ all tests pass (0 failures) |
| `scripts/validate_project_governance.sh .` | ✅ 0 warnings |
| `git diff --check` | ✅ clean |

## Maintainer Terminal Acceptance (2026-07-22)

The maintainer completed the guided real-terminal walkthrough after the two acceptance repairs and
reported all checks passing. This closes the previously deferred wizard interaction gate.
