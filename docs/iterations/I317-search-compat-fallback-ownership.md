# Iteration I317: Own Zero-Key Search Compatibility Helpers

> Document status: Active (governance #851 merged `7b482a4243b37a4f79ef3fe460b69b0740400682`; implementation #860 Draft)
> Published plan date: 2026-11-05
> Planned objective: make the protected production Auto search path independent of `rust_websearch` utility/helper calls while preserving the existing zero-key compatibility fallback.
> Baseline rule: preserve current search semantics and authority; no E/F provider admission or `rust-websearch` retirement.
> MVP deliverable: Talos-owned HTML/entity/DDG redirect helpers and User-Agent selection with pinned upstream parity tests, clean protected production dependency boundary, and no default behavior change.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-K K1: production-side HTML, DDG URL/entity, and UA helper ownership plus parity fixtures; no source/transport/permission/dependency/rollback changes |
| Claimed At | 2026-11-05 |
| Source Issue | #850 |
| Governance Claim PR | #851 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Single-maintainer docs claim #851 merged `7b482a4243b37a4f79ef3fe460b69b0740400682` after exact-head CI3840 and both governance validators passed 0 warnings; independent reviewer unavailable (GitHub rejected Copilot assignment). K1 code requires independent network/security review. |
| Implementation PR | #860 (Draft, code acceptance pending) |
| Last Updated | 2026-11-06 |
| Handoff / Release Condition | Effective claim on main. Draft implementation #860 unaccepted; must pass exact-head locked tests, native CI, independent security review and merge CAS before closure. |

The claim and Active state **became effective** on main at `7b482a4243b37a4f79ef3fe460b69b0740400682` (#851).
The separately scoped K1 code now lives in Draft PR #860 and has no delivery/closure authority before validation and merge.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| [SEARCH-001-K](../backlog/active/SEARCH-001-K-compat-fallback-ownership.md) / #850 | SEARCH-001 / #624 | Ready for scoped compatibility refactor | Accepted ADR-085/090, C/#643 and H/#832 complete | Own final production helper calls and prove bounded parity |

### Inventory and dependency disposition

- I316 / PROCESS-001 / #853: Complete / Closed after implementation #855 and accepted closeout on 2026-11-05; **retain distinct process-launcher/native memory scope**. This K1 effort is parallel, independently owned, and does not touch launchers, shell/exec, process permission or their iteration documents.
- I310 / PROVIDER-008-B: separately active Gemini protocol/Vertex authentication, retain.
- I302 / TEST-003: Review with unresolved Windows diagnostics, retain.
- I277 / DESKTOP-001-D3: Review / deferred human/device acceptance, retain.
- I249 / DEPENDENCY-003-A: Planned / Unclaimed, defer dependency upgrades.
- I164: historical Paused; retain.
- I314 / SEARCH-001-H: Complete/Closed, do not reopen.
- SEARCH-001-D / #644: Complete/Closed **BLOCKED / ZERO-ADMISSION**, preserve.
- SEARCH-001-D2 / #848, Draft #849: Proposed/Unclaimed research only, no provider admission or overlap.
- SEARCH-001-E/F/G: blocked on D admission; SEARCH-001-I: gated by G/H; no implementation authority.
- PR #838 (Gemini) and #849 (D2) are independent of the K1 production helper imports.

Parallel scope is limited to the maintainer's explicit request to continue K1 and does not
automatically authorize any new provider, release or cross-iteration ownership. Refresh
current competing-owner status before merge CAS.

### Scope

- Introduce private `talos-tools/search_backend` helper implementation with attribution to the pinned MIT source.
- Replace only protected-production helper uses of `rust_websearch::html_utils` and `rust_websearch::anti_detection::random_user_agent` (DDG/Bing/Instant Answer parsers and compatibility preparation/chain).
- Include exact upstream helper parity fixtures, malformed encoding and HTML edge cases, UA pool and frozen-header assertions.
- Preserve 4-source Auto order, query-free probes, 30s cache, shared 135s/24 send budget, transport permission/SSRF/redirect/proxy, Wikipedia last resort and all user-facing schemas.
- Retain legacy `WebSearchTool` and pinned `rust-websearch` dependency/lockfile for independent rollback and old-vs-new fixture evidence.

### Non-goals

No new native provider admission, E/F/G/I work, new route, startup network probe, scraping
permission assumption, CAPTCHA bypass, paid auto selection, changes to Network authority,
model-facing public API or Cargo dependency removal.

### Acceptance

1. Protected source helper imports contain no `rust_websearch` references, excluding legacy test/rollback.
2. Pinned upstream parity for HTML entities, cleaning, DDG redirect encoding, UA pool and selection; existing chain/reachability/legacy-parity tests pass.
3. Existing source order, fail-closed parsing, permissions, cancellation/deadline, proxy/budget and result formatting remain stable.
4. Locked network-feature tests and standard Linux/Windows CI pass; independent API/network/security review accepts exact implementation head.
5. Evidence distinguishes helper de-risk from entire dependency retirement; parent #624 remains open, D/#644 blocked, E/F/G/I unactivated.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo test -p talos-tools --features network --locked`
- `cargo clippy -p talos-tools --features network --all-targets --locked -- -D warnings` (document any pre-existing lints separately; don't misstate as green)
- Required workspace preflight and native Windows/Linux CI, exact-head independent network/permission/API review.
- No live upstream search, paid provider, or mainland-China network success is claimed from fixtures.

### Documentation and rollback

Update exact-source provenance, ownership inventory, and completion record. Attribute copied MIT
implementation and retain licence text. Roll back via versioned code revert, not through runtime
unprotected direct legacy fallback. Do not modify E/F or I state.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-11-05 | Atomic claim+activation proposal | #851 proposed I317/SEARCH-001-K, ineffective prior to its target-branch merge; separate I316/I310 owners retained. |
| 2026-11-06 | Claim effective | #851 merged into main as `7b482a4243b37a4f79ef3fe460b69b0740400682` after CI3840 and both governance validators. |
| 2026-11-06 | Draft implementation | #860 K1 private helper replacement and parity fixtures submitted; code CI/review pending. |

## Verification Evidence

- Governance claim #851 merged at `7b482a4243b37a4f79ef3fe460b69b0740400682`. Its exact-head CI3840/run 42177971137 completed success; both governance validators passed with zero warnings. Single-maintainer exception recorded; no independent reviewer approval is claimed.
- Draft implementation PR #860 begins from the effective claim merge. Initial commit `f38fd1bdaacad77edb9c369931eb843ac0ee5f90` adds a private upstream-parity helper and tests. CI and independent implementation security review are **pending**, not passing evidence.

### 2026-11-06 Historical CI acceptance boundary

- **CI3847/run [42179784778](https://github.com/wjhuang88/talos/actions/runs/42179784778)** passed all **8/8** jobs on historical head `f2c978ea71bb0c6e9d603908f46c444a991a4650`: macOS release preflight, native Windows Rust/workspace, Linux/Windows Desktop, installer/classifier and remote owner checks. CI3842 had failed on rustfmt before the formatting fix; CI3844 was canceled. These are historical outcomes, not a claim that the rebased head passed.
- Current-main replay preserves separately accepted #768 fixture repair and QUALITY-001. Fresh exact-head code/native validation and an independent API/network/security review are **still required**. Implementation #860 remains Draft; no merged K1 code or Completion Commit.

## Completion Evidence

Pending a separately accepted implementation commit. A governance-only commit must not cite itself.

## Variance And Residuals

Original D/#644 remains zero-admission; separate D2/#848 possible hosted route does not change K1
fallback ownership. The old crate remains in Cargo until the separately gated I decision.

## Retrospective

Pending actual implementation and closeout.
