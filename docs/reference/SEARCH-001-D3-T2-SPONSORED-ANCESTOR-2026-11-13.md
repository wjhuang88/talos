# SEARCH-001-D3-T2 — Explicit Sponsored-Ancestor Bing Parser Correction (2026-11-13)

> Source Issue: [#947](https://github.com/wjhuang88/talos/issues/947);
> [owner](../backlog/active/SEARCH-001-D3-T2-bing-sponsored-ancestor-rejection.md) /
> [I327](../iterations/I327-search-bing-sponsored-ancestor-rejection.md).
> **Synthetic-only behavior correction; no real Bing data acquired.**
> **Accepted source**: [PR #949](https://github.com/wjhuang88/talos/pull/949), Completion Commit `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`; [exact-head CI43465355386](https://github.com/wjhuang88/talos/actions/runs/43465355386) attempt 2, 8/8 SUCCESS and [independent read-only review](https://github.com/wjhuang88/talos/pull/949#issuecomment-6859139961). No new endpoint/network authority was admitted.

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

These tests were accepted in source PR #949; exact-head CI43465355386 **attempt 2 completed 8/8 SUCCESS** after a first-attempt unrelated macOS `talos-session` recovery test failure (`durable.rs:2363`, expected 1 / observed 0). The failed macOS job was rerun at the **same SHA** with no changes, ignored tests or disabled gates. The separate first source-head rustfmt failure was corrected in the final reviewed head.

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
  while T2 edits only the result-loop body. At #949 merge, #861 was still
  open/dirty with no same-hunk conflict; final CAS preserved the accepted
  `compat_helpers` import. #861 was not closed or superseded by this work.

## Completion evidence

**Implementation Completion Commit: `5b2fa4358b83698ca4c0e517ce24a6dbba021d99`** — actual [source PR #949](https://github.com/wjhuang88/talos/pull/949), not later documentation. Final accepted source head `ee1265c35685481d8eb9e753cd962975bd153bc0`, CI43465355386 attempt 2 all 8 jobs SUCCESS, independent exact-head no-blocker parser/API/security review and expected-head CAS merge. The initial failed macOS attempt and prior rustfmt-only failure remain documented as historical evidence; neither was suppressed. Owner-first closure must still be published before closing #947. Parent D3/#937 and Epic #624 remain Open.
