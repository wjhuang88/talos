# Iteration I208: Steering Boundary Insertion

> Document status: Review / Claimed
> Planned date: 2026-08-17
> Objective: implement TUI-050 so steering is inserted at an explicit model-response or tool-call
> boundary rather than only after the outer turn completes.

## Selected Story

- `TUI-050` — `docs/backlog/active/TUI-050-steering-insertion-boundary.md`

## Activation Gate

- TUI-048 and TUI-049 contracts are accepted or their interaction is explicitly resolved.
- Current-main inventory and an effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I208 / TUI-050 only: insert accepted steering at explicit model-response or tool-call boundaries, preserving FIFO, Session/generation identity, exactly-once custody and existing transcript semantics. Excludes layout/padding, arbitrary token preemption, parallel model execution, global event bus, persistent cross-Session queues, permission, release and CAP-001 text seam work. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | #487 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I207/TUI-049 is Complete / Closed on main at `2edb914f`; maintainer directed serial execution of I207, I208 and I246. |
| Implementation PR | #488 |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Claim and activation became effective when governance PR #487 merged as `75ca8057`; implementation starts from that merge or a later `main`; independent exact-head review remains required. |

## Activation Checkpoint — 2026-09-05

I208 claim and activation are effective on `main` after governance PR #487 merged as
`75ca80571a42f2d026f507fdf84624f5a103b873`. The claim candidate was reviewed at exact head
`d3b1d94e` with CI `33894155189`; this checkpoint records activation only and is not implementation
evidence. The implementation branch starts at the merge commit above. Published Baseline remains
unchanged.

## Runnable Deliverable

An event-boundary implementation with deterministic ordering tests, error/cancel/restart coverage,
and real-terminal timing evidence.

## Exclusions

No arbitrary token preemption, parallel model execution, global event bus, or release work.

## Acceptance

- [ ] Steering is inserted at the selected model/tool boundary with published ordering semantics.
- [ ] Multiple boundaries, late arrivals, errors, cancellation and restart reconcile exactly once.
- [ ] Locked validation and real-terminal evidence pass at exact head.
- [ ] User-facing steering timing documentation is updated.

## Status

Review / Claimed. Implementation authorization is now limited to the Work Slice above; no release,
permission-policy, CAP-001, Dashboard or Desktop work is authorized.

## Local Convergence Checkpoint — 2026-09-05

- Implemented the first boundary slice locally: a `ToolUse` model-response boundary transfers one
  prepared FIFO batch into the Session command route while the current structured turn remains the
  execution authority; durable receipt custody is tracked separately and adopted at completion.
- Added a focused boundary transfer test and preserved existing queue/continuation tests.
- `cargo test -p talos-cli --locked` passed (360 tests); `cargo clippy -p talos-cli --locked -- -D warnings`
  passed; `cargo test --workspace --locked` passed, including doctests.
- Governance validators and `git diff --check` passed with zero warnings/errors.
- This is a local checkpoint only. Implementation PR, exact-head CI, independent review, terminal
  evidence and completion evidence remain pending.

## Boundary Completion Audit And Local Follow-up — 2026-09-05

PR #488 merged as `5abc6eb837b03367fd2c47a5824cfd3fd428dc23` (candidate
`f5fe914fb0634d502a0f10bc583c3afbede66ce6`, CI `33899464220`, review comment `5544284392`).
That implementation establishes early durable custody but does not prove insertion into the
current Agent continuation. It is intermediate evidence, not I208 completion evidence.

The follow-up is converging locally on `fix/i208-steering-runtime-boundary`, based on that merge.
The Agent requests one accepted user batch after a complete tool batch. It appends the ordered
messages before its next Provider request, publishes a cancellation snapshot, and acknowledges
the handoff to the Actor. The Actor then projects the injection. Injected submissions retain
their queue quota and share the outer Turn's terminal outcome without duplicate Turn events.

Local evidence obtained before the stable candidate:

- `cargo test -p talos-agent --locked --lib`: 343 passed, including deterministic same-Turn
  two-boundary FIFO, cancellation, and Provider-error tests. Tests inspect captured Provider
  messages, durable journal states, shared Turn identity and transcript uniqueness.
- `cargo clippy -p talos-agent -p talos-cli --locked -- -D warnings`: passed.
- These results cover the local tree tested; they do not establish a remote candidate or replace
  final candidate validation.

Remaining before completion: CLI lifecycle/projection and race coverage, generation/restart and
quota checks, required full validation, user-facing documentation, real-terminal timing
acceptance, a stable follow-up PR with fresh CI/review/CAS, and owner-first closeout.
Completion Commit: Pending.

### Follow-up Verification And Review Findings

The three deterministic boundary tests also reconstruct the Session actor and retry the exact
injected identity: Success/Cancelled/Error custody is returned without another Provider call.
The cancellation fixture fills the remaining queue and verifies an injected Running item still
consumes quota. Both CLI boundary tests pass, including automatic transfer of the later batch.

Independent local Agent-role review found a projection ordering race. The implementation now
waits for the Actor to enqueue the injection projection before continuing Provider dispatch;
the three focused Agent tests pass with that acknowledgement barrier.

Still open locally (no stable candidate authorization):

- An accepted but not yet injected batch can become PausedPending on cancellation/error while
  the bridge remains AcceptedByActor or discards its deferred handle. Cover receipt-before and
  receipt-after completion, and provide an explicit recoverable UI lifecycle.
- Verify failed custody finalization and ambiguous Running recovery retain the journal quota
  and cannot silently resume an unaccounted live-session state.
- Lock down projection order relative to prior tool results as well as subsequent Provider
  output; inspect the independent forwarder and Actor producers.

The full preflight started before the acknowledgement-barrier correction; its result cannot
be claimed as validation of that later tree. Refresh full validation after local corrections.

### Accepted-But-Uninjected Recovery Progress

The local bridge now retains a pre-start paused identity on Cancelled/Error, including receipts
arriving after the original completion. A focused test covers both receipt orderings and both
terminal outcomes and confirms exact generation-bound cancellation remains possible. All three
CLI boundary tests pass. This resolves the inoperable waiting state, but is not final acceptance:
TUI-048 requires Esc to activate queued input automatically. I208 must preserve that behavior
for early Actor-owned input too; an extra Esc that discards the retained batch is not an
acceptable substitute. Finish the explicit same-generation resume path and its tests before
publishing the candidate. Do not mark the recovery finding closed yet.

The earlier full preflight completed successfully, including workspace tests and doctests;
its tree predates subsequent local changes. No remote candidate has been pushed. At this
checkpoint available disk space is approximately 4 GiB, with `target/debug/incremental` about
6.5 GiB; manage reproducible build cache before another large validation run.

### Esc Automatic Continuation Correction

The Actor now remembers a matching generation/Turn-targeted interrupt and, after a successful
Cancelled finalization, allows already accepted pending user work to start. Legacy unqualified
Interrupt, Provider Error, failed custody finalization and Scheduler-only pending work do not
gain automatic resume. The bridge retains its accepted lifecycle after requested cancellation,
including late receipt arrival; Error keeps the explicit pre-start paused cancellation path.
This supersedes the temporary extra-Esc fallback for requested cancellation described above.

`boundary_injection_targeted_cancel_resumes_uninjected_user_once` passes: the original Turn is
TerminalCancelled, retained input is Committed under a different Turn, exactly two outer Turns
start, and the retained input is never reported as injected into the cancelled Turn. All four
focused Agent boundary tests and all three CLI boundary tests pass. Agent/CLI Clippy with
`--locked -- -D warnings` passes. Final stable-candidate review, full validation and terminal
acceptance remain pending, along with the previously recorded forwarder-ordering and failed
custody accounting checks.

### Ordering And Recovery Protection Follow-up

CLI's complete binary unit suite passed 362 tests after updating the I206 fixture to allow
early custody transfer while preserving its cancellation/provider-switch assertions. Agent's
344-test library suite passed before the subsequent recovery protection below.

Injection acknowledgement now crosses the existing event forwarder after prior queued progress
has drained. The Actor publishes injection before acknowledging the Agent, which waits before
its next Provider request. Focused tests assert tool-result < injection < next-tool-call order.

After failed custody finalization or a missing completion record, the Actor stops admission and
preserves pending journal state for reconstruction. Reconstruction stops when Running identities
cannot be reconciled from terminal transcript proof, rather than accepting new work with missing
quota accounting. `boundary_injection_ambiguous_running_recovery_fences_new_execution` verifies
two same-Turn Running identities remain frozen and a newly queued item causes zero Provider
calls. All five focused boundary tests pass. The final missing-record cleanup adjustment still
needs its broader validation. Independent review and final candidate validation remain pending.

### Local Revalidation And Remaining Handoff Race — 2026-09-05

The local Agent library suite passed 345 tests and the CLI binary suite passed 362 tests.
The subsequent exact-base preflight passed governance, format, workspace check and Clippy,
but failed `orphan_running_submission_is_never_auto_replayed`: the new recovery fence had
closed the Actor command channel, breaking existing observational reconciliation. This is
a failed full preflight, not completion evidence.

Recovery now retains a read-only reconciliation loop while refusing new execution under
ambiguous Running custody. The existing I169 crash/replay integration suite passes 2 tests,
and the I208 boundary suite passes 5 tests. No historical I169 acceptance was relaxed.

Independent local Agent-role reinspection confirmed the normal projection acknowledgement
barrier but found cancellation could discard acknowledgements already sent to the forwarder.
The forwarder now waits for Agent sender closure and transfers those acknowledgements before
finishing cancellation. Remaining local work: deterministic coverage of this cancellation
window; explicit correlated CLI resolution for a delivered batch never acknowledged by the
Agent; rejection/receipt behavior in recovery-fenced mode; refreshed complete validation and
real-terminal acceptance. Do not move the injected snapshot after an awaited confirmation
without proving the cancellation window between projection and snapshot is safe.

No follow-up candidate has been committed or pushed. The maintainer's brief revalidation
confirmation has not been attributed to this uncommitted tree or treated as exact-head review.
I208 remains Review / Claimed with Completion Commit: Pending; I246 remains unauthorized.

### Unacknowledged Resolution And Disk Recovery — 2026-09-05

The Actor now emits a correlated `SubmissionResolved(TerminalError)` after successful durable
finalization of a delivered-but-unacknowledged batch. CLI clears the matching waiting identity
and explicitly reports no automatic retry. The Actor terminal-resolution test and CLI correlated
resolution/stale-receipt test pass. Recovery-fenced tracked and untracked submissions now receive
`SessionClosed` rejection receipts; all six focused Agent boundary tests pass.

Independent local Agent-role reinspection found no further demonstrated code blocker. The
remaining verification gate is a deterministic end-to-end cancellation at the handoff window,
including transcript uniqueness, mutually exclusive Injected/Resolved events, journal terminal
state, CLI lifecycle and restart non-replay. This is not exact-head approval.

A subsequent full preflight failed with `No space left on device` during workspace compilation.
After the process exited, only the reproducible `target/debug/incremental` cache (9.1 GiB by
`du`) was removed. No source, branch or Session data was removed. Full validation was restarted
with `CARGO_INCREMENTAL=0 COLLABORATION_VALIDATION_BASE=origin/main ./scripts/release_preflight.sh`;
its result is pending and must be checked rather than inferred. No candidate has been pushed.

### Complete Local Validation Checkpoint — 2026-09-05

`boundary_cancel_after_agent_ack_before_actor_projection_preserves_once` now uses a
test-only forwarding gate: the Agent has sent its acknowledgement but Actor projection is
held until a targeted cancellation. It verifies one Provider request, one injected input in
the transcript, shared terminal cancellation, no contradictory unacknowledged resolution,
and no replay after reconstruction. All seven focused Agent boundary tests pass.

The I169 journal-fault fixture now waits for bounded Actor termination after finalization
failure instead of sending Shutdown to an already fenced Actor; transcript/journal/restart
assertions remain intact. Its focused test passes.

Final full command (repository-pinned toolchain, outside the restricted filesystem sandbox):

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
COLLABORATION_VALIDATION_BASE=origin/main ./scripts/release_preflight.sh
```

Result: exit 0, `release preflight: passed`; format, workspace check, Clippy, all workspace
tests and doctests pass, both governance validators report zero warnings. Agent library:
347 passed; CLI binary: 363 passed. Debug-symbol and incremental-cache overrides conserve
disk without disabling debug assertions or tests. A preceding restricted run failed because
the existing skill fixture writes `~/.agents/skills/dedup-test`; it was a filesystem permission
failure, not evidence of flaky I208 behavior. No skill implementation or test was changed.

Remaining gates: local staged-diff review and immutable candidate; real-terminal timing and
cancel acceptance; fresh exact-head independent review/CI and merge-time CAS; owner-first
closeout. Completion Commit remains Pending. I246 is not activated.
