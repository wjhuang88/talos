# PROVIDER-008-B: Native Google Gemini

**Status**: Complete / Closed
**Source Issue**: #810
**Parent Epic**: PROVIDER-008 / #724
**Selected Iteration**: I310

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-B native Gemini wire with AI Studio and Vertex authentication only |
| Claimed At | 2026-10-30 |
| Source Issue | #810 |
| Governance Claim PR | #811 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized completing parent #724; distinct human reviewer unavailable in this session. Exact-head checks, independent protocol/auth review and merge CAS required. |
| Implementation PR | #838 (merged) |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Implementation #838 merged after CI4046, independent technical/document review and merge CAS. No release/tag. |

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

Governance-only #811 proposes Claimed/Active atomically. Both remain ineffective until target
merge; the actual PR number is backfilled. Independent review and exact-head checks remain required.
Completion Commit: Pending. No code, dependency, catalog data or implementation tests in this PR.

## Current implementation checkpoint — 2026-11-02

Effective activation: #811 merge 80b9393375812475901b1b9565cbff6fca273537.
Implementation: Draft #838 on feat/provider008b-gemini, checkpoint 090ec083c158682d9e5bfa3f70a9581a46e93ee0.
Native transport/auth/request/SSE, signed replay, controlled real Agent image/tool rounds,
isolated decisions and request-owned PNG ingress are implemented on that branch, not delivered
on main. Focused Provider 263; catalog-contract Config/Core 268/123 pass. Workspace check,
Clippy and explicit Desktop UI feature check pass at the recorded prior stage. Full preflight
stopped on a Git executable-identity host fixture (586 Agent tests passed, one failed); unchanged
fixture passes with internal Git PATH, and full rerun is pending. See I310 for exact evidence
and retained failures; no final native CI or whole-candidate independent review result yet.

Remaining: browser original-argument/compatibility modes; confirmed-model reasoning settings;
positive lazy ADC/factory integration; host/UI/provenance and separately reviewed catalog data;
sourced protocol fixtures, final documentation, full workspace/preflight and exact-head native
CI / full independent protocol/auth review / Single-maintainer CAS. #132 tracks shared dynamic
authentication residuals; this child does not complete PROVIDER-003 by implication.
Status stays Active / Claimed. Completion Commit: Pending. Parent #724 remains open.

### Successor-session checkpoint — 2026-11-03

Continued the same effective #811 claim and Draft #838 from remote1eabe26f. CI3788's aggregate
and seven applicable native jobs passed; unrelated global matrix reconciliation failed on
#844/#390 and remains an ADR-071 advisory residual. No old-head pass is evidence for new code.
Ordinary Compat/TalosStrict tool rounds, signed-text-preserving continuation and request-local
images now have 282 passing pinned/locked Provider tests plus bounded independent Agent source
approval. Browser ingress remains refused; transient signed browser replay/privacy, final UI,
catalog activation, sourced fixtures and whole-candidate acceptance remain incomplete. Exact
stage evidence, retained cold-build failure and resume steps are in I310. Workspace check and
Clippy passed; full test compilation and the final duplicate-ID focused rerun remain pending.
No final workspace/preflight or new-head native CI pass is claimed. Delivery remains
Active / Claimed; Completion Commit: Pending. #724/#810 stay open; no release or merge.

Final compatibility source checkpoint5c2cde9c has focused Provider282/0 and all-target Clippy
passing. Standard preflight is **failed**, not passed: unchanged MCP socket fixtures return
EPERM; standalone AF_UNIX creation is denied too. A later Runtime socket fixture has the same
host failure. Remaining --no-fail-fast package, MCP and external SDK runs are still collecting
results; native CI and all full B acceptance gates remain required. See I310 for exact tests,
counts, preserved successes and resume instructions. This is a Draft stage, not B completion.

### Native browser replay checkpoint — 2026-11-04

Checkpoint 5349cd3c records native Gemini browser original-argument ingress and live signed
continuation. Mock HTTP tests cover mixed ordinary/browser execution, safe events and exports,
live continuation and durable reopen refusal. At ed1c5c8c, pinned locked library tests passed
Core 129, Agent 664 and Provider 284. Independent source review confirmed the three privacy
corrections recorded in I310. The fixture-only follow-up passed targeted all-target Clippy and
the five Agent replay tests. Full preflight and candidate CI are separate gates.
Browser Compat/TalosStrict, final UI/provenance, catalog data, sourced SSE fixtures and final
acceptance remain open. Status stays Active / Claimed. Completion Commit: Pending.


### Rebased candidate validation — 2026-11-11

Implementation candidate 9da0e8a84df400d4cfc4c7f16fc59f3b4fd21758 is based on
main a320b7a8f27c4206135507d7e006dc5c1a6169aa. CI4045 / run42942828514 completed
SUCCESS. Local pinned locked Gemini tests passed Config 5 / Provider 83, zero failures.
Offline locked Cargo metadata, frozen source/catalog audit (zero mismatches), and diff checks
passed. Prior CI4036 at 82a22a0b failed because the rebased petgraph lock entry lacked the
indexmap version qualifier. Cargo regenerated only that reference to indexmap 2.14.0; no package
version changed. Preserve the earlier failed CI and host-limited workspace results.

Independent Agent source review approved protocol/auth/API/privacy surfaces at exact 9da0e8a8
against a320b7a8. It traced factory/endpoint/lazy ADC/refresh/dispatch, SSE terminal/signature
association, sealed live custody/restoration/persistence and bounded catalog precedence, with no
blocking finding. This was source review under a shared repository identity, not a separate human
GitHub approval or independently rerun tests. Synchronous RSA cannot be preempted, private replay
requires live custody, and future SDK-only catalog drift requires refreeze review, as documented.

Native/Compat/TalosStrict browser replay, sourced SSE fixtures, UI credential/provenance and
bounded exact two-pair Flash catalog inference are implemented on the candidate. Architecture,
SDK, README en/zh and unreleased migration documentation are being synchronized. Full B remains
Active / Claimed with Completion Commit Pending until final documentation validation/review and
merge-time CAS. No merge, release, live-vendor success or parent #724 completion is claimed.


## Completion — 2026-11-11

Completion Commit: dbcf6163f34d73def923484ad409814fd4a98039

PR #838 merged the implementation and documentation after exact-head CI4046
(run42947754213, head836af8eebb3bec143b92f2c65bb068dbb65e0b37) completed SUCCESS
with all eight jobs passing. Both governance validators passed with zero warnings. Independent
protocol/auth/API/privacy source review approved implementation9da0e8a8; the subsequent eight
documents received bounded consistency review, including the private-marker-only reopen limit.
Single-maintainer authorization from #811 applied because a distinct human reviewer was unavailable.
Merge CAS confirmed the claimant/work slice, prerequisites, mergeability and absence of overlapping
Gemini PRs or unresolved review feedback. This is independent Agent review with shared account
identity, not a separate human GitHub approval.

All six acceptance rows are covered by native CI and recorded regressions. Both transports have
permission-gated ordered-image/tool Agent rounds. Signed parts, call/result IDs and privacy
projections remain bound. Malformed/safety/terminal/auth failures are explicit and bounded.
User/service-account ADC initialization, refresh, expiry and cache/destination constraints are
tested. Old configurations, explicit overrides, custom endpoints and UI provenance retain their
contracts. Exact Flash catalog/profile inference passed the frozen-source preservation audit.
README en/zh, protocol reference, architecture, SDK and unreleased migration notes are synchronized.

Historical failed runs above remain failed evidence. Mock HTTP and sourced SDK SSE fixtures do
not constitute live vendor verification. Synchronous RSA cannot be preempted; private replay
needs live custody and fails after reopen/fork when markers survive. Public API changes require
the next pre-1.0 minor release. No release/tag was performed. These documented product boundaries
do not transfer or close #132. Child A is Complete; Azure C and Bedrock D and parent #724 remain open.
