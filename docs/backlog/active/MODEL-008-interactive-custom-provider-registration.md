# MODEL-008: Interactive Custom Provider Registration

| Field | Value |
| --- | --- |
| Story ID | MODEL-008 |
| Type | Product / Configuration Story (Epic) |
| Priority | P2 |
| Status | Refinement — split into child Stories MODEL-008-A (I147) and MODEL-008-B (I148) on 2026-07-20 |
| Source | Maintainer requirement recorded 2026-07-20 |
| Depends on | MODEL-005 / MODEL-006 existing `/connect` flow; configuration schema ADR-013; credential boundary ADR-023 |
| Blocks | — |
| Child Stories | [MODEL-008-A](MODEL-008-A-interactive-custom-provider-wizard.md) (I147 wizard + atomic config) · [MODEL-008-B](MODEL-008-B-model-discovery-manual-fallback-activation.md) (I148 discovery + manual fallback + activation) |

## Problem

Talos can execute a manually configured custom provider, but `/connect` only presents catalog
providers and does not provide a complete interactive path for a user to name a new provider,
choose its protocol, supply its endpoint and make at least one model selectable. A provider entry
without a model is a dead end for `/model`; collecting only name, URL and protocol is therefore
not sufficient.

## Goal / Value

Allow a user to register a new API-compatible provider entirely from `/connect`, without editing
TOML: provider name, protocol, Base URL and API credential are validated against the configured
provider's models interface; the user selects a discovered model (or enters one manually when
discovery is unavailable) to form a valid, persisted configuration that `/model` can immediately
select.

## Scope

`/connect` gains an explicit **Add custom provider** entry, distinct from catalog provider setup.
Its sequential, cancel-safe form is:

1. **Provider name** — required canonical slug: lowercase ASCII letter/digit start; then
   lowercase letters, digits or `-`; length 1–64. An existing name enters an explicit update flow,
   never silently overwrites credentials or models.
2. **Protocol** — required choice from the existing closed set only:
   `openai-chat` or `anthropic-messages`. No free-form protocol string and no new provider adapter
   are implied.
3. **Base URL** — required absolute `https://` URL (or `http://` only for loopback addresses).
   Normalize it with the existing endpoint rules and show the normalized non-secret endpoint before
   save. Protocol-specific endpoint shape must be validated: the Anthropic adapter requires its
   messages endpoint; OpenAI-compatible adapter uses a gateway root.
4. **API key** — required for this story, masked while entered and in all diagnostics. It is used
   only for the user-requested validation/discovery request and is saved through the existing
   ADR-023 persistence/masking boundary. Environment-variable-only setup and OAuth are excluded.
5. **Validate and discover models** — after the first four valid fields, perform a bounded,
   protocol-specific authenticated models-list request to the configured URL:
   - `openai-chat`: derive the standard OpenAI-compatible `GET /models` endpoint from the
     normalized gateway root and use the adapter-compatible authorization header.
   - `anthropic-messages`: derive only the documented models-list endpoint from the normalized
     Anthropic-compatible base URL and use the adapter-compatible required headers. Do not guess
     arbitrary paths or silently fall back to an unrelated host.
   - Apply existing dispatch/first-packet/idle timeout policy, cap response bytes and model count,
     parse only the expected protocol response shape, and show no credential in errors or logs.
   - A successful response presents a searchable, bounded picker of returned IDs. Selecting one
     supplies the initial model ID. Remote price/capability metadata is display-only and is not
     trusted or persisted in this story.
6. **Manual model fallback** — a discovery failure, unsupported endpoint, malformed response or
   empty model list must clearly offer Retry, Edit connection fields, or **Enter model ID manually**.
   A gateway lacking `/models` therefore remains usable; discovery failure must not write partial
   configuration or block manual completion.
7. **Initial model ID** — whether selected or manually entered, it is required, nonempty and an
   opaque provider identifier. It is persisted in `[providers.<name>.models.<model>]` and becomes
   immediately available through `/model`. The implementation must carry `(provider, model_id)`
   structurally rather than serialize it into the ambiguous `/model provider/model` command
   grammar: model IDs containing `/` or `@` must not be misparsed.
8. **Confirmation and atomic save** — display provider name, protocol, normalized URL and model
   ID, but never the key. On confirmation write one coherent config update. Cancellation or any
   validation failure leaves config unchanged.

After successful registration Talos must activate the chosen `(provider, model)` for the current
session, rebuild through the existing model lifecycle, persist it, and show it in `/model` and the
status bar. If provider connectivity testing is added, it must be bounded by existing provider
timeouts; a failed test must offer retry/edit/cancel and must not destroy the prior working config.

## Explicit Exclusions

- OAuth/device flow, token refresh, token-cache storage, or changes to PROVIDER-003.
- Arbitrary provider protocol plugins, custom request JSON, custom headers, or new transport code.
- Automatic catalog import, pricing/capability discovery, or model variants for manually added
  models. The one-time remote discovery list is transient and only its user-selected ID is saved.
- Editing/deleting multiple custom models after registration; that is follow-on work.
- Relaxing the credential masking/persistence boundary in ADR-023.

## Design / Security Constraints

- Reuse `ProviderProtocol`, `ProviderConfig`, endpoint normalization and configuration save logic;
  do not create a second credentials store.
- Never log, render, serialize to UI status, or retain an API key in a panel label/debug surface.
- Validate name/model/base URL before mutation; reject duplicates or require an explicit update
  confirmation. Preserve unrelated provider entries and per-model limits.
- The protocol choice is data configuration only. It does not authorize unsafe code, dynamic code
  loading, or a new network trust boundary.
- Determine and document the public API/semver impact before changing public credential request or
  response types. If a breaking public Rust API change is unavoidable, create an ADR with migration
  guidance before marking this story Ready.

## Acceptance

- Given `/connect`, When the user selects Add custom provider and completes valid fields, Then one
  provider configuration and one model configuration are persisted and `/model` immediately lists
  the model under the new provider.
- Given valid custom connection fields and an endpoint that implements the selected protocol's
  models interface, When validation succeeds, Then Talos presents only the bounded returned model
  IDs and saves the ID explicitly selected by the user.
- Given a compatible gateway that lacks `/models`, returns malformed data, times out or rejects the
  discovery request, When validation finishes, Then no partial config is written and the user can
  Retry, Edit, or enter a nonempty model ID manually; manual completion remains usable.
- Given `openai-chat` and an OpenAI-compatible gateway URL, When saved, Then the normalized base
  URL is used by the existing OpenAI adapter and no extra `/chat/completions` duplication occurs.
- Given `anthropic-messages`, When the user enters a valid endpoint, Then the saved endpoint has
  the messages-compatible form required by the existing Anthropic adapter.
- Given invalid name, protocol selection, URL, empty model ID, duplicate-name conflict, or failed
  confirmation, When the user submits, Then the UI identifies the field and config remains exactly
  unchanged.
- Given cancel at every wizard step, When Esc is pressed, Then no partial provider/model/key is
  saved and the normal composer resumes.
- Given a model ID containing `/` or `@`, When the provider is registered and selected from the
  picker, Then its provider/model identity remains exact and it is not parsed as a command suffix
  or provider prefix.
- Given an existing custom provider, When the user explicitly chooses update, Then unrelated
  providers and models are preserved and secrets remain masked.
- Given `talos config list` / `get`, logs, panel labels or errors after setup, Then the API key is
  absent or represented only as `***`.

## Required Reads

- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `docs/backlog/active/MODEL-006-interactive-model-catalog-browser.md`
- `docs/backlog/active/PROVIDER-003-dynamic-provider-credentials.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/config.rs`
- `crates/talos-config/src/endpoint.rs`
- `crates/talos-cli/src/session_handlers.rs`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-tui/src/panel_state.rs`
- `crates/talos-tui/src/state.rs`

## Minimum Validation

- Config parse/save round trip for both protocols; existing data preserved; key masking assertions.
- Wizard state-machine tests for every field, validation error, cancel point, update conflict and
  no-partial-write property.
- Model picker tests proving the new model is immediately listed and selectable.
- Structured-identity tests for model IDs containing `/` and `@`.
- Mock-provider session rebuild tests for each protocol; no external credential or network required.
- Local mock HTTP tests for successful OpenAI-compatible/Anthropic-compatible models discovery,
  endpoint derivation, authorization header shape, timeout, oversized response, malformed payload,
  empty result and manual fallback. No live provider credential is permitted in tests.
- Documentation update: README, `docs/reference/config.reference.toml`, and the public site
  configuration page must describe the wizard and equivalent TOML.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
