# Iteration I280: Single-Direct-Dependency Runtime SDK

> Document status: Active (proposed; effective only after #583 merges)
> Published plan date: 2026-09-21
> Planned objective: Complete RUNTIME-006 / #234 without requiring another direct Talos dependency.
> MVP deliverable: An independent Cargo consumer implements a provider and tool, configures permissions and sandbox, submits a turn, consumes typed events and shuts down using only talos-runtime Talos imports.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 |
| Work Slice | RUNTIME-006 / #234 curated runtime SDK exports, external fixture, compatibility and user documentation. No execution-policy change. |
| Claimed At | 2026-09-21 |
| Source Issue | #234 |
| Governance Claim PR | #583 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested full #234 development closure on 2026-09-21; standing single-maintainer mode with independent Agent API/security review. |
| Implementation PR | Not started |
| Last Updated | 2026-09-21 |
| Handoff / Release Condition | Finalize claim reference, applicable exact-head CI and independent Agent review before merge; implementation begins only after effective target-branch claim. |

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

## Verification Evidence

- Pending claim validation and implementation acceptance; no implementation completion claimed.

## Residuals

- Unrelated MODEL-007 and I277 deferred acceptance remain with their existing owners.
