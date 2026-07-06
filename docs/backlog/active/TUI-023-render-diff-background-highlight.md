# TUI-023: Diff Rendering Background Highlight

Type: Product Story (aesthetic enhancement)
Parent Epic: None
Status: Complete (SB111, 2026-07-06)

## Identity / Goal / Value

`render_diff` (`crates/talos-tui/src/widgets.rs`) styles added/removed lines with foreground
colors only (green/red fg; metadata italic). Mainstream diff UIs additionally tint line
backgrounds, which reads better for long hunks. Skipped during the 2026-07-03 render_diff fix
per Simplicity First (foreground fix was a correctness issue; background is taste). Recorded so
the option is a deliberate choice, not a forgotten one.

## Scope

- Subtle background tint for `+`/`-` content lines, theme-aware (Nord and Solarized Dark both
  defined in `crates/talos-tui/src/theme.rs`), keeping foreground contrast readable.
- Headers/metadata/context lines unchanged.

## Exclusions

- No word-level (intra-line) diff highlighting.
- No detection-logic changes — the 13 existing `render_diff` tests must pass unchanged.

## Required Reads

- `crates/talos-tui/src/widgets.rs` (`render_diff`, tests)
- `crates/talos-tui/src/theme.rs`

## Acceptance for behavior

- Given a tool result containing a unified diff
  When rendered in scrollback
  Then added/removed lines carry the theme's diff background tints, existing detection tests
  pass unchanged, and both built-in themes define the new colors.

## Implementation (SB111, 2026-07-06)

- `diff_added_bg` / `diff_removed_bg` added to `Theme` struct with subtle tints (Nord: `rgb(52,64,52)` / `rgb(64,48,52)`, Solarized: `rgb(12,49,48)` / `rgb(16,38,50)`).
- `semantic::DIFF_ADDED_BG` / `semantic::DIFF_REMOVED_BG` constants wired.
- `render_diff` now applies `.bg()` on `+`/`-` content lines in addition to existing `.fg()`.
- `render_diff_styles_added_and_removed_lines` test extended with `bg` assertions.
- All 13 existing diff tests pass unchanged; 243 total TUI tests pass.
- Commit: `36c14db fix(tui): unify todo panel status icons and add themed diff line backgrounds (#TUI-022, #TUI-023)`
