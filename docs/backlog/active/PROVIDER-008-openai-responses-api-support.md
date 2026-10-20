# PROVIDER-008: Complete Wire-Protocol Family Coverage And Protocol Mode Configuration

**Status**: Refinement / Unclaimed
**Type**: Provider / Config / Model-Catalog Epic (child Stories decompose at claim time)
**Parent Epic**: None
**Source Issue**: [#724](https://github.com/wjhuang88/talos/issues/724)
**Selected Iteration**: None

Suggested children (each independently claimable, with its own ADR):

- PROVIDER-008-A: protocol architecture (enum/factory dispatch/model-level protocol/catalog signals) + OpenAI Responses adapter (ADR-093).
- PROVIDER-008-B: Google Gemini `generateContent` (AI Studio + Vertex auth boundary decision).
- PROVIDER-008-C: Azure OpenAI transport conventions + per-model shape routing.
- PROVIDER-008-D: Amazon Bedrock Converse + SigV4 credential decision.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #724 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | Intake only: no claim, no activation, no implementation authority. Claiming follows AGENT-COLLABORATION preflight and the accepted ADR-093. |

## Identity / Goal / Value

Users need Talos to speak every mainstream model wire-protocol family, not just two. Per the #724
scope expansion (2026-10-20), grouped by wire format rather than by provider:

- Already supported: `openai-chat` (~210 providers / ~8360 models incl. all openai-compatible
  gateways) and `anthropic-messages` (9 / 82).
- To add: `openai-responses` (6 / 122+, ADR-093), `google-gemini` (2 / 101+), `azure-openai`
  (2 / 194, transport conventions over chat/responses bodies), `bedrock-converse` (1 / 199,
  SigV4). Long-tail native formats (e.g. cohere) stay on their OpenAI-compatible endpoints.

models.dev snapshot 2026-10-20: 231 providers / 9080 models; multi-protocol hosts (opencode, ofox,
zenmux, cloudflare-ai-gateway, github-copilot, azure, google-vertex, amazon-bedrock) carry
model-level `provider: {npm, shape}` overrides, so protocol resolution must be model-level.

Confirmed maintainer decisions (Issue #724): 2026-10-15 — default to a protocol family's native
format when support is confirmed, `openai-chat` otherwise, explicit configuration always wins;
2026-10-20 — full wire-family coverage with family-level grouping; npm→family mapping is external
knowledge to verify per provider, never name-based guessing for gateways.

## Scope

Per [#724](https://github.com/wjhuang88/talos/issues/724) (authoritative detail; children own
implementation):

- One adapter module group per wire family (request build + stream parse + family specifics),
  reusing HTTP/auth/timeout/retry/`ProviderProgress`.
- Factory dispatch refactor (provider_setup, desktop, discovery, models_browser) to
  registry-style per-protocol construction instead of N-way exhaustive matches; prerequisite or
  synchronous work of PROVIDER-008-A.
- Model-level protocol resolution: `ModelConfig.protocol` with
  model-explicit > provider-explicit > catalog model-level signal > confirmed provider default >
  `openai-chat`, with visible provenance (Configured / CatalogInferred / Default).
- Catalog: normalize npm + model-level `provider.{npm,shape}` overrides into family protocol
  hints in `models.toml`; centralized external-knowledge mapping table; regeneration PRs
  separately reviewed.
- Public enum: `#[non_exhaustive]` + new variants in the same pre-1.0 minor release.
- Per-family confirmed-support default rules defined in each child ADR; no cross-protocol
  downgrade retry; stateless everywhere.

## Exclusions

See #724 non-goals: no server-side session state for any family, no provider built-in tools
(Responses web_search/file_search, Gemini grounding/caching billing, Bedrock guardrails config),
no Realtime/Assistants/Batch, no active probing (MODEL-011), no silent switch for unconfirmed
openai-compatible gateways, no long-tail native formats (cohere etc.) unless separately
requested.

## Dependencies / Decision Links

Requires a new ADR-093 (Responses protocol boundary, confirmed-support default, public-enum
`#[non_exhaustive]` migration, catalog signal mapping, reasoning replay policy) superseding the
ADR-034 deferral. Interacts with ADR-013/021/022/023/045/048/050/051/062/075, MODEL-011,
MODEL-014 (branch-only), PROVIDER-004; the claiming PR must re-verify overlap.

## Acceptance (summary; Given/When/Then detail in #724)

- Confirmed-support default: built-in `openai` without explicit protocol uses Responses and is
  revertible via explicit `protocol = "openai-chat"`; unconfirmed gateways stay on `openai-chat`.
- Full-turn correctness on `/responses` for text, tool calls, images and reasoning summaries.
- Old configs with explicit protocols behave identically; protocol provenance is visible.
- No-crash on unknown stream events; bounded encrypted-reasoning handling with no disclosure.

## Planned Validation / Documentation

Locked workspace tests and proptest for parsing; recorded-fixture stream tests; config precedence
matrix tests; catalog regeneration diff reviewed in its own PR; `release_preflight.sh`; README en/zh,
SDK contract, ARCHITECTURE, release notes.

## Residual Destination

Remaining protocol gaps (Responses built-in tools, server-side state, Realtime) stay out of scope
and return to the backlog.

## Required Reads

- Issue #724 (full implementation steps, risks, acceptance).
- `docs/decisions/034-reasoning-thinking-boundary.md`, `013-provider-config-schema-boundary.md`,
  `021-tool-call-protocol-architecture.md`, `062-typed-provider-retry-progress.md`.
- `crates/talos-provider/src/openai*.rs`, `crates/talos-config/src/{types,endpoint,config,model}.rs`,
  `crates/talos-config/build.rs`, `crates/talos-cli/src/provider_setup.rs`.

## Intake Checkpoint — 2026-10-15

Registered from maintainer request with #724 as the external requirement record. No claim, no
iteration, no implementation. Claiming requires: accepted ADR-093, child-Story decomposition,
preflight inventory, and an atomic claim+activation PR per AGENT-COLLABORATION.

## Scope Expansion — 2026-10-20

Maintainer expanded #724 from Responses-only to complete wire-family coverage (Google Gemini,
Azure OpenAI, Amazon Bedrock; grouped by wire format per models.dev inventory). ADR-093 is
amended to carry the family-agnostic umbrella principles while remaining the
PROVIDER-008-A contract; PROVIDER-008-B/C/D require their own child ADRs at claim time.
This owner stays Unclaimed; no child is activated.
