# Iteration I319: Retire `rust-websearch` Build Dependency (L1)

> Document status: Active / Claimed (effective via merged #902; implementation #907 in review)
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
| Authorization Evidence | The claim and ADR A1 became effective on main after #902 merged. This does not grant review approval for implementation #907; independent API/network/security review, native CI and merge-time CAS remain necessary. |
| Implementation PR | L1a #904 merged; L1b #907 Draft / unmerged |
| Last Updated | 2026-11-11 |
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

- L1 implementation authorization was satisfied when [ADR-085 A1](../decisions/085-talos-owned-search-routing-boundary.md#amendment-a1--equivalent-build-package-retirement-2026-11-10-accepted-via-902) and this atomic claim took effect together on main via merged #902.
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
| 2026-11-10 | Atomic claim + ADR A1 activation | Governance PR #902 merged at `135fd80508896c9bd3fa30879d4c125fc272a444`. |
| 2026-11-10 | L1a source oracle accepted | PR #904 merged at `00e4ebdd0da0b683005e115e199d6ffd298c2aa1`, preserving independently pinned upstream fixtures. |
| 2026-11-11 | L1b dependency retirement in review | PR #907 current HEAD is the authoritative mutable implementation candidate. Owned legacy Tool/probe, three Cargo-generated locks without rust-websearch or reqwest 0.12, isolated compression/WebPKI/ambient proxy handling, and serialized process-global concurrent rate limiting. Exact-head acceptance and merge remain pending. |

## Verification Evidence

- L1a [PR #904](https://github.com/wjhuang88/talos/pull/904) merged; source-pinned offline oracle remains independent of `rust-websearch` implementation.
- L1b root and active Runtime SDK/Browser Host external locks were regenerated by Cargo and contain no `rust-websearch` or `reqwest 0.12`. Existing public `WebSearchTool` constructor/tool schema and protected production route remain separate.
- Historical [CI4063](https://github.com/wjhuang88/talos/actions/runs/42973360173) passed all 8 native checks **before** the concurrent limiter correction. [Temporary isolated Cargo re-generation and 4-waiter limiter regression](https://github.com/wjhuang88/talos/actions/runs/43007470234) passed after Gemini main changes; the focused regression executed exactly one test. The temporary workflow is excluded from #907. **Current-head full native CI on #907 is still required**; do not reuse an older run for final acceptance.
- Independent review identified and prompted fixes for DDG/Wikipedia ambient system proxy disclosure, WebPKI/response-encoding compatibility, and concurrent legacy rate-limit reservation. Both proxy subprocess tests and a four-waiter async mutex regression are staged in Draft #907; post-fix source review found no further code blocker. Final-head review and governance acceptance remain separate from source findings.

## Completion Evidence

- L1b candidate: current HEAD of [Draft #907](https://github.com/wjhuang88/talos/pull/907); **not merged / not Complete**. The historical candidate hashes belong only to dated PR discussion. Final implementation merge SHA, exact-head CI and independent reviewer evidence must be recorded in an owner-first closure PR before closing #901.

## Variance And Residuals

- Existing SEARCH-001-I G/H gate is unchanged; D2/E/F/G remain separate.

## Retrospective

- Pending final exact-head CI, independent signoff and post-merge retrospective.
