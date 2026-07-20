# MODEL-008-B: Model Discovery, Manual Fallback, And Immediate Activation

| Field | Value |
| --- | --- |
| Story ID | MODEL-008-B |
| Type | Product / Configuration Story |
| Priority | P2 |
| Status | Refinement — selected into I148 (2026-07-20) |
| Source | Maintainer requirement recorded 2026-07-20; child of MODEL-008 |
| Parent Epic | MODEL-008 |
| Depends on | MODEL-008-A (I147) wizard + atomic config; ADR-013 provider config; ADR-023 credential boundary |
| Blocks | — |

## Problem

MODEL-008-A persists a provider entry, but a provider entry without a selected model is a dead end
for `/model`. A user needs to discover at least one model ID from the registered gateway, or enter
one manually when discovery is unavailable, and immediately activate the selected
`(provider, model)` in the current session.

## Goal / Value

After MODEL-008-A saves a custom provider, call the provider's protocol-specific models endpoint
to discover available model IDs, present a bounded searchable picker, allow manual entry when
discovery fails for any reason, persist only the user-selected opaque model ID, and activate the
new `(provider, model)` in the current session so `/model` and the status bar reflect it
immediately.

## Scope

Protocol-specific discovery:

- `openai-chat`:
  - Derive `GET /models` from the normalized gateway root using the existing endpoint rules.
  - Use the adapter-compatible `Authorization: Bearer <key>` header.
  - Do not generate a duplicate `/chat/completions` path.
- `anthropic-messages`:
  - Use only the already-documented models-list endpoint and required headers.
  - Do not guess paths.
  - Do not fall back across hosts.

Operational requirements:

- Reuse the existing dispatch / first-packet / idle timeout policy.
- Cap response bytes, model count, and displayed model ID length.
- Parse only the expected protocol response shape; no unbounded JSON.
- On success, present a bounded, searchable model ID picker.
- Persist only the user-selected opaque model ID in
  `[providers.<name>.models.<model>]`.
- Remote price/capability metadata is display-only; never trusted or persisted as authoritative.
- Carry `(provider, model_id)` structurally; never reserialize to `/model provider/model` or
  `/model model@variant` command text.
- On registration success, activate the chosen `(provider, model)` for the current session through
  the existing model lifecycle. `/model` and the status bar must reflect the new model immediately.

Failure handling (each must offer Retry / Edit / Enter model ID manually):

- Timeout.
- Authentication failure.
- Malformed response.
- Oversized response.
- Empty model list.
- Gateway does not support `/models`.
- Network error.

No partial configuration may be written on any failure path.

## Explicit Exclusions

- Trusting or persisting remote price/capability metadata as authoritative.
- Cross-host fallback for the Anthropic models endpoint.
- New provider protocols, OAuth, arbitrary custom JSON/headers, or new transport code.
- New `unsafe` blocks or native dependencies.
- Editing/deleting multiple custom models after registration.

## Design / Security Constraints

- Reuse `ProviderProtocol`, dispatch, timeout, and configuration save logic.
- Credentials must remain masked in logs, errors, Debug output, panel labels, and UI surfaces.
- The `(provider, model_id)` identity must be a typed struct, not a command string.
- Config writes are atomic: either the full provider + selected model entry is written, or nothing.
- Model IDs containing `/` or `@` must be preserved exactly — they are panel data, not command
  syntax.

## Acceptance

- Given valid custom-provider fields and a gateway that implements the selected protocol's models
  interface, when validation succeeds, then only the bounded returned model IDs are shown and only
  the user-selected ID is persisted.
- Given a gateway that lacks `/models`, returns malformed data, times out, rejects the discovery
  request, returns an empty list, or has a network error, when validation finishes, then no
  partial config is written and the user can Retry, Edit, or enter a nonempty model ID manually;
  manual completion remains usable.
- Given `openai-chat` and a compatible gateway URL, when saved, then the normalized base URL is
  used by the existing OpenAI adapter and no extra `/chat/completions` duplication occurs.
- Given `anthropic-messages`, when the user enters a valid endpoint, then the saved endpoint has
  the messages-compatible form required by the existing Anthropic adapter.
- Given a model ID containing `/` or `@`, when the provider is registered and selected from the
  picker, then its provider/model identity remains exact and is not parsed as a command suffix or
  provider prefix.
- Given successful registration, when the user selects a model, then the current session is
  activated with the new `(provider, model)` and `/model` plus the status bar reflect it
  immediately.
- Given logs, errors, panel labels, or Debug output after discovery, when inspected, then the API
  key is absent or represented only as `***`.

## Required Reads

- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md`
- `docs/backlog/active/MODEL-008-A-interactive-custom-provider-wizard.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/anthropic_request.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-tui/src/panel_state.rs`

## Minimum Validation

- Two-protocol mock HTTP fixtures for: successful OpenAI-compatible discovery, successful
  Anthropic-compatible discovery, path derivation, authorization header shape, timeout,
  oversized response, malformed payload, empty result, auth failure, network error, and manual
  fallback. No live provider credential is permitted in tests.
- Config atomicity and credential redaction assertions.
- Session rebuild + picker integration tests proving the new model is immediately listed and
  selectable.
- Structured-identity tests for model IDs containing `/` and `@`.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.
