# Iteration I121: TUI Attention And Thinking Clarity

> Document status: Complete (2026-07-13)
> Published plan date: 2026-07-13
> Planned objective: Make approval requests unmistakable and thinking previews concise without
> changing permission or reasoning-storage semantics.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: approval and thinking preview behavior passes narrow-buffer tests and one native
> terminal walkthrough.

## Published Baseline

### Selected Stories

| Story | Owner | Outcome |
|---|---|---|
| F110-F111 | TUI-008 | Prominent approval overlay and unchanged approval decisions |
| F112 | TUI-024 | Standalone-bold thinking title extraction with `thinking` prefix |
| F113 | I121 | Native-terminal acceptance and docs |

### Scope

- Center or inline the existing approval presentation, preserve its event owner and keyboard actions,
  and test 80-column plus narrow terminal buffers.
- Parse the most recent standalone bold heading from transient accumulated thinking. Keep the
  existing fallback when no valid heading exists and retain the animated `thinking` label style.
- Add semantic render tests for placement, styles, clipping, title updates, and export exclusion.

### Non-Goals

- No permission-policy change, auto-approval, new popup framework, persisted title/reasoning,
  provider request change, collapsible reasoning panel, TUI-025/026/027, or broad TUI refactor.

### Acceptance

- Approval is visible and operable at 80 columns and the documented narrow minimum.
- Existing Allow/Ask/Deny routing and keys are unchanged.
- A standalone `**Title**` block yields `thinking: Title`; inline bold does not.
- Default copy/export and session persistence remain unchanged under ADR-034.
- A named terminal/viewport walkthrough records observed results without secrets.

### Validation And Docs

- Targeted `talos-tui` tests, default export regression, native-terminal packet, and the standard
  validation ladder. Update TUI-008, TUI-024, README/help if user-visible wording changes.

### Risks And Fallback

- Layout collision: prefer bounded overlay placement and clipping; do not rebuild input layers.
- Platform rendering variance: assert ratatui semantic buffers and retain one real-terminal record.

## Execution Record

### Gate 0 — 2026-07-13

- Branch: `feature/i121-tui-attention-thinking-clarity` (from `feature/i120-dynamic-diagnostics` at `aaff634`).
- I120 is Complete on the I120 branch; I121 branches from that state.
- `rustc 1.97.0`; `Cargo.lock` present; governance 0 warnings; release_preflight passed in I120 closeout.
- No other iteration is Active (I120 Complete, I122-I123 blocked on I121).

### F110-F111 — Complete (2026-07-13)

- `height_hint(w)` now returns width-aware natural height for approval panels: 6 rows at ≥60 cols,
  6+N at <60 cols (N = wrapped argument lines, max 2).
- `render_approval` rewritten with width-aware layout:
  - Wide (≥60): `⚠ tool_name: args` on one line, args truncated
  - Narrow (<60): `⚠ tool_name` on own line (always complete), args on up to 2 separate lines
- Priority clipping: separator > warning title > 3 approval options > args > help text.
  When height insufficient, help drops first, then args. Options never clipped before args.
- Visual emphasis: warning title retains `TEXT_WARNING` fg + `NORD2` bg + bold; panel body
  keeps `INPUT_BG`; selected item keeps `NORD2` bg (unchanged — preserves selection contrast).
- No `Block::borders` added (would consume internal height and worsen clipping).
- Keyboard handling and permission decisions unchanged.
- `wrap_text_to_lines` and `approval_natural_height` helpers added as `pub(crate)`.
- 14 new tests: height_hint at wide/narrow/empty/capped, wrap_text basics/truncation/empty/newlines,
  natural_height wide/narrow, buffer rendering at 40/60/80/120, selected style distinction,
  insufficient-height option preservation, CJK tool name, no-overflow check.
- Validation: fmt, clippy, release_preflight, all pass.

### F112 — Complete (2026-07-13)

- `extract_thinking_title(text: &str) -> Option<&str>` scans ALL lines and returns the LAST valid
  standalone-bold title.
- Title block rules (TUI-024/OpenCode semantics):
  - Line trimmed must fully match `**Title**`
  - Title must not be empty or contain extra `*`
  - Title must be followed by empty line or EOF (double newline or end of text)
  - Supports `\n` and `\r\n`
  - Inline bold does not match
- Dedicated `parse_standalone_bold` helper — no regex, no reuse of `parse_inline_delimiters`.
- `preview_text_for_state()` in `app.rs` now uses `extract_thinking_title` to display
  `thinking: Title` when a valid title exists, falling back to full thinking text otherwise.
- Ripple animation (`thinking_ripple_spans`) unchanged — operates on the `"thinking"` prefix.
- Export/session persistence unchanged (ADR-034).
- 14 tests: standalone bold, EOF, trailing newline, most-recent-wins, CRLF, inline bold rejection,
  no-blank-line rejection, inline suffix rejection, empty markers, unclosed, inner asterisk,
  no-title fallback, CJK title, multi-title sequence.
- Validation: fmt, clippy, release_preflight, all pass.

### F113 — Complete (2026-07-13)

- TUI-008 owner doc updated: status Complete, acceptance checked, implementation notes added.
- TUI-024 owner doc updated: status Complete, acceptance checked, implementation notes added.
- Binary builds and starts correctly (`cargo build -p talos-cli --locked` exit 0).
- Buffer snapshot tests verify rendering at 40/60/80/120 columns for approval panel.
- All 14 thinking-title edge-case tests pass (TUI-024 acceptance scenarios).
- Real transcript/export regressions now prove default Markdown excludes reasoning, its derived
  title, and surrounding sensitive fixture text, while explicit include-thinking export preserves
  the established opt-in behavior.
- Automated Alacritty attempt used `/Applications/Alacritty.app`, isolated HOME
  `/tmp/talos-i121-xYNB7Y`, mock provider, and an 80×24 screen-backed TUI. The binary and native
  window started, but macOS denied both automation keystrokes (Accessibility permission) and screen
  capture. No approval/thinking visual result is claimed from this attempt.
- Maintainer's native Alacritty check confirmed that `/export` reports
  `write requires interactive approval` without opening the tool approval panel. This is expected:
  the I014 slash-command export wrapper intentionally rejects `PermissionDecision::Ask` and has no
  approval response channel. It confirms that I121 did not change export permission semantics, but
  it is not an approval-panel acceptance trigger. A real provider tool call to `write` or `bash`
  remains required for the interactive panel walkthrough; `--mock` cannot supply that call.
- Maintainer then supplied a native Alacritty screenshot from a real `glm-5.2 (alibaba-cn)` turn.
  The captured live state shows provider reasoning with the independent `Thinking:` marker, real
  read/tree tool output, and a pending `bash` approval panel with the warning marker, command
  summary, all three choices, and navigation help visible. The screenshot is visual evidence only;
  keyboard routing, narrow widths, title parsing, and export boundaries are established by the
  automated tests listed above rather than inferred from the image.

## Retrospective

### Acceptance Verification

| Acceptance | Status | Evidence |
|---|---|---|
| Approval visible and operable at 80 cols and narrow minimum | Pass | `height_hint` width-aware, buffer tests at 40/60/80/120 |
| Existing Allow/Ask/Deny routing and keys unchanged | Pass | Keyboard code untouched; existing approval tests pass |
| Standalone `**Title**` yields `thinking: Title`; inline bold does not | Pass | 14 edge-case tests including OpenCode parity scenarios |
| Default copy/export and session persistence unchanged (ADR-034) | Pass | Real `talos-session::transcript` tests cover default exclusion and explicit include-thinking opt-in |
| Named terminal/viewport walkthrough records observed results | Pass | Maintainer-supplied Alacritty screenshot captures a real provider turn, provider reasoning/tool output, and a live `bash` approval panel; automated tests cover behavior not provable from a still image |

### Residuals

- None for I121. Maintainer removed test-target Clippy from this long task's gate;
  repository-standard Clippy and `release_preflight.sh` pass.
