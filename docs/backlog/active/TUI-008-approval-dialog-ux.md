# TUI-008: Approval Dialog UX Improvement

| Field | Value |
|-------|-------|
| Story ID | TUI-008 |
| Priority | P2 |
| Status | Planned |
| Depends On | None |
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

## Acceptance Criteria

- [ ] Approval dialog is clearly visible and not easily missed
- [ ] Dialog position works at 80-col and narrow terminals
- [ ] No regression in existing approval flow

## Required Reads

- `crates/talos-tui/src/widgets.rs` (ApprovalOverlay)
- `crates/talos-tui/src/app.rs` (approval rendering)
