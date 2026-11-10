# SEARCH-001-I — Rust-Websearch Build Dependency Retirement Preflight

> **Status: Proposal / code-truth research only — Unclaimed and NOT authorized for implementation.**
> Evidence snapshot: `wjhuang88/talos main@f29b34d7b4e2ef833e63c95271e2851394f8f0d1`, 2026-11-10.
> Owner: [SEARCH-001 / #624](../backlog/active/SEARCH-001-zero-config-global-search.md).
> Completed predecessor: [SEARCH-001-K / #850](../backlog/active/SEARCH-001-K-compat-fallback-ownership.md), implementation [#860](https://github.com/wjhuang88/talos/pull/860).
> This packet is a proposed amendment to SEARCH-001-I sequencing, not a new provider,
> not an iteration/claim, not successful CI, and not an approval to retire a dependency.

## 1. Why this review is necessary

The user's next goal is to remove the `rust-websearch` crate from Talos *build and
public dependency graph*, without rebuilding the default search router from scratch.
The shipped protected `web_search` already owns the Lite → DDG Instant Answer →
DDG HTML → Bing CN compatibility family, source admission, timeouts, proxy, permission,
retry/cancellation, parser and Wikipedia final knowledge fallback. K1/#850 moved
production HTML/URL/UA utilities into Talos-owned code. The old crate is now retained
**for legacy standalone API behavior, rollback evidence and test oracles**, not
because Auto still calls its coordinator.

Current SEARCH-001-I's parent dependency is **G + H**; G is blocked by E/F,
which depend on qualification of independent native zero-key providers. That
gate is sensible for **global-search/old-fallback retirement**, but it may be
unnecessarily broad for **compile-time third-party crate removal**, provided
exact legacy/API/rollback behavior is independently proven. An explicit review
decision is required before splitting that gate; do not infer authorization
from this preflight.

## 2. Direct call-site inventory (actual code at snapshot)

| Residual | Where | Dependency/behavior |
|---|---|---|
| Historical standalone `WebSearchTool` and `SearchConfig` | `crates/talos-tools/src/web_search.rs` | Public `WebSearchTool::new` plus `AgentTool` implementation; old `rust_websearch::search()` still executes on standalone legacy use. It is not an internal fallback from the protected SearchRouter. |
| Upstream result conversion | `web_search.rs::convert_rust_websearch` | Depends on the crate's `SearchResult` Rust type; removal needs an owned result shape or removal of the old code path after explicit public API approval. |
| Parity helper oracles | `search_backend/compat_helpers.rs` `#[cfg(test)]` | Live upstream calls for entity replacement, tag cleaning, redirect decode and UA membership. Removing the crate without freezing **independent expected-value fixtures** loses the upstream differential check. |
| Legacy real-transport comparison | `search_backend/transport/compatibility/chain/legacy_parity_tests.rs` | Legacy `WebSearchTool::for_legacy_parity_test` and `rust_websearch::clear_probe_cache`; captures request-recipient order against old implementation. Needs an approved, deterministic replacement oracle before dropping this test. |
| Build declaration | `crates/talos-tools/Cargo.toml` | `network` feature includes `dep:rust-websearch`; `rust-websearch = "0.1"` remains optional. **Optional** does not mean absent from every `network` build. |
| Dependency resolution | `Cargo.lock`, `tests/fixtures/{browser-host-external,runtime-sdk-external}/Cargo.lock`, dependency baseline docs | Removing the manifest entry requires correct locked-graph regeneration and deliberate update/recording of *current* fixture locks and dependency observations. Historical snapshots are not silently rewritten. |
| Attribution | `crates/talos-tools/LICENSES/rust-websearch-MIT.txt` and copied/adapted source headers | **Retain** the MIT notice and provenance for Talos-owned derived implementations, even after Cargo no longer downloads the upstream crate. |

**Non-goal:** re-porting the upstream coordinator. The production chain is already
Talos-owned; doing this twice risks diverging policy, shared budget and source identity.

## 3. Two engineering choices — no choice activated by this document

**Option A: isolated legacy feature / de-risk only.** Move upstream
`rust-websearch` to a separately named, explicit, non-default legacy/parity
feature; remove `dep:rust-websearch` from ordinary `network`. This can prove
that standard `network` / product builds exclude the upstream crate. **It does
not fully retire the dependency** (the feature, all-features build and lockfile
still carry it). Any compile-gating of the public `WebSearchTool` requires
source/SDK compatibility review; do not silently make old code disappear.

**Option B: full crate retirement.** Replace the remaining public
`WebSearchTool` / legacy DDG implementation with a reviewed Talos-owned
compatibility implementation (or propose an explicit API transition with
migration guidance), freeze pinned actual-upstream fixtures, and retire the
direct `rust-websearch` manifest+lock entry after **all supported call sites
and test fixtures compile and retain documented behavior**. Do not route
unprepared standalone `execute()` through a permission-protected prepared
execution path as a secret bypass; choose and test the exact supported
standalone authority behavior. Preserve source code/license attribution.

**Review preference:** accept A only if public-API gating and feature-matrix
risks are demonstrably smaller; otherwise design B as a separate tightly
claimed implementation slice. *Neither option supplies two admitted global
search providers*. If G remains a retirement prerequisite by product policy,
the result of this review is a reproducible blocked disposition, not an
implementation shortcut.

## 4. Proposed sequence (if separately authorized)

1. **I0 — decision / no production edits:** reconcile the authority of
   SEARCH-001-I vs G and the SDK/public `WebSearchTool` contract, select A or B,
   and publish a reviewed decision/rollback design.
2. **I1 — behavior-preserving small implementation under a future effective
   Collaboration Claim + iteration:** freeze real-upstream deterministic
   fixture outputs; preserve public tool name, schema and constructor where
   supported, result formatting/error behavior, protected Network permission,
   source identity, query disclosure, proxy, deadline, cancellation and
   zero-key DDG/Bing order. No provider admission, endpoint addition, or
   runtime fallback to the legacy unprotected tool.
3. **I2 — dependency closure under explicit go/no-go:** remove or isolate the
   dependency as chosen; compare normal/explicit `network`, `coding`,
   all-features, CLI/SDK/workspace and Windows/Linux build graphs. Update only
   necessary current lockfiles/metadata; keep historical archives and MIT
   provenance. Complete only with independent API/security review and
   exact-head cross-platform CI.

## 5. Minimum acceptance/negative controls

- A frozen test for `WebSearchTool::new()` shape and `AgentTool` contract
  (input JSON Schema, description, result formatting, errors and optional
  provider ambient-variable isolation); standalone execution must have
  reviewed authority/permission semantics, not auto-escalate to protected
  permission reuse.
- Deterministic upstream-backed fixtures for HTML entities, malformed
  percent-redirect URLs, UTF-8/tag stripping and fixed UA set. Preserve
  original source/commit/hash evidence and pinned MIT notice after dependency
  retirement; never replace upstream parity with tests that only compare
  Talos output to itself without an independent frozen expected oracle.
- Offline fixed-host transport fixtures exercising request order, terminal
  errors, no-result, proxy, cache/TTL, retry, cancellation, deadline and
  Wikipedia final fallback without fetching live internet or bypassing
  challenge/rate-limit safeguards.
- Exact manifest/lock/build graph: `cargo tree --locked -p talos-tools
  --features network -i rust-websearch` and `--features coding` (with all
  supported targets/all-features as applicable). For **full removal**, upstream
  crate must be absent from product build dependency closure and repository
  manifest; mere optional feature gating is not acceptance for full retirement.
- `cargo fmt --all -- --check`, `cargo test --locked -p talos-tools
  --features network`, affected workspace/SDK/CLI fixtures, both governance
  validators, Windows/Linux/macOS CI and independent API/network/security
  review before implementation merge. A real CI or local run must provide
  evidence: this source inspection is **not** a successful test.
- Rollback must be concrete: a version/deployment revert to a known accepted
  tag/commit, or a documented supported legacy feature if Option A is
  selected, rather than re-enabling the upstream tool inside protected Auto.

## 6. Required governance decision

**Current state stays unchanged:** SEARCH-001-I is **Proposed / Blocked by G+H,
unclaimed**, E/F/G are still blocked by independent provider qualification, and
the repository retains `rust-websearch`. This document requests *review* of
the narrower I0/I1/I2 sequence; it does **not** amend a binding dependency
gate, create a new Issue, authorize code, establish a claim or close the Epic.

If accepted, revise the authoritative owner/child map under normal governance,
inventory Active/Review/Planned/Blocked iterations, ensure no overlapping code
claim, and atomically claim the smallest runnable implementation slice. Only
then implement under a separate production PR. Otherwise record why physical
crate retirement must wait for G, with no false-progress claim.
