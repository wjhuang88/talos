# Iteration I319: Retire `rust-websearch` Build Dependency (L1)

> Document status: Active / Claimed (proposed via #902; no effect before governance PR merge)
> Published plan date: 2026-11-10
> Planned objective: remove third-party `rust-websearch` from the Talos normal network and supported build graphs by owning the residual legacy tool contract and freezing independent pinned-upstream test evidence.
> Baseline rule: preserve public standalone `WebSearchTool` API, protected Auto and all existing permission/source/transport behaviors; do not equate dependency removal with E/F/G global search acceptance.
> MVP deliverable: build/test `talos-tools/network` with no `rust-websearch` package, retain validated public legacy call surface and documented rollback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-L L1: source-level legacy Tool parity + freeze independent oracle + complete current Cargo/fixture dependency graph retirement |
| Claimed At | 2026-11-10 |
| Source Issue | #901 |
| Governance Claim PR | #902 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Single-maintainer documentation-only claim proposal: no formal independent approving maintainer review was available at this checkpoint; Copilot advisory is COMMENTED rather than APPROVED. The user selected this non-overlapping L1 scope. Exact-head governance CI, both validators, no blocking reviews and merge-time CAS are required before #902 merge. The later Rust/Cargo implementation requires independent API/network/permission/security review. |
| Implementation PR | Not started |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | Only after accepted code, independent review, and exact-head multi-platform CI may governance close. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| [SEARCH-001-L](../backlog/active/SEARCH-001-L-legacy-dependency-retirement.md) / #901 | SEARCH-001/#624 | Ready for bounded dependency split, pending authorization | K/#850, H/#832, C/#643 Complete; source decision PR #900 | Eliminate residual third-party crate safely with legacy ABI/API parity |

### Active/Review/Planned/Blocked inventory

- I317/SEARCH-001-K, I314/H, I293/C: **Complete/Closed**; preserve their code.
- I318/TUI-054: **Review / Claimed**, non-overlapping TUI keyboard flow.
- I310/PROVIDER-008-B: independently owned model provider work; no search-overlap.
- I302/TEST-003 and I277/Desktop: Review/Claimed and deferred native testing; remain.
- I249: Planned/deferred; I164: historical Paused; preserve.
- DEPENDENCY-003-A/#502 direct libc migration: separately owned native baseline and lockfile risks;
  **shared `Cargo.lock` and external fixture updates must be synchronized** before changing them,
  avoiding parallel dependency edits. Do not overwrite its owner or implementation PR.
- SEARCH-001-I remains Blocked on E/F/G, SEARCH-001-D2/#848 Proposed/Unclaimed,
  browser #891 Blocked on #452. No E/F/G admission is implied.
- The user's current explicit instruction selects this **non-overlapping parallel L1 slice**;
  only this bounded source/dependency work is proposed, not general implementation
  authority for other Active iterations. A conflicting lockfile change invalidates CAS.

### Scope

- L1 implementation may begin only after the [ADR-085 A1 amendment](../decisions/085-talos-owned-search-routing-boundary.md#amendment-a1--equivalent-build-package-retirement-2026-11-10-proposed-902) and this atomic claim take effect together on main via #902.
- Preserve supported `WebSearchTool` public API and old zero-key behavior with Talos-owned
  helper/compatibility implementation and offline bounded protocol fixtures.
- Convert upstream comparison tests to independent pinned expected-value fixtures
  with provenance, rather than same-library tautologies.
- Remove `rust-websearch` from Rust source use, feature lists, Cargo manifests and
  current generated locked dependency closure; keep MIT attribution.
- Separate code from the G-dependent SEARCH-001-I global-search retirement decision.

### Non-goals

No new SearchBackend/Provider, endpoint, key/credential behavior, Network authority,
protected SearchRouter default chain/routing/proxy/deadline, Browser/CDP #452,
Exa/Parallel D2, mainland-China availability claim, captcha workaround,
dependency-wide upgrade or `rust-websearch` dynamic runtime fallback.

### Acceptance and planned validation

- Given ordinary `network` and `coding` features, then `cargo tree --locked`
  excludes `rust-websearch` and all supported external lock fixtures compile.
- Given previously accepted input fixtures, `WebSearchTool::new()`, `AgentTool`
  parameters and output remain compatible; old/new transport traces and errors
  are demonstrably bounded.
- Given protected production invocation, the exact Network permission,
  endpoint manifests, query-free probes, retry/budget/cancel/redirect and
  source output remain unchanged.
- `cargo fmt --all -- --check`; locked cargo test/check/Clippy for `talos-tools/network`;
  full preflight/CI on Linux, Windows, macOS; independent API/Network/security review.

### Documentation and rollback

- Owner, source-truth preflight, locked graph observations and reviewed variance.
- Revert deployed version/commit; no unprepared fallback from protected path.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-11-10 | Proposed atomic claim+activation | Governance PR #902 proposes both L1 claim and I319 activation; no production code until exact-head governance claim and Active state are merged. |

## Verification Evidence

- None for L1 code; research-only PR #900 CI3970 is not implementation CI.

## Completion Evidence

- No implementation SHA yet; remain Active/Review/Blocked until evidence exists.

## Variance And Residuals

- Existing SEARCH-001-I G/H gate is unchanged; D2/E/F/G remain separate.

## Retrospective

- Pending.
