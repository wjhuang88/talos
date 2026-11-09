# 100: Default Background Completion Continuation

## Status

Proposed — BG-CONTINUE-001. Number provisional until target-branch CAS.
No effective claim, implementation or approval is implied.

## Context

The user requests that background jobs wake the model by default, without another
user message or per-job opt-in. ADR-060 currently forbids automatic continuation.
This proposal supersedes only that product restriction once accepted; process
ownership, permission admission, bounded output and shutdown guarantees remain.

Current terminal events are presentation notifications. The Actor owns dispatch,
recovery fencing, pause, compaction and runtime admission. Reusing ordinary
SubmissionItem text would produce Message::User and misrepresent provenance.

## Proposed decision

### Ownership and triggering

The live Session Actor owns a bounded terminal-completion inbox fed by its existing
supervisor. Terminal UI notification remains independent. No TUI callback, global
bus, polling timer or detached per-job retry task starts model work.

Successful completion, nonzero exit, timeout and asynchronous supervision failure
qualify. Synchronous launch rejection/failure already returned in the launching
tool result does not qualify. User cancellation and shutdown never qualify.

After the active turn commits, reconcile eligible completions and dispatch through
the same recovery, pause, fork, compaction, budget and runtime-admission gates as
other turns. Never overlap provider turns. Already-queued user work has priority;
completion must not be injected into an active tool exchange.

### Identity and cancellation

Use originating generation and job ID as the completion identity. Admission tracks
pending/running/consumed state, with a cancellation epoch captured when the job is
reserved. Explicit Session cancellation invalidates earlier-epoch pending wakeups;
late terminal notifications cannot undo stop. Shutdown closes completion admission
before supervised cleanup. Stale generation events are ineligible.

A process response read by a host is not proof of model consumption. Only terminal
metadata acknowledged under the consumption decision below suppresses a redundant
completion turn. Record the identity, not a heuristic text match. Failure
to prepare a request retains bounded pending work and pauses rather than looping.
Once a completion turn starts, provider failure is recorded, not automatically retried.

### Context and trust

Use a distinct internal runtime origin and Message::Context for a fixed-format
bounded envelope: job identity, terminal state and exit status. No command strings,
stdout/stderr or cleanup-error text are promoted into this envelope. The model may
read output using process, subject to retention and ordinary tool admission.

Provider adapters may map Context to wire role user; this does not make the event
user-authored or authorize any operation. Preserve internal provenance and explicit
runtime-event labeling in the transcript. Never append another Tool result for the
original launch call. Tool permission checks remain unchanged.

### Compatibility and persistence

Do not add variants to public exhaustive SubmissionSource/SubmissionKind in this
slice. Prefer private Actor input and internal request preparation with explicit
input messages. Share existing turn finalization and admission rather than copy
an alternate execution loop. If private integration cannot preserve those gates,
stop and amend this ADR with public API/version migration before implementation.

Live process jobs do not survive restart. Already-started turns retain existing
interrupted-turn recovery protections. Pending live completion identities must not
be replayed as revived process jobs after restart. Do not claim durable completion
delivery without storage evidence.

### Bounded admission decision

Reserve one completion slot together with each background job reservation. Set a
private per-Session limit of 128 outstanding completion slots, counting reserved,
pending and running entries. A failed launch releases its slot. When all slots are
occupied, reject further background launches before spawn with a bounded diagnostic;
foreground execution remains unchanged. Do not discard an admitted terminal event
to make room for another job. This backpressure is separate from the existing
nonterminal-process limit and does not broaden spawn authorization.

Store only identity, generation, cancellation epoch, terminal state and optional
exit status in each slot; no output buffers. Use a coalescing Notify to wake the
Actor and inspect authoritative slot state, not a lossy channel as the only record.
Consumed/suppressed entries release their slots; terminal transitions are monotonic
and delayed notifications cannot reconstruct removed entries. Existing job-output
retention remains separate, so process reads may explicitly report expired output.

### Consumption decision

Track job identities in private turn-local metadata when a successful model-facing
process read/status result contains terminal state. List and host-only reads do not
acknowledge consumption. Reconcile that metadata only after the corresponding tool
result is included in a provider request whose response completes successfully;
record the consumption alongside the turn commit. Merely appending a tool result
before a failed provider request is not sufficient. A cancelled turn suppresses
older-epoch completion through cancellation rather than claiming it was consumed.
Tests must distinguish these cases. No string parsing of rendered tool output is
permitted as an identity or acknowledgement mechanism.

## Consequences

Default behavior consumes additional provider tokens and may initiate further
permission-checked actions within the original task. Document this behavior and
user stop semantics. No new opt-in flag, launcher, unsafe block or dependency.

## Acceptance evidence required

Actor tests cover idle/busy success, failure, timeout, asynchronous monitor failure,
synchronous launch failure, duplicate delivery, model-consumed terminal results,
host-only reads, cancel-before/after-admission, stale generation, shutdown, queue
saturation, output expiry, provider failure, recovery fence and permission denial.
Test wire tool-call/result pairing and transcript provenance. Real CLI smoke test
must continue after job exit without user input. Run pinned locked focused tests,
standard preflight, exact-head CI and independent runtime/security review.

Owner: docs/tasks/background-job-default-continuation.md.
User docs: README.md, README.zh-CN.md and applicable background-tool references.
No implementation or validation evidence exists for this proposed decision.