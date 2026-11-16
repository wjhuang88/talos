# SEARCH-001-D3-T3: Bing Non-CN-First, CN-Fallback Zero-Key HTML Route

**Status**: Review / Proposed claim; offline candidate only — **no live or production endpoint admission**

| Field | Value |
|---|---|
| Story ID | SEARCH-001-D3-T3 |
| Source Issue | [#957](https://github.com/wjhuang88/talos/issues/957) |
| Parent | [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md), [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Predecessors | D3-T1/#941 and D3-T2/#947 Complete/Closed; L/#901 retired `rust-websearch` |
| Selected iteration | [I329](../../iterations/I329-search-bing-dual-host-offline.md) — proposed, ineffective until governance merge |
| Product direction | Non-CN `www.bing.com/search` first, then existing `cn.bing.com/search` on allowed unavailability |
| Implemented by this claim | **T3a offline/test-only** failover policy candidate; no newly authorized host, default route, or live query |
| Production activation | **T3b BLOCKED** on supplier/result rights, network authority, real independent region tests, and a later separately effective claim |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed — **proposed only until claim PR merges** |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T3a / #957: offline pure endpoint-variant selection and fail-closed fallback classification, synthetic deterministic policy tests, bounded design receipt; no production path `www.bing.com` registration or live Bing requests. |
| Claimed At | 2026-11-16 |
| Source Issue | #957 |
| Governance Claim PR | #958 (Draft / ineffective until merge) |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer direction on SEARCH-001: use Bing HTML web pages as zero-key candidate; within Bing prefer non-CN, fallback CN. Actual governance effectiveness requires exact-head CI, independent review, accepted merge/CAS. |
| Implementation PR | None; source must follow accepted claim |
| Last Updated | 2026-11-16 |
| Handoff / Release Condition | Do not start source before effective I329 claim. T3a closeout may describe only offline model acceptance, not product route. T3b remains blocked on additional rights/authority/region gates. |

## Product decision and boundary

The **Bing provider family** has *two variants*: preferred `https://www.bing.com/search` (non-CN) and fallback `https://cn.bing.com/search` (CN), with bare `bing.cn` explicitly **unadmitted**. This priority applies **inside Bing** only. The existing protected Auto provider-chain order DDG Lite → DDG Instant Answer → DDG HTML → Bing remains unchanged unless separately reviewed.

Reuse the existing Talos-owned Bing HTML parser (`li.b_algo`, `h2 a`, `.b_caption p`) and prepared HTTP transport. A real Browser/CDP executor may later be an alternative *page-acquisition transport* under WEB-007/#452; it is not needed for this offline policy and does not transform the two Bing hosts into two independent search providers. Do not use general `BrowserPageRecord.selected_links` as organic SERP results (negative #891).

## I329 T3a accepted work — offline candidate only

1. Introduce an internal, deterministic **pure** descriptor/decision adapter for the two Bing variants; non-CN is considered before CN, but no runtime Network manifest, existing request factory, transport/fallback behavior, Cargo or public schema changes are permitted.
2. Policy distinguishes genuine temporary *availability failures* (DNS, connect, source-local timeout; bounded 500/502 where independently identified as outage) from **no-fallback** events: 403/429, challenges/CAPTCHA, rate-limit-like 503, access refusal, unsupported redirect/host, invalid/malformed source, authorization denial, cancellation/stale state, deadline or physical-request exhaustion. No cross-host throttle or challenge evasion.
3. A valid HTTP 200 with no organic results must not be mistaken for network failure; current Bing parser emits `Empty`, so candidate tests must explicitly characterize "valid empty" versus challenge/DOM drift rather than auto-select CN. If a case is indistinguishable, fail closed and document it.
4. Test with hand-authored local fixtures: preferred success/no fallback; temporary failure then CN; CN unavailable exhausted; 403/429/challenge/redirect no fallback; denied permission; stale/cancelled; exact ordering; per-variant source identity remains **one** Bing family; query secrecy/normalization and no startup request. These are **synthetic**, not live Bing assertions.
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
