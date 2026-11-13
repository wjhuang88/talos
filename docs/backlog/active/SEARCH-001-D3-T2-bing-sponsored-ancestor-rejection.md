# SEARCH-001-D3-T2: Reject Explicit Sponsored Ancestors in Existing Bing Parser

**Status**: Ready / proposed In Progress — *ineffective until governance PR merges*
**Type**: Narrow search compatibility parser hardening / regression
**Source Issue**: [#947](https://github.com/wjhuang88/talos/issues/947)
**Parent**: [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md), [SEARCH-001 Epic / #624](SEARCH-001-zero-config-global-search.md)
**Selected Iteration**: [I327](../../iterations/I327-search-bing-sponsored-ancestor-rejection.md) — proposed only
**Predecessor**: [SEARCH-001-D3-T1 / #941](SEARCH-001-D3-T1-bing-synthetic-negative-conformance.md) Complete / Closed, implementation [#944](https://github.com/wjhuang88/talos/pull/944) Completion Commit `9cbc615cebb119ead9706a4d00159d7151639606`, closeout [#946](https://github.com/wjhuang88/talos/pull/946) merged `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed — proposed Claimed, ineffective until governance merge |
| Responsible Actor | @wjhuang88 (proposed) |
| Executing Agent | ChatGPT / GPT-6 (proposed) |
| Work Slice | Only existing `bing_parser.rs` response-body filtering of `li.b_algo` beneath explicit `b_ad` ancestor class + targeted `bing_parser/tests.rs` synthetic regression; optional dated narrow evidence receipt. No new host, transport/Network/default/API/Cargo changes or live Bing. |
| Claimed At | 2026-11-14 (proposed, no effective claim yet) |
| Source Issue | #947 |
| Governance Claim PR | Pending — backfill proposed PR number |
| Authorization Mode | Single-maintainer merge (proposed; requires exact-head CI, independent applicable review, CAS) |
| Authorization Evidence | User-directed continuation of completed #941; separate #947 issue. Proposed branch has no authority until governance review/CI and target-branch merge. |
| Implementation PR | Not started |
| Last Updated | 2026-11-14 |
| Handoff / Release Condition | Claim effective only upon governing PR merge; separately reviewed code/tests and exact-head native CI before implementation merge; owner-first completion thereafter. |

## Goal and evidence

The accepted [D3-T1 synthetic coverage receipt](../../reference/SEARCH-001-D3-T1-OFFLINE-COVERAGE-2026-11-13.md) and
`synthetic_sponsored_wrapper_with_b_algo_documents_known_false_positive_gap`
show that an artificial `<aside class="b_ad"><li class="b_algo">...`
is currently reported as a candidate. This is an **explicit sponsored
container** in a Talos-authored offline fixture, not a claim about all real
Bing ads or about `www.bing.com` / `bing.cn` DOM shapes.

This T2 corrects that *specific* false-positive class without changing the
existing Bing CN compatibility family's request authority, provenance,
ordering, payload bounds or public output contract.

## Selected source and acceptance

- Modify **only** `crates/talos-tools/src/search_backend/bing_parser.rs`
  (parser-local `b_ad` ancestor-class check before extracting result fields)
  and `crates/talos-tools/src/search_backend/bing_parser/tests.rs`
  (synthetic ad-only/mixed/nested/exact-token/limit/order regression).
- Use existing `scraper` dependency and no new feature, Cargo, external
  parser, retry, user agent, permission or API.
- Do not skip/ignore suppressed tests. Preserve `li.b_algo` organic
  structure, first `h2 a` and `.b_caption p`, redacted errors, status and
  challenge ordering, max results, bounding and existing compound
  `SearchBackendId::DuckDuckGo` compatibility source identity.
- A row inside an explicit `b_ad` **ancestor class token** should not
  become a candidate; an ad-only synthetic response should be Empty,
  while mixed organic rows retain output order. Do not invent extra
  ad-class heuristics or claim all sponsored results can be identified.
- Existing T1 known-gap test must be updated to a rejecting assertion; test
  positive and negative class token cases, nesting, mixed max cap and
  preserved non-ad parser behavior.

## Important non-goals

No live search, scraped fixtures, automated Bing traffic, `www.bing.com`
or bare `bing.cn` admission, source priority or fallback change, default
query path, Network grant, proxy/redirect handling, permission budget,
new search source/provider identity, Microsoft licensing conclusion, geo-IP
or mainland-China reachability assertion. D3/#937 stays **Intake /
Unclaimed**, D/#644 remains **BLOCKED / ZERO-ADMISSION**, E/F/G/I and
parent #624 remain gated/Open. A response-body improvement alone cannot
admit or qualify a new Bing route.

## Dependency, conflict, and validation plan

- D3.1 research #938 and D3-T1 #941 are accepted/Closed.
- I324/#919 Git preflight, I325/#670 storage, and other open/review owners
  remain independently claimed; this slice touches only the isolated
  search parser and its synthetic unit tests. No ownership transfer.
- Review `docs/sop/AGENT-COLLABORATION.md`,
  `START-ITERATION.md`, `CHANGE-CONTROL.md`, ADR-085/090,
  and SearchRouter's existing response/result admission invariants.
- After effective claim, run locked focused parser and compatibility
  tests, cargo fmt/check/clippy/test and full applicable native Windows,
  macOS and Linux CI; require independent read-only API/network/security
  review on exact implementation head and merge-time CAS.
- If `b_ad` ancestor classification is ambiguous with existing real
  wire fixtures or breaks prior parity, record a review blocker, do not
  expand heuristics or claim completed product organic filtering.
- Rollback: revert only parser-local exclusion and corresponding synthetic
  assertions. No migration, network setup or user configuration change.

## Evidence and completion status

I327 governance activation **not yet merged**. No implementation PR, native
source CI or Completion Commit exists. A governance-only PR is never proof
of implementation completion or SEARCH-001 provider admission.
