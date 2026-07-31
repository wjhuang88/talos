# TOOL-024: Background Command Jobs And Session Result Delivery

**Status**: Refinement (2026-07-29)
**Priority**: P1
**Type**: Epic
**Source**: [GitHub Issue #59](https://github.com/wjhuang88/talos/issues/59) and maintainer request —
long-running `bash`, Windows PowerShell, and `exec` work must not block an interactive conversation;
bounded completion output must remain model-readable through supervised job controls.
**Depends on**: TOOL-023-C for Windows PowerShell identity; TOOL-024-A decision output; RUNTIME-005
for bounded resource finalization; PERM-006-C before the final cross-surface permission integration.

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

## Required Invariants

- Background execution is opt-in through a typed tool input, never inferred from shell syntax such
  as `&`, `nohup`, or PowerShell jobs.
- `bash`/PowerShell and `exec` receive one consistent job identity, lifecycle, output-bound, timeout,
  cancellation, and session-delivery contract.
- A background start passes permission evaluation before spawning and must have distinct, visible
  background intent in the approval surface; foreground approval or an existing `always` grant must
  not silently authorize a longer-lived background execution.
- Every process is owned by a supervisor and reaches one terminal state: completed, failed, timed
  out, cancelled, or spawn/monitor failure. Terminal result delivery is exactly once.
- Output is bounded while running and at completion. The result records truncation without silently
  discarding the fact that output was truncated.
- User cancellation, `/quit`, Ctrl+C/Esc semantics, terminal restore, process-tree cleanup, and
  unexpected supervisor failure must have defined behavior before implementation.
- Windows support follows TOOL-023-C's PowerShell boundary; Unix support preserves the existing
  shell hardening boundary. No `unsafe`, Job Object implementation, or new dependency is authorized
  by this Epic.

## Children

| ID | Title | Type | Status | Depends On | Deliverable |
|---|---|---|---|---|---|
| TOOL-024-A | Background Job Lifecycle And Permission Contract Spike | Spike | Ready | None | ADR and implementation-ready contract for ownership, approval, cancellation, result delivery, and persistence. |
| TOOL-024-B | Managed Background Execution Core | Product/State Story | Blocked | TOOL-024-A Accepted; TOOL-023-C Complete | Supervisor, typed tool input/result, bounded capture, and exact-once terminal events. |
| TOOL-024-C | Interactive Job Projection And Control | Product/TUI Story | Blocked | TOOL-024-B Complete | Non-blocking TUI state, transcript projection, job status and cancellation controls. |

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

## Completion Condition

All children are Complete with existing implementation commit evidence, the lifecycle ADR is
Accepted, focused and workspace validation pass, and real Unix/Windows interactive walkthroughs
prove that background work does not block the conversation and returns exactly one bounded result.
