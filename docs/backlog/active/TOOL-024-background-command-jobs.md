# TOOL-024: Background Command Jobs And Session Result Delivery

**Status**: Partial (A/B/C/D1-A/D1-B Complete; D2 Review; deferred validation remains)
**Priority**: P1
**Type**: Epic
**Source**: [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) and maintainer request —
long-running `bash`, Windows PowerShell, and `exec` work must not block an interactive conversation;
bounded completion output must remain model-readable through supervised job controls.
**Depends on**: Completed TOOL-023-C for Windows PowerShell identity, TOOL-024-A decision output,
RUNTIME-005 bounded resource finalization and PERM-006-C permission orchestration. Production work
still requires a separate child owner, runnable iteration and effective claim.

## Outcome

Talos can start an explicitly requested command as a managed background job, keep the interactive
conversation usable, and surface a bounded, attributable terminal result back into the live
conversation when that job exits or fails. The feature must never create an unmanaged child process
or silently broaden command permissions.

## Product Boundary

This Epic is deliberately narrower than a durable autonomous task runtime:

- A background job belongs to the currently running Talos process and its live session.
- Its result is delivered as a normal, explicitly identified tool-result event in that conversation.
- Restart survival, cross-session task queues, scheduling, retry policy, remote workers, and
  unattended execution are out of scope; those belong to `TASK-001`.
- The initial result delivery must not automatically initiate another model turn. Whether a later
  explicit "resume the model on completion" policy is desired is a separate product decision.
- A model can inspect and control existing jobs through one bounded `process` tool contract; the
  terminal result is not smuggled into a later provider request as an unsolicited tool result.

## Required Invariants

- Background execution is opt-in through a typed tool input, never inferred from shell syntax such
  as `&`, `nohup`, or PowerShell jobs.
- `bash`/PowerShell and `exec` receive one consistent job identity, lifecycle, output-bound, timeout,
  cancellation, and session-delivery contract. Delivery is staged: B enables Unix shell/single-exec
  only, while Windows remains fail-closed until D's Job Object gate.
- A background start passes permission evaluation before spawning and must have distinct, visible
  background intent in the approval surface; foreground approval or an existing `always` grant must
  not silently authorize a longer-lived background execution.
- Every process is owned by a supervisor and reaches one terminal state: completed, failed, timed
  out, cancelled, or spawn/monitor failure. Terminal result delivery is exactly once.
- Output is bounded while running and at completion. The result records truncation without silently
  discarding the fact that output was truncated.
- User cancellation, `/quit`, Ctrl+C/Esc semantics, terminal restore, process-tree cleanup, and
  unexpected supervisor failure must have defined behavior before implementation.
- `process` read/status/list/cancel operations use stable job identity, ordered cursor-based output
  reads and explicit truncation/expiry semantics. They do not accept arbitrary shell syntax.
- Windows support follows TOOL-023-C's PowerShell identity but not its direct-child-only cleanup.
  Unix support preserves the existing shell hardening/`setsid` boundary. ADR-060 proposes the only
  narrow future Unix group-signal `unsafe` authorization; Windows Job Object code, wider `unsafe`,
  and new dependencies remain unauthorized until a separately accepted D decision.

## Children

| ID | Title | Type | Status | Depends On | Deliverable |
|---|---|---|---|---|---|
| TOOL-024-A | Background Job Lifecycle And Permission Contract Spike | Spike | Complete / I188 / PR #228 | None | Accepted ADR-060 and current-path matrix for ownership, approval, cancellation, result delivery, and persistence; Completion Commit `245eddeb`. |
| [TOOL-024-B](TOOL-024-B-managed-background-execution-core.md) | Managed Background Execution Core | Product/State Story | Complete / Closed; PR #382 merged | TOOL-024-A Accepted; TOOL-023-C Complete; RUNTIME-005 Complete; PERM-006-C Complete | Unix session-owned supervisor, explicit non-daemonizing shell/single-exec background input, bounded capture, same-group cleanup, and exact-once terminal state; Windows fails closed. |
| [TOOL-024-C](TOOL-024-C-model-readable-process-job-control.md) | Model-Readable Process Job Control | Product/Tool Story | Complete / Closed; PR #386 merged | TOOL-024-B Complete | Bounded `process` read/status/list/cancel operations with stable identity and ordered cursors. |
| [TOOL-024-D1-A](TOOL-024-D1-A-windows-job-object-decision.md) | Windows Job Object Security And OS-ABI Decision | Architecture/Process-Security Decision | Complete / Closed | TOOL-024-C Complete; ADR-060/057 Accepted | Current-path matrix and Accepted ADR-068 define assigned-before-exec ownership, bounded OS-ABI, migration, rollback and D1-B test contract; no behavior change. |
| [TOOL-024-D1-B](TOOL-024-D1-B-windows-job-object-ownership.md) | Windows Job Object Process-Tree Ownership | Product/Process-Security Story | Complete / Closed | TOOL-024-D1-A Complete; ADR-068 Accepted | PR #394 merged as `d4d7cb25`; exact candidate `83557863` passed CI `32849330531` and independent Windows/process/security/API review `5410840103`. |
| [TOOL-024-D2](TOOL-024-D2-interactive-projection-and-acceptance.md) | Interactive Projection And Cross-Platform Acceptance | Product/TUI Story | Review / Claimed | TOOL-024-D1-B Complete | I228 owns CLI/TUI projection, user/model docs and integrated Unix/Windows acceptance; no supervisor, permission, Dashboard or `/auto` authority. |

## Major Risks

- A detached child can outlive Talos with no cancel path or result owner.
- Returning a background result directly to a model can unexpectedly spend tokens or start work the
  user did not request.
- Shell background syntax is interpreter-specific and bypasses consistent lifecycle control.
- Process-tree cleanup differs between Unix and Windows; pretending the current single-child kill is
  sufficient would leak descendants.
- Durable-session records can duplicate a late result after resume unless job/result identity and
  exactly-once delivery are designed together.

## Required Reads

- `docs/backlog/active/TASK-001-persistent-task-runtime-spike.md`
- `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md`
- `docs/backlog/active/TOOL-023-C-windows-powershell.md`
- `docs/decisions/007-process-hardening-unsafe.md`
- `docs/decisions/012-exec-policy-dsl-boundary.md`
- `docs/decisions/040-command-access-evidence-sandbox.md`
- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-conversation/src/`
- `crates/talos-session/src/`

## Current Implementation Baseline (2026-08-14)

- `ExecInput` and `BashInput` expose bounded foreground timeouts but no explicit
  background flag or job identity. The exec tool documentation explicitly says
  that background jobs are not performed.
- No `ProcessJobManager`, model-readable `process` tool, ordered output cursor or
  background cancellation/reap lifecycle exists in the workspace.
- Existing direct-child timeout behavior and Windows shell identity are inputs;
  neither proves descendant supervision or background ownership.
- TOOL-023-C is Complete. Claim PR #196 merged as `02a35588`; I188/TOOL-024-A decision artifacts
  are under independent exact-head review in PR #228.
- The current-path matrix proves Unix `setsid` is terminal isolation rather than whole-tree cleanup;
  Windows retains ADR-057's direct-child-only residual. ADR-060 therefore keeps Windows spawn
  fail-closed until D rather than weakening the no-unmanaged-child invariant.
- RUNTIME-005 remains Refinement/Unclaimed. PERM-006-A/I189 is Planned/Claimed but unactivated;
  both ordered chains still gate production B.

## TOOL-024-A Acceptance Checkpoint (2026-08-17)

- I188/TOOL-024-A is Complete at Completion Commit: `245eddebae762d1d0c7ee796baea50d0bb080bd5`;
  PR #228 merged as
  `1db1211e2fedeab277db366c3c76db0239691732` from independently reviewed exact head `d7d4fe7a`.
- ADR-060 is Accepted as a decision contract only. No production background process, tool/API,
  permission-policy, persistence, dependency, `unsafe`, Desktop or Dashboard behavior was added.
- TOOL-024-B is Complete / Closed through PR #382 merge `8671edf45c168612bfa4a4bbb65a9847026e1b96`. TOOL-024-C remains
  blocked on B, and Windows spawn remains fail-closed until D's separate Job Object/OS-ABI gate.

## Completion Condition

All children A-D are Complete with existing implementation commit evidence, the lifecycle ADR is
Accepted, focused and workspace validation pass, and real Unix/Windows interactive walkthroughs
prove that background work does not block the conversation, every job is reaped, and the model can
retrieve exactly one bounded terminal result through the documented `process` contract.

## RUNTIME-005 Dependency Completion Checkpoint (2026-08-21)

RUNTIME-005 A/B/C is Complete/Closed through Completion Commits `6719c876`, `c123328d` and
`44e840d7`; C implementation PR #348 merged as `6e5fa8c3` after exact-head CI and independent
runtime architecture review. This clears the runtime half of TOOL-024-B's prerequisite conjunction
only. TOOL-024-B remains Blocked because PERM-006-C is incomplete and no separate B claim or
implementation PR exists. TOOL-024-C remains blocked on B, Windows spawn remains fail-closed until
D, and Issue #59 stays open.

## Decision Links

- [ADR-060: Supervised Background Command Job Lifecycle](../../decisions/060-supervised-background-command-jobs.md)
- [I188 current-path characterization](../../reference/I188-BACKGROUND-JOB-CURRENT-PATH.md)

## PERM-006-C Dependency Completion Checkpoint (2026-08-23)

PERM-006-C / I221 completed at implementation commit `49d1546c`; PR #376 merged as `f9e6706d`
after exact-head CI `32640691772`, independent permission/security/API approval `5386153429` and
merge-time CAS. All recorded TOOL-024-B prerequisites are therefore Complete, so B is Ready /
Unclaimed rather than Blocked. This readiness does not activate work: production B still requires
its own child owner, runnable/testable iteration, effective Collaboration Claim, implementation PR,
security review and exact-head evidence. C/D remain blocked in order, Windows spawn remains
fail-closed until D, and Issue #59 stays open.

## Issue #59 Long-Task Planning Checkpoint (2026-08-23)

The separate TOOL-024-B owner, I222 iteration and Issue #59 long-task record are finalized in claim
PR #379 from `main@e1c375e6`; I223 and Issue #378 own deferred human/device evidence. B/I222 is
Active/Claimed only as a proposal and has no implementation authority until #379 merges. Maintainer
authorization `5386904546` permits only the exact non-overlapping I213/I222-B pair with stable
changed-file inventory and CAS gates; C/D cannot reuse it. The B contract excludes
self-daemonizing/detached commands and documents ADR-060's 32-terminal oldest-first/session-end
retention policy rather than adding an unapproved clock TTL.

## TOOL-024-B Activation Checkpoint (2026-08-24)

Claim PR #379 exact head `5f0816aa` passed CI `32650593056`, independent Agent-role claim review
`5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. B/I222 is now
Active/Claimed. Implementation starts from that merge or later `main`; the authorization remains
limited to the exact I213/I222-B non-overlapping pair and does not activate C/D or Windows spawn.

## TOOL-024-B Completion Checkpoint (2026-08-24)

Implementation PR #382 was merged into `main` as `8671edf45c168612bfa4a4bbb65a9847026e1b96`.
The I222 and TOOL-024-B owners record that pre-existing merge as their Completion Commit. Exact-head
CI `32690533253` passed 5/5 for head `01aa8b6a`; independent process/permission/unsafe/API review
approved the implementation at `fc28d821`, and the final governance-only exact-head review approved
`01aa8b6a` with both validators passing. TOOL-024-C/D and I223 remain separately governed.

The preceding checkpoint is a preserved pre-C-merge record. TOOL-024-C is now Complete / Closed
under the completion checkpoint below; D1/D2 and I223 remain separately governed.

## TOOL-024-C Completion Checkpoint (2026-08-24)

Implementation PR #386 exact head `d42c060d618e61218c4c1efe0651e74830807256` was based on
`ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`, passed exact-head CI `32719779528` 5/5, and received
independent permission/security/API/process approval `5394777902`. It merged as
`60b0367cf749397bf1167e189e820e82e32baf03` after merge-time CAS. TOOL-024-C/I224 is now
Complete / Closed. TOOL-024-D1/D2 and I223 remain separately governed, Windows remains fail-closed,
and Issue #59 remains open until the remaining child and deferred-validation evidence is complete.

## TOOL-024-D1-A Claim Preparation Checkpoint (2026-08-24)

I224/TOOL-024-C is Complete/Closed through owner-first closeout PR #387 merge `3cb4eff8`. A fresh
inventory found no D1 owner, ADR-068 or competing implementation PR, so TOOL-024-D1-A and I225 now
own only the prerequisite Windows Job Object security/OS-ABI decision. Claim PR #388 merged as
`2afcdc3e` after exact-head review/CI/CAS; decision work starts from that merge or later `main`.

This decision slice changes no Rust, Cargo, dependency, `unsafe`, Windows process behavior,
CLI/TUI, Dashboard/I213, `/auto`, release or publication surface. D1-B implementation, D2 projection
and I223/Issue #378 remain separate and unauthorized; Windows background mode stays fail-closed.

## TOOL-024-D1-A Decision Candidate Checkpoint (2026-08-25)

Proposed ADR-068 and the I225 current-path/migration matrix now define the Windows ownership
contract without enabling Windows background execution. D1-B must create/configure a Job Object and
allowlisted stdio handle set while suspended, assign before resume, use kill-on-close, and fail
closed with complete partial-failure cleanup. Exact decision review and acceptance remain pending.

## I228 Activation And Local Candidate Checkpoint (2026-08-26)

Claim PR #402 merged as `da9a79cd`, making I228 effective on the current `main`. The local D2
candidate is in `Review / Claimed` and covers only CLI/TUI projection, terminal-event delivery,
display-safe bounded summaries and SDK guidance. The candidate preserves the foreground path and
does not change supervisor, permission, Job Object, Dashboard, persistence, `/auto` or release
behavior. A stable implementation PR and exact-head protected validation remain required.
