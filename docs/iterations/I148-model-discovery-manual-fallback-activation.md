# Iteration I148: MODEL-008-B Model Discovery, Manual Fallback, And Immediate Activation

> Document status: Review
> Published plan date: 2026-07-20
> Last updated: 2026-07-22 (P1-fix4)
> Planned objective: let a custom provider call its protocol-specific models
> endpoint to discover available model IDs, with a safe manual fallback, and
> immediately activate the selected `(provider, model)` in the current session.
> Baseline rule: preserve this objective; changed targets use a new iteration ID.
> MVP deliverable: after `/connect` wizard saves a custom provider, the user can
> discover models from the provider's models endpoint or enter one manually, and
> the selected model is immediately active in the current session.
> Review evidence: commits 23db287 (P1 tests), 187f13d (P1-fix provider_hint),
> 4d5f8d7 (P1-fix2 bridge integration), 834400b (P1-fix3 handler integration +
> ADR-048 semver), and this fix (P1-fix4 unsafe removal + doc status sync).
> Remaining human gate: maintainer real-terminal walkthrough of discovery →
> selection → activation → status sync.

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
| 2026-07-20 | Implementation | Discovery core (`provider_discovery::discover_provider_models`) + 9 mock HTTP fixture tests + wired to `handle_register_custom_provider`. Manual fallback via printed config instructions. Marked Review. |
| 2026-07-21 | Owner acceptance (NO-GO) | Owner feedback: discovery merely prints the list and tells the user to edit config + use `/model`. R9 rework required: discovery success should atomically persist discovered models and surface them through a picker. |
| 2026-07-21 | R9 rework | `handle_register_custom_provider` now runs discovery BEFORE the atomic `Config::save()` and persists up to `MAX_DISCOVERED_MODELS_TO_PERSIST = 32` discovered model IDs into `providers.{name}.models`. The existing `/model` picker surfaces them; selecting one runs the existing atomic provider+model save + session rebuild. Registration is decoupled from discovery success. 2 new R9 tests cover atomic persistence (mock /models endpoint) and provider-saved-when-discovery-fails. Residual: a dedicated DiscoveredModels TUI panel that auto-opens on registration remains a separate future iteration. Status: **Partial** (core atomic flow delivered; dedicated picker UX is the documented gap). |
| 2026-07-22 | P1 closeout | 7 new P1 tests prove the discovery → picker visibility → structured identity → activation closed loop. Tests cover: (1) OpenAI-compatible discovery models appear in `config.all_models()`; (2) Anthropic-compatible discovery same; (3) credential redaction — no API key in UI output; (4) structured identity — model IDs with `/` and `@` preserved exactly; (5) duplicate provider update preserves manually-added models; (6) manual fallback — after discovery failure, manually adding a model makes it visible; (7) selection → activation — setting active model to discovered ID produces correct identity. The code path was already implemented in the R9 rework; this commit adds test-only coverage. Status: **Partial → Review** (mock-proven closed loop; real-terminal walkthrough remains the human gate). |
| 2026-07-22 | P1-fix (NO-GO) | Owner returned NO-GO: (1) bridge dropped provider identity from `UserInput::SwitchModel`; (2) tests called `set_active_model` directly instead of going through the real lifecycle; (3) discovery failure semantics contradictory. Fixes: `provider_hint: Option<String>` added to `ModelSwitchRequest`, bridge forwards provider, `handle_session_model` uses `provider/model_id` to disambiguate. 3 P1-fix tests added. I148 reverted to **Partial**. |
| 2026-07-22 | P1-fix2 (NO-GO) | Owner returned NO-GO again: (1) still no real bridge→lifecycle integration test; (2) semver migration note missing for `ModelSwitchRequest.provider_hint`. Fixes: 2 bridge integration tests (`bridge_switch_model_forwards_provider_hint`, `bridge_switch_model_empty_provider_yields_none_hint`) proving `UserInput::SwitchModel → SessionLifecycleRequest::ModelSwitch` carries provider_hint. ADR-049 amended with migration note. Status remains **Partial** pending real-terminal walkthrough. |
| 2026-07-22 | P1-fix3 (NO-GO) | Owner returned NO-GO: (1) tests still didn't go through `handle_session_model`; (2) semver note in wrong ADR. Fixes: 2 real handler integration tests (`p1fix3_handle_session_model_success_rebuilds_once`, `p1fix3_handle_session_model_failure_no_rebuild`) proving bridge_rx_update exactly once on success, zero on failure, old config/session unchanged. Semver note moved to ADR-048. Status remains **Partial**. |
| 2026-07-22 | P1-fix4 (Review) | Owner returned GO for code, two doc/cleanup items: (1) 4 `unsafe { set_var }` blocks introduced by P1-fix3 tests — replaced with `with_isolated_home` helper (no new unsafe). (2) Document status header was still `Planned` — synced to **Review**. Status: **Partial → Review**. Remaining human gate: maintainer real-terminal walkthrough. |
