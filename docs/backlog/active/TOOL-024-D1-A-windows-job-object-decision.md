# TOOL-024-D1-A: Windows Job Object Security And OS-ABI Decision

> Document status: Planned / Unclaimed

| Field | Value |
|---|---|
| Story ID | TOOL-024-D1-A |
| Type | Architecture / Process-Security Decision |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I225 (proposed; inactive until an effective claim reaches `main`) |
| Depends On | TOOL-024-C / I224 Complete; ADR-060 and ADR-057 Accepted |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #59 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Merge an independently reviewed atomic I225 claim before decision work; later D1-B implementation needs a separate owner, iteration and effective claim. |

## Identity / Goal / Value

Decide the smallest Windows Job Object and process-creation boundary that can place a newly created
PowerShell or single-exec background process under Talos ownership before any user code runs, keep
kill-on-close ownership for its descendants, and fail closed on every uncertain setup path.

This is a prerequisite decision, not Windows enablement. Its output makes a later D1-B
implementation runnable without weakening ADR-060's no-unmanaged-child invariant.

## Required Decision Surface

- Inventory the current Windows foreground/background spawn, pipe, timeout, cancellation,
  supervisor and shutdown paths after I224.
- Compare viable Rust-native Windows API bindings and exact feature/dependency closure; select the
  smallest maintainable boundary and record license, MSRV and public-API impact.
- Decide an assigned-before-exec process-creation sequence. The contract must address suspended
  creation, Job Object creation/configuration, assignment, primary-thread resume, async stdout/
  stderr/wait integration and ownership transfer.
- Define handle ownership and RAII for process, primary thread and Job Object handles, including
  partial-construction cleanup and exactly-once close/terminate behavior.
- Define `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, explicit cancel/timeout/shutdown behavior, and how
  child/grandchild termination is proven without `taskkill` or shell-job fallbacks.
- Define nested/existing Job Object behavior, assignment and resume failures, unsupported Windows
  versions/configurations, channel closure, deadline races and supervisor-finalizer failures. Every
  uncertain path must terminate the suspended child when one exists and return a typed failure.
- Bound all `unsafe`/OS-ABI sites, required access rights, checked conversions and error handling;
  no wider native surface is implied.
- Preserve foreground behavior, PowerShell identity, permission resources, public SDK types and
  Unix process-group behavior unless the decision explicitly records an additive compatibility
  seam.
- Produce the D1-B implementation file/authority inventory, Windows test matrix, migration,
  rollback and reversal triggers. Keep D2 CLI/TUI projection and I223 evidence cleanup separate.

## Non-Goals

- No Rust, Cargo, dependency, lockfile, build-script or executable behavior change.
- No Windows background spawn, Job Object implementation, `unsafe` block or public API change.
- No CLI/TUI projection, provider continuation, persistence, scheduler, PTY/stdin or restart
  survival.
- No Dashboard/I213, `/auto`, PERM-006-D/E, release, publication, Desktop or unrelated work.
- No acceptance claim based only on Unix tests or documentation.

## Acceptance

- A current-path and authority matrix identifies every Windows creation, ownership, wait,
  cancellation, output and shutdown seam affected by D1-B.
- Proposed ADR-068 selects one assigned-before-exec design and rejects race-prone alternatives,
  including spawn-then-assign, direct-child-only kill and `taskkill`.
- The decision defines fail-closed behavior for every pre-assignment, post-assignment/pre-resume,
  resume, monitor, cancellation and teardown failure state.
- The decision defines a bounded `unsafe`/dependency contract, compatibility and migration plan,
  rollback path, exact D1-B authority inventory and independent Windows/process-security test
  matrix.
- Independent process/Windows/unsafe/API review binds the exact decision head; both governance
  validators, exact-head CI, YAML and diff checks pass.
- The accepted decision contains no production implementation and cannot be reused as D1-B or D2
  implementation authority.

## Required Reads

- `docs/decisions/060-supervised-background-command-jobs.md`
- `docs/decisions/057-windows-powershell-process-boundary.md`
- `docs/decisions/007-process-hardening-unsafe.md`
- `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`
- `docs/reference/I188-BACKGROUND-JOB-CURRENT-PATH.md`
- `docs/backlog/active/TOOL-024-background-command-jobs.md`
- `docs/backlog/active/TOOL-024-B-managed-background-execution-core.md`
- `docs/backlog/active/TOOL-024-C-model-readable-process-job-control.md`
- `crates/talos-tools/src/process_boundary.rs`
- `crates/talos-tools/src/background_jobs.rs`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`

## User-Facing Documentation

The decision must identify later D1-B and D2 documentation changes, but I225 changes no shipped
help or behavior. Windows remains explicitly unsupported for background start until D1-B completes.

## Residual Destination

- TOOL-024-D1-B: Accepted ADR-068 implementation and real Windows child/grandchild evidence.
- TOOL-024-D2: CLI/TUI projection, user documentation and integrated Unix/Windows acceptance.
- I223 / Issue #378: deferred human/device evidence and final integrated validation.
