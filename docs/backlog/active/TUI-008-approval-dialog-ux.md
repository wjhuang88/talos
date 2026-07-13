# TUI-008: Approval Dialog UX Improvement

| Field | Value |
|-------|-------|
| Story ID | TUI-008 |
| Priority | P2 |
| Status | Complete (I121 F110-F111, 2026-07-13) |
| Depends On | TUI-010 for shared popup evaluation; CMD-001 command boundary |
| Origin | User feedback 2026-06-15 — tool call approval dialog appears at bottom right, easy to miss |

## Problem

When the agent needs user approval for a tool call (e.g., `bash`), the approval dialog
appears at the bottom right corner of the TUI. This position is:

- Easy to overlook — user may not notice it and think the session is stuck
- Visually disconnected from the conversation flow
- No visual cue (highlight, flash, animation) to draw attention

## Proposed Fix

- Move approval dialog to a more prominent position (center overlay or inline)
- Add a brief attention animation or border highlight on appearance
- Ensure the dialog is visible even when terminal is narrow
- Once TUI-010's input-layer popup stack exists, evaluate rendering approval
  through that shared layer instead of maintaining a separate bottom-right
  overlay.
- BuiltinCommand and PluginCommand handlers cannot render or resolve approvals directly. Any
  permission request must use the existing unified event/permission flow; the popup stack is only
  the presentation owner.

## Acceptance Criteria

- [x] Approval dialog is clearly visible and not easily missed
- [x] Dialog position works at 80-col and narrow terminals
- [x] No regression in existing approval flow

## I121 F110-F111 Implementation (2026-07-13)

- `height_hint(width)` now returns correct natural height: 6 rows wide, 6+N narrow (N = wrapped arg lines).
- Width-aware layout: ≥60 cols shows `⚠ tool_name: args` inline; <60 shows tool_name on own line + args on up to 2 wrapped lines.
- Priority clipping: separator > warning title > 3 options > args > help. Options never clipped before args.
- Warning title: TEXT_WARNING fg + NORD2 bg + bold. Body: INPUT_BG. Selected: NORD2 (unchanged).
- No Block::borders added (preserves internal height).
- Keyboard handling and permission decisions unchanged.
- 14 buffer/height tests at 40/60/80/120 columns including CJK and overflow checks.

## Required Reads

- `crates/talos-tui/src/widgets.rs` (ApprovalOverlay)
- `crates/talos-tui/src/app.rs` (approval rendering)
- `docs/backlog/active/TUI-010-slash-command-menu.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
