# SEARCH-001-D3-T3a / I329 — Offline Bing Variant Decision (2026-11-16)

**Evidence class**: synthetic source-policy contract, **not** live Bing SERP evidence,
network permission, vendor license/usage grant, regional measurement or production
source admission.

## Provenance and effective authority

- Parent [SEARCH-001/#624](https://github.com/wjhuang88/talos/issues/624);
  research [D3/#937](https://github.com/wjhuang88/talos/issues/937);
  implementation child [D3-T3/#957](https://github.com/wjhuang88/talos/issues/957).
- Governed I329 claim [PR #958](https://github.com/wjhuang88/talos/pull/958)
  merged at **`58207831675365e35cc80601097b96f9432a68aa`** after
  [CI #4273](https://github.com/wjhuang88/talos/actions/runs/43805545051)
  all four applicable jobs SUCCESS, four code-scope jobs SKIPPED,
  and [independent exact-head no-blocker review](https://github.com/wjhuang88/talos/pull/958#issuecomment-6907540766);
  merge executed with expected-head CAS.
- Separate implementation branch
  `feat/i329-search-bing-offline-failover` starts at that governance merge.
  Implementation CI, independent code review, CAS and source Completion Commit
  **PENDING**. No claim of tested source before CI.

## Code seam

New private `crates/talos-tools/src/search_backend/bing_variants.rs`, registered
as a standalone private module in `search_backend.rs`. It is **not called**
by `production.rs`, `SearchRouter`, `PreparedCompatibilityChain`, HTTP
request preparation, BrowserPageConnector, native Browser/CDP, or any Network
permission evaluator. Static query-free host labels do **not** create a
`ToolNetworkEndpoint`, a manifest, an approved redirect, an external query,
a new `SearchBackendId` or a release-ready default.

- `BingVariant::NonCn` is the first hypothetical Bing-family option,
  `www.bing.com/search`, followed by `BingVariant::Cn` with existing
  `cn.bing.com/search`. Neither bare `bing.cn` nor another provider exists
  in the contract.
- `BingAvailabilityFailure` distinguishes DNS, connection establishment,
  connection refused and source-local timeout **only**; a real caller would
  need a separately reviewed error taxonomy and execution deadline before
  mapping into this helper.
- `BingFallbackGuards` requires a **preprepared CN request**, explicit
  Network permission, prepared proxy route, live invocation and remaining
  physical-send budget. Default guards are all false.
- `next_variant` returns `TryCn` only when the preferred NonCn attempt
  reports one of those four typed local availability failures and all guards
  are satisfied. There is exactly one potential fallback step, never a
  reverse CN → non-CN or a host rotation loop.
- Organic results always win without another query; valid HTTP 200
  with no organic results stops and never equates to unreachability.
- Every HTTP status, explicitly **all 5xx including 500/502/503**,
  403/429, challenge/CAPTCHA, access denial, redirect mismatch,
  malformed/encoding/parser drift, permission denial, cancellation,
  overall deadline, and exhausted budget is terminal. No rate or
  access-control bypass by switching hosts.

## Synthetic negative tests (not genuine SERP)

| Fixture class | Expected decision |
|---|---|
| First variant and exact two query-free host labels | NonCn preferred, Cn only fallback candidate |
| Organic NonCn or Cn | Complete, no further query |
| DNS/connection/refused/source-local timeout on NonCn with all guards | TryCn |
| Any corresponding failure on Cn | Stop, no third try |
| Missing preprepared CN, Network grant, proxy route, live clock or budget | Stop |
| HTTP 200 status, valid 200 empty, 302, 403, 429, 500/502/503/504 | Stop |
| Rate limited, challenge, access denial, unknown redirect, malformed/DOM drift, encoding | Stop |
| Cancelled, overall expired, physical budget exhausted | Stop |
| Default guard with source timeout | Stop |

The tests contain **no live network call, query text, fixture acquired from
Bing or simulated CAPTCHA bypass**. The private policy itself cannot
establish rights, reachability, SSR compatibility or the user country.

## Later T3b activation gate (unchanged)

`www.bing.com` must **not** be added to the production manifest or shipped
as default until separately determined Microsoft service-access **and**
URL/title/snippet/LLM-context/attribution/cache rights, an effective T3b
implementation claim, ADR-090 per-hop prepared authority and Network approval,
proxy/redirect/privacy/budget/cancellation review, and permitted true
mainland-China + independent comparison-network evidence. The existing
DDG-first overall compatibility sequence and CN-only Bing request are
retained. Two Bing entrypoints are **one logical source family**, not two
independent search providers; E/F/G/I remain blocked. Parent #937 and Epic
#624 stay open; #644 historical ZERO-ADMISSION is not reversed.

## Verification status

The source PR must show focused `bing_variants` test results, locked
workspace validation, exact-head native Linux/macOS/Windows CI, independent
API/Network/security review, and merge-time expected-head CAS **before** an
implementation Completion Commit may be recorded. This research receipt is
not itself verification.
