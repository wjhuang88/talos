# SEARCH-001-K: Talos-Owned Zero-Key Compatibility Fallback

**Status**: Complete / Closed — Implementation Completion Commit `e32ea0a6220889e8346676d368110310171e2cce` (PR #860), I317 governance closeout merged in PR #870 (`45a1733ec88b7925e9f1931082562bf3b240e513`)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-K |
| Type | Compatibility / dependency decoupling Story |
| Parent Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#850](https://github.com/wjhuang88/talos/issues/850) |
| Baseline | `main` `eed60f5a8587afc1a2cbc2483ee26ac2eb39ea08` (2026-11-04) |
| Selected Iteration | [I317](../../iterations/I317-search-compat-fallback-ownership.md) (effective after governance #851 merge) |
| Related | A/B/C (completed), H/#832 (completed), D/#644 (blocked-zero-admission), D2/#848 (research), I (gated) |
| Evidence | [2026-11-04 code-truth inventory](../../reference/SEARCH-001-K-COMPATIBILITY-OWNERSHIP-2026-11-04.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-K K1: replace production-side rust-websearch HTML/URL and User-Agent helpers only, add deterministic exact-upstream regression fixtures; preserve dormant legacy rollback and cargo dependency |
| Claimed At | 2026-11-05 |
| Source Issue | #850 |
| Governance Claim PR | #851 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #851 was merged as `7b482a4243b37a4f79ef3fe460b69b0740400682` after exact-head CI3840 (success), both governance validators (0 warnings), and documented single-maintainer CAS because independent reviewer was unavailable. Security-sensitive code still requires independent review. |
| Implementation PR | #860 — merged `e32ea0a6220889e8346676d368110310171e2cce` |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | Implementation #860 merged `e32ea0a6220889e8346676d368110310171e2cce` with exact-head CI3862 8/8, independent API/network/security review and merge CAS; synchronize governance/Issue #850 on this docs-only closeout merge. No release or parent closure. |

## Why this work is narrower than a new native provider

Protected production `web_search` already uses Talos-owned prepared network invocation, backend
chain, parser, authorization, transport budget, query-free reachability and Wikipedia fallback,
with actual-legacy fixture comparisons. It does **not** invoke the opaque
`rust_websearch::search()` routing coordinator.

At the original K1 intake baseline, remaining production-side crate calls were small helpers (HTML entity/text cleanup, DDG redirect
URL decoding, random UA choice); these have now been internalized in the reviewed K1 code. The dormant public `WebSearchTool`, old behavior and
old-versus-protected parity tests still depend on `rust-websearch` for rollback.

The requested deliverable is **make the already-shipped compatible fallback self-contained at the
runtime call boundary**, not to reimplement or re-admit DDG/Bing under a second router.

## Bounded K1 work slice

- Add one private, Talos-owned compatibility helper module with MIT provenance to replace
  `rust_websearch::html_utils::{clean_html,resolve_duckduckgo_url,decode_html_entities}`
  and `rust_websearch::anti_detection::random_user_agent` in **protected production** calls.
- Preserve the existing fixed 4-source chain, probe paths/cache/TTL, bounded retry,
  Wikipedia final legacy fallback, proxy authority/snapshot, shared deadline, cancellation,
  endpoint/URL admission, source reporting, and all model-facing tool behavior.
- Add byte/behavioral parity tests against pinned upstream helper implementation, including
  percent-decoding, HTML entities, URL normalization, arbitrary UTF-8, UA pool membership, error
  boundaries and isolation from ambient optional-provider variables.
- Preserve the dormant legacy tool and parity reference to `rust-websearch`; do not remove
  `rust-websearch` from Cargo yet. Retiring the build/dependency is SEARCH-001-I's later gate.
- Record all observed compatibility differences and limitations; do not overclaim third-party
  regional reachability, automated scraping permission, or live endpoint success.

## Non-goals and safety

No new search source, independent provider admission, E/F authorization, paid usage, network
probes at app startup, CAPTCHA workaround, query leakage, new transport policy or multiple generic
retry layers. The existing compatible DDG/Bing family remains **one legacy compatibility route**.
SEARCH-001-D2 Exa/Parallel remains independently researched and unadmitted.

## Acceptance gates

1. Protected production path no longer calls `rust_websearch::html_utils` or
   `rust_websearch::anti_detection`; dormant rollback and old parity test stay available.
2. Exact request/source ordering, official Network permission, proxy/redirect, deadline,
   bounded retry, result/URL safety and output are unchanged, with offline deterministic tests.
3. Locked network-feature tests and required cross-platform CI pass on exact reviewed head.
4. Independent network/permission/security compatibility review and merge CAS are recorded.
5. Parent #624 and historical #644 blocked disposition remain unchanged.

The claim became effective in #851 (`7b482a4243b37a4f79ef3fe460b69b0740400682`); all K1 implementation acceptance gates later passed and #860 merged as existing Completion Commit `e32ea0a6220889e8346676d368110310171e2cce`. This owner is complete upon the docs-only governance closeout merge; #850 is synchronized/closed afterward.

## Code validation checkpoint — 2026-11-06

CI3847/run [42179784778](https://github.com/wjhuang88/talos/actions/runs/42179784778) completed **8/8 success** on historical pre-main-sync SHA `f2c978ea71bb0c6e9d603908f46c444a991a4650`. CI3842 rustfmt failure and canceled CI3844 are preserved. Latest-main replay requires new exact-head CI, independent API/network/security review, and merge CAS. Issue #768 was independently closed after #862; QUALITY-001 is independently owned. Do not mark K1 Complete or retire `rust-websearch`.

## Code and independent review checkpoint — 2026-11-07

CI3855/run [42203333658](https://github.com/wjhuang88/talos/actions/runs/42203333658) passed **8/8** on historical pre-sync code SHA `7dbe82c65c034a2c0c827dd699975c4442c64930`; independent Copilot [read-only review](https://github.com/wjhuang88/talos/pull/860#issuecomment-6682872016) reported **no blocking API/network/security/upstream-parity findings** for exactly that SHA. The single non-blocking stale `headers.rs` comment was corrected in this replay. Latest main `4685c2979453ef8a89675477032053ec0e15acec` includes independent #865/#866 QUALITY-001 closure and search timeout fixtures. **Fresh-head CI, exact-head re-review and merge remain required**; do not promote Draft until accepted.

## Authoritative post-merge completion — 2026-11-07

- **Delivery: Complete / Closed (upon this governance record's merge).** Existing production implementation Completion Commit: `e32ea0a6220889e8346676d368110310171e2cce`, [PR #860](https://github.com/wjhuang88/talos/pull/860) squash-merged after exact-head CAS of `d79d3d0ecbe51b19246d81a8dcc0acdd7ae6b13e`. This governance commit is not a substitute implementation SHA.
- **Exact-head CI**: [CI3862/run 42340480867](https://github.com/wjhuang88/talos/actions/runs/42340480867) **8/8 successful**, including Windows Rust/workspace, macOS Release Preflight, Linux/Windows Desktop, installer, classifier and owner reconciliation. Actual preflight logs show all six new pinned-upstream parity helper tests passed and `release preflight: passed`.
- **Independent API/network/security/upstream parity review**: [Copilot #6702564750](https://github.com/wjhuang88/talos/pull/860#issuecomment-6702564750) **no blocking findings** for the merged exact head; previous [review #6682872016](https://github.com/wjhuang88/talos/pull/860#issuecomment-6682872016) flagged only stale UA provenance comments, corrected before final acceptance.
- **Merge-time latest-main CAS**: current base #867 was docs-only PERM-004/#22, with clean GitHub merge and no K1 code overlap. GitHub confirmed main equals the implementation merge SHA and preserves the unrelated governance changes.
- **Delivered**: private Talos-owned parity helpers for seven-entry HTML entity replacement, text/tag cleaning, DDG `uddg` redirect decoding, and pinned ordered 12-UA pool with uniform rejection sampling. Existing four-source zero-key compatibility fallback plus Wikipedia final knowledge fallback, permission/source/budget/proxy/redirect/deadline behavior and model-facing schema are unchanged.
- **Retained**: dormant `WebSearchTool`, `rust-websearch` Cargo dependency and MIT provenance for rollback/upstream differential tests; no live DDG/Bing regional availability, scraping permissions or native-provider admission proven. D/#644 remains BLOCKED / ZERO-ADMISSION; D2/#848 stays separate; E/F/G/I remain gated; parent SEARCH-001/#624 remains Open.
- **Issue synchronization**: #850 was auto-closed by implementation merge, then intentionally reopened until owner-first docs are accepted. Close #850 only after this governance PR merges; do not close #624.

Earlier Draft/pending entries are historical checkpoints and do not override this terminal-state record.
