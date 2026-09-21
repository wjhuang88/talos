# Iteration I280: Single-Direct-Dependency Runtime SDK

> Document status: Complete
> Published plan date: 2026-09-21
> Planned objective: Complete RUNTIME-006 / #234 without requiring another direct Talos dependency.
> MVP deliverable: An independent Cargo consumer implements a provider and tool, configures permissions and sandbox, submits a turn, consumes typed events and shuts down using only talos-runtime Talos imports.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 |
| Work Slice | RUNTIME-006 / #234 curated runtime SDK exports, external fixture, compatibility and user documentation. No execution-policy change. |
| Claimed At | 2026-09-21 |
| Source Issue | #234 |
| Governance Claim PR | #583 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested full #234 development closure on 2026-09-21; standing single-maintainer mode with independent Agent API/security review. |
| Implementation PR | #584 |
| Last Updated | 2026-09-21 |
| Handoff / Release Condition | Complete at implementation merge `007ca29d`; closeout `ade0b68c` records owner synchronization. |

## Published Baseline

### Inventory And Selection

- Main baseline: `1915857d68b783c321b83c36613ad61723e68c81`; remote refreshed 2026-09-21; no open PRs.
- I277 remains Review with deferred human/device acceptance, outside this SDK slice.
- I249 remains Planned/Unclaimed and unselected; I164 remains Paused/superseded.
- No other current Active or Blocked iteration; historical remediation headings do not activate work.
- I279 and the v0.10.0 release are complete. MODEL-007 remains Refinement/Unclaimed;
  the maintainer's explicit #234 request selects this slice first without cancelling MODEL-007.
- Selected Story: RUNTIME-006, parent ARCH-031; ADR-024 and ADR-052 govern composition.

### Design And Compatibility

- Add explicit, curated re-exports of canonical types, including the associated types needed to
  implement provider/tool/sandbox traits and consume typed session events. No glob facade or
  duplicate protocol types; keep existing imports and type identity compatible.
- Custom providers implement the facade-exported canonical trait. Built-in providers remain the
  optional `talos-provider` convenience crate; it is never mandatory for the runtime contract.
- Preserve minimal construction, permission checks, Deny precedence, approval fail-closed behavior
  and sandbox fallback defaults. No new dependencies merely to implement the facade.
- Additive public paths remain pre-1.0 semver-bound; document supported versus internal surfaces.
  Existing paths remain valid, so no breaking replacement or release is authorized.

### Scope And Acceptance

- Inventory public builder/handle and trait-signature type closure; expose the supported provider,
  tool, message/event, permission and sandbox composition paths through runtime.
- A standalone fixture declares exactly one direct Talos dependency, `talos-runtime`; third-party
  async/serialization dependencies are allowed. No direct internal Talos imports or dev-dependency
  leakage from the workspace may satisfy the fixture.
- Execute a deterministic provider/tool turn, assert typed events and results, and bounded shutdown.
- Exercise injected sandbox and permission configuration; assert denied writes do not execute and
  unavailable isolation does not silently bypass default policy. No credentials or live LLM needed.
- Update rustdoc, SDK contract, quickstart/examples, README and README.zh-CN SDK guidance, plus
  migration notes identifying the new supported import paths and retained legacy compatibility.
- Prove fixture manifest/import constraints mechanically and run it in an independent Cargo root.

### Validation

- Pinned Rust 1.97.0, locked focused runtime tests/examples, fixture check/run and documentation checks.
- `./scripts/release_preflight.sh` including locked workspace tests before stable implementation push.
- Both governance validators and `git diff --check`; governance-only claim does not run Rust tests.
- Independent Agent API/security review, exact-head applicable CI and merge-time CAS before merge.
- Owner-first closeout cites an existing implementation SHA, synchronizes Board/index and closes #234
  only after all acceptance passes. No release or Desktop acceptance is part of this task.

### Risks And Rollback

- Incomplete transitive signature coverage could leave hidden internal-crate dependencies; the
  external fixture and explicit public API audit must cover required associated types.
- Re-exporting internals accidentally broadens support promises; export named supported types only.
- Preserve execution code and defaults; revert additive paths before release if compatibility review
  finds a blocker. Already-published versions/tags remain immutable.

## Actual Activation And Execution

- 2026-09-21: Prepared governance-only claim proposal; not effective until finalized claim merges.
- 2026-09-21: #583 merged as `b553b9933c4bb9c20526168d25758c2c0694cc96`.
  Exact head `b9747bd99f89d06adf9a74f59b742f9c9158b4a5`, base
  `1915857d68b783c321b83c36613ad61723e68c81`; CI `35548998972` passed applicable
  docs jobs, independent Agent approval `5754011166`, merge-time CAS `5754026580`.
  Claim now effective; implementation branch starts at this merge. Existing external SDK fixture
  is multi-dependency and stale at 0.9.0; migrate it while retaining durability/shutdown coverage.

## Verification Evidence

- `cargo check --locked -p talos-runtime`: passed after initial explicit exports.
- `cargo check --locked -p talos-runtime --examples`: passed after facade import migration.
- Independent external default fixture: passed including typed tool-result/lifecycle assertions,
  permission/sandbox counter matrix and scoped-approval versus fallback separation. Later durability
  reopen and coding read assertions are included in the pending final preflight.
- External coding build exposed a missing `dep:talos-text` edge in `shared-composition`; fixed the
  existing optional-dependency feature declaration. Workspace lockfile unchanged. Fixture lockfile
  updated for its independent 0.10.0 local-source resolution; no production dependency upgrade.
- First full preflight reached test linking but failed with disk exhaustion (`os error 28`), not a
  passing test result. After all compiler processes exited, removed rebuildable incremental cache;
  retry uses `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 ./scripts/release_preflight.sh`.
- The retry also exhausted disk at test linking. With no live compiler processes, cleaned only
  rebuildable `target/debug` through `cargo clean --profile dev` (24.7 GiB). Final full-scope retry
  uses `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0
  CARGO_PROFILE_TEST_DEBUG=0 ./scripts/release_preflight.sh`; this disables debug symbols, not
  tests, debug assertions, permission checks or locked dependency validation.
- The debug-symbol-free run completed compilation and reached workspace tests, then the existing
  `talos-skill::tests::test_dedup_project_shadows_shared` was denied access to its HOME-based test
  directory by the execution sandbox. The same full preflight is being rerun with approved
  unrestricted execution; no test is ignored or weakened and the failure is not counted as a pass.
- Pre-submit API/security inspection by independent Agent `/root/durability_diagnosis` found no
  blocking defect. This is not exact-head approval; stable-commit review and remote CI remain required.
- Implementation remains local and incomplete until full preflight, final fixture modes and
  exact-head review/CI pass. No implementation completion claimed.

## Residuals

- Unrelated MODEL-007 and I277 deferred acceptance remain with their existing owners.
- Existing Windows I226 marker-readiness timing defect is deferred by the maintainer to the
  next requirement on 2026-09-21. A CI retry is not evidence that this defect is repaired.

## Stable Candidate Checkpoint (2026-09-21)

- Implementation #584: `dfe63ea65e2ca1cf18c460ba7843920aff42ce07`, base
  `b553b9933c4bb9c20526168d25758c2c0694cc96`. Earlier pending/local statements above
  describe intermediate execution; the full unrestricted preflight subsequently exited 0,
  including locked workspace validation and independent default/coding fixture execution.
- Quickstart executed successfully. Runtime rustdoc built with broken intra-doc links denied.
- Independent Agent API/security review approved that implementation head in #584 comment
  `5754547935`; shared-account Agent-role separation only, not independent human identity.
 - Delivery is Complete/Closed. Completion Commit: `007ca29df62fe63da9e8dda07865284b09f67bc6`.
   Closeout merge: `ade0b68cf5bc37cff81dabe10ab47b3994d484ca`.

## Implementation Changed-File Inventory

- SDK implementation: `crates/talos-runtime/src/lib.rs` (explicit canonical exports/rustdoc),
  `crates/talos-runtime/Cargo.toml` (existing optional text dependency enabled by its consumer feature).
- SDK examples: `crates/talos-runtime/examples/quickstart.rs`,
  `crates/talos-runtime/examples/custom_tool.rs`, `crates/talos-runtime/examples/approval.rs`,
  `crates/talos-runtime/examples/common/mod.rs` (facade paths; self-contained quickstart provider).
- Independent acceptance: `tests/fixtures/runtime-sdk-external/Cargo.toml`,
  `tests/fixtures/runtime-sdk-external/Cargo.lock`, `tests/fixtures/runtime-sdk-external/src/main.rs`,
  `tests/fixtures/runtime-sdk-external/src/provider.rs`,
  `tests/fixtures/runtime-sdk-external/src/safety.rs`,
  `tests/fixtures/runtime-sdk-external/src/api_surface.rs`.
- Regression entrypoint: `scripts/validate_runtime_sdk_fixture.py`, `scripts/release_preflight.sh`.
- User/API documentation: `README.md`, `README.zh-CN.md`,
  `docs/reference/RUNTIME-SDK-CONTRACT.md`, `docs/reference/I280-RUNTIME-FACADE-MIGRATION.md`.
- Owner-first execution records: this iteration, `docs/backlog/active/RUNTIME-006-single-dependency-sdk-facade.md`,
  `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, `docs/BOARD.md`,
  `.agent-governance/manifest.yaml`.
- No permission/sandbox implementation, CLI/TUI/Desktop production behavior, main workspace lockfile,
  release version or tag changes. Fixture-local dependency resolution is independent by design.
