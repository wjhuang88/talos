# Iteration I317: Own Zero-Key Search Compatibility Helpers

> Document status: Complete / Closed (implementation #860 merged `e32ea0a6220889e8346676d368110310171e2cce`; governance closeout published on merge)
> Published plan date: 2026-11-05
> Planned objective: make the protected production Auto search path independent of `rust_websearch` utility/helper calls while preserving the existing zero-key compatibility fallback.
> Baseline rule: preserve current search semantics and authority; no E/F provider admission or `rust-websearch` retirement.
> MVP deliverable: Talos-owned HTML/entity/DDG redirect helpers and User-Agent selection with pinned upstream parity tests, clean protected production dependency boundary, and no default behavior change.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-K K1: production-side HTML, DDG URL/entity, and UA helper ownership plus parity fixtures; no source/transport/permission/dependency/rollback changes |
| Claimed At | 2026-11-05 |
| Source Issue | #850 |
| Governance Claim PR | #851 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Single-maintainer docs claim #851 merged `7b482a4243b37a4f79ef3fe460b69b0740400682` after exact-head CI3840 and both governance validators passed 0 warnings; independent reviewer unavailable (GitHub rejected Copilot assignment). K1 code requires independent network/security review. |
| Implementation PR | #860 — merged `e32ea0a6220889e8346676d368110310171e2cce` |
| Last Updated | 2026-11-07 |
| Handoff / Release Condition | All K1 code acceptance gates completed by #860 merge; close Issue #850 after this owner-first governance closeout merges. SEARCH-001/#624, D/#644 and E/F/G/I remain separate. |

The claim and Active state **became effective** on main at `7b482a4243b37a4f79ef3fe460b69b0740400682` (#851).
K1 code was implemented and merged in PR #860 at `e32ea0a6220889e8346676d368110310171e2cce`; the I317 terminal governance record is published in this docs-only closeout.

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
| 2026-11-06 | Draft implementation | #860 K1 private helper replacement and parity fixtures submitted; initial CI3842 rustfmt failure was fixed before successful later heads. |
| 2026-11-07 | Accepted implementation merge | #860 exact head `d79d3d0` passed CI3862 8/8 and independent security/API review, then merged into main as `e32ea0a6220889e8346676d368110310171e2cce` by expected-head CAS. |

## Verification Evidence

- Governance claim #851 merged at `7b482a4243b37a4f79ef3fe460b69b0740400682`. Its exact-head CI3840/run 42177971137 completed success; both governance validators passed with zero warnings. Single-maintainer exception recorded; no independent reviewer approval is claimed.
- Implementation PR #860 began after effective claim; the original `f38fd1b` failed rustfmt in CI3842, fixed by `f2c978e`. Final code head `d79d3d0` passed CI3862 8/8 and independent exact-head security/API review, then merged `e32ea0a6220889e8346676d368110310171e2cce`.

### 2026-11-06 Historical CI acceptance boundary

- **CI3847/run [42179784778](https://github.com/wjhuang88/talos/actions/runs/42179784778)** passed all **8/8** jobs on historical head `f2c978ea71bb0c6e9d603908f46c444a991a4650`: macOS release preflight, native Windows Rust/workspace, Linux/Windows Desktop, installer/classifier and remote owner checks. CI3842 had failed on rustfmt before the formatting fix; CI3844 was canceled. These are historical outcomes, not a claim that the rebased head passed.
- Current-main replay preserves separately accepted #768 fixture repair and QUALITY-001. Fresh exact-head code/native validation and an independent API/network/security review are **still required**. Implementation #860 remains Draft; no merged K1 code or Completion Commit.

## Completion Evidence

**Complete / Closed** on publication of this governance closeout.

Actual implementation Completion Commit: `e32ea0a6220889e8346676d368110310171e2cce`

[CI3862](https://github.com/wjhuang88/talos/actions/runs/42340480867) passed 8/8, [independent final-head API/security review](https://github.com/wjhuang88/talos/pull/860#issuecomment-6702564750) found no blockers, and GitHub clean target-branch CAS and expected-head merge succeeded. This docs-only closeout is not the Completion Commit.

## Variance And Residuals

Original D/#644 remains zero-admission; separate D2/#848 possible hosted route does not change K1
fallback ownership. The old crate remains in Cargo until the separately gated I decision.

## Retrospective

K1 kept scope private and narrow, preserving protected zero-key behavior and upstream rollback. The historical rustfmt discrepancy and nonblocking stale-UA provenance comment were repaired and independently rechecked. Cross-platform CI verified source-parity fixtures; it did not prove live search reachability.

## Independent review and CI after historical main synchronization — 2026-11-07

CI3855/run [42203333658](https://github.com/wjhuang88/talos/actions/runs/42203333658) passed **8/8** on historical pre-sync code SHA `7dbe82c65c034a2c0c827dd699975c4442c64930`; independent Copilot [read-only review](https://github.com/wjhuang88/talos/pull/860#issuecomment-6682872016) reported **no blocking API/network/security/upstream-parity findings** for exactly that SHA. The single non-blocking stale `headers.rs` comment was corrected in this replay. Latest main `4685c2979453ef8a89675477032053ec0e15acec` includes independent #865/#866 QUALITY-001 closure and search timeout fixtures. **Fresh-head CI, exact-head re-review and merge remain required**; do not promote Draft until accepted.

That was the historical pre-merge checkpoint. The actual merged implementation now exists as `e32ea0a6220889e8346676d368110310171e2cce`; governance and #850 closeout remain conditional on this documentation PR merging.

## Final authoritative I317 closure — 2026-11-07

- K1 code Completion Commit: `e32ea0a6220889e8346676d368110310171e2cce`
- Accepted implementation head `d79d3d0ecbe51b19246d81a8dcc0acdd7ae6b13e` passed CI3862 8/8, [independent Copilot security/API/parity review](https://github.com/wjhuang88/talos/pull/860#issuecomment-6702564750), clean merge-time CAS.
- Six offline exact-upstream helper tests passed, including malformed encodings and 12-UA-byte fingerprint, with no source/permission/budget/proxy/redirect/retry/transport/API change.
- Dormant `rust-websearch` package, legacy `WebSearchTool`, MIT notice and direct upstream test oracle intentionally retained. Dependency retirement is separately gated SEARCH-001-I; D/#644 zero native-provider admission and regional reachability limitations persist.
- Parent SEARCH-001/#624 stays **Open**; E/F/G/I not activated. #850 had been automatically closed by #860 merge and reopened until this closeout merges; update/close only after published governance CAS.
- Older Active/Draft statements are historical and superseded by this terminal-state section.
