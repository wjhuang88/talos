# TUI-040: Approval Option Contrast And Discoverability

| Field | Value |
| --- | --- |
| Story ID | TUI-040 |
| Type | Product / TUI visual-reliability story |
| Priority | P1 |
| Status | Complete — maintainer real-terminal review passed 2026-07-29 |
| Source | Maintainer report: unselected approval options are so faint that users do not discover the available choices. |
| Parent Epic | None; bounded follow-up to completed TUI-008 approval UX |
| Depends On | TUI-008 Complete; TUI-035/I156 Alternate-Screen renderer Complete |
| Blocks | None |
| Selected Iteration | I167 (Complete; implementation `3356aac`) |

## Goal / Value

Make every actionable approval choice visually discoverable. The selected option must retain its
selection treatment, while unselected options must use the normal readable foreground instead of
the muted metadata color.

## Scope

- Change only the approval-panel style for unselected actionable menu options.
- Preserve the selected option's accent/background treatment.
- Use an existing semantic theme role; do not introduce a theme, palette, or terminal-color mode.
- Add a buffer-level regression test that asserts unselected approval option text uses the readable
  primary foreground and remains distinct from the selected row.

## Exclusions

- No permission-policy, approval-choice, keyboard, mouse, modal, or tool-execution changes.
- No layout, height, wrapping, cursor, alternate-screen, transcript, or native-scrollback change.
- No global theme redesign or custom user theme support.
- No new dependency, `unsafe`, release, tag, publish, or version change.

## Required Reads

- `docs/backlog/active/TUI-008-approval-dialog-ux.md`
- `docs/backlog/active/TUI-035-narrow-viewport-and-resize-rendering-robustness.md`
- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md`
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-tui/src/tests.rs`
- `crates/talos-tui/src/theme.rs`

## Acceptance

- Given an approval panel with one selected choice and multiple unselected choices, when it is
  rendered, then each unselected actionable label uses the readable semantic primary foreground
  rather than `DIM_TEXT`.
- Given the same panel, when a different option is selected, then the selected row keeps its
  existing accent/background distinction and unselected rows remain readable.
- Given narrow and wide approval layouts, when rendered into a buffer, then all choices remain
  present, bounded, and style-distinguishable.
- `cargo test --locked -p talos-tui` and full locked workspace validation pass.

## State / Status Owners

- Rendering and tests: `crates/talos-tui/src/scrollback.rs` and `crates/talos-tui/src/tests.rs`.
- Story state: this document.
- Execution evidence: I167.

## User-Facing Documentation

No text documentation change is required: this is a presentation-only correction. The buffer test
and I167 runtime walkthrough are the user-visible evidence.

## Residuals

- A future global contrast/theme audit, if requested, requires a separate Story. This slice changes
  only approval option discoverability.
- Manual terminal gate: passed 2026-07-29. The maintainer opened an approval prompt, confirmed all
  choices were visible before navigation, and verified the accent/background distinction remained
  clear after Up/Down movement.

## Completion Evidence

- Completion Commit: `3356aac52e755a29c4bfbdd43854c47e851569d9`.
- Maintainer review: passed 2026-07-29; unselected approval choices are now discoverable and the
  selected treatment remains clear.
