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
metadata successfully committed to a model-visible tool-result path suppresses a
redundant completion turn. Record the identity, not a heuristic text match. Failure
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
delivery without storage evidence. Queue bounds and output expiration remain
observable; saturation must not silently evict an eligible completion. Implementation
must define and test the exact bound and fail-closed diagnostic before acceptance.

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