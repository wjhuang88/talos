# TUI-001: TUI Completion

## Outcome

Users can see tool/plugin provenance in the TUI and can explicitly copy or export transcript
content.

## Status

Planned. Selected into I014.

## Priority

P1.

## Required Reads

- `docs/iterations/I014-tui-completion.md`
- `docs/iterations/I009-extensible-agent.md`
- `docs/backlog/active/EXT-001-tui-provenance-plugins.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [ ] TUI renders provenance markers for native and MCP tools.
- [ ] `/plugins` or equivalent TUI command lists available plugin/tool provenance.
- [ ] Transcript copy/export is explicit and does not silently write files without user intent.
- [ ] User-facing README or usage docs are updated if commands or keybindings change.
- [ ] Runtime verification drives the TUI or an end-to-end TUI-facing test path.

## Residual Work Destination

Unfinished UI affordances stay in I014 notes, not in this compact backlog entrypoint.

