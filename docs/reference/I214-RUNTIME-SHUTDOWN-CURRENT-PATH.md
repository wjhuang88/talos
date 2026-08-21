# I214 Runtime Shutdown Current-Path Characterization

**Date**: 2026-08-21

**Source baseline**: `14531bbc70db4e401b922cf68f8983d33e15ad46` (`main`, I214 activation)

**Purpose**: code-grounded evidence for RUNTIME-005-A/I214. This document changes no runtime,
Session, persistence or public API behavior.

## Result

The supported SDK currently has one consuming `RuntimeHandle::shutdown(self)` operation. It sends
the existing `SessionOp::Shutdown`, ignores a send failure, and waits without a deadline for the
Session actor task. The actor rejects its in-memory pending queue, pauses unstarted durable
submissions, cancels the active turn token and waits for the turn task to publish a terminal
record. ADR-058 finalization therefore provides a real active-turn durability seam.

There is no shared shutdown state, pre-send admission fence, caller-selected policy, total
deadline, finalizer registry or structured shutdown report. Concurrent/repeated shutdown callers
also cannot share a result because the only public operation consumes the sole runtime handle.

## Current Ownership Matrix

| Stage | Current owner and source | Current behavior | Missing contract / handoff |
|---|---|---|---|
| SDK construction | `crates/talos-runtime/src/lib.rs:436-459` | Builder restores durable history, constructs `AppServerSession`, injects durable persistence and spawns one actor task. | No shutdown coordinator or finalizer registry is constructed. |
| SDK handle ownership | `crates/talos-runtime/src/lib.rs:469-474` | One handle owns the command sender, event receiver and actor `JoinHandle`. | The handle is not cloneable; no separately cloneable shutdown observer/controller exists. |
| Compatibility admission | `crates/talos-runtime/src/lib.rs:476-494` | `submit` and `preview_request` await a send to the bounded Session queue and return `CommandChannelClosed` only when the receiver is closed. | A successful send is not a shutdown admission decision. There is no atomic fence between concurrent submit and shutdown calls. |
| Interrupt | `crates/talos-runtime/src/lib.rs:497-503` | Sends the legacy unqualified `SessionOp::Interrupt`. | No shutdown-specific wait/interrupt policy or accepted-plan arbitration exists. |
| Public shutdown | `crates/talos-runtime/src/lib.rs:510-515` | Consumes the handle, sends `SessionOp::Shutdown`, ignores send failure, awaits actor join, and returns only `Result<()>`. | No deadline, repeated/concurrent result, stage outcome or finalizer report. |
| Drop behavior | `crates/talos-runtime/src/lib.rs:353-360` and absence of a `Drop` implementation | Documentation prefers explicit shutdown. Dropping the handle drops its channels and detaches the Tokio task handle; the actor observes SQ closure after all senders are gone. | Drop is best-effort channel closure, not a bounded completion contract. It cannot truthfully report reconciliation or resource cleanup. |
| Session queue | `crates/talos-agent/src/session.rs:93-107`; `crates/talos-core/src/session.rs:21-82` | The SQ is a bounded channel of 512 public `SessionOp` values. `Shutdown` carries no policy, deadline or response channel. | Changing the serialized `SessionOp::Shutdown` shape would be a compatibility change; the SDK coordinator should preserve it as an internal actor signal. |
| Actor admission loop | `crates/talos-agent/src/session.rs:197-229`, `268-399` | While not shutting down, the actor can pop pending work before its next select receives `Shutdown`. | No actor-visible SDK fence prevents a just-finished active turn from starting the next pending item before the shutdown op is observed. |
| SQ disconnect | `crates/talos-agent/src/session.rs:268-280` | Closed SQ sets `shutting_down`, cancels the token, rejects in-memory pending work and attempts to pause unstarted durable work. | Errors are emitted only as live events; the caller receives no structured outcome. |
| Explicit actor shutdown | `crates/talos-agent/src/session.rs:463-476` | Sets `shutting_down`, cancels the active token, rejects in-memory pending work, pauses durable unstarted work and keeps only active size accounting. | Policy is always immediate cancellation. Pause failure does not change the public shutdown result. |
| Actor exit | `crates/talos-agent/src/session.rs:227-229` | The loop exits only after shutdown is set, the active turn is gone and the in-memory pending queue is empty. | No absolute deadline bounds the active turn or actor join. |
| Active-turn cancellation | `crates/talos-agent/src/session/turn.rs:139-210` | Cancellation aborts the provider/agent task, drains the latest stable snapshot, applies persistence projection and attempts an Error/Cancelled terminal record. | This is the correct ADR-058 seam for an interrupt policy, but timeout/failure is not surfaced to a shutdown report. |
| Turn completion arbitration | `crates/talos-agent/src/session.rs:231-267` | The actor joins the turn task, commits its returned messages, finalizes structured custody and pauses after non-success or custody failure. | Shutdown has no independent stage outcome for success, cancellation, panic, persistence failure or missing record. |
| Durable active-turn finalization | `crates/talos-agent/src/session/turn.rs:145-210`, `327-389`, `392-489` | Success, Error and Cancelled paths use the ADR-058 durable finalizer or compatible outcome record. Persistence failure becomes an Error completion. | Shutdown must consume this result rather than inventing a second persistence format or writer. |
| Durable pending custody | `crates/talos-agent/src/session/custody.rs:114-125`; `crates/talos-agent/src/session.rs:468-475` | In-memory pending items are rejected as `SessionClosed`; durable unstarted items are paused. | The pause result is not returned to SDK callers. A failed pause needs a redacted reconciliation status. |
| Restart reconciliation | `crates/talos-agent/src/session/custody.rs:18-54`; `crates/talos-agent/src/session.rs:169-172` | Actor startup examines Running submissions and reconciles them against durable transcript outcome, retaining ambiguous records conservatively. | Shutdown must report unresolved custody and leave startup reconciliation authoritative; it must not claim a fabricated terminal state. |
| Runtime-owned finalizers | no registry in `talos-runtime`, `talos-agent` Session or builder | None. | RUNTIME-005-C must add ordered, bounded, exactly-once finalization without a global event bus. |
| Background-job consumer | ADR-060 section 6; `docs/reference/I188-BACKGROUND-JOB-CURRENT-PATH.md` | No production supervisor exists. ADR-060 requires a later supervisor to consume RUNTIME-005 finalization. | TOOL-024 cannot own or precede the runtime coordinator, and remains blocked on completed RUNTIME-005 and PERM-006-C. |

## Existing Deterministic Evidence

| Evidence | What it proves | What it does not prove |
|---|---|---|
| `crates/talos-agent/src/session/tests.rs:1044-1066` | An idle actor receiving `SessionOp::Shutdown` exits within the test's two-second outer timeout. | No public deadline, report or active-turn/finalizer bound. |
| `crates/talos-agent/src/session/tests.rs:1068-1100` | EQ consumer loss does not prevent a simple submit/shutdown actor exit. | No durable reconciliation result reaches the caller. |
| `crates/talos-agent/tests/i169_durable_custody.rs:249-324` | Shutdown cancels one running structured submission and leaves two unstarted durable submissions `PausedPending`. | No shared shutdown arbitration or arbitrary resource cleanup. |
| ADR-058 and SESSION-008-B | One actor-owned atomic/idempotent terminal finalizer preserves a display-safe stable prefix. | No generic runtime finalizer registry or total shutdown deadline. |
| ADR-060 / I188 | A future background supervisor must register as a bounded finalizer consumer. | No production job spawn or runtime finalizer implementation. |

## Reproduction Commands

Run from repository root at the recorded baseline:

```bash
rg -n "pub struct RuntimeHandle|pub async fn shutdown|SessionOp::Shutdown" \
  crates/talos-runtime/src/lib.rs crates/talos-agent/src/session.rs

rg -n "shutting_down|pause_unstarted|release_in_memory_pending_on_shutdown" \
  crates/talos-agent/src/session.rs crates/talos-agent/src/session/custody.rs

rg -n "cancel_token.cancelled|agent_task.abort|finalize_turn|persist_terminal_outcome" \
  crates/talos-agent/src/session/turn.rs

rg -n "shutdown_pauses_unstarted_durable_submissions|test_shutdown" \
  crates/talos-agent/src/session/tests.rs crates/talos-agent/tests/i169_durable_custody.rs
```

## Gate Disposition

| Gate | Evidence-backed disposition |
|---|---|
| RUNTIME-005-A / I214 | Active decision-only slice. This matrix plus ADR-063 are its deliverable. |
| RUNTIME-005-B | Blocked until ADR-063 is accepted. It owns coordinator state, admission, active-turn policy, total deadline and structured report. |
| RUNTIME-005-C | Blocked until B completes. It owns ordered build-time finalizers, final durable reconciliation and compatibility closure. |
| PERM-006-A / I189 | Planned/Claimed and deliberately unactivated. No authority transfers from I214. |
| TOOL-024-B/C/D | Still blocked or unauthorized. A later supervisor may consume the completed C registry only after the existing permission and platform gates. |

## Conclusion

The current actor cancellation and ADR-058 persistence paths are reusable; replacing them would add
risk without closing a gap. RUNTIME-005 should add one SDK-owned coordinator around those paths,
not a second Session lifecycle or a TOOL-024-owned shutdown system.
