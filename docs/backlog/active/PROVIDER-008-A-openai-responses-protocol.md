# PROVIDER-008-A: Protocol Architecture And OpenAI Responses

**Status**: Review / Claimed; implementation merged, acceptance residual remains
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
| Implementation PR | #749 (code), #775 (catalog data), #792 (model endpoint runtime) |
| Last Updated | 2026-10-28 |
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

## Independent Review Correction Stage — 2026-10-27

The three blockers from the independent bb31d012 review have working-source corrections: real
request-local tool image continuation with no automatic repost and expiry, explicit Compat/Strict
configuration-to-runtime preservation and shared text parsing, and parallel plus sourced SDK stream
replay. The new CLI scenarios use read_image without --attach and exercise both textual tool modes;
they do not substitute attachment history for a one-shot tool image. Upstream fixture hashes,
license and acquisition limits are explicit in the fixture README. Independent WIP re-review
approves those corrections; stable-head CI/review/publication remain pending. Details and retained
initial failures are in I306. Status stays Active / Claimed, with no completion or release claim.


## Merged Implementation / Acceptance Review — 2026-10-28

Code #749 merged as `60e27eb6653b3a8fef638a2b9a7294a036b78db0`; the separately reviewed
catalog #775 merged as `6af8654c6ee6e422d4d3f4e5dad171c78604cafd`. Claim #747 remains effective,
with the same Responsible Actor and Work Slice. This owner is now Review / Claimed.
Completion Commit: Pending. Merged implementation evidence does not by itself close acceptance.
Earlier Draft/Active checkpoints above are historical; the published scope and acceptance remain.

Code head `1388874d` passed CI3498 / run40510753909 (8/8), and final data head `76647990`
passed CI3509 / run40537545803 (8/8), including full release preflight, native Windows workspace,
Linux/Windows Desktop and both governance validators. Exact-head independent read-only Agent
technical review APPROVE covered provider/API corrections, source provenance, all 6,474 catalog
rows, final trees and relevant-base integration. This shared-account review is not a distinct
natural-person approval. Single-maintainer merge authorization and final CAS are recorded in each PR.

Acceptance evidence on the merged implementation:

| Acceptance area | Evidence / coverage |
|---|---|
| Protocol precedence, explicit Chat rollback, unconfirmed gateway, ambiguity and provenance | Config/resolver/request tests and packaged-catalog snapshot tests; explicit protocol remains authoritative. |
| Stateless text, ordered images, native tools and reasoning | Dedicated Responses mapping/stream tests plus the actual CLI binary scenarios in I306. |
| Request-local tool image continuation | Real read_image -> read -> final round sends the image once, without --attach; expiry and bounds are tested. |
| Native, Compat and TalosStrict tools | Four real CLI Agent scenarios cover native attachments, one-shot tool image, Compat and Strict; merged config retains tool_protocol. |
| Parallel tools, usage, terminal/error, unknown events and EOF | Synthetic interleaved streams, property tests and sourced byte-identical SDK recordings; fixture README preserves provenance and limitations. No live Talos endpoint capture is claimed. |
| Catalog source and non-protocol preservation | Frozen 2026-10-27 source plus exhaustive offline audit: 4,010 hints on 6,474 preserved models; no non-protocol mismatch. |

Still open: distinct model-endpoint routing is unimplemented. The catalog withholds all 56 such
records, alongside absent models, unsupported transports and provider-root drift, rather than
claiming their runtime support. This remains an acceptance/coverage residual under parent #724;
this checkpoint neither waives the requirement nor declares the multi-protocol-host goal complete.
Before Complete, resolve that residual through implementation or an explicit accepted scope decision,
then synchronize owner, indexes, Board and #748 with existing implementation evidence.
Gemini/Azure/Bedrock remain unclaimed siblings; MODEL-014 and active probing remain separate.
The next release must use the documented pre-1.0 minor migration for the public enum/default change;
no tag or release is authorized by this checkpoint.

Earlier local full-preflight MCP 26 passed / 21 failed and lost/interrupted validation records remain
historical failures; green native CI does not turn them into local successes. The original 2026-10-26
catalog input remains unavailable; the distinct archived 2026-10-27 source is the reviewed data evidence.


## HTTP Model Endpoint Correction Stage — 2026-10-28

The maintainer requested continued implementation of the remaining routing residual under the same
effective #747 / #748 / I306 Work Slice and Responsible Actor. This stage starts from main
`e03a2984267eacbd4f65d0085be1d028a8c420ed`, preserving inherited I308/I293 work. It adds explicit
`ModelConfig.base_url`, CLI get/set and effective endpoint/source resolution in the shared adapter
factory. Model connection overrides provider connection. Catalog endpoints require an exact built-in
provider/model, the unchanged catalog provider root and a matching effective wire; custom-provider
unique model matching never authorizes moving its credentials to the catalog vendor endpoint.
An explicit different wire suppresses the catalog wire's connection. Protocol inference is suppressed
when a model supplies an unconfirmed connection, just as for a replaced provider root.

The generator preserves supported HTTP model roots with their protocol. HTTPS or loopback HTTP is
validated by the existing locked `url` 2.5.8 parser (one additional talos-config dependency edge, no
crate/version change). Userinfo, query/fragment, malformed roots and remote cleartext HTTP are denied.
Environment templates remain literal at generation/config persistence and are validated after actual
runtime expansion. Missing environment variables fail before an HTTP request is built. Provider
construction errors propagate through CLI/RPC/TUI rather than panicking; failed TUI preparation must
preserve the old config, session and runtime generation. Desktop already returns construction errors.

The frozen source's 56 endpoint-different records comprise 52 supported HTTP candidates (14 Responses,
34 Messages, 4 Chat), including four Freemodel cross-origin defaults; four Alibaba records are WSS
Realtime transports, an existing explicit non-goal, not HTTP Chat endpoints. Endpoint and credential
destination metadata requires a separately reviewed data projection. This code stage does not change
`models.toml`, activate those rows, or claim runtime SDK authentication wrappers. The subsequent data
stage must prove compressed packaged catalog -> resolver -> actual adapter/binary routing, including
Neon environment roots, Freemodel origin changes, replaced-root suppression, custom-name non-relocation
and explicit wire rollback. Explicit model-URL tests alone do not close that automatic-catalog gate.

Independent WIP review found two blockers (userinfo loopback bypass and CLI construction panic);
source corrections and regression coverage were read-only re-reviewed without a new blocking finding.
Final exact-head object review, unfiltered affected checks, standard preflight and fresh CI remain
separate gates recorded in the carrier PR. One local CLI link attempt failed with an object-file
section-header offset beyond EOF, exit 101; no source root cause is asserted. Its log is retained,
and only generated target artifacts were cleaned. The complete affected-package rerun uses fixed
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`;
these flags change build artifacts, not test selection or assertions. No unfinished run is a pass.
Status stays Review / Claimed, Completion Commit: Pending; data and final acceptance remain open.
No sibling implementation, scope waiver or release is claimed.


## Reviewed Endpoint Data Candidate — 2026-10-28

Code carrier #792 (`f43cd990`) and the separately reviewed endpoint data stage continue the same
#747 claim. The frozen 2026-10-27 source admits 45 supported, vendor-reviewed HTTP endpoint pairs:
Neon 14 Responses, Ofox 10 Messages, Zenmux 17 Messages and Freemodel 4 Chat. The data candidate adds
only paired model `protocol` / `api_base_url` metadata and validates original non-routing identity,
prices, limits, capabilities, variants and provider records with the original semantic digest.
Total candidate hints are 4,055 (3,857 Chat / 65 Messages / 133 Responses).

The data activation stage also keeps a reviewed model's original vendor root on wire rollback when
no provider URL was persisted; it cannot leak that provider credential to an unrelated adapter default.
This exact-model safeguard does not change custom gateways or older records without API metadata.

Primary vendor root/authentication evidence, exact destination tuples, all 45 rows, hashes and actual
compressed-catalog-to-binary tests are recorded in
[the endpoint ledger](../../reference/PROVIDER-008-A-CATALOG-PROTOCOLS.md#reviewed-http-endpoint-projection-candidate--2026-10-28).
Code-first merge and fresh exact-head data validation/review/CAS remain required; candidate data is
not represented as activated before merge. Seven Copilot Messages endpoints remain unreviewed for
vendor authentication/transport compatibility. Four WSS endpoints are the already-excluded Realtime
family, not HTTP Chat. Neither distinction is a scope waiver or implementation claim.
Keep Review / Claimed with Completion Commit: Pending until the remaining acceptance residual is
resolved through implementation/confirmation or an explicit accepted scope decision. #724/#748 stay
open; no sibling implementation or release is claimed.


### Code-first Main Integration — 2026-10-28

Code PR #792 merged as `e53c61c7f0344308a5c5535bb1521fefebbacc70` after exact-head
CI3524 / run40628003993 passed all eight jobs and independent read-only Agent review approved.
The endpoint data branch integrates that actual merged main before first publication. The additional
main delta is #788's two Search backend files; no catalog/source regeneration or claim transfer is
included. Final combined-tree test, standard preflight, independent object review and fresh data-head
CI evidence are recorded in the data PR. Earlier failed local checks remain retained evidence.
Status stays Review / Claimed, Completion Commit: Pending; seven unreviewed Copilot endpoints remain
an acceptance residual. #724/#748 stay open, with no completion or release claim.
