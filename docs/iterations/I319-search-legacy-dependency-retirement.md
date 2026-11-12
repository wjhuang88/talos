# Iteration I319: Retire `rust-websearch` Build Dependency (L1)

> Document status: Complete / Closed upon owner-first governance closeout publication (implementation #907 already merged)
> Published plan date: 2026-11-10
> Planned objective: remove third-party `rust-websearch` from the Talos normal network and supported build graphs by owning the residual legacy tool contract and freezing independent pinned-upstream test evidence.
> Baseline rule: preserve public standalone `WebSearchTool` API, protected Auto and all existing permission/source/transport behaviors; do not equate dependency removal with E/F/G global search acceptance.
> MVP deliverable: build/test `talos-tools/network` with no `rust-websearch` package, retain validated public legacy call surface and documented rollback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-L L1: source-level legacy Tool parity + freeze independent oracle + complete current Cargo/fixture dependency graph retirement |
| Claimed At | 2026-11-10 |
| Source Issue | #901 |
| Governance Claim PR | #902 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim and ADR-085 A1 effective via #902; independent API/network/security review, native CI4121 8/8 on exact implementation head, and expected-head CAS merge of #907 completed. |
| Implementation PR | L1a #904 merged; L1b #907 merged as `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` |
| Last Updated | 2026-11-12 |
| Handoff / Release Condition | Fulfilled for L1 only: code PR #907 merged after exact-head 8/8 CI4121 and independent security/API review. Publish this owner-first closeout, then close #901; parent #624/E/F/G/I and release gates remain separate. |

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
| 2026-11-11 | L1b candidate review (historical) | PR #907 underwent iterative compatibility/security review: owned legacy Tool/probe, three Cargo-generated locks without rust-websearch or reqwest 0.12, isolated compression/WebPKI and ambient proxy handling, and serialized process-global rate limiting. Final acceptance followed on 2026-11-12. |
| 2026-11-12 | L1b source acceptance and merge | PR #907 exact head `faa4982caed19e3f35b9e5648c47c36184c24f52`, full native CI4121 8/8 SUCCESS, independent read-only security/API review without source blocker; GitHub squash merge with expected-head CAS produced **Completion Commit `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970`**. |

## Verification Evidence

- L1a [PR #904](https://github.com/wjhuang88/talos/pull/904) merged; source-pinned offline oracle remains independent of `rust-websearch` implementation.
- L1b root and active Runtime SDK/Browser Host external locks were regenerated by Cargo and contain no `rust-websearch` or `reqwest 0.12`. Existing public `WebSearchTool` constructor/tool schema and protected production route remain separate.
- Final **exact-head [CI4121](https://github.com/wjhuang88/talos/actions/runs/43195189353)** succeeded **8/8** at `faa4982caed19e3f35b9e5648c47c36184c24f52`: Windows workspace, Windows Rust workspace, macOS format/check/Clippy/test, Linux and Windows Desktop, installer fixture, classifier and remote issue/owner reconciliation. Historical CI4063/CI4078 and [temporary isolated Cargo and four-waiter validation](https://github.com/wjhuang88/talos/actions/runs/43007470234) are supporting evidence, not replacements for CI4121.
- Independent read-only exact-head [review](https://github.com/wjhuang88/talos/pull/907#issuecomment-6820413223) found no remaining source/routing/dependency/ADR blockers after fixes for DDG/Wikipedia ambient proxies, WebPKI/compression compatibility, concurrency, and unbounded legacy wire-body accumulation. Final client streams via `reqwest::Response::chunk()` and caps **accepted/accumulated wire bytes** and **decoded output** separately at 8 MiB; chunk frames and Vec capacity mean **no strict 8 MiB peak process memory guarantee**. Focused [wire/expansion test run](https://github.com/wjhuang88/talos/actions/runs/43194684946) succeeded; these tests are in full CI4121. Protected SearchRouter source/authority is unchanged.

## Completion Evidence

Completion Commit: `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` — actual implementation #907; this documentation closeout is not the implementation commit.

- **Implementation Completion Commit:** `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` ([PR #907](https://github.com/wjhuang88/talos/pull/907)), merged using expected head `faa4982caed19e3f35b9e5648c47c36184c24f52` after [CI4121 8/8 SUCCESS](https://github.com/wjhuang88/talos/actions/runs/43195189353) and [independent final-head source review](https://github.com/wjhuang88/talos/pull/907#issuecomment-6820413223) with no code blocker. This **docs-only closeout is not** the implementation Completion Commit; close #901 only upon its owner-first governance merge.

## Variance And Residuals

- Existing SEARCH-001-I G/H gate is unchanged; D2/E/F/G remain separate.

## Retrospective

- **Accepted:** frozen independent upstream oracle first (#904), then replace residual public legacy Tool/probe without changing prepared production SearchRouter authority; explicit negative proxy/privacy, decompression, wire-size and concurrent fairness fixtures prevented hidden compatibility regressions.
- **Observed:** multiple mainline changes and independent governance issue registration moved the target branch. Rebased with force-with-lease and generated Cargo locks preserving newer provider dependencies; kept archived green source commits for audit. Repository-wide missing Issue rows (#914 then #919/#923) were repaired by separately owned governance PRs, not by bypassing validators.
- **Residual:** neither L1 nor its CI establishes search provider E/F/G admission, global geographic reachability, strict process RSS ceiling or a release qualification. Rollback is an explicit version/deployment revert to pre-L1 main; do not restore an unreviewed protected-to-legacy fallback.
