# TUI-001: TUI Completion

## Outcome

Users can see tool/plugin provenance in the TUI and can explicitly copy or export transcript
content.

## Status

Closed via CMD-001 (2026-06-19). The shared `CommandDefinition` registry now drives help,
completion, and dispatch. `/copy last`, `/copy all`, and permission-gated `/export <path>` are
restored as typed `UiOutput::CopyToClipboard` and `UiOutput::ExportToFile` actions dispatched
through the conversation engine and handled by the TUI. The clipboard module uses OSC 52 with
pbcopy fallback; the export module routes through `talos-permission::PermissionEngine`.

## Priority

P1 — ✅ Closed.

## Required Reads

- `docs/iterations/I014-tui-completion.md`
- `docs/iterations/I009-extensible-agent.md`
- `docs/backlog/active/EXT-001-tui-provenance-plugins.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [x] TUI renders provenance markers for native and MCP tools.
- [x] `/plugins` or equivalent TUI command lists observed plugin/tool provenance.
- [x] Transcript copy/export is explicit and does not silently write files without user intent.
- [ ] User-facing README or usage docs are updated if commands or keybindings change.
- [x] Runtime verification drives the TUI or an end-to-end TUI-facing test path.

## Residual Work Destination

Restore `/copy last`, `/copy all`, and permission-gated `/export <path>` through CMD-001. Do not
re-advertise them until end-to-end tests prove that the active TUI route executes them.
