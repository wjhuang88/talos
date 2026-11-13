# SEARCH-001-D3-T1 — Synthetic Negative Conformance Evidence (2026-11-13)

> [SEARCH-001-D3-T1 / #941](../backlog/active/SEARCH-001-D3-T1-bing-synthetic-negative-conformance.md)
> and [I326](../iterations/I326-search-bing-synthetic-negative-conformance.md).
> **Scope: Talos-authored synthetic tests ONLY.**
> This is a code-to-test inventory, not a real Bing SERP acquisition,
> production change, China-region measurement or a Microsoft use-rights approval.
> Results and exact-head CI must be filled from actual validated test executions
> before acceptance. D/#644 ZERO-ADMISSION and parent #624 remain unchanged.

## Pre-existing code/test truth — do not duplicate

| Source test | Existing coverage | D3 BF reference |
|---|---|---|
| `bing_parser/tests.rs::selectors_cleanup_title_fallback_and_compound_family_identity_are_preserved` | `li.b_algo`, first `h2 a`, `.b_caption p`, text cleanup, compound-family ID | BF-01 / BF-13 |
| `bing_parser/tests.rs::status_and_raw_challenge_markers_keep_upstream_classification` | 403, 429, 503, raw challenge markers | BF-03 / BF-04 |
| `bing_parser/tests.rs::empty_decoy_and_malformed_synthetic_documents_have_closed_outcomes` | HTML without organic candidates => Empty | BF-10 |
| `bing_parser/tests.rs::input_and_requested_count_bounds_are_checked_before_dom_work` and `field_limits_reject_whole_response_including_raw_href_before_trim` | response/field bounds and ordered fail-closed admission | BF-05 |
| `bing_parser/tests.rs::encoding_and_debug_errors_are_content_free` | invalid UTF-8/BOM and redacted errors | BF-05 / BF-08 |
| `transport/compatibility/chain/tests.rs::all_payload_validation_precedes_any_draw_including_late_get_bounds_and_scope` | frozen four fixed destinations and prepared failure before request/header draw | BF-08 / BF-12 |
| `transport/tests/headers.rs::frozen_headers_do_not_authorize_an_unscoped_redirect_target` | synthetic unauthorized redirect cannot receive request | BF-07 / BF-12 |
| `transport/compatibility/chain/legacy_parity_tests.rs` | Existing CN-only compatible request/fallback parity and provenance | BF-13 |
| `transport/compatibility/chain/reachability_tests.rs::both_unreachable_families_send_no_query_bearing_request` | No request if both existing families unreachable | BF-09 |

## Targeted new `#[cfg(test)]` cases

| Test | Synthetic fixture / assertion | Boundary |
|---|---|---|
| `synthetic_sponsored_containers_without_organic_rows_do_not_yield_results` | `aside.b_ad` links without `li.b_algo` return Empty; mixed ad and organic yields only selected structural organic row | BF-02 / BF-10; **not** real ad classification |
| `synthetic_sponsored_wrapper_with_b_algo_documents_known_false_positive_gap` | Artificial `b_ad` wrapper containing `li.b_algo` **currently produces a candidate** | BF-02 known parser false-positive risk; not a production fix or evidence of truly organic output |
| `synthetic_locales_do_not_hide_unverified_result_dom_selector_drift` | Mock zh-CN/en-US labels keep the existing structure; alternate manually invented selector returns Empty | BF-06; actual www.bing.com DOM remains **UNKNOWN** |
| `new_bing_host_candidates_are_not_admitted_by_existing_fixed_chain` | Construct candidate URIs in memory and show fixed manifest contains only existing `cn.bing.com` query authority; no outbound request | BF-07/BF-08 negative host authorization |
| `query_free_bing_reachability_does_not_turn_a_failed_query_into_success` | Simulated query-free `Reachability { bing: true }` but loopback query path HTTP 403 → error, never a valid result | BF-09; no new probe/health algorithm |

## Residuals / explicitly not proven

- **Parser false organic positive:** an ad wrapper containing `li.b_algo`
  is selected as a candidate by the current structure-only parser. The
  new test intentionally *documents* this behavior; filtering actual
  sponsored results would require separately governed production behavior,
  provider-use rights and authorized real fixtures. No security/quality
  claim is made for arbitrary Bing DOM.
- New `www.bing.com` and bare `bing.cn` endpoints remain **not
  admitted**, actual redirects and templates **unverified**. Two Bing
  surfaces are one provider family; no second native independent source.
- No dynamic endpoint-health selection or startup probe; BF-11 is a
  future design proposal, not implemented or tested.
- No actual mainland-China execution region or Bing result source proof,
  no Microsoft permission to embed or redistribute snippets or query,
  and no new live automated Bing request.
- Existing test-only loopback sockets are local fixtures, not service
  observations. Exact-head CI results, platform coverage, independent
  review and final status are **pending** at the initial stage.
