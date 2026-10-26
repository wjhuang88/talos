# PROVIDER-008-A: Protocol Architecture And OpenAI Responses

**Status**: Active / Claimed; selected for I306
**Type**: Provider / Config / Model-Catalog Story
**Parent Epic**: [PROVIDER-008](PROVIDER-008-openai-responses-api-support.md) / [Issue #724](https://github.com/wjhuang88/talos/issues/724)
**Source Issue**: [#748](https://github.com/wjhuang88/talos/issues/748)
**Selected Iteration**: [I306](../../iterations/I306-provider008a-openai-responses.md)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 / talos开发 #724 continuation 2026-10-26 |
| Work Slice | PROVIDER-008-A protocol architecture plus OpenAI Responses only; Gemini, Azure and Bedrock remain separate children. |
| Claimed At | 2026-10-22 |
| Source Issue | #748 |
| Governance Claim PR | #747 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of parent #724 in the 2026-10-22 talos开发 session; child requirement #748 records this bounded slice. Governance-only #747 must pass exact-head applicable CI, both governance validators, provider/API technical review and merge-time CAS before this claim is effective. |
| Implementation PR | #749 |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Claim activates only when #747 reaches main with accepted ADR-093. Sibling families remain unclaimed; Complete requires existing implementation commit evidence. |

## Goal

Introduce the shared protocol-selection architecture required by PROVIDER-008 and deliver the first new
wire family, OpenAI Responses, without changing unconfirmed OpenAI-compatible gateways away from
`openai-chat`.

The runnable deliverable is one complete agent round over `POST /v1/responses`, including text,
images, native function calls/results, reasoning summaries, usage, terminal/error handling and
protocol provenance, while preserving old explicit protocol behavior.

## Scope

- Add `openai-responses` to the public provider protocol contract and mark the public enum
  `#[non_exhaustive]` in the same pre-1.0 minor migration boundary defined by ADR-093.
- Refactor provider construction so protocol-specific adapter creation is centralized rather than
  duplicated across CLI/Desktop/discovery/model-browser exhaustive matches.
- Add model-level protocol configuration and resolution provenance with this precedence:
  model explicit > provider explicit > catalog model signal > confirmed provider default >
  `openai-chat`.
- Preserve explicit `openai-chat`; only confirmed support may select Responses by default.
- Normalize the catalog signals needed for Responses (`shape = responses/completions`, official
  provider signal) without name-based inference. Catalog regeneration remains a separately
  reviewable diff/stage.
- Implement a dedicated Responses request/stream adapter module group. Do not add protocol branches
  into the existing OpenAI Chat adapter.
- Map Responses input/output/tool/reasoning/usage semantics into existing Talos provider events and
  retry/progress boundaries.
- Keep requests stateless: `store: false`; never use `previous_response_id`.
- Preserve current stateless Responses `encrypted_content` reasoning items when replay is applicable;
  legacy `include = ["reasoning.encrypted_content"]` is compatibility-only, not a required modern
  OpenAI request field. Replay remains subject to the existing same-model rule.
- Return a clear unsupported-protocol error with an explicit `protocol = "openai-chat"` escape hatch;
  never retry across protocols.
- Surface effective protocol and source in model-selection/debug surfaces covered by #724/#748.

## Non-Goals

- Google Gemini, Azure OpenAI and Amazon Bedrock implementation; they remain PROVIDER-008-B/C/D.
- Active endpoint probing (MODEL-011) or MODEL-014 capability/variant expansion.
- Responses built-in tools (`web_search`, `file_search`), Realtime, Assistants, Batch, server-side
  conversation state or implicit cross-protocol fallback.
- Long-tail native formats such as Cohere native.
- Unrelated provider retry-policy, permission, storage, session or runtime redesign.

## Dependencies / Decision Links

- ADR-093 becomes Accepted atomically with this claim/activation in #747.
- Preserve ADR-013 provider schema, ADR-021 native tool protocol, ADR-034 reasoning model identity,
  ADR-050/051 image continuation, ADR-062 typed retry progress and ADR-075 bounded decision calls.
- PROVIDER-004 tool-call identity behavior must not regress.
- MODEL-014 is Refinement / Unclaimed and remains a separate owner; this Story may only add protocol
  metadata required by #724/#748, not general capability inference.

## Acceptance

- Given built-in official `openai` with no explicit protocol, when a confirmed Responses-capable
  model is selected, then effective protocol is `openai-responses` with visible non-configured
  provenance.
- Given `protocol = "openai-chat"`, then Chat Completions behavior is preserved exactly and no
  automatic Responses retry occurs.
- Given an unconfirmed OpenAI-compatible gateway, then default remains `openai-chat`.
- Given model-level catalog `responses`/`completions` signals, then the model-level protocol wins
  over provider defaults; ambiguous custom-model matches inherit nothing.
- Given text, ordered multi-image input, native function calls/results and reasoning settings, then
  `/v1/responses` request mapping is correct and stateless.
- Given recorded Responses streaming fixtures, then text, thinking summaries, parallel tool calls,
  usage and terminal state map correctly; unknown events do not panic; EOF without terminal event
  is an error.
- Given a gateway that rejects `/responses`, then Talos returns an actionable protocol error and
  does not reissue the request through another wire family.
- Old configs round-trip and retain behavior except the documented confirmed-support default change.

## Planned Validation

- Request mapping unit tests for text, ordered images, tools/tool results, reasoning, output limit and
  encrypted reasoning replay eligibility.
- Recorded SSE fixtures for normal output, parallel tools, usage, API errors, unknown events and
  missing terminal events.
- `proptest` coverage for stream/chunk parsing boundaries as required by repository protocol rules.
- Config round-trip, illegal-value text and protocol precedence/source matrix tests.
- Catalog mapping tests for provider/model `npm`/`shape` signals, exact/prefix/ambiguous/no custom
  model matching and explicit override.
- Mock HTTP end-to-end agent round including tool call and ADR-051 image continuation.
- Pinned locked focused checks during convergence, then `cargo test --workspace --locked` and
  `./scripts/release_preflight.sh` before candidate submission.

## Documentation

- ADR-093 and protocol/config reference docs.
- README / README.zh-CN provider configuration examples and migration note for explicit
  `openai-chat` rollback.
- Model/browser/debug surfaces describing protocol provenance.
- Release note for the public non-exhaustive enum and confirmed-support default behavior.

## Residual Destination

PROVIDER-008-B/C/D retain Gemini, Azure and Bedrock. Responses built-in tools, server-side state,
Realtime/Assistants/Batch and active probing remain backlog-only unless separately requested.

## Preflight — 2026-10-22

Target main at selection: `0caaf83241b8f94cba0cbdca66e3e166068f024d`.
Parent issue #724 and all then-current comments were re-read. Before #747 was opened, no open PR
matched #724 / PROVIDER-008 / OpenAI Responses and no matching `724` or `provider` implementation
branch was found. During claim preparation the bounded child requirement was split into #748 so the
parent epic and child owner do not share one Source Issue. PROVIDER-008 remains Refinement / Unclaimed
at the epic level; MODEL-014 remains Refinement / Unclaimed and does not authorize overlapping
capability inference.

Relevant current iteration disposition from target-branch Board/owners: I293/I294 retain independent
Search ownership; I300's structured Git claim remains separate; I302 remains Review on Windows
resource attribution; I305 is Complete/Closed; PROVIDER-007/I297 is Complete/Closed. No existing
active provider-protocol implementation owner was found. This Story deliberately claims only child A
so siblings B-D remain independently schedulable.

No implementation code, test, dependency, generated catalog or production configuration is part of
#747. This Claimed/Active record is only a proposal until the finalized exact head reaches main under
`docs/sop/AGENT-COLLABORATION.md`; implementation starts from that merge or a later main commit.

## Current OpenAI Contract Check — 2026-10-22

The OpenAI developer documentation was re-checked before implementation. Current stateless Responses
behavior returns reasoning `encrypted_content` by default with `store: false`; the legacy include
value remains compatibility-only. Function-call results remain `function_call_output` items bound by
`call_id`, and the core streaming lifecycle still includes `response.created`,
`response.output_text.delta`, `response.completed` and `error`. Recorded fixtures and implementation
must follow the current API contract rather than older examples.

## Execution Checkpoint — 2026-10-26

Governance #747 is effective at merge `4722fee827c71b807bb58458a08d67abe32050e4`; Draft implementation
#749 is active. The maintainer requested continuation after the previous session became overlong.
The same claimant and bounded child-A scope remain in force. Runtime/config resolver convergence,
interactive configuration, omission preservation, protocol provenance, reasoning-preview redaction,
binary regressions and property tests are recorded in the
[I306 continuation checkpoint](../../iterations/I306-provider008a-openai-responses.md#continuation-checkpoint--2026-10-26).

Status remains Active / Claimed. Catalog regeneration, complete validation, recorded-fixture audit,
current-head CI and independent API review remain acceptance gates. No completion or release is
claimed; the non-exhaustive public enum must ship only in a pre-1.0 minor release.

## Stream Lifecycle Follow-up — 2026-10-26

Rebased code CI3457 and stacked catalog CI3458 passed all six jobs, including full macOS preflight
and Windows workspace testing. Prior local failures remain recorded without a root-cause claim.
Review then found that the Responses HTTP reader waited for EOF after semantic completion/failure;
regressions reproduced the retained-connection timeout and duplicate terminal/framing error. The
correction and exact validation are recorded in I306. New code requires fresh exact-head CI.

Recorded upstream SSE capture (including parallel tools/reasoning/error paths), independent API
review and the distinct-endpoint residual remain open. Existing inline mocks and non-stream SDK
snapshots are not relabeled as captured upstream streams. Status stays Active / Claimed; no
completion, merge or release is claimed.
