# TUI-007: Theme System

| Field | Value |
|-------|-------|
| ID | TUI-007 |
| Title | Theme System |
| Priority | P3 |
| Status | Planned |
| Depends on | Centralized color constants in `crates/talos-tui/src/theme.rs` and `crates/talos-cli/src/colors.rs` |
| Blocks | Runtime theme selection; user-configurable palettes |

## Outcome

Talos exposes a small theme system so terminal colors are selected through semantic roles instead
of hard-coded palette constants. Users can eventually switch between built-in themes without
changing rendering code.

## Motivation

Current colors are centralized, but the active theme is still compile-time Nord. This is good
enough for consistency, but not enough for accessibility, light terminals, high-contrast use, or
future user preference support.

## Scope

- Define a `Theme` or equivalent semantic palette structure for TUI surfaces.
- Keep Nord as the default built-in theme.
- Route TUI rendering through semantic theme roles such as `input_bg`, `preview_fg`, `status_dim`,
  `markdown_code`, `prefix_assistant`, and `tip_error`.
- Decide whether CLI ANSI colors should share the same semantic theme source or remain a separate
  lightweight terminal-output palette.
- Add validation for contrast-sensitive built-in theme combinations.

## Acceptance Criteria

- No TUI rendering path uses raw RGB values outside the theme module.
- Nord remains the default and current snapshots/tests keep equivalent output.
- A second built-in theme can be added by changing theme data, not render logic.
- Theme roles are documented enough for future contributors to choose the right role.
- User configuration format is either implemented or explicitly deferred with a follow-up story.
- `cargo test -p talos-tui` and `cargo test --workspace` pass.

## Non-Goals

- No custom user theme parser in the first slice unless explicitly selected.
- No runtime theme switching UI unless a later UX story defines it.
- No syntax-highlighting theme categories until CODE-001/TUI-006 decides the tree-sitter path.

## Required Reads

- `crates/talos-tui/src/theme.rs`
- `crates/talos-cli/src/colors.rs`
- `docs/backlog/active/TUI-006-code-block-rendering.md`
- `docs/backlog/active/CODE-001-tree-sitter-code-analysis-research.md`
- `docs/reference/REFERENCE-PROJECTS.md` §Nord theme palette
