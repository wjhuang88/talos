# SEARCH-001-D3-T2: Reject Explicit Sponsored Ancestors in Existing Bing Parser

**Status**: Complete / Closed upon owner-first closeout publication — Implementation Completion Commit `5b2fa4358b83698ca4c0e517ce24a6dbba021d99` (PR #949)
**Type**: Narrow search compatibility parser hardening / regression
**Source Issue**: [#947](https://github.com/wjhuang88/talos/issues/947)
**Parent**: [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md), [SEARCH-001 Epic / #624](SEARCH-001-zero-config-global-search.md)
**Selected Iteration**: [I327](../../iterations/I327-search-bing-sponsored-ancestor-rejection.md) — Complete / Closed upon owner-first closeout publication
**Predecessor**: [SEARCH-001-D3-T1 / #941](SEARCH-001-D3-T1-bing-synthetic-negative-conformance.md) Complete / Closed, implementation [#944](https://github.com/wjhuang88/talos/pull/944) Completion Commit `9cbc615cebb119ead9706a4d00159d7151639606`, closeout [#946](https://github.com/wjhuang88/talos/pull/946) merged `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | Only existing `bing_parser.rs` response-body filtering of `li.b_algo` beneath explicit `b_ad` ancestor class + targeted `bing_parser/tests.rs` synthetic regression; optional dated narrow evidence receipt. No new host, transport/Network/default/API/Cargo changes or live Bing. |
| Claimed At | 2026-11-13 |
| Source Issue | #947 |
| Governance Claim PR | #948 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Atomic I327 claim+activation PR #948 merged `5704bb66aca22919347705a5bff40a84d641c3c9` after exact-head CI43462581361 SUCCESS and independent no-blocker review/CAS. Source PR #949 accepted on exact head `ee1265c35685481d8eb9e753cd962975bd153bc0`, CI43465355386 attempt 2 all 8 jobs SUCCESS, independent read-only parser/API/security review and expected-head CAS, merged `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`. |
| Implementation PR | #949 merged as `5b2fa4358b83698ca4c0e517ce24a6dbba021d99` |
| Last Updated | 2026-11-14 |
| Handoff / Release Condition | Bounded I327 implementation accepted; close Issue #947 only after this owner-first closeout merges. Separate D3/#937, Epic/#624, rights/region/provider gates remain OPEN/BLOCKED. |

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
  remain independently claimed; no owner Work Slice transfer. However,
  **open stale Draft [PR #861](https://github.com/wjhuang88/talos/pull/861)**
  also edits `bing_parser.rs` at its legacy `clean_html` import. Its
  head `eec87383ea14ab62f7b48546f81f437888473a14` is currently
  unmergeable against the updated main; its K/I317 source work has already
  been superseded by accepted #860/#870 and L/#901 retirement. This is a
  **real file-level overlap**, not the same sponsored-ancestor behavior claim.
  I327 implementation must start from then-current main, refresh #861 state,
  compare the precise overlapping hunk, avoid reverting/cherry-picking old
  helpers, and stop for explicit reconciliation/re-review if #861 moves
  into a conflicting state. No broad file ownership exclusivity is claimed.
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

Completion Commit: `5b2fa4358b83698ca4c0e517ce24a6dbba021d99` — actual source implementation [PR #949](https://github.com/wjhuang88/talos/pull/949), **not** this later docs-only closeout.

- **Effective claim:** docs [#948](https://github.com/wjhuang88/talos/pull/948) merged `5704bb66aca22919347705a5bff40a84d641c3c9` after native exact-head CI43462581361 and independent review/CAS.
- **Source correction:** [#949](https://github.com/wjhuang88/talos/pull/949) excludes `li.b_algo` only if an ancestor has an exact `b_ad` class token. Updates T1 known false-positive regression and adds three synthetic tests for nested/mixed ordering and max slots, exact token/ancestor vs substring/descendant, and skipped ad invalid fields. Three changed paths only (parser, its `#[cfg(test)]` tests, [dated receipt](../../reference/SEARCH-001-D3-T2-SPONSORED-ANCESTOR-2026-11-13.md)); no Cargo, transport, permission, new host or route/API change.
- **Exact-head native CI:** [CI43465355386](https://github.com/wjhuang88/talos/actions/runs/43465355386), attempt **2**, **8/8 SUCCESS** at head `ee1265c35685481d8eb9e753cd962975bd153bc0`. All Windows/Linux jobs and owner reconciliation succeeded on the first attempt. The first macOS `Release preflight` failed an **unrelated** `talos-session::durable::tests::fork_recovery_skips_active_lock_and_retires_only_unpublished_intent` (`durable.rs:2363`, expected 1 / observed 0); the same macOS failed job was rerun with **no changes or suppressed tests**, and succeeded. Preserve both facts; the first failure is not erased.
- **Independent source review:** [exact-head read-only review](https://github.com/wjhuang88/talos/pull/949#issuecomment-6859139961) found no parser/API/security blocker, confirmed scraper 0.27 ancestors/class token semantics and source-only boundary. Prior format-only failure on older source head was corrected before this CI.
- **Merge CAS and overlap:** source #949 merged using expected head `ee1265c35685481d8eb9e753cd962975bd153bc0` at `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`. Concurrent STORE-002 #935 main commit `027b8a59194f5ba496b0048c7183ad236e2e2dd2` touched unrelated files; no source overlap. Historical dirty Draft #861 still edits only an old import hunk, not the result loop; accepted `super::compat_helpers::clean_html` was preserved. #861 is **not** closed here.
- **Known limitations:** only explicitly marked `b_ad` ancestors are recognized; arbitrary ads or real Bing DOM accuracy were not proven. No live Bing traffic, `www.bing.com`/`bing.cn` admission, Microsoft usage rights, mainland China coverage, new independent source or E/F/G/I activation. D3/#937 and Epic/#624 stay **Open**, #644 ZERO-ADMISSION remains unchanged.
- **Owner-first final gate:** merge this documentation/status reconciliation PR after exact-head CI and independent review, then close only #947. Rollback is a localized source/test revert.
