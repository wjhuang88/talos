# TOOL-015: Write And Edit Result Visibility

| Field | Value |
|-------|-------|
| Story ID | TOOL-015 |
| Priority | P2 |
| Status | Planned |
| Source | [GitHub Issue #13](https://github.com/wjhuang88/talos/issues/13) |
| Relates To | TOOL-003, TUI-019 |

## Requirement

`write` and `edit` tool results should show useful write/edit content so users can verify file
changes without opening the file manually.

## Scope

- For `write`, show target path, byte count, and bounded content preview.
- For `edit`, show a bounded diff of the replacement.
- Keep model-facing raw result bounded and deterministic.
- Preserve full file content outside the display path only when explicitly returned by the tool.

## Acceptance Criteria

- [ ] `write` success includes path, size, and bounded preview.
- [ ] `edit` success includes a readable diff.
- [ ] Large outputs are truncated predictably.
- [ ] TUI rendering remains readable.
- [ ] Unit tests cover write preview and edit diff output.

## Required Reads

- [GitHub Issue #13](https://github.com/wjhuang88/talos/issues/13)
- `docs/backlog/active/TOOL-003-posix-tool-set.md`
- `crates/talos-tools/src/file_tools/`
- `crates/talos-tui/src/tool_display.rs`
