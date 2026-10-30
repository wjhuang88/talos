# PROVIDER-008-B: Native Google Gemini

**Status**: Ready / Unclaimed
**Source Issue**: #810
**Parent Epic**: PROVIDER-008 / #724
**Selected Iteration**: I310

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-B native Gemini wire with AI Studio and Vertex authentication only |
| Claimed At | 2026-10-30 |
| Source Issue | #810 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized completing parent #724; distinct human reviewer unavailable in this session. Exact-head checks, independent protocol/auth review and merge CAS required. |
| Implementation PR | Not started |
| Last Updated | 2026-10-30 |
| Handoff / Release Condition | Claim and activation are ineffective until finalized governance PR merges; no implementation before then. |

## Scope and deliverable

One stateless native Gemini Agent round over streamGenerateContent, with text, ordered multiple
images, parallel/sequential tools and results, reasoning summaries/signatures, usage and terminal
errors. AI Studio API-key and standard Vertex project/location OAuth are distinct transport
profiles of the same adapter. Preserve existing Chat/Messages/Responses and centralized protocol
selection/provenance. Catalog activation is separately reviewed from runtime implementation.

## Acceptance

- Given either configured transport, a real permission-gated mock HTTP Agent completes image/tool
  turns using Gemini contents/parts and correct authentication, without cross-wire retries.
- Given signed model parts, preserve opaque signatures at their original positions; keep real
  function-call IDs when supplied, generate unambiguous internal IDs only when absent, correlate
  results without relying only on function name, and prevent readable secret/signature disclosure.
- Given safety blocking, malformed/unknown events, missing terminal or invalid credentials, return
  bounded explicit outcomes without panic, silent success or credential relocation.
- Given expired Vertex credentials, refresh through the reviewed credential provider; service-account
  and local ADC flows must work. A static externally supplied token alone cannot close this scope.
- Given old configs, custom gateways and explicit protocol overrides, retain behavior and expose
  effective protocol/source through picker/browser/debug surfaces.
- Given frozen reviewed catalog metadata, select only confirmed Gemini protocol/transport pairs;
  preserve non-routing fields and suppress inference for replaced/custom endpoints.

## Validation and documentation

Pinned Rust with --locked; request tests include two distinguishable images in order, tools/results,
reasoning and output limits. Sourced recorded SSE fixtures with license/hash/provenance; proptest
byte/UTF-8/chunk/EOF boundaries; parallel tool association; config precedence/round-trip and
catalog matrices; real Agent image/tool rounds; credential expiry/refresh/redaction/destination
isolation tests. Standard ./scripts/release_preflight.sh, native exact-head CI, independent
protocol/auth security review and merge CAS. Update README en/zh, protocol reference, architecture,
SDK contract, unreleased notes and owner/Board/index/Issue. No live credentials in tests.

## Non-goals

Interactions API or server-side sessions, grounding/caching billing, built-in vendor tools,
Azure/Bedrock, active endpoint probing, general capability inference, release/tag. Vertex express
API-key mode is not silently substituted for standard OAuth. No shelling out to gcloud at runtime.

## Dependencies and decisions

Child A Complete (#806 merge5d366db2fbb63301cdce83369fc63fedd1f28c5a) supplies resolver/factory.
ADR-094 defines Gemini boundary; ADR-013/021/034/050/051/062/075 remain constraints. Auth library
version/features and transitive native dependencies require explicit audit before any dependency
commit; no dependency is introduced by this governance candidate.

## Iteration inventory and disposition — 2026-10-30

Inventory of all nonterminal iteration status headers on main d15c7688:
I249 Planned: defer dependency pilot, separate scope; I277 Review: retain deferred human Desktop
acceptance; I293 and I294 Active: retain Search owners, no overlap (open #808); I302 Review:
retain Windows attribution evidence; I308 Active: retain permission/MCP owner, no overlap.
I306 Complete/Closed is the satisfied prerequisite; I307/I309 are not nonterminal candidates.
No existing Gemini owner or overlapping implementation PR was found. Parent #724 stays open;
C Azure and D Bedrock remain unclaimed. This inventory does not transfer those owners.

## Execution / completion

Governance preparation only. Draft record is Unclaimed until actual PR-number backfill and
reviewable atomic Claimed/Active proposal. Both remain ineffective until target merge.
Completion Commit: Pending. No code, dependency, catalog data or implementation tests in this PR.
