# DEPENDENCY-003-A: libc Stable Baseline Migration

> Document status: Active / Claimed

| Field | Value |
|---|---|
| Parent | DEPENDENCY-003 |
| Type | Dependency Migration / Native Boundary Compatibility |
| Priority | P2 |
| Status | Active / Claimed |
| Source | GitHub Issue #502 |
| Selected Iteration | None |
| Depends On | I250 completion; ADR-007; ARCH-034-R04; completed #502 libc route research |
| Preserved behavior | Existing Unix process-hardening call sites, configured resource limits, process-group behavior, environment filtering, Windows cfg isolation, public APIs and release targets |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-5.6 Sol / talos开发 session |
| Work Slice | Stabilize Talos direct Unix `libc` dependency from the 1.0 prerelease line to stable 0.2, refresh only resulting lock/fixture resolution, and produce compatibility/platform evidence without changing Rust hardening behavior. |
| Claimed At | 2026-09-23 |
| Source Issue | #502 |
| Governance Claim PR | #604 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-007 and the #502 2026-09-23 maintainer route decision establish the protected dependency boundary; PR #604 requires exact-head independent security/escape-vector review, applicable CI, both governance validators and merge-time CAS before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-09-23 |
| Handoff / Release Condition | Merge #604 only after exact-head independent protected-scope review, CI, both governance validators and merge-time CAS. Start the implementation branch only from that merge or a later main commit; no Cargo or production implementation is authorized before then. |

## Decision Basis

The 2026-09-22/23 #502 research pass established the following bounded facts:

- Talos directly declares `libc = "1.0.0-alpha.3"` in `talos-sandbox` and `talos-tools`, while the root lock remains on alpha.3 and the independent runtime SDK fixture has resolved alpha.4.
- The repository history examined did not identify a technical requirement for the 1.0 prerelease line.
- Talos's current explicit libc call surface does not show a static source/API/ABI blocker to the stable 0.2 line on the inspected macOS/Linux targets.
- The manifest prerelease requirement is not an exact pin, so external consumers can resolve a later prerelease independently of the repository root lock.
- The maintainer selected a stable 0.2 direct-dependency baseline in Issue #502 on 2026-09-23.
- ARCH-034-R04-AG1 remains a separate process-hardening security residual and is not implementation authority for this dependency migration.

This decision basis is compatibility/risk evidence, not a claim that stable 0.2 is vulnerability-free or that platform validation has already passed.

## Scope

The implementation slice is intentionally narrow:

1. Change the Unix direct `libc` requirement in:
   - `crates/talos-sandbox/Cargo.toml`
   - `crates/talos-tools/Cargo.toml`
   from the 1.0 prerelease requirement to the stable `0.2` line.
2. Refresh `Cargo.lock` only as required by that manifest change.
3. Refresh `tests/fixtures/runtime-sdk-external/Cargo.lock` only as required to prove the external-consumer path no longer receives a Talos-direct 1.0 prerelease dependency.
4. Preserve every existing Rust libc call site and behavior unless validation demonstrates an unavoidable compatibility defect; any such defect stops this slice and requires a new reviewed scope rather than an opportunistic code change.
5. Record locked dependency-tree evidence that distinguishes Talos direct dependencies from unrelated transitive versions.
6. Validate the supported Unix release targets and Windows cfg isolation before merge.

## Explicit Exclusions

This owner does **not** authorize:

- edits to `crates/talos-sandbox/src/hardening.rs`, `crates/talos-tools/src/process_boundary.rs`, or other Rust hardening/runtime behavior;
- remediation of ARCH-034-R04-AG1, including `unsetenv` placement or ignored `setrlimit` return values;
- permission, sandbox-policy, process-limit, process-tree, timeout, or unsafe-contract changes;
- dependency deduplication solely to make the graph contain one libc version;
- migration to libc 1.0 final when it is eventually released;
- closure of #502's advisory/deprecation evidence-source residual;
- release or publication changes.

If the 0.2 migration requires any excluded behavior change, stop and route that finding to a separately reviewed owner.

## Compatibility / Security Review Questions

Independent review must confirm:

- the actual Talos libc call surface remains available with compatible signatures, constant values/types, symbol linkage and relevant struct/type layout on supported Unix targets;
- changing only the dependency line cannot silently weaken the intended resource-limit, environment or process-group behavior;
- Windows continues to exclude the Unix libc dependency/call path through target cfgs;
- lockfile changes are explainable by the selected direct dependency change and do not smuggle unrelated upgrades;
- the external SDK fixture resolves a stable 0.2 Talos-direct libc path;
- ARCH-034-R04-AG1 remains explicitly unresolved rather than being represented as fixed by this dependency migration.

## Acceptance

Before implementation merge:

- both Talos direct Unix manifests use the stable `libc = "0.2"` line;
- no Talos direct dependency path resolves `libc 1.0.0-alpha.*`;
- root and independent fixture lockfile diffs are limited to consequences of the approved dependency change;
- the libc call-site/API/ABI matrix is rechecked against the actually resolved stable 0.2 release;
- locked build/test evidence covers the supported macOS and Linux release targets, and Windows locked build/test evidence proves cfg isolation;
- `cargo test --workspace --locked` and repository-required validation pass on the applicable CI targets;
- the external runtime SDK fixture passes its default and coding/shared-composition paths;
- an independent ADR-007 / escape-vector review approves the exact implementation candidate;
- no Rust process-hardening implementation is changed under this owner.

## Rollback

If compatibility, ABI, platform, fixture, or protected-scope review fails, restore the pre-migration manifest and lockfile resolution before merge. Do not keep a partial 0.2 migration or compensate by weakening tests, cfgs, hardening or security expectations.

## Residuals

- ARCH-034-R04-AG1 remains separately owned and unclaimed unless independently activated.
- DEPENDENCY-003 continues to own the unrelated advisory/deprecation evidence-source residual after this child completes.
- Future libc 1.0 adoption requires fresh evidence and separately governed work; completion of this child does not pre-authorize it.

## Completion Evidence

- Completion Commit: pending
