# TUI-038: Startup Inline Composer Continuity

| Field | Value |
| --- | --- |
| Story ID | TUI-038 |
| Type | Product / rendering story |
| Priority | P1 |
| Status | Paused — I164 baseline is superseded by TUI-039 after rebuilt-binary manual verification (2026-07-28) |
| Source | Maintainer request 2026-07-28 |
| Parent Epic | None |
| Depends On | I163 Complete; TUI-035 Complete; ADR-054 Accepted |
| Blocks | None |
| Selected Iteration | I164 (Paused; historical implementation authority) |

## Identity / Goal / Value

Restore the compact startup composition: in a new session, the composer starts
approximately two display rows below the Logo instead of at the bottom of a
tall terminal. This keeps the Logo, initial input, preview, and first history
rows visually contiguous, so the first turn flows naturally into conversation
history rather than jumping across unused screen space.

## Scope

- Add a bounded startup-layout mode for a newly created or newly resumed empty
  session with no submitted user message and no active modal/panel that needs
  the normal bottom layout.
- Render the composer after the display-only Logo virtual prefix with exactly
  two display-only spacer rows between the final Logo row and the composer at
  normal usable heights.
- Render the startup status/hint immediately below the startup composer where
  space permits; it must remain part of the same Alternate-Screen full frame.
- When the first user message is submitted, transition once to the existing
  normal full-frame layout. The submitted message appears below the Logo prefix
  as current ADR-054 behavior requires; subsequent preview and history rows
  occupy the normal continuous history surface.
- Keep the Logo virtual/display-only and preserve the geometry-free
  `TranscriptStore`, logical scroll state, one-size-snapshot render transaction,
  mouse-wheel history behavior, and Alternate-Screen lifecycle.
- Define an explicit narrow/short-terminal fallback: if there is insufficient
  height for Logo + two spacers + composer + status, use the existing bounded
  layout without out-of-bounds rectangles, duplicated components, or cursor
  placement outside the visible composer.

## Exclusions

- No return to Primary Screen, native terminal scrollback, DECSTBM, reverse
  index, or a second renderer.
- No change to transcript persistence, export, session semantics, provider
  context, tool execution, or session title behavior.
- No change to normal post-first-submit composer placement, modal/panel layout,
  queue semantics, or history scroll policy beyond the minimal startup-mode
  integration required here.
- No dashboard/logo-link work, logo artwork redesign, animation, or new
  configuration option.

## Decision Links And Constraints

- ADR-054 requires one Alternate-Screen full-frame renderer and a Logo that is
  a display-only virtual history prefix. Startup layout must be a projection
  decision inside that renderer, not an ANSI prelude or terminal-side viewport.
- TUI-035 owns the accepted resize/isolation architecture. This story must not
  mutate `TranscriptStore` to manufacture startup spacing or let fixed rows
  enter terminal scrollback.
- `AppLayout` remains the authority for final bounded component rectangles and
  cursor targets. A startup layout may extend that authority but may not bypass
  its extreme-size invariants.
- The phrase "two rows" means two terminal display rows at usable normal
  dimensions; it is a visual spacer, not stored history content.

## State / Status Owners

- Startup mode state and render transaction: `talos-tui`.
- Component rectangle allocation and cursor visibility: `talos-tui`.
- Story status: this document; I164 owns implementation evidence after
  activation.

## User-Facing Documentation

- Update TUI/startup UX documentation or release notes to state that a fresh
  session begins near the Logo and transitions into the normal conversation
  layout after the first submission.
- Record the short-terminal fallback as a visual degradation, not a history or
  transcript behavior change.

## Required Reads

- `AGENTS.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/iterations/I164-startup-inline-composer-continuity.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/backlog/active/TUI-005-logo-splash.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `crates/talos-tui/src/app.rs`
- `crates/talos-tui/src/app_layout.rs`
- `crates/talos-tui/src/splash.rs`
- `crates/talos-tui/src/history_projection.rs`
- `crates/talos-tui/src/inline_terminal.rs`

## Acceptance

- Given a normal-height new session with no submitted user message and no modal,
  when the first full frame is drawn, then the Logo is visible and the composer
  starts exactly two display rows after the final Logo row rather than at the
  terminal bottom.
- Given a user types, edits, wraps, or clears the first draft, when startup
  layout redraws, then Logo rows remain visible, the composer cursor remains in
  its final visible composer rectangle, and no transcript entry is created.
- Given the first user message is submitted, when the next frame is drawn, then
  the message appears below the Logo virtual prefix and the renderer transitions
  once to the normal full-frame history/composer layout without blank duplicate
  rows, Logo disappearance, transcript mutation beyond the submitted message,
  or a stale cursor.
- Given preview/reasoning/tool output begins after the first submit, when rows
  grow and wrap, then preview/history remain continuous under the Logo prefix
  and fixed components retain current full-frame isolation.
- Given width/height resize occurs before or during the first submission, when
  the frame redraws, then all component rectangles and the cursor are bounded,
  spacer rows are projection-only, and no fixed content enters primary-screen
  scrollback.
- Given a terminal too short to show the requested Logo/spacer/composer/status
  arrangement, when startup renders, then Talos uses the documented bounded
  fallback without panic, overlap, off-screen cursor, or transcript change.
- Given mouse wheel navigation and Logo-prefix history scrolling, when startup
  mode is active or after the transition, then the Logo remains display-only and
  wheel behavior does not regress into composer-input history navigation.
- Unit/layout/full-frame and real rebuilt-binary Alternate-Screen walkthroughs
  cover normal startup, first draft, first submit, resize, CJK/multiline input,
  narrow/short fallback, wheel navigation, and terminal restore; `cargo test
  --workspace --locked` passes.

## Risks And Rollback

- Risk: treating startup spacing as transcript rows would corrupt logical
  history, scroll anchors, export, or resize behavior.
- Mitigation: keep Logo and spacer rows projection-only and test transcript
  snapshots before/after every startup transition.
- Rollback: remove the startup-layout branch while retaining the existing
  ADR-054 full-frame renderer and normal bottom-composer layout.

## Residuals

- This story deliberately defines startup as "before the first submitted user
  message." Any request to keep the composer beside the Logo during later turns
  is a separate layout product decision.
- I163 is Complete and I164 is paused. I157 remains a published Planned
  baseline and is explicitly deferred until I165 disposition by the
  maintainer's 2026-07-28 priority shift.
- Manual verification rejected the specified post-first-submit transition to a
  bottom composer. This document preserves that published baseline and its
  implementation evidence; TUI-039/I165 owns the replacement behavior.
