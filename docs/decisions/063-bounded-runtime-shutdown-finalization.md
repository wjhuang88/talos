# ADR-063: Bounded Runtime Shutdown And Finalizer Coordination

> Status: Proposed
> Date: 2026-08-21
> Owner: RUNTIME-005-A / I214
> Proposal evidence: content commit `648a35d3`; PR #338; exact final head, CI and independent
> architecture review pending.

## Context

`RuntimeHandle::shutdown(self)` currently sends `SessionOp::Shutdown` and waits without a deadline
for the Session actor. The actor immediately cancels an active turn, rejects its in-memory pending
queue, attempts to pause durable unstarted submissions and exits after the active turn returns a
terminal record. ADR-058 already makes the actor the sole turn-finalization and event-order owner.

This path is orderly but not a bounded SDK contract:

- concurrent or repeated callers cannot join one result because shutdown consumes the only handle;
- submit and shutdown have no shared admission fence;
- callers cannot choose whether an active turn gets a bounded chance to finish or is interrupted;
- no one total deadline covers turn resolution, durable reconciliation, finalizers and actor join;
- runtime-owned resources have no ordered finalizer registry;
- callers receive no redacted structured report when persistence, cleanup or join is incomplete.

ADR-060 also requires a future background-job supervisor to consume a generic runtime finalizer.
It cannot own that contract without creating the dependency cycle that RUNTIME-005 exists to avoid.

## Constraints

| Constraint | Type | Consequence |
|---|---|---|
| The Session actor owns turn arbitration and ADR-058 finalization. | Hard / ADR-039, ADR-058 | Shutdown coordinates the actor; it does not create another turn finalizer or durable writer. |
| The host owns process signals and application exit. | Hard / RUNTIME-005 | Talos exposes a bounded API and never installs process-signal handlers in the SDK. |
| Public crate APIs are semver-bound. | Hard / AGENTS.md | Add new types/methods; retain `RuntimeHandle::shutdown(self) -> RuntimeResult<()>` as a compatibility wrapper. |
| No global message bus. | Hard / ADR-006 | One runtime-local coordinator and existing Session channels carry the lifecycle. |
| Reports cannot leak model or tool data. | Hard / security boundary | Report only typed states, bounded timings/counts and fixed error categories. |
| A managed resource cannot outlive failed cleanup silently. | Hard for registered finalizers | Registration requires cancellation-safe containment; timeout is explicit and never reported as success. |
| TOOL-024 is downstream. | Hard / ADR-060 | RUNTIME-005 defines the registry without depending on a process supervisor or permission policy. |
| Thirty seconds for the legacy wrapper is a policy default. | Soft | It is documented and caller-overridable through the new structured API; changing it later is a policy change. |

## Decision

### 1. One runtime-local shutdown coordinator

RUNTIME-005-B adds one coordinator created by `RuntimeBuilder` and shared by the SDK handle, the
Session actor wrapper and a cloneable shutdown controller. The intended public shape is:

```text
RuntimeHandle::shutdown_controller(&self) -> RuntimeShutdownHandle
RuntimeShutdownHandle::shutdown(&self, options) -> RuntimeResult<ShutdownReport>
RuntimeHandle::shutdown_with(&self, options) -> RuntimeResult<ShutdownReport>
RuntimeHandle::shutdown(self) -> RuntimeResult<()>
```

Names may change during implementation only when the semantics and migration notes remain exact.
`ShutdownOptions` has private fields and is created only through a validating constructor or a
validated deserialization path. The structured methods borrow their handle and never accept an
unvalidated options draft; only the legacy compatibility wrapper consumes the primary handle.
`RuntimeShutdownHandle` is cloneable but exposes no submit, event-consumption or resource mutation
surface. A cancelled caller future does not cancel shutdown: after the first accepted request, a
runtime-owned driver completes the plan and caches its terminal report.

The coordinator state is monotonic:

```text
Open -> Closing(accepted plan) -> Closed(shared report)
```

No transition reopens admission. The terminal report is immutable and cloneable.

### 2. First valid request wins; every later caller joins

A request contains a total deadline and one active-turn policy. An unvalidated draft cannot be
passed to `shutdown_with`: `ShutdownOptions` construction validates the positive total deadline,
finish grace and policy relationship before it touches the coordinator or consumes the primary
handle. Its fields remain private, and any serde/config conversion performs the same validation.
Invalid construction therefore changes no runtime state and leaves the primary handle available;
it cannot accidentally invoke primary-handle Drop. A future API that accepts an unvalidated draft
must borrow a controller or return ownership on validation failure rather than consume the primary
handle.

The first submitted validated request that wins the `Open -> Closing` transition establishes the
policy and absolute monotonic deadline. `RuntimeHandle::shutdown_with(&self, options)` and the
controller method borrow their handles, so validation failure or a rejected request cannot trigger
primary-handle Drop. The consuming legacy `shutdown(self)` constructs the valid documented default,
joins the same structured path and cannot introduce an invalid-options branch.

Concurrent or repeated valid requests after `Closing`:

- cannot replace the policy;
- cannot shorten or extend the deadline;
- cannot run finalizers again;
- wait for and receive the same terminal report, including the same accepted-plan identifier and
  stage outcomes.

This is scheduling arbitration, not authorization. The accepted-plan identifier is a
runtime-generated opaque value and is never derived from prompt, tool or credential data.

### 3. One admission/start arbiter closes before the actor shutdown signal

RUNTIME-005-B adds one runtime-local admission/start arbiter shared by supported SDK admission,
the shutdown coordinator and the Session actor's pending-to-active transition. The winning
`Open -> Closing` transition while holding this arbiter is the shutdown fence linearization point.
It is not a separate SDK CAS followed by an actor check.

An SDK submission reserves bounded channel capacity before acquiring the arbiter. While holding
the arbiter it rechecks `Open` and commits the reserved send; if it observes `Closing` or `Closed`,
it drops the reservation and returns typed `RuntimeClosing` without enqueueing. It never awaits
channel capacity while holding the arbiter. A committed send is pre-fence even if the actor has not
yet received it.

The actor refactors start into preparation and a short start-commit step. Preparation may inspect
or build an internal turn plan but performs no externally visible model/tool work. At the final
commit point the actor acquires the same arbiter and, only while it remains `Open`, atomically moves
one item from Pending to `StartCommitted` and installs its cancellation token. It then releases the
arbiter before durable I/O, provider/tool work, task await or compaction. `StartCommitted` is
treated as active by shutdown: `Interrupt` cancels its installed token before external work can
begin, while `FinishCurrent` permits it within the accepted grace. Every asynchronous stage after
start commit, including compaction and provider/tool execution, observes that same token before and
during external work. A failure after start commit is terminalized through the existing ADR-058
actor path.

If shutdown wins the arbiter, no pending item can become `StartCommitted`; the item remains pending
for the actor's existing reject/pause handling. If the actor wins, the item is unambiguously active
before the fence. This removes the check-to-start gap without holding a lock across an await or
provider work. Direct lower-level Session construction outside the supported runtime facade retains
its current default-open control unless it explicitly installs this arbiter seam.

The implementation preserves the serialized public `SessionOp::Shutdown` shape. It must not add a
policy payload to that enum merely for the SDK coordinator. Already queued work is handled by the
Session actor's existing order: active work follows the selected policy, in-memory pending work is
rejected `SessionClosed`, and durable unstarted work becomes `PausedPending`.

If enqueue, actor start commit and shutdown race, the shared arbiter determines exactly one order.
Returning success from `submit` still means accepted by the SDK queue, not successful model
completion; user-facing documentation must keep that distinction.

### 4. Active-turn policy vocabulary

The public caller chooses one of two policies:

| Policy | Required behavior |
|---|---|
| `FinishCurrent { grace }` | Stop admission and give the already-active turn at most `grace` within the total deadline. If it does not reach an ADR-058 terminal result, issue the same actor-owned interrupt used below. Pending work never starts during the grace period. |
| `Interrupt` | Stop admission and immediately cancel the active turn through its Session cancellation token. The turn task aborts the provider/agent future, closes the latest display-safe stable prefix and invokes the ADR-058 Cancelled/Error finalizer. |

There is deliberately no caller-selectable `AbortWithoutFinalization` policy. Such a mode could
discard an admitted tool fact or bypass the sole durable finalizer. Forced task abort exists only
as final deadline containment after the actor-owned interrupt failed to finish; it is reported as
`Unreconciled`, never as Cancelled or clean shutdown.

`FinishCurrent.grace` must be less than the total deadline. The implementation reserves the
remaining interval for interrupt finalization, durable reconciliation and registered finalizers;
it does not let a finish grace silently redefine the total deadline.

### 5. One absolute monotonic deadline

The winning request converts its duration into one monotonic absolute deadline at the successful
state transition. Queueing the actor signal, finish grace, interrupt finalization, durable custody,
each finalizer and actor join all consume the same budget.

- A stage receives only the remaining time.
- A per-finalizer cap may shorten its own time but never extend the global deadline.
- Timeout, retry, panic handling or report assembly cannot reset the clock.
- Once no time remains, outstanding tasks are contained/aborted, unstarted finalizers are marked
  `NotRunDeadline`, and the terminal report is published promptly with `deadline_exhausted=true`.
- Wall-clock changes do not affect deadline accounting.

The compatibility wrapper uses `Interrupt` with a 30-second total deadline. The structured API
allows a caller to choose another positive bounded duration.

### 6. Ordered finalizer registry

RUNTIME-005-C adds a runtime-owned registry configured before `RuntimeBuilder::build`. Runtime
startup freezes it; registration after build is rejected. Each entry has:

```text
fixed static identifier + unique order + per-finalizer cap + cancellation-safe finalizer
```

Identifiers are code-owned constants, not caller-supplied display strings. Duplicate identifiers
or order values fail the build. Finalizers run exactly once in ascending order after active-turn
terminalization and durable pending reconciliation. A finalizer error or caught panic is recorded
and later finalizers still run while budget remains. A timed-out finalizer is cancelled and
contained; subsequent finalizers run only if global time remains.

Registration requires the finalizer implementation to prove that cancellation/drop leaves its
resource in a fail-closed contained state. A resource that cannot meet this rule cannot register
and must remain disabled. This is the integration gate a future TOOL-024 supervisor must satisfy;
RUNTIME-005 does not import, construct or authorize that supervisor.

### 7. Durable reconciliation and stage order

The coordinator uses this order:

1. close SDK admission and freeze the accepted plan;
2. resolve the active turn according to the selected policy through the Session actor;
3. record the active turn's existing ADR-058 terminal result and pause/reject unstarted custody;
4. run registered runtime-owned finalizers in frozen order;
5. join or contain the actor and publish one immutable report.

The Session actor remains the only owner of transcript entries, turn outcome markers and pending
submission state. Shutdown does not infer terminal success from entries, rewrite ambiguous Running
custody or replay a submission. If persistence/custody fails, the report records a fixed
`DurableReconciliationFailed` category and the existing startup reconciliation remains
authoritative.

The initial C registry accepts only Talos runtime-owned finalizers installed by reviewed
composition code. It is not a public arbitrary callback/plugin API. A later third-party finalizer
extension requires its own panic, identifier, semver and resource-containment review; embedders can
still coordinate host-owned resources after receiving the shared report.

### 8. Redacted structured report

`ShutdownReport` is an additive, non-exhaustive public DTO. It exposes only:

- accepted plan identifier and selected policy;
- total elapsed duration and whether the deadline was exhausted;
- active-turn outcome: idle, finished, interrupted-and-finalized, failed, or unreconciled;
- durable reconciliation outcome and bounded counts by state;
- each fixed finalizer identifier with completed, failed, panicked, timed-out or not-run status;
- actor joined, failed, or contained status.

It never contains prompts, reasoning, messages, tool names or arguments, raw tool output, provider
payloads, filesystem paths, credentials, arbitrary error strings or finalizer-supplied free text.
Errors are closed enums with bounded numeric counts. Logs and hooks receive the same redacted
projection; there is no richer hidden debug report.

### 9. Compatibility and semver boundary

- Existing `RuntimeHandle::shutdown(self) -> RuntimeResult<()>` remains source-compatible. It calls
  the structured path with the documented default, discards a clean report and preserves existing
  `ActorJoin` error behavior. An incomplete structured result maps to a new bounded
  `ShutdownIncomplete` error; it must not fabricate `Ok(())`.
- Adding `ShutdownIncomplete` to today's exhaustive public `RuntimeError` is a pre-1.0 breaking
  change. B/C must land with a next-minor migration note that tells exhaustive downstream matches
  to add a fallback arm, marks the enum non-exhaustive for future additions and compiles an external
  migration fixture. It cannot be slipped into a patch release or an unrelated implementation PR.
- New shutdown types use `serde` and `schemars` where they cross config/protocol boundaries, and
  non-exhaustive enums/accessors where downstream exhaustive matching would prevent additive growth.
- The serialized `SessionOp::Shutdown` representation remains unchanged.
- Existing `submit`, `preview_request`, `interrupt`, event ordering and durable TLOG format remain
  unchanged outside the closing state.
- No persistence migration is required. Removing the compatibility wrapper, changing default
  policy after shipment, or changing serialized report fields requires a new ADR and migration plan.

`RuntimeHandle` is the unique primary handle; `RuntimeShutdownHandle` is a cloneable controller,
not another primary. Dropping the primary while the coordinator is still `Open` initiates the same
default shutdown plan through a non-blocking runtime-owned wakeup but cannot wait or return a
report from Drop. Dropping a controller never initiates shutdown, and retained controllers do not
prevent primary Drop from closing admission. Once any explicit request has reached `Closing`,
primary Drop can only leave or join that accepted plan; it never proposes a second default.
Explicit `shutdown` remains the only API that proves terminal cleanup to the host; Drop remains
best-effort initiation, and panic/unwinding must not block.

## Required Race Semantics

| Race / failure | Required result |
|---|---|
| Two valid shutdown callers race while Open | One plan wins the compare-and-set; both receive the same report. |
| Invalid options draft races a valid request | Draft validation fails before coordinator or primary-handle access; the valid request may win. A later explicit primary Drop is a separate default request, not an effect of validation. |
| Submit wins the admission gate before shutdown | It is queued pre-fence and then completed, cancelled, rejected or paused by the accepted shutdown plan. |
| Shutdown wins the gate before submit | Submit returns `RuntimeClosing` and never reaches the Session queue. |
| Actor pauses after preparation and shutdown wins before start commit | The item never becomes `StartCommitted`, performs no model/tool work and is rejected or paused by the actor. |
| Actor start commit wins before shutdown | The item is pre-fence active; shutdown applies `FinishCurrent` or `Interrupt` to its preinstalled cancellation token. |
| Primary Drop races explicit structured shutdown | The first valid plan wins; Drop never installs a second default plan. Controller Drop changes no state. |
| Active Success finalizes before interrupt wins | Report `Finished`; ADR-058 Success remains authoritative. |
| Interrupt wins before Success finalization | ADR-058 first-terminal-outcome rules decide Cancelled/Error; report the observed durable result. |
| Finish grace expires | Issue actor-owned interrupt using the remaining global budget; do not start pending work. |
| Durable pause/finalization fails | Continue safe cleanup, report `DurableReconciliationFailed`, and leave restart reconciliation authoritative. |
| One finalizer fails or panics | Record a typed failure and continue later finalizers while time remains. |
| One finalizer times out | Contain it, record timeout, and never report that resource clean. |
| Total deadline expires | Contain outstanding work, mark unrun stages, publish one incomplete report; never reset the deadline. |
| Caller awaiting shutdown is cancelled | Runtime-owned driver continues; a later caller receives the cached terminal report. |

## Implementation Split

| Slice | Runnable/testable deliverable | Principal surfaces | Exit gate |
|---|---|---|---|
| RUNTIME-005-B | Cloneable coordinator, validated shutdown options, one admission/start arbiter, explicit primary/controller Drop semantics, two active-turn policies, one absolute deadline, cached redacted report and deterministic idle/active/concurrent/racing fixtures. | `talos-runtime`, existing Session coordination seam, SDK reference docs | ADR-063 Accepted; exact-head Unix/Windows workspace tests; no finalizer registry or TOOL-024 dependency. |
| RUNTIME-005-C | Frozen build-time ordered finalizer registry, durable reconciliation outcomes, compatibility wrapper closure and failure/panic/timeout ordering fixtures using runtime-owned test finalizers. | `talos-runtime`, minimal lower-layer finalizer handle seam, SDK/reference docs | B Complete; independent architecture review; legacy `shutdown()` and durable-session regressions pass. |

Neither slice authorizes TOOL-024 production behavior. A later TOOL-024-B implementation may
register its supervisor only after RUNTIME-005-C and PERM-006-C are Complete and ADR-060's platform
gates remain satisfied.

## Validation Gate

Before this ADR becomes Accepted, independent exact-head architecture review must verify:

- first-valid-request arbitration and caller-cancellation independence;
- the submit/shutdown fence and actor start commit share one linearization mechanism with no
  check-to-send or check-to-start race and no arbiter held across asynchronous/external work;
- invalid options cannot consume the primary handle, and primary/controller Drop cannot create a
  second shutdown plan;
- `FinishCurrent` cannot consume or extend the total deadline;
- ADR-058 remains the only turn finalizer and durable reconciliation owner;
- finalizer order, panic, timeout and cancellation containment are implementable without a global bus;
- the report has no free-text or sensitive payload channel;
- the compatibility wrapper and unchanged `SessionOp::Shutdown` shape have a credible migration path;
- B and C are independently runnable/testable and create no TOOL-024 dependency cycle.

Implementation must then add deterministic idle, active success, interrupt, concurrent caller,
submit race, paused-before-start-commit shutdown, invalid-options plus primary/controller Drop,
persistence failure, finalizer ordering/failure/panic/timeout and total-deadline tests.
Observable SDK documentation must stay marked planned until the implementation commit exists.

## Consequences

- Embedders can coordinate shutdown from more than one host component without duplicating cleanup.
- The current immediate-cancel behavior remains the legacy default, but gains a finite bound and
  truthful incomplete outcome.
- A finish grace is available without allowing new work to start or starving cleanup indefinitely.
- Runtime-owned resources gain one deterministic integration seam; background jobs remain a later
  consumer rather than a prerequisite.
- The coordinator and report add public SDK surface. Careful non-exhaustive typing and migration
  notes are required even before 1.0.
- A deadline can produce an incomplete report. Bounded does not mean falsely successful; the host
  decides whether to retry observation, log a redacted incident or terminate its process.

## Reversal Triggers

Revisit this decision if implementation shows that:

- one admission/start arbiter cannot serialize submit, shutdown and actor start commit without
  changing established queue semantics, holding across external work or introducing deadlock;
- actor-owned ADR-058 finalization cannot fit within one absolute deadline without losing admitted
  durable facts;
- a registered resource cannot provide cancellation-safe containment;
- a fixed ordered registry creates a dependency cycle or requires a global event bus;
- the structured report cannot remain useful without sensitive/free-text payloads.

A replacement must preserve monotonic admission closure, exactly-once shared completion, one total
deadline, actor-owned durable finalization and fail-closed reporting.

## Related

- ADR-006: Event Architecture Boundary
- ADR-039: Runtime Event Semantic Single-Flow Boundary
- ADR-042: Embedded Durable Runtime Session Boundary
- ADR-058: Partial-Turn Durable Finalization Boundary
- ADR-060: Supervised Background Command Job Lifecycle
- RUNTIME-005 / Issue #49
- TOOL-024 / Issue #59
