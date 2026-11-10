# Default model continuation after background job completion

## Status and authority

Historical intake, preserved during branch consolidation. Superseded by
[BG-CONTINUE-001](../tasks/background-job-default-continuation.md) and
[ADR-101](../decisions/101-background-completion-continuation.md), accepted in
PR #893. The text below records the earlier proposal; current authority and
implementation status belong to those documents.

Proposal; not an effective Collaboration Claim, accepted ADR, or implementation.
The user explicitly requested that completed background jobs wake the model by default,
without a separate opt-in switch. This replaces the earlier suggested opt-in design.
ADR-060 remains the effective historical contract until a successor decision and claim
are accepted on the target branch. Preserve its original completion evidence.

## Deliverable

In the originating live Session, an authorized background job reaching success,
failure, or timeout causes the model to inspect its result and continue without
another user message. When the Session is busy, completion waits for a safe dispatch
boundary rather than starting a concurrent provider turn. CLI/TUI presentation must
not be the owner of continuation.

## Constraints

- Hard: retain permission checks for every subsequent tool action; a completion is
  data, not user authorization or a privileged instruction.
- Hard: retain Session identity/generation isolation, bounded queues and output,
  process-tree ownership, and shutdown finalization. No global event bus.
- Product decision requested by user: continuation is default, not per-job opt-in.
- Proposed cancellation rule: explicit user cancellation or Session shutdown must
  suppress pending automatic continuation; later notifications must not undo stop.
- No durable background-process recovery, cross-session delivery, new process
  launcher, automatic command retry, spinner changes, or steering UI work.
- Never emit a second provider tool result for the original launch call ID.

## Source findings and design questions

- `docs/decisions/060-supervised-background-command-jobs.md:46` prohibits automatic
  continuation; `docs/backlog/active/TOOL-024-background-command-jobs.md:28-31`
  requires a separate product decision.
- `crates/talos-agent/src/background_jobs.rs`, `emit_summary`, emits
  `SessionEvent::BackgroundJobTerminal`; `crates/talos-cli/src/tui_bridge.rs`
  displays it as System content without submitting work.
- `crates/talos-agent/src/scheduler.rs`, `scheduled_submission` and
  `deliver_until_accepted`, demonstrate stable producer IDs, generation checking,
  tracked admission and receipt reconciliation. Reuse these principles, not a fake
  Scheduler source or synthetic user-authored message.
- `crates/talos-core/src/submission.rs` currently exposes exhaustive public enums:
  SubmissionSource only User/Scheduler/Compatibility and SubmissionKind only
  UserTurn/PreviewRequest. A new variant is not automatically source compatible;
  decide internal Actor input versus a versioned public extension in the successor
  ADR, with storage/schema and SDK migration impact reviewed before implementation.
- Define terminal-event ownership before notification: UI event consumption cannot
  be the reliability boundary. Queue-full, lost receipt, stale generation, shutdown,
  job retention expiry, and cancellation-after-admission need explicit behavior.
- Prevent duplicate continuation when an active turn has already consumed the
  terminal result through `process`; choose and test a consumption identity/rule.
- Spawn/monitor failures require explicit classification; do not schedule a second
  turn merely because a launch already returned a synchronous failure.

## Acceptance and validation plan

1. Idle Session + background success/failure/timeout: a mock provider receives the
   attributable bounded completion context without additional user input.
2. Busy Session: completion is queued, current provider work is not interrupted,
   and at most one provider turn is active.
3. Duplicate notification or lost admission receipt: no duplicate consumption;
   reconciliation uses the same completion identity.
4. Explicit cancellation, Session shutdown, and stale generation: no automatic
   restart. Test cancellation both before and after queue admission.
5. Saturated queue and unavailable custody: bounded, visible failure/pending state;
   no silently lost continuation or unbounded retry tasks.
6. Already-read terminal result: no redundant follow-up according to the selected
   consumption rule. Expired output yields explicit unavailability, not invention.
7. Provider request preserves tool-call/result pairing and completion provenance;
   process output is not elevated to a user/system instruction.
8. Exercise Actor-level behavior without TUI and a real CLI background-job smoke
   test. Retain existing foreground, permission and cleanup regression coverage.
9. Run repository-pinned locked focused tests and standard release preflight;
   obtain exact-head CI and independent runtime/security review.

User documentation targets: README.md, README.zh-CN.md, and applicable tool/runtime
reference pages. Residual destination: the future authoritative Story owner.

## Execution gates

Before selecting/activating implementation, inventory all Active, Review, Planned
and Blocked iteration owners, check remote ownership/PR overlap, assign a new Story
and successor ADR, and merge an effective target-branch Collaboration Claim. The
current main worktree contains unrelated preserved changes; do not overwrite or
mix them into this slice. This proposal does not authorize implementation or claim
completion. No production code or runtime validation has been performed for it.
