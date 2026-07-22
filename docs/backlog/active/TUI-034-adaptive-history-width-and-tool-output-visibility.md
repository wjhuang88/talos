# TUI-034: Adaptive History Width And Tool Output Visibility

**Status**: Refinement
**Priority**: P1
**Source**: Maintainer request 2026-07-22

## Problem

The TUI history area, especially tool-result output, wastes available horizontal space.  The
tool-result display path currently truncates every retained output line at a fixed character
budget even when a wide terminal could show the rest.  This makes otherwise useful command,
diff, and diagnostic output appear as an ellipsis with no way to recover the omitted portion
from the viewport.

The problem is distinct from the existing vertical-volume controls: TUI-015 deliberately limits
very long non-summary outputs to head and tail lines, and summary-eligible tools deliberately
avoid showing their raw content.  Those policies must not be accidentally removed while fixing
width utilisation.

## Outcome

History and retained tool-output lines use the live scrollback viewport width, measured in
terminal display cells, instead of a fixed character cap.  At a wider terminal, a user can see
more of the same retained output; at a narrower terminal, the output is safely wrapped or
continued without overflowing, splitting a wide character, or fabricating an extra omission.

## Scope

1. Inventory every fixed character/display-width cap reachable from normal TUI history rendering,
   including `tool_display.rs` and any reachable legacy bubble widget.  Classify each cap as a
   history-result rule, an intentional one-line interaction rule, or dead code to remove.
2. Replace fixed per-line caps for tool-result scrollback and TUI-015 head/tail retained lines
   with a render-time, display-width-aware policy based on the actual history viewport width.
   Continuation rows must preserve tool-result styling and make it clear that they belong to the
   preceding logical line.
3. Use Unicode display width rather than bytes or scalar count.  CJK, emoji, combining sequences,
   and explicit newlines must neither overflow the render area nor be split into invalid UTF-8.
4. Preserve existing vertical-content controls: the shared 30-line decision threshold, the 3/3
   head+tail policy, the summary-eligible tool set, and its summary text remain unchanged.
5. Preserve TUI-025's one-line semantics for tool-call arguments and approval arguments.  Those
   surfaces may use their actual available width, but this story must not start wrapping them or
   change permission/approval behavior.
6. Verify the full history path, rather than only a helper: tool display output through the
   scrollback/inline-terminal renderer must have correct row accounting at narrow, normal, and
   wide terminal widths.

## Non-Goals

- Do not make raw output from summary-eligible tools visible by default.
- Do not remove the head+tail policy or make the history area an unbounded tool-output viewer.
- Do not change model-visible tool results, session/export persistence, permissions, or tool
  execution behavior.
- Do not alter composer, status-bar, modal, queue-preview, or panel width policies unless the
  renderer inventory proves a shared history primitive must change; any such expansion requires
  explicit change control.
- Do not add a user configuration knob, a fullscreen viewer, or an external pager in this story.

## Acceptance

- Given a retained ASCII tool-result line whose content is longer than 120 characters but fits in
  a wide history viewport, when it is rendered, then it is visible without an ellipsis introduced
  solely by a fixed 120-character limit.
- Given the same logical line at a narrower width, when it is rendered, then all retained content
  is represented in width-bounded continuation rows; no row exceeds the viewport width.
- Given CJK and emoji-containing output, when it crosses a display-cell boundary, then rendering
  uses display width and does not split a character or produce invalid UTF-8.
- Given a non-summary tool result over 30 lines, when it is rendered, then it still displays the
  first three lines, the existing omitted-line indicator, and the last three lines; each retained
  line uses the adaptive-width policy.
- Given a summary-eligible tool result, when it is rendered, then its existing summary behavior
  and information boundary remain unchanged.
- Given a tool-call or approval argument that does not fit, when it is rendered, then it retains
  TUI-025 single-line truncation semantics using the actual available width.
- Given the legacy `ToolCallBubble` or an equivalent component is reachable, when the inventory is
  complete, then it follows the classified policy and has regression coverage; if unreachable, it
  is removed or its non-production status is documented with evidence.

## Verification

- Focused renderer tests using an actual `Buffer`/`InlineFrame` (or the active equivalent) at at
  least 80, 120, and 160 columns.
- Focused tests for wide CJK text, emoji, explicit newlines, and content that crosses the former
  120-character boundary.
- Regression tests for TUI-015 head+tail and summary-tool behavior, plus TUI-025 one-line
  argument behavior.
- Real-terminal walkthrough in Alacritty and one second terminal implementation at narrow and
  wide sizes, including a long `bash`/diagnostic tool result.
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --locked -- -D warnings`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

## Refinement Questions

The first implementation checkpoint must establish whether the current scrollback segment model
can represent continuation rows with correct style and height accounting.  If it cannot, record
the smallest internal rendering design before changing output policy; do not rely on terminal
autowrap as the layout mechanism.  The inventory must also establish whether the 200-character
legacy bubble cap is reachable in the active TUI.

## Required Reads

- `crates/talos-tui/src/tool_display.rs`
- `crates/talos-tui/src/scrollback.rs`
- `crates/talos-tui/src/inline_terminal.rs`
- `crates/talos-tui/src/widgets.rs`
- `docs/backlog/active/TUI-015-head-tail-truncation.md`
- `docs/backlog/active/TUI-025-tool-argument-line-fit-display.md`
- `docs/backlog/active/TUI-032-composer-multiline-wrap.md`
- `docs/decisions/035-tui-history-scrollback-boundary.md`
