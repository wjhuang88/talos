# TUI-019: Tool Output Visual Hierarchy

| Field | Value |
|-------|-------|
| Story ID | TUI-019 |
| Priority | P3 |
| Status | Planned |
| Source | [GitHub Issue #14](https://github.com/wjhuang88/talos/issues/14) |
| Relates To | TUI-007, TOOL-015 |

## Requirement

Tool output rendering should distinguish primary result lines from secondary detail lines.

## Scope

- Keep primary status/result lines visually prominent.
- Render detail/preview lines with a softer semantic style.
- Prefer existing theme roles before adding new theme fields.

## Acceptance Criteria

- [ ] Status/result line uses primary result style.
- [ ] Detail lines use secondary result style.
- [ ] Existing themes remain readable.
- [ ] Tests cover style classification.

## Required Reads

- [GitHub Issue #14](https://github.com/wjhuang88/talos/issues/14)
- `docs/backlog/active/TUI-007-theme-system.md`
- `docs/backlog/active/TOOL-015-write-edit-result-visibility.md`
- `crates/talos-tui/src/tool_display.rs`
- `crates/talos-tui/src/theme.rs`
