# TUI-001: TUI Completion

## Outcome

Users can see tool/plugin provenance in the TUI and can explicitly copy or export transcript
content.

## Status

Regression open (2026-06-19 audit). I014 delivered provenance markers, `/plugins`, `/copy`, and
`/export`, but the copy/export dispatch path was lost during later TUI/conversation refactoring.
`/plugins` remains executable. The unavailable commands have been removed from `/help` and
completion until their typed TUI action path is restored under CMD-001.

## Priority

P1.

## Required Reads

- `docs/iterations/I014-tui-completion.md`
- `docs/iterations/I009-extensible-agent.md`
- `docs/backlog/active/EXT-001-tui-provenance-plugins.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [x] TUI renders provenance markers for native and MCP tools.
- [x] `/plugins` or equivalent TUI command lists observed plugin/tool provenance.
- [ ] Transcript copy/export is explicit and does not silently write files without user intent.
- [ ] User-facing README or usage docs are updated if commands or keybindings change.
- [ ] Runtime verification drives the TUI or an end-to-end TUI-facing test path.

## Residual Work Destination

Restore `/copy last`, `/copy all`, and permission-gated `/export <path>` through CMD-001. Do not
re-advertise them until end-to-end tests prove that the active TUI route executes them.
