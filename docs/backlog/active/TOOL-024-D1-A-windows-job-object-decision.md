# TOOL-024-D1-A: Windows Job Object Security And OS-ABI Decision

> Document status: Complete / Closed

| Field | Value |
|---|---|
| Story ID | TOOL-024-D1-A |
| Type | Architecture / Process-Security Decision |
| Priority | P1 |
| Status | Complete / Closed |
| Source | [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) |
| Selected Iteration | I225 Active / Claimed |
| Depends On | TOOL-024-C / I224 Complete; ADR-060 and ADR-057 Accepted |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 decision session 2026-08-24 |
| Work Slice | Decide only the Windows Job Object prerequisite: inventory the current Windows process path; define assigned-before-exec creation/assignment/resume, allowlisted child-handle inheritance, handle RAII, kill-on-close, nested-job and partial-failure semantics; select a bounded dependency/OS-ABI/`unsafe` boundary; freeze compatibility, migration, rollback, reversal triggers, D1-B authority inventory and Windows test matrix in ADR-068/current-path documentation. No Rust/Cargo/dependency, process behavior, Windows enablement, CLI/TUI, I223 execution, Dashboard/I213, permission, `/auto`, release, publication or Desktop change. |
| Claimed At | 2026-08-24 |
| Source Issue | #59 |
| Governance Claim PR | #388 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer's persistent active goal authorizes completion of Issue #59. I224 closed through reviewed PR #387 merge `3cb4eff8`; claim PR #388 exact head `e0c65c52`, CI `32729210800`, independent Windows/process/unsafe/API governance approval `5395556844`, merge-time CAS and merge `2afcdc3e` establish this claim on `main`. Shared GitHub account establishes Agent-role separation only, not natural-person identity separation. |
| Implementation PR | Not started |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | ADR-068 is Accepted on `main@0021690e`; later D1-B implementation needs a separate owner, iteration and effective claim. |

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
- Decide an allowlisted child-handle inheritance boundary for suspended/raw process creation and
  stdio integration in a multithreaded host. ADR-068 must select `STARTUPINFOEX` with
  `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` or an equivalently proven safe binding, inherit only the
  required child stdio handles, prevent unrelated inheritable-handle disclosure, close every
  parent/child duplicate on partial failure, and reject any design whose inheritance set cannot be
  proven.
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
- The decision and D1-B test matrix prove that only explicitly allowlisted child stdio handles are
  inherited during concurrent process creation; unrelated inheritable handles cannot leak, and all
  attribute-list/pipe/duplicate handles close on every partial-failure path.
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
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-agent/src/background_jobs.rs`
- `crates/talos-agent/src/process_tool.rs`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-agent/src/session.rs`
- `crates/talos-core/src/background_job.rs`
- `crates/talos-runtime/src/lib.rs`

## User-Facing Documentation

The decision must identify later D1-B and D2 documentation changes, but I225 changes no shipped
help or behavior. Windows remains explicitly unsupported for background start until D1-B completes.

## I225 Decision Execution Checkpoint (2026-08-25)

Decision work started from `main@e6980722` after claim PR #388 activation and owner-first sync PR
#389 preparation. Read-only source mapping is recorded in
`docs/reference/I225-WINDOWS-JOB-OBJECT-CURRENT-PATH-2026-08-25.md`. Proposed ADR-068 selects an
assigned-before-exec Job Object sequence, an allowlisted stdio handle-inheritance boundary,
kill-on-close descendant ownership, explicit partial-failure cleanup and fail-closed migration.
No Rust/Cargo/dependency/unsafe or Windows behavior changed. ADR-068 remains Proposed until its
decision PR receives exact-head CI and independent Windows/process/unsafe/API review.

## I225 Completion Checkpoint (2026-08-25)

ADR-068 decision PR #391 merged as `0021690e` from exact head `fca45c46`. Exact-head CI
`32797375011` was green and independent Windows/process/unsafe/API review `5404361120` approved the
decision. The decision remains documentation-only; Windows background admission is still fail-closed.

Completion Commit: `fca45c467466cd67b52d4391e88c776abfbea198`.

## Residual Destination

- TOOL-024-D1-B: Accepted ADR-068 implementation and real Windows child/grandchild evidence.
- TOOL-024-D2: CLI/TUI projection, user documentation and integrated Unix/Windows acceptance.
- I223 / Issue #378: deferred human/device evidence and final integrated validation.
