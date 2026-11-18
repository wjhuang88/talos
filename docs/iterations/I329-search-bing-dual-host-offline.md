# Iteration I329: Bing Non-CN-First/CN-Fallback Offline Contract

> Document status: Complete / Closed upon owner-first closeout merge for I329 T3a offline-only; T3b BLOCKED / Unclaimed. Claim #958 merged `58207831675365e35cc80601097b96f9432a68aa`; source Completion Commit `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` (PR #959).
> Selected: 2026-11-16
> Story: [SEARCH-001-D3-T3 / #957](../backlog/active/SEARCH-001-D3-T3-bing-dual-host-zero-key-route.md)
> Parent: [D3/#937](https://github.com/wjhuang88/talos/issues/937) / [SEARCH-001/#624](https://github.com/wjhuang88/talos/issues/624)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | D3-T3a/#957: deterministic offline non-CN-first/CN-fallback decision candidate and synthetic tests only. No live requests, production endpoints, Auto policy, Network grants or scope expansion. |
| Claimed At | 2026-11-16 |
| Source Issue | #957 |
| Governance Claim PR | #958 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I329 activated by #958 merge `58207831675365e35cc80601097b96f9432a68aa`; exact-head CI #4273 4 applicable SUCCESS/4 skipped; independent no-blocker review #6907540766 and CAS. |
| Implementation PR | #959 merged at `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` (source Completion Commit) |
| Last Updated | 2026-11-17 |
| Handoff / Release Condition | T3a offline accepted/closed by owner-first closeout only. Keep #957 Open for blocked/unclaimed T3b; no production admission or release. |

## Baseline and active-work inventory

SEARCH-001-C/#643 and L/#901 are Complete; D3-T1/#941 and D3-T2/#947 are Complete, while D3/#937, D2/#848, SEARCH Epic/#624 remain Open/Unclaimed and E/F/G/I unactivated. The actual existing compatibility chain is DDG Lite → Instant Answer → DDG HTML → **CN Bing**. Reuse existing parser, exact host request and prior source identity.

Parallel owned slices remain separate: I328/STORE-002 E activation, I325 D and I327 T2 closed; I324/GIT-COMPAT currently review/draft; I323 Azure, I318 TUI review and other Active/Planned/Blocked iterations remain their owners. This I329 source slice must not edit those owners or shared decisions; re-check active claims and changed paths before starting implementation. Existing obsolete K alternate #861 closed unmerged.

## Deliverable / offline acceptance

- Pure, isolated variant and failover policy for **`www.bing.com` first, `cn.bing.com` second** within one Bing family. No implicit country inference. Bare `bing.cn` not selected.
- Only typed DNS/connect establishment/refusal and source-local timeout with remaining invocation authority/budget qualify as source-unavailable. **All HTTP 5xx (500/502/503)** are **no-fallback** pending separately governed evidence; explicit denial/403/429, rate limits, CAPTCHA/challenge, unknown redirect, insufficient Network approval, stale/cancelled/budget-exhausted invocation and malformed/uncertain results also never fail over.
- Explicit tests distinguish valid 200 empty, parser drift, restriction responses including **500/502/503 all blocked**, and successful organic results. Same-family provenance; retained external chain and existing CN-only production behavior.
- Handwritten fixture tests and a compact reviewable rights/permission/rollback decision. No live query or assumed independent native provider.
- A later T3b activation requires independently cleared access+output rights, accepted per-invocation network/redirect authority, actual controlled China/comparator evidence, fresh native CI and independent review.

## Execution order and gates

1. This governance-only claim+iteration PR merges after exact-head docs CI and independent review; until then do **not** create source implementation branch.
2. Implement bounded offline candidate on current main; locally converge targeted tests and full locked validation; stage one reviewable implementation PR.
3. Native exact-head Windows/macOS/Linux CI; independent API/network/security review, permission/redirect negative controls, and source-head CAS merge.
4. Owner-first governance closeout with the **actual prior source Completion Commit**, then close the bounded T3a claim only; preserve parent open and T3b blocked.

## Explicit exclusions

No production `www` request, browser/CDP activation, new SearchBackend identity, public `web_search` schema, Cargo/lock changes, network manifest or budget widening, new proxy flow, startup probes, rate/challenge bypass, GeoIP selection, user cookies or secrets. No supplier/region/rights assertion. Rollback of T3a is isolated candidate/test revert; there is **no** production route switch to roll back.

## Claim activation and implementation checkpoint — 2026-11-16

Claim #958 merged at `58207831675365e35cc80601097b96f9432a68aa`, after [CI #4273](https://github.com/wjhuang88/talos/actions/runs/43805545051) and independent exact-head no-blocker review. No source implementation was started before that merge. The new branch includes only private inert Bing variant decision/tests and an evidence note; at that checkpoint source CI, independent security/API review and CAS were **not yet accepted**; all T3a source gates subsequently passed as recorded in the closeout below. Production `www.bing.com` activation T3b remains BLOCKED, no new Network authority was granted.

## T3a accepted completion and validation — 2026-11-17

Completion Commit: `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` (actual source PR #959 merge on target main).

1. Governance claim #958 merged `58207831675365e35cc80601097b96f9432a68aa` before source work. Implementation [PR #959](https://github.com/wjhuang88/talos/pull/959) merged with expected HEAD `71d752880b6ede90c8e8bd89c5a87cc6f2217b9e`; **Completion Commit** `18c3ce58553b8e2c4b6ac37f056e536aa2cc2e8a` exists on target main.
2. Locked exact-head [CI #4333 / run 44011362862](https://github.com/wjhuang88/talos/actions/runs/44011362862) **8/8 SUCCESS**: macOS fmt/check/clippy/test; Windows workspace and aggregate, Linux/Windows Desktop, installer, scope classification and remote issue/owner reconciliation. Preserve original #4284 attempt-1 Windows `talos-agent` lease-test failure and same-head successful rerun; no flaky-root-cause certification.
3. Independent read-only [Copilot technical APPROVE](https://github.com/wjhuang88/talos/pull/959#issuecomment-6941738278) to exact source HEAD and current target main, zero blocking findings, role limits disclosed. GitHub clean mergeability, existing owner/work-slice and no overlap rechecked before squash merge; expected-head CAS succeeded.
4. Seven offline pure synthetic tests cover Non-CN-first/CN-only fallback, five positive guards, typed DNS/connect/refusal/source-local timeout only; all HTTP status including 5xx/403/429, challenge/access-denial, invalid/redirect, valid-empty, cancelled/deadline/budget-exhausted stop. No live `www.bing.com` request or production Network/manifest, Browser/CDP, source-order, backend identity, public API or distribution change.
5. **T3b separately BLOCKED** on qualified service-access and result/output-use rights, ADR-090 Network/proxy/redirect/budget and authorized real mainland-China/comparator observations. I329 closes **only T3a offline work**. Leave #957 Open for residual T3b, parents D3/#937 and Epic/#624 Open, historical #644 ZERO-ADMISSION intact. Any later production activation needs separate effective claim, native CI, independent review and rollback.
