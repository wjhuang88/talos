# SEARCH-001-D3-T2 — Explicit Sponsored-Ancestor Bing Parser Correction (2026-11-13)

> Source Issue: [#947](https://github.com/wjhuang88/talos/issues/947);
> [owner](../backlog/active/SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md) /
> [I327](../iterations/I327-search-bing-sponsored-ancestor-rejection.md).
> **Synthetic-only behavior correction; no real Bing data acquired.**
> Source PR/CI/independent review and implementation Completion Commit pending
> at this initial evidence packet's publication.

## Origin and scope

D3-T1/#941 Complete/Closed exposed a **specific** known false-positive
in the existing **compound Bing CN compatibility response parser**:
`synthetic_sponsored_wrapper_with_b_algo_documents_known_false_positive_gap`
previously returned a synthetic `li.b_algo` candidate nested within
`<aside class="b_ad">`. This did not prove a live Bing SERP template.

D3-T2 makes only a narrow response-parsing change: before extracting a
candidate's fields, inspect the candidate's DOM ancestor chain, skip it
if any ancestor has an **exact `b_ad` CSS class token**. No substring,
URL/title/snippet heuristics or classifier model are introduced. The
current selected `li.b_algo`, `h2 a`, `.b_caption p` structure
and normal candidate pipeline remain unchanged.

## Intended native proof — hand-authored fixtures only

| Test | Required observed outcome | Scope |
|---|---|---|
| `synthetic_sponsored_wrapper_with_b_algo_is_rejected` | Explicit `aside.b_ad` wrapping matching `li.b_algo` yields `BingParseError::Empty` if no other candidates | T1 known false-positive regression |
| `synthetic_nested_sponsored_rows_do_not_consume_result_slots` | Nested explicit `b_ad` is skipped, two ordinary entries retain order, snippet, compound family identity, and max-results=1 returns the first organic entry | Bounded source-only behavior |
| `synthetic_sponsored_class_matching_is_exact_and_ancestor_only` | Similar but nonmatching `b_advertisement` is not excluded; a `b_ad` decoration **descendant** within an ordinary row does not exclude its parent | Avoid overly broad substring filtering |
| `synthetic_sponsored_rows_skip_invalid_fields_before_bounds_check` | Oversized raw ad-only href is skipped **before** any field extraction; an accompanying valid ordinary result succeeds | No exposure to excluded fields |
| Pre-existing `bing_parser/tests.rs` cases | Status/challenge, BOM/UTF-8, input/field/result limits, dedup, ordered title/snippet selection unchanged | Regression parity |
| Existing prepared compatibility chain tests | No change in query receiver, host/redirect permission or compound source identity | Authorization parity |

These entries are an **intended test matrix**, not a declaration that CI
passed; acceptance requires actual locked focused tests, cross-platform
native CI and independent source review on the exact implementation head.

## Explicit residual limitations

- **Not a general sponsored-content classifier**: ads lacking an explicit
  `b_ad` ancestor marker can still match `li.b_algo` and be returned.
  No known 2026 Bing HTML snapshot was scraped or evaluated.
- `b_ad` class-token membership is the *only* new exclusion; this may
  also exclude a legitimate row inside a wrapper marked sponsored.
- `www.bing.com` and `bing.cn` are not approved query recipients.
  The existing frozen `cn.bing.com` compatibility path, Network
  authorization, cancellation, proxy and budget are unchanged.
- Bing-family request sources do **not** become two independent native
  zero-key providers; D/#644 ZERO-ADMISSION and E/F/G/I remain gated.
- Microsoft distributed-product usage/result rights and real mainland
  China / independent-network reachability remain BLOCKED / UNKNOWN.
  This source-only correction does not authorize live automated access.
- Open stale Draft [PR #861](https://github.com/wjhuang88/talos/pull/861)
  modifies the same parser source **import** on an old unmergeable head,
  while T2 edits only the result-loop body. Before merge, recheck #861
  and current main, refuse to replace accepted `compat_helpers`
  import, and stop on actual same-hunk conflict or newly active claim.

## Completion evidence

**Pending** source PR, exact-head CI, independent review, merge-time CAS
and actual implementation Completion Commit. Final owner-first closeout
must distinguish the real source commit from any later docs-only
governance merge. Parent research D3/#937 and Epic #624 remain Open.
