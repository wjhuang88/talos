# PROVIDER-008: Complete Wire-Protocol Family Coverage And Protocol Mode Configuration

**Status**: Refinement / Unclaimed (child A Complete / Closed; B/C/D unimplemented)
**Type**: Provider / Config / Model-Catalog Epic (child Stories decompose at claim time)
**Parent Epic**: None
**Source Issue**: [#724](https://github.com/wjhuang88/talos/issues/724)
**Selected Iteration**: None

Children:

- [PROVIDER-008-A](PROVIDER-008-A-openai-responses-protocol.md) / [#748](https://github.com/wjhuang88/talos/issues/748): protocol architecture (enum/factory dispatch/model-level protocol/catalog signals) + OpenAI Responses adapter (ADR-093); I306 / effective governance claim #747; code #749 and catalog #775 merged; Review / Claimed.
- PROVIDER-008-B: Google Gemini `generateContent` (AI Studio + Vertex auth boundary decision) — unclaimed.
- PROVIDER-008-C: Azure OpenAI transport conventions + per-model shape routing — unclaimed.
- PROVIDER-008-D: Amazon Bedrock Converse + SigV4 credential decision — unclaimed.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Parent epic remains unclaimed; child A (#748) is separately claimed by effective #747. |
| Claimed At | Not applicable |
| Source Issue | #724 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-10-28 |
| Handoff / Release Condition | Child Stories are independently claimed. #747 activates only PROVIDER-008-A/#748; B/C/D stay unclaimed. |

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

Per [#724](https://github.com/wjhuang88/talos/issues/724) (authoritative epic detail; child #748 owns
A implementation):

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

ADR-093 is the umbrella + Responses contract. #747 proposes its acceptance together with
PROVIDER-008-A / #748 / I306 claim+activation; this parent remains unclaimed. Interacts with
ADR-013/021/022/023/045/048/050/051/062/075, MODEL-011, MODEL-014 (unclaimed), PROVIDER-004.
Sibling family claims must add their own ADRs and re-run overlap preflight.

## Acceptance (summary; Given/When/Then detail in #724/#748)

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

- Issue #724 (epic) and #748 (child A implementation requirement).
- `docs/decisions/034-reasoning-thinking-boundary.md`, `013-provider-config-schema-boundary.md`,
  `021-tool-call-protocol-architecture.md`, `062-typed-provider-retry-progress.md`.
- `crates/talos-provider/src/openai*.rs`, `crates/talos-config/src/{types,endpoint,config,model}.rs`,
  `crates/talos-config/build.rs`, `crates/talos-cli/src/provider_setup.rs`.

## Intake Checkpoint — 2026-10-15

Registered from maintainer request with #724 as the external requirement record. No parent-epic
claim or implementation. Claiming follows child decomposition and AGENT-COLLABORATION.

## Scope Expansion — 2026-10-20

Maintainer expanded #724 from Responses-only to complete wire-family coverage (Google Gemini,
Azure OpenAI, Amazon Bedrock; grouped by wire format per models.dev inventory). ADR-093 is
amended to carry the family-agnostic umbrella principles while remaining the
PROVIDER-008-A contract; PROVIDER-008-B/C/D require their own child ADRs at claim time.

## Child A Selection — 2026-10-22

Maintainer requested implementation of #724. Child requirement #748 was created for PROVIDER-008-A
so the parent epic retains a single Source Issue/owner relationship. Governance-only #747 creates
PROVIDER-008-A / I306, proposes ADR-093 acceptance, and claims only protocol architecture + OpenAI
Responses. The parent stays Refinement / Unclaimed and siblings B/C/D stay unclaimed. #747 has no
effect until its finalized exact head reaches main; implementation must start from that merge or
later.


## Child A Merged Checkpoint — 2026-10-28

Child A / #748 / I306 is Review / Claimed after code #749 (`60e27eb6`) and catalog #775
(`6af8654c`) merged with exact-head eight-job CI, technical review and CAS. Its owner retains
Completion Commit: Pending. Distinct model-endpoint routing remains unimplemented (56 catalog rows
withheld) and must be resolved through implementation or an explicit accepted scope decision before
child acceptance closes. This residual tracking does not satisfy the multi-protocol-host requirement.
B/C/D remain unclaimed; parent #724 stays open and is not Complete. Earlier proposal-only checkpoints
are historical; no sibling implementation or release is claimed.


## HTTP Endpoint Stage — 2026-10-28

Child A's #792 runtime and separate data activation stage cover 45 reviewed HTTP endpoint pairs
(Neon 14 Responses, Ofox/Zenmux 27 Messages, Freemodel 4 Chat), including a vendor-root rollback
safeguard. Seven Copilot Messages endpoints remain unreviewed for vendor authentication/transport;
four WSS records are the already-excluded Realtime family. The owner remains Review / Claimed with
Completion Commit: Pending. Neither a staged candidate nor moving a residual into this parent is
completion or a scope waiver. B/C/D remain unclaimed; #724/#748 stay open and no release is claimed.


## Child A acceptance closure — 2026-10-29

Child A / #748 / I306 closes owner-first with existing implementation evidence
`dd6c5f82ed88cc963b47758dee06fa506cb407a1` (#803) and the earlier implementation chain
recorded in its owner. All 52 supported reviewed HTTP endpoint pairs are now active; the seven
Copilot records are no longer an implementation residual. CI3570 passed 8/8 and exact-object
technical review approved. Four WSS records remain the original Realtime exclusion. Prior
checkpoints above are historical. Parent #724 remains open and incomplete: B Gemini, C Azure
and D Bedrock need independent claims, ADRs, runnable implementations and full acceptance.
No release or sibling completion is claimed. #748 closes only after the owner closure reaches main.
