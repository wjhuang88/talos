# TUI-023: Diff Rendering Background Highlight

Type: Product Story (aesthetic enhancement)
Parent Epic: None
Status: Planned

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
