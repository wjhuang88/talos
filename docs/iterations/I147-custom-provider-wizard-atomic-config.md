# Iteration I147: MODEL-008-A Custom Provider Wizard And Atomic Config

> Document status: Planned
> Published plan date: 2026-07-20
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
