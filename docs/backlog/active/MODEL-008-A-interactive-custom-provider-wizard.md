# MODEL-008-A: Interactive Custom Provider Wizard And Atomic Config

| Field | Value |
| --- | --- |
| Story ID | MODEL-008-A |
| Type | Product / Configuration Story |
| Priority | P2 |
| Status | Complete — I147 maintainer terminal acceptance (2026-07-22) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-008 |
| Parent Epic | MODEL-008 |
| Depends on | MODEL-005 / MODEL-006 existing `/connect` flow; ADR-013 provider config schema; ADR-023 inline api_key boundary; TUI-033 parameterless commands (I146) |
| Blocks | MODEL-008-B (I148) |

> Completion Commit: `1c843b2` — provider-wizard rendering, cursor targeting, and visible protocol-choice acceptance repairs.

## Problem

`/connect` only presents catalog providers. A user who wants to register an OpenAI-compatible or
Anthropic-compatible gateway currently has to hand-edit `~/.talos/config.toml`. The first half of
MODEL-008 — a cancel-safe wizard that collects provider name, protocol, base URL, and API key, then
persists them atomically — is a prerequisite for any in-TUI model discovery.

## Goal / Value

Let a user register a new API-compatible provider entirely from `/connect` without editing TOML,
through a five-step wizard (name → protocol → base URL → API key → confirm) that is cancel-safe,
validates every field before save, never leaks the API key, and never partially writes configuration.

## Scope

The wizard sequence and validation rules:

1. **Provider name** — 1–64 character canonical slug: starts with lowercase ASCII letter or digit;
   continues with lowercase letters, digits, or `-`. An existing name enters an explicit update
   flow that preserves unrelated providers, models, and secrets. No silent overwrite.
2. **Protocol** — required choice from the existing closed set only:
   - `openai-chat`
   - `anthropic-messages`

   No free-form protocol string, no new provider adapter, no arbitrary request JSON, and no
   custom headers.
3. **Base URL** — required absolute `https://` URL. `http://` is allowed only for loopback
   addresses (`127.0.0.1`, `::1`, `localhost`). Reuse the existing endpoint normalization; show
   the normalized non-secret endpoint before save. Protocol-specific endpoint shape is validated
   per the existing adapters.
4. **API key** — required for this story, masked while entered and in every diagnostic surface.
   Used only for the user-requested validation/discovery request (which lives in MODEL-008-B).
   Saved through the existing ADR-023 persistence/masking boundary.
5. **Confirm** — display provider name, protocol, normalized URL, and an explicit `***` key
   placeholder. On confirmation, write one coherent config update. Cancellation, field error,
   or confirmation failure leaves config exactly unchanged.

## Explicit Exclusions

- OAuth / device flow / token refresh / token-cache storage (PROVIDER-003 remains separate).
- Arbitrary provider protocol plugins, custom request JSON, custom headers, new transport code.
- Model discovery itself (MODEL-008-B owns discovery + manual fallback + activation).
- New `unsafe` blocks or native dependencies.
- Relaxing the ADR-023 credential masking/persistence boundary.
- Editing/deleting multiple custom models after registration.

## Design / Security Constraints

- Reuse `ProviderProtocol`, `ProviderConfig`, endpoint normalization, and configuration save logic.
  Do not create a second credential store.
- Never log, render, serialize to UI status, or retain an API key in a panel label/debug surface.
- Validate name/protocol/base URL before any mutation.
- Reject duplicates or require an explicit update confirmation; preserve unrelated provider entries.
- On confirm, write one atomic config update. On cancel or any failure, leave config unchanged.
- No partial write on any path (cancel, field error, confirmation failure, I/O failure).
- Determine and document the public API/semver impact before changing public credential request or
  response types. If a breaking public Rust API change is unavoidable, create an ADR with
  migration guidance before marking this story Ready for implementation — this is the I147
  baseline, so any ADR must exist before I147 starts implementation.

## Acceptance

- Given `/connect`, when the user selects Add custom provider, then a five-step wizard opens with
  provider name as the first field and no mutation to config.
- Given an invalid name (empty, >64 chars, non-slug), when the user submits, then the UI
  identifies the field and config remains exactly unchanged.
- Given a name that already exists, when the user submits, then an explicit update flow is
  offered; unrelated providers and models are preserved; secrets remain masked.
- Given a non-`openai-chat` / non-`anthropic-messages` protocol input, when the user submits, then
  the field is rejected and config is unchanged.
- Given a non-HTTPS URL (except loopback), when the user submits, then the field is rejected and
  config is unchanged.
- Given valid fields up to confirm, when the user cancels at any step, then no partial
  provider/model/key is saved and the composer resumes.
- Given valid fields and confirmation, when the user confirms, then exactly one config update is
  written and `talos config list` masks the key as `***`.
- Given logs, panel labels, errors, or Debug output after save, when inspected, then the API key
  is absent or represented only as `***`.

## Required Reads

- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/config.rs`
- `crates/talos-config/src/endpoint.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/state.rs`

## Minimum Validation

- Wizard state-machine tests for every field, validation error, cancel point, update conflict,
  and the no-partial-write property.
- Name/protocol/URL validation unit tests (including loopback HTTP, non-HTTPS rejection,
  non-slug rejection, length boundaries).
- Config parse/save round trip for both protocols; existing providers preserved; key masking
  assertions.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.
