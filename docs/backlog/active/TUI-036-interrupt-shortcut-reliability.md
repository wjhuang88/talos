# TUI-036: Interrupt Shortcut Reliability And Semantics

| Field | Value |
| --- | --- |
| Story ID | TUI-036 |
| Type | Product / input-reliability story |
| Priority | P1 |
| Status | Refinement — deferred behind I156 / TUI-035 real-terminal acceptance |
| Source | Maintainer report 2026-07-27: Ctrl+C can stop interrupting turns permanently until Talos is restarted |
| Parent Epic | None |
| Depends On | TUI-009, TUI-035; existing conversation cancellation boundary |
| Blocks | None |

## Identity / Goal / Value

Make interruption reliable and unambiguous. Ctrl+C must be reserved for local
composer clearing and the explicit idle exit affordance; Esc must request
cancellation of the current turn. A failed or stale cancellation path must not
leave the session permanently unable to interrupt subsequent turns.

## Scope

- Trace the existing Ctrl+C path from terminal event through TUI state and the
  conversation cancellation owner; record a reproducible state matrix for the
  reported "works until it does not" failure.
- Change the normal active-turn shortcut from Ctrl+C to Esc.
- Keep Ctrl+C local: clear non-empty idle composer text; retain only the
  explicit idle double-press/close behavior when the composer is empty.
- Define Esc priority for approval, credential/provider panels, slash menus,
  and a normal active turn without allowing one key event to both close a panel
  and cancel a turn.
- Ensure cancel requests are scoped to the currently active turn, are emitted
  at most once per key press, and recover after completion, failure, timeout,
  or a rejected/failed cancellation request.
- Add deterministic entry-point tests plus real-terminal validation for the
  supported keyboard-protocol and portable fallback paths.

## Exclusions

- No change to tool execution, permission decisions, provider protocol, or
  session persistence semantics.
- No new global event bus or parallel cancellation channel.
- No promise that every terminal reports modified keys; the implementation must
  preserve a documented portable path.
- No work on I156/TUI-035 architecture beyond the minimum integration needed
  when this story is selected by a later iteration.

## Decision Links And Constraints

- TUI-009's completed active-turn Ctrl+C acceptance is superseded for this
  follow-up only; it remains historical evidence for idle clearing and exit
  behavior.
- ADR-054 keeps interactive rendering application-owned; shortcut changes must
  not reintroduce terminal-native history or resize behavior.
- The current I156 program permits no new implementation scope until its
  real-terminal acceptance matrix passes.

## Uncertainty And Validation Path

The triggering condition for the stuck Ctrl+C report is not yet known. Before
marking this story Ready, document the cancellation state machine and exercise
at least: normal streaming, tool execution, approval visibility, modal input,
cancel during shutdown, timeout/failure, repeated Esc, and a new turn after
each terminal state. If the root cause lies outside the existing TUI/
conversation boundary, record the exact owner and required decision rather
than broadening this story silently.

## State / Status Owners

- TUI key dispatch and visible feedback: `crates/talos-tui`.
- Active-turn cancellation state and acknowledgement: current
  `talos-conversation` owner.
- Story state: this document; iteration selection is a future owner.

## User-Facing Documentation

- Update the composer hint, keyboard-help text, README keyboard guidance, and
  any command reference that states Ctrl+C interrupts.
- Document Esc cancellation and the retained Ctrl+C idle clear/exit behavior.

## Required Reads

- `docs/backlog/active/TUI-009-input-and-session-exit-polish.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/state.rs`
- `crates/talos-tui/src/inline_terminal.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-conversation/src/types.rs`

## Acceptance

- Given an idle composer with text, when Ctrl+C is pressed, then only the
  composer content is cleared and no cancellation request is emitted.
- Given an idle composer without text, when Ctrl+C is pressed according to the
  documented exit affordance, then Talos exits cleanly without treating it as a
  turn cancellation.
- Given an active turn with no higher-priority modal, when Esc is pressed, then
  exactly one cancellation request for that active turn is emitted and visible
  feedback is shown.
- Given approval, credential/provider, or slash UI is active, when Esc is
  pressed, then its documented local dismissal/cancel action takes priority and
  no active turn is cancelled unless the UI explicitly delegates to it.
- Given any completed, failed, timed-out, or rejected cancellation, when a
  later turn begins, then Esc can cancel that later turn without restarting
  Talos.
- Given repeated Esc during one turn, when cancellation is already pending,
  then duplicate requests do not corrupt state and the UI remains responsive.
- Entry-point, state-machine, and real-terminal tests cover the above matrix;
  `cargo test --workspace --locked` passes.

## Residuals

- The report's original trigger remains to be discovered during refinement.
- Selection requires a new iteration after I156/TUI-035 reaches its documented
  completion gate.
