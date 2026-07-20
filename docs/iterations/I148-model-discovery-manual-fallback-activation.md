# Iteration I148: MODEL-008-B Model Discovery, Manual Fallback, And Immediate Activation

> Document status: Planned
> Published plan date: 2026-07-20
> Planned objective: let a custom provider call its protocol-specific models
> endpoint to discover available model IDs, with a safe manual fallback, and
> immediately activate the selected `(provider, model)` in the current session.
> Baseline rule: preserve this objective; changed targets use a new iteration ID.
> MVP deliverable: after `/connect` wizard saves a custom provider, the user can
> discover models from the provider's models endpoint or enter one manually, and
> the selected model is immediately active in the current session.

## Published Baseline

- Selected Ready story: MODEL-008-B, under ADR-013 (provider config) and ADR-023 (credential boundary).
- Dependencies satisfied: MODEL-008-A (I147, Review — wizard + atomic config).
- Protocol-specific discovery:
  - `openai-chat`: derive `GET /models` from the normalized gateway root using existing endpoint rules; use adapter-compatible `Authorization: Bearer <key>` header; do not generate duplicate `/chat/completions`.
  - `anthropic-messages`: use only the documented models-list endpoint and required headers; do not guess paths; do not cross-host fallback.
- Reuse existing dispatch / first-packet / idle timeout policy.
- Cap response bytes, model count, and displayed model ID length.
- Parse only the expected protocol response shape; no unbounded JSON.
- On success, present a bounded, searchable model ID picker.
- Persist only the user-selected opaque model ID.
- Failure handling (each must offer Retry / Edit / Enter model ID manually):
  - Timeout, auth failure, malformed response, oversized response, empty model list, gateway does not support `/models`, network error.
- No partial configuration on any failure path.
- On registration success, activate the chosen `(provider, model)` for the current session through the existing model lifecycle. `/model` and the status bar must reflect the new model immediately.
- Carry `(provider, model_id)` structurally; never reserialize to command text.
- Model IDs containing `/` or `@` must be preserved exactly.
- Remote price/capability metadata is display-only; never trusted or persisted as authoritative.

## Explicit Non-Goals

- Trusting or persisting remote price/capability metadata as authoritative.
- Cross-host fallback for the Anthropic models endpoint.
- New provider protocols, OAuth, arbitrary custom JSON/headers, or new transport code.
- New `unsafe` blocks or native dependencies.
- Editing/deleting multiple custom models after registration.

## Acceptance

- Given valid custom-provider fields and a gateway that implements the selected protocol's models interface, when validation succeeds, then only the bounded returned model IDs are shown and only the user-selected ID is persisted.
- Given a gateway that lacks `/models`, returns malformed data, times out, rejects the discovery request, returns an empty list, or has a network error, when validation finishes, then no partial config is written and the user can Retry, Edit, or enter a nonempty model ID manually; manual completion remains usable.
- Given `openai-chat` and a compatible gateway URL, when saved, then the normalized base URL is used by the existing OpenAI adapter and no extra `/chat/completions` duplication occurs.
- Given `anthropic-messages`, when the user enters a valid endpoint, then the saved endpoint has the messages-compatible form required by the existing Anthropic adapter.
- Given a model ID containing `/` or `@`, when the provider is registered and selected from the picker, then its provider/model identity remains exact.
- Given successful registration, when the user selects a model, then the current session is activated with the new `(provider, model)` and `/model` plus the status bar reflect it immediately.

## Planned Validation

- Two-protocol mock HTTP fixtures for: successful OpenAI-compatible discovery, successful Anthropic-compatible discovery, path derivation, authorization header shape, timeout, oversized response, malformed payload, empty result, auth failure, network error, and manual fallback. No live provider credential is permitted in tests.
- Config atomicity and credential redaction assertions.
- Session rebuild + picker integration tests proving the new model is immediately listed and selectable.
- Structured-identity tests for model IDs containing `/` and `@`.
- Locked fmt/check/clippy/test and `scripts/validate_project_governance.sh .`; `git diff --check`.

## Required Reads

- `docs/backlog/active/MODEL-008-B-model-discovery-manual-fallback-activation.md`
- `docs/backlog/active/MODEL-008-A-interactive-custom-provider-wizard.md`
- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `crates/talos-provider/src/openai.rs`
- `crates/talos-provider/src/anthropic_request.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-tui/src/panel_state.rs`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-20 | Planning | Baseline published. Activation follows I147 completion. |
