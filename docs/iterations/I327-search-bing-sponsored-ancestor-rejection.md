# Iteration I327: Existing Bing Parser Explicit Sponsored Ancestor Rejection

> **Status**: Active / Claimed proposed via Draft #948; ineffective until governance merge
> Published: 2026-11-13
> Source: [SEARCH-001-D3-T2 / #947](../backlog/active/SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md).
> Baseline: D3-T1/#941 Complete/Closed, implementation #944 `9cbc615cebb119ead9706a4d00159d7151639606`, owner-first closeout #946 `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-D3-T2/#947: strictly bounded response-only `b_ad` ancestor exclusion in existing Bing parser and owned synthetic parser tests; no network/request/route/provider change. |
| Claimed At | 2026-11-13 |
| Source Issue | #947 |
| Governance Claim PR | #948 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User-directed D3 continuation after closed T1 and source Issue #947; #948 is proposed atomic claim with exact-head CI, independent review and CAS pending before code. |
| Implementation PR | Not started |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | Claim effective only when main incorporates finalized I327 governance. Source completion and owner-first closeout are separately required. |

## Selected Story and Work Slice

| Story | Dependency | Status at selection | MVP |
|---|---|---|---|
| [D3-T2/#947](../backlog/active/SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md) | T1/#941 Complete/Closed, parent D3 research open | Ready — proposed Active after claim merge | Ignore candidates nested inside explicit `b_ad` ancestor while preserving standard Bing CN compatibility parsing and bounds |

### Output acceptance

1. `<aside class="b_ad">...<li class="b_algo">...</li>...</aside>`
   synthetic response returns `BingParseError::Empty` if no ordinary
   result exists, rather than the prior T1 false-positive.
2. Mixed sponsored and ordinary result structures keep original ordinary
   result order, dedup, bounded max_results; skipped ads do not consume
   result slots, and an unrelated class token is not filtered.
3. Nested explicit sponsored structures are excluded. No real Bing
   response template or general ad-detection claim is made.
4. Status/challenge/encoding/limit, compatibility attribution and existing
   locked parser and chain tests remain accepted with no output schema
   change, extra feature or dependencies.
5. Source PR gets focused and exact-head applicable native CI and
   independent API/network/security review, merge-time expected-head CAS.

## Non-goals / risks

- No new search host, permission, redirect, default route, fetch, relay,
  provider count, real Bing request, API key, proxy, health check or
  mainland-China product acceptance.
- No licensed redistribution or Microsoft product-use rights inference.
- This T2 closes only the **explicitly marked sponsored-wrapper** failure
  mode; differently shaped ad containers and actual vendor DOM remain
  UNKNOWN, requiring separate authorization/evidence for broad coverage.
- If selector ancestry correction would violate prior parser semantics or
  produce ambiguity, record as Review/Blocked with concrete fixture.
- Rollback is localized parser/test revert, not a production route switch.

## Native validation

After claim: `cargo fmt --all --check`; `cargo test -p talos-tools
--features network --locked bing_parser`; relevant compatibility chain
regressions; `cargo check --workspace --locked`; `cargo clippy
--workspace --locked -- -D warnings`; `cargo test --workspace --locked`,
and applicable Linux/macOS/Windows workflows. Record actual commands,
results and exact-head CI, not planned commands as executed facts.
Both governance validators, independent security/API/parser review and
merge-time CAS required. No actual Bing network request.

## Parallel work and ownership

I324 gix-native Git 2.36 preflight, I325 storage delegation, current
Provider/Desktop/TUI owners and parent D3 research remain separate.
**Open stale Draft [PR #861](https://github.com/wjhuang88/talos/pull/861)
(I317/K)** is a known file-level overlap: it changes the
`bing_parser.rs` `clean_html` import on an old, currently unmergeable
head `eec87383ea14ab62f7b48546f81f437888473a14`. K/#850
was completed via #860/#870, and L/#901 later retired the legacy dependency.
The behavioral Work Slices are distinct, but they share a file. **Before
touching or merging parser source**, refresh main and #861 status, compare
old and current import hunk, preserve currently accepted helper identity,
and halt/re-review on any actual implementation conflict or newly active
same-scope claim. Do not silently override #861 or assert zero overlaps.
I327 remains a narrow distinct behavior claim, not D2/#848, D/#644,
E/F/G/I or parent #624 product acceptance.

## Activation / Completion evidence

This iteration is a **governance proposal**; no implementation or test
source changes before an effective owner and iteration record on main.
Completion Commit: **Pending** (must refer to a future real merged source
commit, never the governance PR).
