# DEPENDENCY-003-A: libc Stable Baseline Migration

> Document status: Review / Claimed

| Field | Value |
|---|---|
| Parent | DEPENDENCY-003 |
| Type | Dependency Migration / Native Boundary Compatibility |
| Priority | P2 |
| Status | Review / Claimed |
| Source | GitHub Issue #502 |
| Selected Iteration | None |
| Depends On | I250 completion; ADR-007; ADR-082; ARCH-034-R04; completed #502 libc route research plus current-main supervisor-surface refresh |
| Preserved behavior | Existing Unix process-hardening call sites, configured resource limits, ADR-082 owned-command supervisor/process-group cancellation behavior, environment filtering, Windows cfg isolation, public APIs and release targets |

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
| Authorization Evidence | ADR-007, ADR-082 and the #502 2026-09-23 maintainer route decision establish the protected native/dependency boundary; PR #604 requires exact-head independent security/escape-vector review covering the complete current direct-libc surface, applicable CI, both governance validators and merge-time CAS before merge. |
| Implementation PR | #645 |
| Last Updated | 2026-10-04 |
| Handoff / Release Condition | #604 is merged; #645 must satisfy every acceptance gate, exact-head CI and independent protected-scope review before merge-time CAS. Post-merge baseline and owner-first closeout remain separate. |

## Decision Basis

The 2026-09-22/23 #502 research pass established the following bounded facts:

- Talos directly declares `libc = "1.0.0-alpha.3"` in `talos-sandbox` and `talos-tools`, while the root lock remains on alpha.3 and the independent runtime SDK fixture has resolved alpha.4.
- The repository history examined did not identify a technical requirement for the 1.0 prerelease line.
- The 2026-09-22/23 research checked the then-enumerated `hardening.rs` / `process_boundary.rs` surface and found no static source/API/ABI blocker to stable 0.2 on the inspected macOS/Linux targets. Current `main` subsequently includes the ADR-082 `crates/talos-sandbox/src/supervisor.rs` libc surface (including `waitid` / `siginfo_t`, process-group signaling, and Darwin `proc_listpids`-related bindings). That newer ABI-sensitive surface is not treated as previously validated and must be included in the authoritative migration matrix and exact-candidate review.
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
4. Preserve every existing Rust libc call site and behavior, including `crates/talos-sandbox/src/hardening.rs`, `crates/talos-tools/src/process_boundary.rs`, and the ADR-082 `crates/talos-sandbox/src/supervisor.rs` owned-command cancellation surface. If validation demonstrates an unavoidable compatibility defect, stop this slice and require a new reviewed scope rather than an opportunistic code change.
5. Record locked dependency-tree evidence that distinguishes Talos direct dependencies from unrelated transitive versions.
6. Validate the supported Unix release targets and Windows cfg isolation before merge.

## Explicit Exclusions

This owner does **not** authorize:

- edits to `crates/talos-sandbox/src/hardening.rs`, `crates/talos-tools/src/process_boundary.rs`, `crates/talos-sandbox/src/supervisor.rs`, or other Rust hardening/runtime behavior;
- remediation of ARCH-034-R04-AG1, including `unsetenv` placement or ignored `setrlimit` return values;
- permission, sandbox-policy, process-limit, process-tree, timeout, or unsafe-contract changes;
- dependency deduplication solely to make the graph contain one libc version;
- migration to libc 1.0 final when it is eventually released;
- closure of #502's advisory/deprecation evidence-source residual;
- release or publication changes.

If the 0.2 migration requires any excluded behavior change, stop and route that finding to a separately reviewed owner.

## Compatibility / Security Review Questions

### 2026-10-04 Exact-Candidate ABI Matrix

Candidate `1d23edc545c215c8fbaae09f3723c601c004c866` resolves `libc 0.2.186` in
the workspace and `libc 0.2.189` in the independent SDK fixture. The source inventory
below includes the ADR-082 additions absent from the earlier research pass.

| Caller | API / ABI surface | Stable 0.2 comparison and boundary |
|---|---|---|
| `talos-sandbox/src/hardening.rs` | `rlimit`, `rlim_t`, `setrlimit`, `RLIMIT_CORE/CPU/AS` | Unix 0.2 retains the two-field 64-bit `rlimit` layout and target-specific resource parameter; existing casts and error handling are unchanged. |
| `talos-tools/src/process_boundary.rs` | `setsid`, `unsetenv`, `setrlimit`, negative-PGID `kill`, `SIGTERM/SIGKILL`, `ESRCH` | Signatures and constants remain available under Unix cfg; existing ignored `setrlimit` result remains ARCH-034-R04-AG1, not fixed here. |
| `talos-sandbox/src/supervisor.rs` | `pid_t`, `id_t`, `waitid(P_PID, WEXITED/WNOHANG/WNOWAIT)`, `siginfo_t::si_pid`, `setsid`, negative-PGID `kill`, `ECHILD/ESRCH/EPERM` | Stable 0.2 preserves the target-specific signatures, flags and `siginfo_t` accessor layout on Darwin and Linux musl x86_64/aarch64; existing ownership and fail-closed checks are unchanged. |
| Darwin-only `supervisor.rs` | `proc_listpids(u32,u32,*mut c_void,c_int)`, local `PROC_PGRP_ONLY=2`, `pid_t` buffer | Stable 0.2 signature matches the macOS SDK `libproc.h`; selector value matches `sys/proc_info.h`. Two `pid_t` entries provide eight bytes and the one-entry result remains four bytes. |
| Windows | Unix target dependencies and `cfg(unix)` callers | No Windows libc dependency or call path is introduced. |

Independent read-only Agent security/API review approved this static comparison for the
exact candidate and base `510dd64699846d44e553166eef3e1b43fa1683cb` on 2026-10-04.
This is not a runtime ABI proof or a substitute for locked CI/platform tests. Existing
group-escape limitations and ARCH-034-R04-AG1 remain unresolved.

### 2026-10-04 Validation Checkpoint

- PR #645 at `1d23edc545c215c8fbaae09f3723c601c004c866` / base
  `510dd64699846d44e553166eef3e1b43fa1683cb`: CI run `37179680972`
  passed its six jobs, including macOS workspace preflight, Linux Desktop and Windows workspace.
  A later head containing this checkpoint requires fresh exact-head CI/review binding.
- Local `cargo check --workspace --all-targets --all-features --locked` passed; focused
  `talos-sandbox` and `talos-tools` tests passed (41 each, plus two sandbox doc tests).
  Both governance validators passed with zero warnings.
- The all-target/all-feature Clippy gate initially exposed test-only lints in the affected
  packages. Those tests were updated to use descriptive `expect` diagnostics, and
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` now passes.
  No permission, process-hardening, or other production behavior was changed for these lint fixes.
- A focused all-feature test run for `talos-agent`, `talos-core`, `talos-text`, and `talos-tools`
  initially failed while linking with `No space left on device`; after removing only the
  regenerable `target/debug/incremental` cache, the rerun completed compilation and all selected
  tests except the opt-in `installed_language_plugins_drive_all_four_symbol_tools` test. That
  test requires `TALOS_LANGUAGE_BUNDLE_ROOT` pointing at real packaged Rust/Python Bundles and
  failed locally before exercising the migration. The missing fixture is an environment gate,
  not a passing test result; the full workspace test and packaged-Bundle validation remain open.
- Rust 1.95 is the declared workspace MSRV; stable libc 0.2.186 declares 1.65.
  A full workspace 1.95 build has not run because that toolchain is not installed locally.
  All-feature workspace tests and all explicit minimal/default/optional combinations
  are not yet evidenced. No acceptance exception is inferred from lack of evidence.

Independent review must confirm:

- the complete current Talos direct-libc call surface remains available with compatible signatures, constant values/types, symbol linkage and relevant struct/type layout on supported Unix targets, including `hardening.rs`, `process_boundary.rs`, and ADR-082 `supervisor.rs` (`waitid` / `siginfo_t`, process-group signaling, Darwin `proc_listpids` and related constants/types);
- changing only the dependency line cannot silently weaken ADR-007 resource-limit/environment/process-group behavior or ADR-082 owned-command supervisor/process-group cancellation guarantees;
- Windows continues to exclude the Unix libc dependency/call path through target cfgs;
- lockfile changes are explainable by the selected direct dependency change and do not smuggle unrelated upgrades;
- the external SDK fixture resolves a stable 0.2 Talos-direct libc path;
- ARCH-034-R04-AG1 remains explicitly unresolved rather than being represented as fixed by this dependency migration.

## Acceptance

Before implementation merge:

- both Talos direct Unix manifests use the stable `libc = "0.2"` line;
- no Talos direct dependency path resolves `libc 1.0.0-alpha.*`;
- root and independent fixture lockfile diffs are limited to consequences of the approved dependency change;
- the authoritative libc call-site/API/ABI matrix is refreshed from current `main` and rechecked against the actually resolved stable 0.2 release, explicitly including the ADR-082 supervisor surface;
- locked build/test evidence covers the supported macOS and Linux release targets, and Windows locked build/test evidence proves cfg isolation;
- `./scripts/release_preflight.sh` and the repository's current testing contract pass for the stable candidate;
- locked all-target/all-feature dependency validation passes, including workspace `cargo check` and Clippy with `--all-targets --all-features --locked`, plus all-feature workspace tests;
- affected minimal/default/optional feature combinations are exercised explicitly, including the `talos-tools` shell/coding paths that activate the Unix libc boundary;
- workspace/package MSRV compatibility is checked independently of the pinned development compiler, with the declared workspace MSRV (currently Rust 1.95) compared against the actually resolved stable libc dependency graph;
- `bash scripts/test_dependency_audit.sh` passes deterministically without live registry lookup;
- `cargo test --workspace --locked` and repository-required validation pass on the applicable CI targets;
- the external runtime SDK fixture passes its default and coding/shared-composition paths;
- an independent ADR-007 + ADR-082 native/escape-vector review approves the exact implementation candidate and covers the complete current direct-libc surface;
- no Rust process-hardening implementation is changed under this owner.


## Post-Merge Baseline Closeout

The implementation merge does not by itself make this owner Complete.

After the accepted libc 0.2 implementation exists in target `main` history:

1. regenerate `docs/reference/dependency-baseline.json` from that exact merged dependency tree using the repository snapshot workflow and record the source commit, UTC acceptance time and validation evidence;
2. verify the regenerated baseline records the stable Talos-direct libc requirement/resolution and contains no unexplained dependency drift;
3. audit the generated snapshot against the accepted tree;
4. perform owner-first closeout using the already-existing implementation SHA, then synchronize derived indexes/Board/Issue state.

Baseline generation must not run from an unmerged candidate and must not be represented as self-certifying completion. Until this closeout is merged, DEPENDENCY-003-A remains non-Complete even if the dependency implementation itself has merged.

## Rollback

If compatibility, ABI, platform, fixture, or protected-scope review fails, restore the pre-migration manifest and lockfile resolution before merge. Do not keep a partial 0.2 migration or compensate by weakening tests, cfgs, hardening or security expectations.

## Residuals

- ARCH-034-R04-AG1 remains separately owned and unclaimed unless independently activated.
- DEPENDENCY-003 continues to own the unrelated advisory/deprecation evidence-source residual after this child completes.
- Future libc 1.0 adoption requires fresh evidence and separately governed work; completion of this child does not pre-authorize it.

## Completion Evidence

- Completion Commit: pending

### 2026-10-04 Stable Candidate Local Convergence

- Current implementation head: `06fb5717e945bb326339b3dd6e51382a7dbfec0e`, rebased on
  `main` base `9582733cca444c3ccfe70faf9daff869757c7fcf`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo check --workspace --all-targets --all-features --locked`: passed, including the
  Rust 1.95 check.
- `cargo test --workspace --all-features --locked --quiet`: 430 passed, 0 failed, with real
  Rust/Python language Bundles supplied through `TALOS_LANGUAGE_BUNDLE_ROOT`.
- Explicit `talos-tools` minimal, `shell`, and `coding` feature test runs passed.
- `bash scripts/test_dependency_audit.sh`, both governance validators, and `git diff --check`
  passed. `./scripts/release_preflight.sh` passed, including the external runtime SDK fixture.
- The all-feature Clippy correction is test-only diagnostic wording; no production behavior,
  permission path, process hardening, or sandbox implementation changed.

This checkpoint is local evidence for the new exact candidate. Remote CI/review evidence from
the prior candidate is not reused and must be refreshed after pushing this head.
