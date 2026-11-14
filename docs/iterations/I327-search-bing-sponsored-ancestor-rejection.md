# Iteration I327: Existing Bing Parser Explicit Sponsored Ancestor Rejection

> **Status**: Complete / Closed upon owner-first closeout merge; source PR #949 already merged
> Published: 2026-11-13
> Source: [SEARCH-001-D3-T2 / #947](../backlog/active/SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md).
> Baseline: D3-T1/#941 Complete/Closed, implementation #944 `9cbc615cebb119ead9706a4d00159d7151639606`, owner-first closeout #946 `73ec650b6ef9804d0c9c42a6f1c5e7c9d9f0baa7`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-D3-T2/#947: strictly bounded response-only `b_ad` ancestor exclusion in existing Bing parser and owned synthetic parser tests; no network/request/route/provider change. |
| Claimed At | 2026-11-13 |
| Source Issue | #947 |
| Governance Claim PR | #948 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Effective governance #948 merged `5704bb66aca22919347705a5bff40a84d641c3c9` after exact-head CI43462581361, review/CAS; implementation #949 at head `ee1265c35685481d8eb9e753cd962975bd153bc0` merged as `5b2fa4358b83698ca4c0e517ce24a6dbba021d99` after CI43465355386 attempt 2, 8/8 SUCCESS, independent source review and CAS. |
| Implementation PR | #949 merged as `5b2fa4358b83698ca4c0e517ce24a6dbba021d99` |
| Last Updated | 2026-11-14 |
| Handoff / Release Condition | Source test/correction delivered; release I327 claim through this owner-first closeout merge, then close only #947. Parent research #937 and #624 remain Open. |

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

I327 was activated with [claim #948](https://github.com/wjhuang88/talos/pull/948) (merged `5704bb66aca22919347705a5bff40a84d641c3c9`) before implementation. **Completion Commit: `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`**, the actual [source PR #949](https://github.com/wjhuang88/talos/pull/949) merge, **not** this documentation closeout.

- Source head `ee1265c35685481d8eb9e753cd962975bd153bc0` passed native [CI43465355386](https://github.com/wjhuang88/talos/actions/runs/43465355386), attempt **2**, all **8/8 SUCCESS** including Windows/Linux, macOS full preflight, installer and remote issue/owner reconciliation.
- Earlier attempt 1 failed in unrelated `talos-session` `durable.rs:2363` recovery test (314 passed / 1 failed within crate); that single failed macOS job was rerun against the **same exact SHA**, without source changes or test suppression, and passed. Separate earlier head failed rustfmt before the final corrected head.
- [Independent exact-head source review](https://github.com/wjhuang88/talos/pull/949#issuecomment-6859139961) found no blockers for scraper 0.27 ancestor/class-token filter, synthetic HTML5 cases and unchanged request, permission, backend ID and parser bounds.
- Merge-time CAS checked latest main had nonoverlapping STORE-002 #935 only; stale open #861 is a different old import hunk and no competing sponsored-ancestor claim. Squash merge preserved `compat_helpers::clean_html`.
- Accepted behavior: skip candidates with **explicit `b_ad` ancestor token** before field extraction; matching ads-only => Empty, organic order/max/dedup/ID preserved. No real Bing acquisition or broad organic-quality guarantee.
- **Residuals:** real Bing DOM, other ad shapes, Microsoft product/result rights, regional reachability, additional host admission, independent provider and E/F/G/I remain BLOCKED/UNKNOWN; D/#644 ZERO-ADMISSION, D3/#937 and Epic #624 remain unchanged.

## Variance And Retrospective

One format-only fix was necessary; a subsequent unrelated `talos-session` macOS test failed intermittently on CI attempt 1 and passed when rerun unchanged on attempt 2. Keeping the failure record avoids mistaking a green rerun for proof against intermittent recovery-test risk. No session code was changed under search ownership. Narrow ancestor-only filtering minimizes compatibility drift and leaves wider sponsor recognition explicitly unresolved.
