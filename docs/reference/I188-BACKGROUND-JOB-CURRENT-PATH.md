# I188 Background Job Current-Path Characterization

**Date**: 2026-08-14

**Source baseline**: `556b5a4319085bf5250bccf4920e0dec0c6646c8` (`origin/main`)

**Purpose**: runnable/read-only evidence for TOOL-024-A. This document changes no runtime behavior.

## Result

Talos has hardened foreground command paths but no background-job owner. Unix children are detached
from the controlling terminal with `setsid`, yet timeout/cancellation kills only the direct child.
Windows has the same direct-child cleanup limitation. A late background completion also cannot be
represented as a second normal provider tool result without duplicating a tool-call ID. RUNTIME-005
and PERM-006-C are therefore real production prerequisites, not documentation dependencies.

## Ownership Matrix

| Stage | Current owner and source | Current behavior | Background gap / handoff |
|---|---|---|---|
| Typed shell input | `talos-tools/src/bash_tool.rs:30-39` | `command` plus foreground `timeout_secs`; no job intent or ID | Add default-false typed background field only in B. |
| Platform shell identity | `talos-tools/src/bash_tool.rs:59-83` | Windows `powershell.exe -NoLogo -NoProfile -NonInteractive -Command`; Unix `sh -c` | Preserve ADR-057 identity; never infer jobs from shell syntax. |
| Shell permission facet | `talos-tools/src/bash_tool.rs:307-320` | One Execute/Command resource | Background needs an additional explicit `background:` resource facet after PERM-006-C. |
| Shell spawn/hardening | `talos-tools/src/bash_tool.rs:130-189`; `process_boundary.rs:7-32` | Piped output, null stdin, dangerous-env scrub, Unix limits, Unix `setsid` | No supervisor registration or group termination. |
| Shell output | `talos-tools/src/bash_tool.rs:191-277` | stdout/stderr lines append to one unbounded `String` | Background capture must use a bounded ordered ring; current collector is not reusable. |
| Shell timeout | `talos-tools/src/bash_tool.rs:245-255` | Kill/wait direct child; intentionally stop waiting for descendant-held pipe EOF | Descendants can survive; whole-group cleanup required. |
| Typed exec input | `talos-tools/src/exec_tool.rs:52-80` | direct command, steps, pipes, mode, timeout; no job intent or ID | B accepts only one top-level Unix command in background; complex shapes fail before spawn. |
| Exec output bound | `talos-tools/src/exec_tool.rs:24`, `121-149`, `641-668` | 32 KiB retained independently for stdout and stderr | Useful bound precedent, but no cross-stream cursor or incremental reads. |
| Exec spawn/timeout | `talos-tools/src/exec_tool.rs:671-753`; `process_boundary.rs:7-32` | null/piped stdin, bounded readers, Unix `setsid`; timeout kills/waits direct child | No process-group cancellation or lifecycle owner. |
| Agent tool sequencing | `talos-agent/src/tool_execution.rs:235-345` | Tool executes inside the turn; result is projected, appended as `Message::Tool`, and emitted as `AgentEvent::ToolResult` | Initial job receipt can use this path; late completion cannot reuse it as a second provider result. |
| Permission evaluation | `talos-agent/src/tool_execution.rs:424+`; `talos-runtime/src/lib.rs:404-509` | Agent and runtime composition paths both evaluate/resolve permission today | PERM-006-C must establish one authoritative pipeline before spawn. |
| `always` grant | `talos-runtime/src/lib.rs:549-581`; CLI approval equivalents | Runtime adds nature/resource allow rules for facets | Background needs a distinct exact resource; a foreground grant must not be copied. |
| Runtime interrupt | `talos-runtime/src/lib.rs:375-381`; `talos-agent/src/session.rs` | Turn-scoped cancellation token interrupts the active turn | Esc/Ctrl+C must remain turn-scoped; job cancel is an explicit supervisor operation. |
| Session events | `talos-core/src/session.rs:84-184` | Non-exhaustive UI-neutral runtime event boundary exists | Add one live terminal-job event in B; do not auto-submit it. |
| Provider tool result | `talos-core/src/message.rs:26-46`, `228-233` | Every result carries the originating `tool_use_id` and enters message history | A second late result would duplicate the call; use `process` for later model reads. |
| Runtime shutdown | `talos-runtime/src/lib.rs:388-393` | Send `SessionOp::Shutdown`, then wait actor task with no public deadline/report | RUNTIME-005 must supply admission close, ordered finalizers, deadline, and report. |
| Persistence/resume | `talos-session`; current turn finalization | Normal turn messages/results persist; no job record/schema exists | B/C remain live-process-only; old IDs expire after restart. |
| CLI/TUI projection | existing `SessionEvent` consumers | Foreground tool lifecycle only | D projects job terminal/control events after shared runtime semantics exist. |

## Read-Only Reproduction Commands

Run from repository root at the recorded baseline:

```bash
rg -n "struct BashInput|struct ExecInput|background jobs are performed" \
  crates/talos-tools/src/bash_tool.rs crates/talos-tools/src/exec_tool.rs

rg -n "setsid|child.kill|descendant pipe|MAX_STREAM_BYTES" \
  crates/talos-tools/src/process_boundary.rs crates/talos-tools/src/bash_tool.rs \
  crates/talos-tools/src/exec_tool.rs

rg -n "Message::Tool|AgentEvent::ToolResult|tool_use_id" \
  crates/talos-agent/src/tool_execution.rs crates/talos-core/src/message.rs

rg -n "PermissionDecision::|AlwaysApprove|add_always_allow_rules" \
  crates/talos-agent/src/tool_execution.rs crates/talos-runtime/src/lib.rs \
  crates/talos-cli/src/approval.rs

rg -n "SessionOp::Shutdown|pub async fn shutdown|CancellationToken" \
  crates/talos-agent/src/session.rs crates/talos-agent/src/session/turn.rs \
  crates/talos-runtime/src/lib.rs
```

## Existing Deterministic Evidence

- `bash_tool::timeout_does_not_wait_for_descendant_pipe_eof` records that descendant-held stdout
  cannot extend the direct-child timeout. It is evidence of the residual, not proof of cleanup.
- `exec_tool` bounded-output tests prove the 32 KiB per-stream foreground cap and truncation marker.
- I191/ADR-007 prove controlling-terminal isolation through `setsid`; they do not signal or reap an
  entire process group.
- I170/ADR-057 explicitly retains Windows direct-child-only timeout cleanup and leaves Job Objects
  outside that iteration.
- I193/ADR-058 establishes durable finalization for interrupted turns; it does not add arbitrary
  resource finalizers or a job supervisor.

## Gate Disposition

| Gate | Evidence-backed disposition |
|---|---|
| TOOL-023-C | Complete at merge `592254d73a98166df48da0139a02df67e9cd2cd6`; shell identity is available. |
| TOOL-024-A / I188 | Effective claim merge `02a3558894a13204a28a48907fa39ca79a420d70`; this decision implementation is now active. |
| RUNTIME-005 | Refinement/Unclaimed; A → B → C must complete before production spawn. |
| PERM-006 | I189/A is Planned/Claimed but unactivated; A → B → C must complete before production spawn. |
| Unix process tree | Existing `setsid` makes group ownership feasible; ADR-060 proposes the narrow group-signal authorization for B. |
| Windows process tree | Still blocked; D requires a separately accepted Job Object/OS-ABI decision and real Windows evidence. |

## Conclusion

TOOL-024-B can be made implementation-ready as a Unix-only slice after ADR-060 acceptance and the
RUNTIME/PERM chains. Windows must fail closed until D. This is the smallest scope that satisfies
the Epic's “never unmanaged child” invariant without pretending that direct-child kill is enough.
