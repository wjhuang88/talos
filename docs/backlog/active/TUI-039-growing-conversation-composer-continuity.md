# TUI-039: Growing Conversation Composer Continuity

| Field | Value |
| --- | --- |
| Story ID | TUI-039 |
| Type | Product / rendering story |
| Priority | P1 |
| Status | In Progress — I165 Active |
| Source | Maintainer correction after I164 rebuilt-binary verification (2026-07-28) |
| Depends On | TUI-035 Complete; ADR-054 Accepted; I164 paused |
| Selected Iteration | I165 (Active; sole implementation authority) |

## Goal

Keep the composer and status visually adjacent to the Logo and the growing
conversation while the projected content fits in the Alternate-Screen frame.
Only when that flow exhausts the available height may the history viewport grow
to its bounded height and the composer become bottom-fixed.

## Scope

- Retain the display-only Logo prefix and one startup spacer row.
- In FollowTail with no active modal, cap history to its current projected
  content height so composer/status immediately follow the visible flow.
- When the combined flow no longer fits, use the existing bounded full-frame
  history viewport and bottom composer behavior.
- Preserve one renderer, geometry-free transcript facts, logical anchors,
  wheel history scrolling, resize behavior, and final `AppLayout` rectangles.
- Add focused layout/input/full-frame tests for first submit, progressive
  history growth, overflow transition, resize, and anchored-history fallback.

## Exclusions

- No primary-screen/native-scrollback/DECSTBM work, second renderer, or
  transcript/session/export format change.
- No changes to modal layout, terminal lifecycle, provider/tool behavior,
  I157, I158-I162, release/version work, or dashboard behavior.

## Acceptance

- After first submit, a short conversation continues below the Logo rather than
  jumping the composer to the terminal bottom.
- As rendered history grows, the composer/status move down with it without
  overlap or transcript mutation.
- Once content exceeds available frame height, history becomes bounded and
  the composer/status remain at the bottom; Logo participates in normal
  app-owned history scrolling.
- Resize, CJK/multiline content, mouse-wheel history navigation, extreme
  dimensions, and anchored scroll state remain bounded and deterministic.
- `cargo test --workspace --locked` and rebuilt-binary human acceptance pass.

## Change-Control Record

I164/TUI-038's published behavior deliberately transitioned to normal bottom
layout after first submit. Manual rebuilt-binary verification rejected that
experience. This is a changed acceptance target, not a correction to I164's
published baseline; I164/TUI-038 is paused and this story is the sole current
implementation authority.

## Automated Evidence

- `8e6ffe9` adds FollowTail natural-history allocation and tests first-submit
  continuity, progressive growth, overflow fallback, and anchored-history
  bottom placement.
- `cargo test --locked -p talos-tui --lib` = 464 passed; locked workspace
  tests, format, check, Clippy, governance validation, diff check, and CLI
  build all pass. Rebuilt-binary terminal acceptance remains required.
