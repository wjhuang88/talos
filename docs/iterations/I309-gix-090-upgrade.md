# Iteration I309: Upgrade gix To Latest Stable 0.90.0

> Document status: Active (claim #774 effective at 2789b97bfe807f486778f3b44c7053e2a0e6e13d)
> Published plan date: 2026-10-26
> Planned objective: upgrade gix 0.87.1 to the latest observed stable, non-yanked 0.90.0 while preserving existing Git behavior and permission boundaries.
> Baseline rule: preserve this target; append execution facts rather than rewriting acceptance.
> MVP deliverable: existing embedded Git status/diff/revision/worktree operations run against gix 0.90.0 with validated locked dependency resolution.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | I309: gix 0.87.1 to 0.90.0 dependency graph, necessary consumer migrations, Git regression validation and dependency documentation; excludes Auto policy and search/provider implementation. |
| Claimed At | 2026-10-26 |
| Source Issue | None; explicit maintainer request in handoff session |
| Governance Claim PR | #774 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer explicitly requested latest gix upgrade; claim requires exact-head review, applicable CI, both validators and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Verified upgrade and existing implementation commit evidence; no release authorized. |

Claim and activation became effective when governance-only #774 merged at
`2789b97bfe807f486778f3b44c7053e2a0e6e13d`. Implementation follows that record;
the published governance candidate itself contained no dependency/source implementation.

## Selection Inventory And Disposition

Inventory at main `881a27b890e5310055380e6e3f5b0da78f9fdd3c`:

| Owner | State | Disposition |
|---|---|---|
| I249 | Planned | Retain broad dependency-pilot deferral; I309 selects only the newly requested gix upgrade, not that pilot. |
| I277 | Review | Retain deferred human/device acceptance. |
| I293 / I294 | Active | Retain search transport/provider evidence ownership; no search changes. |
| I302 | Review | Retain Windows native-resource residual #717. |
| I306 | Active | Retain provider protocol ownership; no provider changes. |
| I308 | Active / Partial | Retain T8 scope and stable Runtime candidate; dependency upgrades are excluded from its baseline. I309 runs as the explicitly requested independent dependency stage, without changing I308 policy. |
| I164 | Paused | Retain superseded startup target. |
| I300 | Complete | Preserve T7 completion and historical gix observations; do not reopen it. |

Open PRs #767 (search), #749 (provider), #705 (Windows diagnostics) do not own the gix graph.
Refresh this check before merge. Sequence source work after the Runtime candidate is stable;
do not mix changes or claim its old validation covers upgraded dependencies.

## Published Baseline

### Selected Work

Standalone maintainer-requested dependency upgrade; no new product feature or host-Git replacement.

### Scope

- Upgrade the tools optional gix declaration and affected root/fixture locks without deleting locks.
- Preserve `default-features = false` and existing basic/status/revision/blob-diff/index/sha1/sha256 feature intent.
- Migrate only APIs required by the new version; inspect feature/native/license/advisory/duplicate changes.
- Verify crate MSRV (workspace 1.95) independently from pinned development compiler 1.99.0.
- Update operator-facing Git/dependency documentation with actual results and remaining host-Git boundary.

### Non-Goals

- No removal date or claim that gix replaces host Git, new unsafe code, new direct dependency, Auto-policy redesign, search/provider changes, release or tag.
- No upgrade of unrelated direct dependencies or advancement of an accepted audit baseline before implementation merge.

### Acceptance

1. Manifest and locked resolution use observed latest stable gix 0.90.0; all consumer and SDK fixture locks remain coherent.
2. Existing status, diff, revision and linked-worktree fixtures pass; SHA-1/SHA-256 coverage and structured Git permission behavior remain intact.
3. Default/minimal/git feature combinations, required workspace/all-feature checks and regression suites pass. Windows cross-build uses cargo xwin; native CI is distinct evidence.
4. Exact graph, MSRV, feature/native/license/advisory findings and rollback are recorded honestly; unknown advisory coverage stays unknown.
5. Independent security/API review, exact-head CI and merge CAS pass; owner completion cites an already-existing implementation SHA.

### Planned Validation

- `bash scripts/dependency_audit.sh --format json` before/after; `bash scripts/test_dependency_audit.sh`.
- `./scripts/release_preflight.sh` with locked workspace checks/test contracts.
- `cargo check --workspace --all-targets --all-features --locked`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- `cargo test --workspace --all-features --locked`.
- Affected tools no-default/default/git feature checks/tests; runtime SDK fixture; cargo xwin Windows check and native platform CI.
- Both governance validators and staged-diff inspection; domain regression coverage above.

### Documentation Targets

- This owner, `docs/iterations/README.md`, `docs/BOARD.md` and the existing Git user/reference documentation identified during implementation.

### Risk And Rollback

Pre-1.0 minor releases may break APIs and expand transitive graph. Keep dependency resolution
and necessary source migrations separately reviewable/revertible; revert the upgrade stage to
restore 0.87.1 without altering T7/T8 policy. Do not conceal a failed validation by narrowing scope.

## Evidence And Execution

- Initial observation: clean main source `881a27b890e5310055380e6e3f5b0da78f9fdd3c`; current manifest/lock gix 0.87.1.
- crates.io sparse index `https://index.crates.io/3/g/gix`, fetched 2026-10-26: latest stable non-yanked 0.90.0, declared Rust 1.88. This is registry evidence, not a validated graph/MSRV result.
- Upstream release: https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.90.0 (2026-10-22).
- Dependency graph migration, runtime validation and completion remain unimplemented.


### Initial implementation checkpoint — 2026-10-26

- Claim #774 merged at `2789b97bf`; implementation branch starts from that main commit.
- Both direct consumers (`talos-tools` optional Git and `talos-tui`) now request 0.90.0; root and external SDK fixture locks resolve that version. Existing feature selections remain unchanged.
- Initial check found one upstream API rename: `gix::path::try_from_bstr` to `from_bstr`. Inspected gix-path 0.14.1 source: Unix byte preservation and non-Unix invalid-UTF-8 error remain, with no lossy conversion. After migration, locked tools Git + TUI check passes.
- Resolved non-gix changes: faster-hex 0.10.0→0.10.1, prodash 31.0.0→31.0.1, defmt 0.3.101 added, sha1/sha1-checked replaced by sha1dc 0.1.5. Downloaded changed manifests declare MSRV at most 1.88 (workspace 1.95); this metadata comparison is not an executed MSRV build. Changed package licenses are MIT, Apache-2.0 or dual MIT/Apache-2.0. sha1dc has no build dependency or native binding in its manifest. Full feature/native closure and advisory validation remain pending.
- Formatting passes. Git regression suite is running; no full upgrade completion, platform acceptance or accepted-baseline advancement is claimed.

### Broader validation checkpoint — 2026-10-27

- Standard locked release preflight passed (release preflight: passed): /private/tmp/i309-preflight-final.log. All-target/all-feature workspace check passed: /private/tmp/i309-all-feature-check.log. These do not prove all-feature test acceptance.
- All-feature workspace tests exited 101: installed_language_tool_obeys_production_agent_deny requires real packaged language Bundles and TALOS_LANGUAGE_BUNDLE_ROOT; the direct invocation did not provide them. Full all-feature test acceptance remains outstanding. Log: /private/tmp/i309-all-feature-test.log. Use the existing scripts/validate_language_plugins.sh packaging flow to provide actual bundles before rerunning; do not suppress this test.
- All-target/all-feature Clippy exited 101 at large_enum_variant in talos-cli event_loop and unused_io_amount in talos-tools HTTP fixture tests. These files are unchanged by this candidate, but baseline reproduction and resolution remain outstanding; no waiver or completion is claimed. Log: /private/tmp/i309-all-feature-clippy.log.
- The upgrade remains Partial: executed MSRV, Windows cross-build/native evidence, graph security review, final exact-head review/CI and merge are outstanding.

- Executed MSRV evidence: cargo +1.95.0 check -p talos-tools --no-default-features --features git -p talos-tui --locked passed exit 0 (1m00s), isolated target /private/tmp/talos-i309-msrv; log /private/tmp/i309-msrv.log. This establishes affected-consumer compilation on workspace MSRV, not native Windows execution.
- Parsed before/after audit records confirm both tools and TUI resolve gix 0.87.1 before and 0.90.0 after. Both offline reports explicitly retain security/deprecation unknown; equal file sizes did not mean equal contents.

### Validation repair and advisory lookup — 2026-10-27

- All-feature Clippy repairs are implemented locally but not yet validated: HTTP loopback fixtures now consume complete bounded request headers (16 KiB / five seconds), handling partial reads and premature EOF; the CLI event queue boxes its large approval-request variant without changing the request or responder. These are upgrade-validation prerequisites, not gix feature changes. Root I308 preflight is still running, so no concurrent compilation was started here.
- Queried upstream RustSec advisory-db at `3279c99911354c400e1d41e47c7d60e39258b147`, comparing all 56 new name/version lock entries against package advisory records. The 14 matching records all have a declared patched lower bound satisfied by the resolved version: faster-hex 0.10.1 (RUSTSEC-2026-0306); gix-attributes 0.37.1 (2024-0359); gix-date 0.18.1 (2025-0140); gix-fs 0.24.1 (2024-0350); gix-index 0.58.0 (2024-0348/0352); gix-path 0.14.1 (2024-0355/0367/0371); gix-ref 0.69.1 (2024-0351); gix-transport 0.62.0 (2023-0064, 2024-0335); gix-worktree 0.59.0 (2024-0349/0353). In particular, the upgrade's faster-hex 0.10.0→0.10.1 removes an x86/x86_64 AVX2 out-of-bounds read covered by the first advisory. Source: https://rustsec.org/advisories/RUSTSEC-2026-0306.html .
- Evidence: `/private/tmp/i309-changed-package-advisories.json` contains versions, full advisory headers and upstream database commit. This is a bounded manual changed-package comparison, not cargo-audit or an assertion about the full unchanged workspace graph; cargo-audit is not installed. No known matching unpatched advisory was identified for the changed entries in that database snapshot. Unknown vulnerabilities and full-graph audit coverage remain unknown.
- Independent same-session read-only dependency/security/API inspection found no demonstrated migration blocker, but did not approve delivery or execute builds. `sha1dc 0.1.5` is pure Rust (`build = false`, no production dependencies/native bindings), with dependency-owned unsafe SIMD loads/stores and CPU target dispatch. Pure Rust does not mean unsafe-free. No Talos-authored unsafe was added; ADR-007 is not evidence approving this dependency's internals. Source inspection is not a proof of SIMD soundness; final feature/native closure and platform testing remain required.

### Rebased validation repair — 2026-10-28

- Rebased implementation onto main `9d38a7365c494ade99a484bdd158476f850f4428`; upgrade commits are `6b9ec521a` and `990e01cd4`. This is not merged upgrade evidence.
- The earlier statement that both Clippy repairs were implemented locally was not durable: the rebased tree contained the merged HTTP header-read fix, but its CLI approval event was still unboxed. Current all-feature Clippy reproduced `large_enum_variant` (at least 360-byte enum); log `/private/tmp/i309-current-all-feature-clippy.log`. The CLI variant is now boxed at the channel boundary and unboxed when queued, preserving the original request/responder. Formatting and diff whitespace checks pass; full all-feature Clippy retry remains running, not passed.
- The pinned 1.99.0 WASM target is installed. Real Rust language guest compilation has started with Homebrew LLVM's WASM-capable Clang/llvm-ar; real packaged bundles and full all-feature test acceptance remain outstanding.
- Maintainer-requested `cargo clean` removed 19,327 files / 10.8 GiB before this retry; no worktree/source was removed. Only root and the active I309 worktree remain. Old validation logs are preserved, but old-base results do not certify this candidate.
- Dependency audit regression tests and both project/Collaboration Claim validators passed (exit 0): `/private/tmp/i309-current-audit-tests.log`, `/private/tmp/i309-current-governance.log`, `/private/tmp/i309-current-claims.log`.
- Both real language guests compiled and packaged successfully into `/private/tmp/i309-real-language-bundles` (not installed/activated): Rust SHA-256 `e681c7cc74e676a6b081df63a6233f56496ea6ae432757d2ccdd8f6894bde6e1`; Python `040bd1ab7c8fa07b0743b5bcf16293797b220570212a0691a95a65e0d204a1b6`. The first Rust packaging attempt failed because the task-specific parent directory did not exist; creating it and retrying succeeded. Bundle creation is a test prerequisite, not full all-feature test acceptance.
- The Clippy retry progressed past the CLI variant but failed at `field_reassign_with_default` in the existing config protocol test helper. Changed only helper construction to a struct initializer; assertions/provider semantics are unchanged. This is a validation prerequisite, not a provider feature change. Formatting and whitespace checks pass. Full all-feature tests with the real bundle root are running (log `/private/tmp/i309-current-all-feature-test.log`); final Clippy is queued behind Cargo's build lock (log `/private/tmp/i309-current-all-feature-clippy-final.log`). Neither is reported passed.
