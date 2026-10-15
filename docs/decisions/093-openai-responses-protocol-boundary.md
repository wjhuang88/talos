# ADR-093: OpenAI Responses Protocol Boundary

## Status

Proposed — contract for claiming PROVIDER-008 / #724. Ineffective until an accepted claim merges;
it authorizes no implementation by itself. It supersedes the ADR-034 deferral
("The Responses API is a separate future decision") only for the boundaries defined here.

## Context

Talos speaks two wire protocols: `openai-chat` and `anthropic-messages`. OpenAI's newer
capabilities (reasoning summaries, encrypted reasoning replay, newer models) are delivered through
the Responses API (`POST /v1/responses`), and models.dev catalog data now carries provider/model
signals (`shape`, official `@ai-sdk/openai`) that indicate Responses support. Issue #724 requests
adapter support, catalog/custom-model recognition and configurable protocol mode.

## Decision

### 1. Protocol value and resolution

`ProviderProtocol` gains `OpenAIResponses` (`"openai-responses"`), mirrored in
`CatalogProviderProtocol`. Resolution order: model-level explicit config > provider-level explicit
config > catalog model-level confirmed signal > confirmed provider default (built-in official
`openai` → `openai-responses`) > `openai-chat`.

**Confirmed-support default (maintainer decision, 2026-10-15):** default to Responses only when
support is confirmed — built-in official `openai` provider without an explicit protocol, catalog
`shape = "responses"` (or an ADR-amended equivalent), or a custom model uniquely matching such a
catalog entry. Name-based guesses, `@ai-sdk/openai-compatible` gateways and ambiguous matches are
not confirmed and stay on `openai-chat`. Explicit user configuration (including explicit
`openai-chat`) always wins; config must distinguish "unset" from "explicitly chat", so the protocol
field must become `Option`-based or carry a source marker.

### 2. Public-API migration

`ProviderProtocol` is a public non-`non_exhaustive` enum; adding a variant breaks downstream
exhaustive matches. The enum is marked `#[non_exhaustive]` in the same minor release that adds the
variant, with a migration note (add catch-all arms). No patch-release protocol additions.

### 3. Statelessness

Requests use `store: false` and never `previous_response_id`. Conversation truth remains local
history/TLOG, so session restore, compaction and forking stay consistent.

### 4. Reasoning replay and confidentiality

With `store: false`, encrypted reasoning content is requested via
`include: ["reasoning.encrypted_content"]` and replayed only when `assistant_reasoning.model ==
model` (existing rule); otherwise it is dropped. Encrypted blobs are opaque, size-bounded,
never rendered in UI, logs, or debug snapshots (same class as ADR-023/045 masking), and never
persisted in readable form.

### 5. Catalog and matching

models.dev signals map at build time: model-level `shape = "responses"`/`"completions"` →
model-level protocol hint; provider `@ai-sdk/openai` → confirmed provider default. Signals absent
from data default to current behavior. Custom-model protocol inheritance follows the existing
unique-match rule (exact or single-prefix-strip; ambiguous/no match → no inheritance), applies only
when the user has not configured a protocol, and the effective protocol plus its source
(Configured / CatalogInferred / Default) is visible in model UI and debug snapshots. Catalog
regeneration is a separately reviewed diff.

### 6. Error handling

A gateway that lacks `/responses` must produce a clear error suggesting `protocol = "openai-chat"`;
no automatic downgrade/retry to a different protocol (avoids duplicate billing or repeated tool
side effects). Unknown stream event types are ignored or reported, never a panic (hard
constraint 9); streams that end without a terminal event are errors.

### 7. Interactions

Tool calls map to Responses `function_call`/`function_call_output` items bound by `call_id`
(Native tool protocol per ADR-021; `TalosStrict`/`Compat` keep the text protocol). Typed
`ProviderProgress` and retry boundaries match ADR-062, including no whole-stream retry after
output began. Image parts and one-shot continuation follow ADR-050/051. Bounded decision calls
follow ADR-075. MODEL-011 (active probe) and MODEL-014 (capability inference, branch-only) remain
separate scopes; overlap must be re-verified at claim time.

## Consequences

Confirmed-support users default to a new request path — a visible behavior change that must be
documented, reversible via explicit protocol, and covered by provenance display. The public enum
change gates a pre-1.0 minor release. Implementation requires child-Story decomposition, recorded
stream fixtures, proptest coverage, and independent review for the provider/security boundary.
