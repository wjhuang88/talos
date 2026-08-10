# ADR-054: Alternate-Screen Application-Owned Transcript Rendering

- Status: Accepted (2026-07-27; reinstated with startup-splash amendment)
- Date: 2026-07-27
- Owners: TUI / runtime maintainers
- Related: TUI-035, I156, ADR-035

> I184 amendment status: Proposed. The accepted mouse-capture behavior below remains authoritative
> until the native-selection amendment is independently reviewed and accepted.

> Decision history: the initial implementation was temporarily rejected when
> its startup sequence printed the logo on Primary Screen and then hid it by
> entering Alternate Screen. A primary-screen ADR-055 trial restored native
> history, but the maintainer subsequently selected one Alternate Screen mode.
> This accepted amendment fixes the actual startup defect: enter Alternate
> Screen first, then render the logo in the first full application frame.

## Context

Repeated Alacritty resize leaked fixed hint, composer, and status rows into
primary terminal scrollback. Cleanup after `Event::Resize` cannot reliably
reverse terminal-side reflow that occurs before the event reaches Talos.

The primary-screen inline renderer treats native terminal scrollback and
DECSTBM/reverse-index insertion as history storage while drawing fixed controls
as ordinary terminal cells. Terminal emulators are free to reflow those cells
on a width change, so there is no protocol boundary that identifies fixed UI as
non-history. The current architecture therefore cannot meet TUI-035's resize
isolation acceptance criterion.

## Decision

Talos interactive TUI uses alternate screen. Talos owns the logical transcript
and history scroll state. All visible content is rendered through one full-frame
renderer. No runtime correctness depends on terminal native scrollback, DECSTBM
history insertion, reverse index, or primary-screen reflow behavior.

One terminal-size snapshot is read for each frame and is used for layout,
history projection, drawing, and cursor placement. Resize invalidates a frame;
it never mutates transcript facts.

The startup logo is display-only application state. `TerminalSession` enters
and clears Alternate Screen before the first draw; the full-frame renderer then
draws the logo as a virtual prefix of the history rectangle. The first user
message is appended below that prefix, and increasing projected history
naturally scrolls Logo rows out of the rectangle. The Logo never enters
`TranscriptStore` and is never printed to Primary Screen.

## Rejected Alternatives

- More `ClearType` commands.
- Resize debounce as a correctness mechanism.
- An Alacritty-specific workaround.
- Resetting DECSTBM only after resize.
- Keeping native scrollback while trying to protect fixed bottom rows.
- Maintaining native scrollback and an app-owned transcript as equal sources of truth.

## Consequences

Positive consequences:

- deterministic resize and cross-terminal rendering;
- one rendering model and testable full-frame projection;
- Talos-controlled history reflow and scroll anchors;
- fixed-pane isolation from primary-screen scrollback.

Costs:

- interactive history scroll must be app-owned;
- leaving alternate screen does not naturally leave a live transcript on the
  primary screen;
- history viewport and anchors require explicit implementation.

## Exit Policy

Default: restore primary screen and print a compact session summary. An optional
explicit mode may print the transcript on exit. Talos never mirrors the live
transcript into primary scrollback during interactive execution.

## Implementation Notes

- `TranscriptBlock` stores tool calls and results as logical display facts; only
  `HistoryProjection` applies a current-frame width.
- History scrolling uses a logical entry/row anchor, with a documented nearest
  surviving-entry fallback after reflow.
- `AppLayout` bounds history and the bottom frame allocation for zero and
  short terminal sizes; cursor placement is clamped to that frame.
- Fill tokens are projected scalar-by-scalar and never exceed the viewport.
- `TerminalSession` records each terminal mode transition and rolls back only
  transitions that completed when initialization fails.
- `viewport_splash_lines` is the single Logo representation. It is rendered
  after Alternate Screen entry, reserves one display-only spacer row above the
  wordmark, uses a compact wordmark on narrow terminals, and participates only
  as a virtual frame-history prefix excluded from transcript/session/export
  facts.
- Mouse-wheel navigation addresses the Logo prefix and projected Transcript as
  one continuous frame-history surface. Prefix positions remain display-only;
  positions inside the Transcript resolve to logical content anchors, so the
  Logo can scroll out and back into view without becoming a transcript fact.
- The old primary-screen insertion recovery helpers and their test-only writer
  seam have been removed. Real-terminal acceptance remains an I156/TUI-035
  completion gate rather than an unresolved architecture decision.
- Logical history anchors identify `(entry_id, logical_line, scalar_offset)`;
  rendered rows carry a stable range rather than a width-specific row number.
- Final component rectangles are assigned by `AppLayout`, with composer/status
  reserved before optional preview, queue, tips, and panels.
- Terminal restoration attempts every enabled cleanup action, retains failed
  state for retry, and propagates failure before any primary-screen summary.
- Mouse capture is a tracked terminal lifecycle state. Wheel events move the
  application-owned logical history projection; successful/failed capture
  transitions follow the same exhaustive rollback and retry rules.
- User-message history rows preserve `INPUT_BG` across the full projected row,
  including their padding rows. Projection suppresses synthetic leading
  assistant prefix-only rows without mutating transcript content.
- Rendered anchor ranges use half-open logical intervals. Only the last
  projected row accepts logical-line EOF; semantic empty and fill-only lines
  use an explicit zero-length logical range.
- Logical offsets come from original transcript scalars, never from
  projection-only markers, fill characters, or rendered text length.
- `AppLayout` separates resource allocation priority from visual placement,
  owns above-input/below-input panel placement, and assigns every input cursor
  from its final component rectangle.
- A modal text cursor is visible only when its active semantic field or
  selection row exists in the final panel rectangle. Vertical coordinates are
  never clamped onto another semantic row; the panel renderer and cursor target
  use the same title/instruction/input-row convention.

## Proposed I184 Amendment: Application-Owned Selection Owns The Default Pointer Path

### Additional Context

Issue #134 reports that ordinary mouse-drag selection cannot establish an arbitrary visible range
in the interactive TUI. Code inventory on claim merge `66d0f932` confirms that `TerminalSession`
unconditionally enables terminal mouse reporting, while Talos consumes only wheel events. Pointer
down, drag and release events do not implement an application-owned selection model. Alternate
Screen is therefore not established as the cause; the conflict is the combination of enabled mouse
reporting and no Talos selection consumer.

The existing `/copy last` and `/copy all` commands copy semantic transcript scopes. They do not
replace partial-line or cross-component selection of already visible terminal cells. Keyboard
PageUp/PageDown and Ctrl+Home/Ctrl+End already navigate the application-owned history independently
of mouse events.

### Proposed Decision

The native-only policy is rejected as a complete default for Issue #134. On the captured Alacritty
0.17.0/macOS 26.5.2 baseline, ordinary drag requires Shift, wheel scrolling moves the application
without the selection tracking the projected content, dragging past the viewport has no edge
autoscroll, and resizing clears the selection. Native clipboard copying itself works, but these
interaction gaps violate the required default contract.

Terminal.app 2.15 on the same macOS baseline supplies an independent terminal implementation:
neither ordinary drag nor Shift+drag selects while mouse reporting is enabled. Disabling Terminal's
Allow Mouse Reporting menu item restores exact visible-text selection, but wheel and edge-drag then
move terminal scrollback rather than Talos history, and repeated resize clears the selection. This
confirms that disabling mouse reporting is a diagnostic workaround, not a complete default policy.

TUI-046-B should therefore implement a bounded application-owned selection over visible projected
cells, including edge autoscroll while dragging and explicit resize behavior. The selection must
remain isolated from transcript storage, composer, modal, approval and execution state, and copy
only the visible projected text. This recommendation is a gate for B, not authorization to change
Rust in I184.

The default application-owned history contract is keyboard navigation: PageUp/PageDown move by a
viewport and Ctrl+Home/Ctrl+End jump to the beginning/end. Mouse-wheel history navigation is not a
default Talos guarantee when native selection owns the pointer path. Terminal-specific alternate
screen wheel translation is recorded as observed behavior, not represented as Talos history input.
TUI-046-B must verify that the selected environments do not turn an ordinary selection gesture into
composer, modal, approval, session or execution mutations.

The previously proposed terminal-owned TUI-046-B scope is superseded by the evidence above. The
implementation scope for the separately claimed B slice is:

- define and render an application-owned visible-cell selection;
- support ordinary pointer drag without a Shift modifier and autoscroll at viewport edges;
- preserve or explicitly resolve selection across wheel scrolling and terminal resize;
- preserve Alternate Screen, raw mode, bracketed paste, keyboard enhancement, cursor and exhaustive
  restoration behavior, with mouse event routing reviewed for safety;
- retain keyboard history navigation and existing `/copy` semantics;
- keep the selection buffer bounded to visible content and out of transcript persistence;
- record exact-head results on the maintainer terminal and one materially different platform
  terminal using the I184 matrix schema before the implementation PR is accepted.

An optional mouse-capture mode, terminal-specific modifier contract, application-owned selection or
restored wheel-history feature requires a separate owner and decision. TUI-042/#79 retains ownership
of no-op application wheel transitions and is neither absorbed nor completed by this amendment.

### Acceptance Gate

This proposal becomes an Accepted amendment only after:

- the captured Alacritty and Terminal.app evidence supports mouse reporting, rather than Alternate
  Screen alone, as the causal boundary and rejects native-only selection as the complete default;
- independent architecture review approves the policy and the exact TUI-046-B scope;
- both governance validators, exact-head CI and merge-time CAS pass.

Cross-platform real-terminal execution is a TUI-046-B implementation acceptance gate, not a
pre-development gate. B may begin after this amendment is Accepted and its own Collaboration Claim
is effective; its implementation PR may not be accepted until the exact implementation head passes
the two-environment matrix.

PR #187 remains Proposed decision evidence through merge. A governance-only I184 closeout must then
cite the already-existing #187 merge commit, exact-head review and CI, change this amendment marker
to Accepted, and mark I184 Complete. Only that closeout's target-branch merge unlocks a TUI-046-B
claim; the status-changing closeout commit cannot serve as its own completion evidence.

If the causal matrix is inconclusive, this amendment remains Proposed, the earlier accepted
mouse-capture behavior remains authoritative, and TUI-046-B stays Blocked.
