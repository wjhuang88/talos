# BG-CONTINUE-001: Actor continuation design

Status: Draft; no effective claim, accepted ADR or implementation.

## Requested behavior

Background success, failure and timeout wake the originating live Session model
by default. No per-job opt-in. Busy Sessions queue completion rather than starting
concurrent provider turns. Explicit cancellation and shutdown suppress wakeups.
ADR-060's historical evidence remains unchanged; a successor decision must change
its no-continuation product boundary.

## Integration direction

The Session Actor owns admission, cancellation and dispatch. Presentation consumers
must not trigger continuation. Evaluate a private supervisor-to-Actor pending inbox
with bounded notification, stable Session-generation/job identity, and reconciliation
after current-turn commit. Retain terminal UI notifications independently.

Completion is bounded runtime data, not user authorization, a privileged instruction,
or a second Tool result for the original launch call. Subsequent tools retain normal
permission checks. Document the additional provider usage of default continuation.

## Request projection findings

`Agent::structured_session_inputs` in request_plan.rs currently converts every
text-only SubmissionItem to Message::User. Therefore routing completion through an
unchanged structured submission would falsely label it as user input internally.
The existing Message::Context variant is a candidate for attributable runtime data;
OpenAI maps it to wire role user, so wire role alone cannot establish provenance.
Keep an explicit bounded background-completion envelope in content, avoid arbitrary
process output in that envelope, and retain distinct internal/transcript identity.
Use job ID and terminal status to let the model retrieve output through process.
This is a proposed projection, not implementation or proven provider coverage.

`Session::run_owned` fences execution when Running custody cannot be reconciled.
Its regular dispatch also gates on pending-store pause, fork work, compaction and
runtime admission. A new continuation input must traverse equivalent gates and
commit/terminal recording, not spawn a provider task from a notification handler.

## Decisions required before implementation

- Select model-context representation and admission integration. SubmissionSource
  and SubmissionKind are exhaustive public enums; additions require compatibility
  review and a migration decision, not an assumption of source compatibility.
- Define cancellation epochs so late notifications cannot undo user stop.
- Define terminal-result consumption at the model-visible delivery boundary; host
  process reads alone must not suppress continuation. Avoid duplicate follow-ups
  after an active turn has already consumed terminal results.
- Define bounded overflow, queue ordering, paused/error behavior and output expiry.
  Do not silently discard continuation or create unlimited retries.
- Distinguish synchronous launch failures already returned from later monitor failure.
- Resolve pending-state recovery without claiming background process restart survival.

## Verification gates

Actor mock-provider tests cover idle/busy completion, failure, timeout, duplicate
notification, already-consumed results, cancel-before/after-admission, shutdown,
stale generation, saturation, expired output and provider failure. Verify permission
checks on follow-up tools and correct tool-call/result pairing. Add real CLI smoke
coverage, pinned locked checks, standard preflight, exact-head CI and independent
review. No passing tests or delivered behavior are claimed by this draft.