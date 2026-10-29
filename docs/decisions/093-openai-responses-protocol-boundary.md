# ADR-093: Wire-Protocol Family Expansion Boundary (OpenAI Responses First)

## Status

Accepted — PROVIDER-008-A / I306 contract, proposed atomically by governance PR #747 on 2026-10-22.
The decision becomes effective only when that exact reviewed claim/activation reaches `main`; before
merge, the branch status is proposal state only and authorizes no implementation. It supersedes the
ADR-034 deferral ("The Responses API is a separate future decision") only for the boundaries defined
here.

**Scope amendment (2026-10-20):** #724 expanded to complete wire-protocol family coverage
(OpenAI Responses, Google Gemini, Azure OpenAI, Amazon Bedrock; grouped by wire format). Sections
1, 3, 4, 6 and 7 below are restated as **family-agnostic umbrella principles** binding every
PROVIDER-008 child; section 2 (enum migration), 5 (catalog signals) and the Responses-specific
mapping remain this ADR's PROVIDER-008-A contract. Each sibling family (Gemini, Azure, Bedrock)
requires its own child ADR covering its wire mapping, auth and error semantics before its claim
becomes effective.

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

With `store: false`, current OpenAI Responses semantics return reasoning items with opaque
`encrypted_content` by default. Talos must preserve that stateless output when replay is enabled;
the legacy `include: ["reasoning.encrypted_content"]` request value may be emitted only as a
compatibility aid for older compatible implementations and must not be treated as a required modern
OpenAI parameter. Replay occurs only when `assistant_reasoning.model == model` (existing rule);
otherwise encrypted reasoning is dropped.

Encrypted blobs are opaque, size-bounded, never rendered in UI, logs, or debug snapshots (same class
as ADR-023/045 masking), and never persisted in readable form. Reasoning summaries are a separate,
explicitly requested display-safe output and must not be confused with encrypted reasoning state.

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
separate scopes; overlap was re-verified during the #747 claim preflight and remains excluded from
I306.

### 8. Model endpoint routing (2026-10-28)

A protocol signal alone does not implement a different model endpoint. Runtime model connection
resolution is separate from wire resolution: explicit model `base_url` > eligible packaged model
endpoint > the existing provider/adapter connection. An explicitly replaced provider root disables
packaged endpoint routing; model-level explicit configuration remains authoritative.

Packaged endpoint selection requires exact built-in provider/model identity, the unchanged catalog
provider root, a supported HTTP transport and a matching effective protocol. Explicit wire rollback
does not reuse the endpoint for another wire. Custom-provider unique-name matching inherits no vendor
endpoint: it must never relocate gateway credentials merely because a model name matches.

Catalog model endpoints are reviewed credential destinations, like built-in provider roots; a
cross-origin change must be visible in the separately reviewed catalog diff and provenance ledger.
Only committed, offline data participates in routing, not freshly fetched model metadata. Environment
roots (e.g. Neon) are resolved before dispatch; missing variables or unsupported explicit model
transports are errors. WebSocket/Realtimes, vendor auth wrappers and unsupported SDK transports
remain excluded, even if an upstream shape says `completions` or `responses`.

This adds optional `ModelConfig.base_url` and an effective endpoint/provenance query. It does not
change retry policy, endpoint probing, credentials, or admit new packaged rows without data review.

### 9. Public Copilot static HTTP authentication (2026-10-28)

The remaining public Copilot model endpoints can reuse the three existing native HTTP wire adapters
with a destination-specific static Bearer header contract. This is a bounded implementation of the
I306 connection residual, not admission of the excluded SDK/OAuth/vendor-token wrappers in section 8.
No account discovery, token exchange, refresh, browser login, enterprise routing or external runtime
is introduced. Existing `api_key` / `api_key_env` lookup supplies the caller's raw Copilot-compatible
credential. The packaged provider advertises `GITHUB_TOKEN` to connection setup. Manual TOML
must explicitly select `api_key_env = "GITHUB_TOKEN"` or, for example,
`api_key_env = "COPILOT_GITHUB_TOKEN"`; no implicit environment lookup is added.

Only the exact `https://api.githubcopilot.com:443` authority and known native routes qualify:
Chat `/chat/completions`, Responses `/responses`, Messages `/v1/messages`. If no URL was persisted,
the exact selected `github-copilot` provider uses its packaged provider default; it must never send
that credential to a generic adapter's OpenAI/Anthropic default. This does not infer a wire from the
provider name or inherit a vendor URL for a custom gateway. Bare or `/v1` API roots
are resolved to the selected wire's native root; a full endpoint for another wire is rejected.
Userinfo, query/fragment, other ports and unsupported paths are refused. Replaced gateways and
custom-provider unique-model matching cannot inherit Copilot authentication or relocate credentials.
The selected Copilot client refuses redirects, sends `Authorization: Bearer` (never `x-api-key` for
Messages), keeps `anthropic-version: 2023-06-01` for Messages, and adds truthful `talos` attribution,
a random process-stable interaction ID, and the reviewed API version. Previews completely mask
Copilot credentials. No retry crosses wire families or changes the selected credential destination.

Primary evidence: GitHub's [AWF auth matrix](https://github.com/github/gh-aw-firewall/blob/492391885dc25aeae354afeb4d19b0659c3fbabf/docs/auth-matrix.md)
and its [static auth implementation](https://github.com/github/gh-aw-firewall/blob/492391885dc25aeae354afeb4d19b0659c3fbabf/containers/api-proxy/providers/copilot-auth.js)
plus [Copilot adapter](https://github.com/github/gh-aw-firewall/blob/492391885dc25aeae354afeb4d19b0659c3fbabf/containers/api-proxy/providers/copilot.js).
Microsoft's published `@vscode/copilot-api` 0.5.2 independently defines the three native routes,
including direct `ChatMessages` dispatch, and `X-GitHub-Api-Version: 2026-08-01`.
Its tarball SHA256 remains `7a817c4d468abb7811964af047de00aeee13e70ac97e1f698d3b774d1580d6a4`.
Only the HTTP contract is used; no SDK code or dependency is imported.

This evidence does not guarantee any caller's subscription, token scopes, integration quota or model
allowlist. The service retains those authorization decisions. Talos identifies itself, never as
VS Code, the Copilot CLI or AWF; SDK Mission Control integration-ID examples are not treated as a
raw Messages model-allowlist guarantee. No live vendor success or universal token compatibility is
claimed. Seven packaged rows remain withheld until a separate, reviewed data stage follows exact
runtime validation. The archived model source and all non-routing model/provider fields stay fixed.

## Consequences

Confirmed-support users default to a new request path — a visible behavior change that must be
documented, reversible via explicit protocol, and covered by provenance display. The public enum
change gates a pre-1.0 minor release. PROVIDER-008-A / I306 owns implementation under this contract;
Gemini, Azure and Bedrock still require separately claimed child Stories and child ADRs.

Implementation requires recorded stream fixtures, proptest coverage, exact-head CI and independent
provider/API review. #747 is governance only: no production implementation is authorized until its
Accepted/Claimed/Active records become target-branch truth.

## Current API Verification — 2026-10-22

Before implementation, the OpenAI developer documentation was re-checked against the 2026-10-20
intake assumptions. Current stateless Responses documentation says `store: false` output reasoning
items include `encrypted_content` by default and retains `reasoning.encrypted_content` in `include`
only for compatibility. Current function-calling documentation still binds `function_call_output` to
the originating `function_call` with `call_id`, and current streaming guidance identifies
`response.created`, `response.output_text.delta`, `response.completed` and `error` as core text
lifecycle events. I306 tests must use recorded fixtures representative of the current API rather than
freezing older documentation examples.
