# SEARCH-001-K: Talos-Owned Zero-Key Compatibility Fallback

**Status**: Active / Claimed (#851 merged 2026-11-06; K1 implementation Draft #861)

| Field | Value |
|---|---|
| Story ID | SEARCH-001-K |
| Type | Compatibility / dependency decoupling Story |
| Parent Epic | [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md) |
| Source Issue | [#850](https://github.com/wjhuang88/talos/issues/850) |
| Baseline | `main` `eed60f5a8587afc1a2cbc2483ee26ac2eb39ea08` (2026-11-04) |
| Selected Iteration | [I317](../../iterations/I317-search-compat-fallback-ownership.md) (activation effective through #851) |
| Related | A/B/C (completed), H/#832 (completed), D/#644 (blocked-zero-admission), D2/#848 (research), I (gated) |
| Evidence | [2026-11-04 code-truth inventory](../../reference/SEARCH-001-K-COMPATIBILITY-OWNERSHIP-2026-11-04.md) |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-K K1: replace production-side rust-websearch HTML/URL and User-Agent helpers only, add deterministic exact-upstream regression fixtures; preserve dormant legacy rollback and cargo dependency |
| Claimed At | 2026-11-05 |
| Source Issue | #850 |
| Governance Claim PR | #851 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly requested continuation of SEARCH-001-K on 2026-11-05 while I316 and I310 remain separate; concurrent scope is disjoint. Exact-head governance validators, CI, and no-blocker review required before the claim can merge; effective only on target branch. |
| Implementation PR | [#861](https://github.com/wjhuang88/talos/pull/861) — Draft, validation pending |
| Last Updated | 2026-11-06 |
| Handoff / Release Condition | #851 effective on main as `7b482a4243b37a4f79ef3fe460b69b0740400682`; K1 #861 requires independent API/network review, locked Rust tests, exact-head CI and merge CAS before implementation closure. |

## Why this work is narrower than a new native provider

Protected production `web_search` already uses Talos-owned prepared network invocation, backend
chain, parser, authorization, transport budget, query-free reachability and Wikipedia fallback,
with actual-legacy fixture comparisons. It does **not** invoke the opaque
`rust_websearch::search()` routing coordinator.

Remaining production-side crate calls are small helpers (HTML entity/text cleanup, DDG redirect
URL decoding, random UA choice). The dormant public `WebSearchTool`, old behavior and
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

The atomic claim became effective when #851 merged into main as `7b482a4243b37a4f79ef3fe460b69b0740400682` after exact-head docs-only CI3840. K1 code is staged in Draft #861 at `d8b5046a7c3e3aaa130044d1087968f2bd6eff5b`; CI3843 and independent review are pending. No K1 completion is claimed, and #850 remains Open.
