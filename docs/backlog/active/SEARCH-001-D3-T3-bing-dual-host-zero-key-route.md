# SEARCH-001-D3-T3: Bing Non-CN-First, CN-Fallback Zero-Key HTML Route

**Status**: **T3a offline-only Complete / Closed upon owner-first closeout merge** — Completion Commit `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` (PR #959); **T3b production/live activation remains BLOCKED / Unclaimed**

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D3-T3 |
| Source Issue | [#957](https://github.com/wjhuang88/talos/issues/957) |
| Parent | [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md), [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Predecessors | D3-T1/#941 and D3-T2/#947 Complete/Closed; L/#901 retired `rust-websearch` |
| Selected iteration | [I329](../../iterations/I329-search-bing-dual-host-offline.md) — T3a Complete / Closed upon owner-first closeout merge; governance #958 effective |
| Product direction | Non-CN `www.bing.com/search` first, then existing `cn.bing.com/search` on allowed unavailability |
| Implemented by this claim | **T3a offline/test-only** decision policy and synthetic tests accepted/merged via PR #959; no newly authorized host, default route, or live query |
| Production activation | **T3b BLOCKED** on supplier/result rights, network authority, real independent region tests, and a later separately effective claim |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T3a / #957: offline pure endpoint-variant selection and fail-closed fallback classification, synthetic deterministic policy tests, bounded design receipt; no production path `www.bing.com` registration or live Bing requests. |
| Claimed At | 2026-11-16 |
| Source Issue | #957 |
| Governance Claim PR | #958 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance PR #958 accepted/merged `58207831675365e35cc80601097b96f9432a68aa` after exact-head [CI #4273](https://github.com/wjhuang88/talos/actions/runs/43805545051) 4 applicable SUCCESS/4 documentation-scoped SKIPPED, independent [no-blocker review](https://github.com/wjhuang88/talos/pull/958#issuecomment-6907540766) and head CAS. |
| Implementation PR | #959 merged as `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` (Completion Commit) |
| Last Updated | 2026-11-17 |
| Handoff / Release Condition | Offline T3a source delivered and I329 claim closed only by this owner-first closeout publication. Keep #957 Open for blocked/unclaimed T3b; do not imply deployed `www` routing or a vendor/access/output rights grant. |

## Product decision and boundary

The **Bing provider family** has *two variants*: preferred `https://www.bing.com/search` (non-CN) and fallback `https://cn.bing.com/search` (CN), with bare `bing.cn` explicitly **unadmitted**. This priority applies **inside Bing** only. The existing protected Auto provider-chain order DDG Lite → DDG Instant Answer → DDG HTML → Bing remains unchanged unless separately reviewed.

Reuse the existing Talos-owned Bing HTML parser (`li.b_algo`, `h2 a`, `.b_caption p`) and prepared HTTP transport. A real Browser/CDP executor may later be an alternative *page-acquisition transport* under WEB-007/#452; it is not needed for this offline policy and does not transform the two Bing hosts into two independent search providers. Do not use general `BrowserPageRecord.selected_links` as organic SERP results (negative #891).

## I329 T3a accepted work — offline candidate only

1. Introduce an internal, deterministic **pure** descriptor/decision adapter for the two Bing variants; non-CN is considered before CN, but no runtime Network manifest, existing request factory, transport/fallback behavior, Cargo or public schema changes are permitted.
2. Policy permits failover **only** for typed DNS resolution failure, connection establishment failure/refusal, or source-local timeout with a still-live invocation budget. Treat **all HTTP 5xx including 500/502/503 as no-fallback**, because a status response alone does not distinguish transient outage from server-side throttling; future evidence-based changes require their own policy review. Also **no fallback** for 403/429, challenges/CAPTCHA, access refusal, unsupported redirect/host, invalid/malformed source, authorization denial, cancellation/stale state, deadline or physical-request exhaustion. No cross-host throttle or challenge evasion.
3. A valid HTTP 200 with no organic results must not be mistaken for network failure; current Bing parser emits `Empty`, so candidate tests must explicitly characterize "valid empty" versus challenge/DOM drift rather than auto-select CN. If a case is indistinguishable, fail closed and document it.
4. Test with hand-authored local fixtures: preferred success/no fallback; temporary failure then CN; CN unavailable exhausted; 403/429/**all 5xx including 500/502/503**/challenge/redirect no fallback; denied permission; stale/cancelled; exact ordering; per-variant source identity remains **one** Bing family; query secrecy/normalization and no startup request. These are **synthetic**, not live Bing assertions.
5. Record how an eventual production variant would preserve one invocation's separately approved `www` and `cn` endpoint manifest, proxy/redirect per-hop authority, immutable prepared requests, one deadline, retry/backoff and accurately expanded request-budget bounds. No newly observed `www` DOM equivalence or mainland-China reliability claim.

## T3b later, explicit external acceptance blocker

Do **not** connect T3a to production or issue live automated Bing search requests until:

- Written, applicable Microsoft/rightsholder or qualified legal determination for distributed user-invoked automated Bing HTML *access*, and separate result URL/title/snippet/LLM-context/attribution/cache *use*. Current [2026-09-30 Microsoft Services Agreement](https://www.microsoft.com/en/servicesagreement) does not establish that permission for this design.
- A separate effective production implementation/activation claim and reviewed ADR-090 source, redirect, Network permission, proxy, privacy, cancellation and bounded physical-request-budget amendment; no silent reuse of old CN-only grant.
- Authorized actual observations on a **genuine mainland-China network** and distinct comparator, including effective host, redirect, challenge/429, valid query-level results and timing, preserving secrecy and complying with service restrictions.
- Independent security/API/network review and exact-head native CI; rollback preserves current CN-only chain and previous accepted behavior. Never count `www` + `cn` as two independent providers or unlock E/F/G/I automatically.

## Concurrent work and validation

Main has I328/STORE-002 E work, I324/GIT-COMPAT, Provider/TUI/Server owners. T3a touches only bounded Search source/test files after I329 claim; inspect actual changed paths before source PR, no cross-scope owner or Board status edits. Prior K alternative #861 is closed unmerged as superseded by #860.

After effective claim: `cargo fmt --all -- --check`, `cargo test --locked -p talos-tools --features network` focused policy tests, full workspace check/Clippy/tests, applicable native Windows/macOS/Linux CI, both governance validators, independent Security/API review bound to exact HEAD and merge-time CAS. A historical green SHA does not validate subsequent changed code.

**Acceptance criterion**: demonstrate correct deterministic failover policy and negative controls *offline*, then record actual source Completion Commit in owner-first closeout. No production code path is changed by this governance PR. Parent D3/#937 and Epic/#624 remain OPEN, prior D/#644 remains ZERO-ADMISSION.

## T3a implementation checkpoint — 2026-11-16

I329 was activated by governance-only [PR #958](https://github.com/wjhuang88/talos/pull/958), merged at `58207831675365e35cc80601097b96f9432a68aa` after exact-head CI #4273 success and independent no-blocker review. A separate implementation branch from that exact merge introduces a private, **unwired** `search_backend::bing_variants` policy candidate and hand-authored tests. This may be compiled/tested but is not permitted to change the protected SearchRouter or manifest, register `www.bing.com`, perform live requests, activate Browser/CDP or imply Microsoft service/output rights. The [offline contract/evidence note](../../reference/SEARCH-001-D3-T3-OFFLINE-DECISION-2026-11-16.md) pins those boundaries. At that source-stage checkpoint completion and CI were pending; **actual delivery and acceptance** are recorded in the 2026-11-17 owner-first closeout below.

## T3a owner-first source closeout — 2026-11-17

- **Completion Commit**: `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a`, existing on target `main`, created by squash merge of implementation [PR #959](https://github.com/wjhuang88/talos/pull/959). Source candidate exact HEAD `71d752880b6ede90c8e8bd89c5a87cc6f2217b9e`, against target main `6d5651df91c9c5fc20ee57517d1c43a1a2e0a338` at merge.
- **Authority**: separate I329 atomic claim [#958](https://github.com/wjhuang88/talos/pull/958) effective on `58207831675365e35cc80601097b96f9432a68aa`. Only the private, unconnected T3a variant/failover decision and seven synthetic offline unit tests were authorized or delivered.
- **Exact-head native validation**: [CI #4333 / run 44011362862](https://github.com/wjhuang88/talos/actions/runs/44011362862) **SUCCESS, 8/8 jobs** (native macOS fmt/check/clippy/tests, Windows workspace and aggregate, Linux/Windows Desktop, installer, change-scope and remote issue/owner reconciliation). Earlier exact-head [CI #4284](https://github.com/wjhuang88/talos/actions/runs/43830156419) had one Windows `talos-agent::checkpoint_history` lease-test failure in attempt 1 (648 passed/1 failed, untouched by the SEARCH PR), then passed on the same-head failed-job-only rerun; retain original failure and **do not** assert a proven flaky root cause.
- **Independent security/API/network review**: [Copilot formal technical APPROVE](https://github.com/wjhuang88/talos/pull/959#issuecomment-6941738278) for exact HEAD `71d752880b6ede90c8e8bd89c5a87cc6f2217b9e`, including later target-main changes, zero blockers, no source/Network/public API interaction. Reviewer is a separate technical reviewer, not legal/rightsholder or human-maintainer sign-off. Final owner/work-slice/overlap, exact-CI, approval and expected-head CAS passed; GitHub [PR #959](https://github.com/wjhuang88/talos/pull/959) returned merged=true and Completion Commit above.
- **Accepted behavior**: private/unwired `bing_variants` models Non-CN-first with only typed DNS, connect establishment/refusal or source-local timeout eligible for CN **and only with five positive guards**. All HTTP statuses including 403/429/5xx, valid-empty, challenge, access-denial, redirect, parser/encoding faults, cancellation, deadline and budget exhaustion are no-fallback. CN never rotates back. **Existing production** DDG Lite → DDG Instant Answer → DDG HTML → CN Bing chain and query recipients are unchanged; no newly distributed/authorized `www.bing.com` Network manifest, Browser/CDP path, new provider count or user query traffic.
- **Residual**: T3b remains **BLOCKED, unclaimed and unshipped** pending separate Microsoft automated SERP access **and** result/LLM-output-use determination, effective Network/ADR-090 host/redirect/proxy/budget authority, permitted actual mainland-China + independent comparator evidence, independent review and native CI. **#957 remains Open** for this residual; D3 parent #937 and Epic #624 remain Open; historical #644 ZERO-ADMISSION and E/F/G/I gates unchanged. I329 T3a may be recorded Complete / Closed only as this offline slice. No release activation is implied.
