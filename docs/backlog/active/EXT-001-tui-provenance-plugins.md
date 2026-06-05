# EXT-001: TUI Provenance Markers and `/plugins`

## Outcome

The backend provenance shipped in I009 becomes visible to TUI users.

## Status

Deferred into TUI-001 / I014.

## Priority

P1 because this closes a user-facing gap from completed extensibility work.

## Required Reads

- `docs/iterations/I009-extensible-agent.md`
- `docs/iterations/I014-tui-completion.md`
- `docs/backlog/active/TUI-001-completion.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [ ] TUI distinguishes native, native tool-pack, and MCP-remote tools where provenance is available.
- [ ] `/plugins` lists plugin/provenance information without changing the agent loop.
- [ ] Missing provenance degrades clearly instead of panicking or hiding the tool.
- [ ] Tests cover rendering or state construction for provenance-bearing tool events.

## Residual Work Destination

If I014 only lands markers or only lands `/plugins`, remaining user-facing work stays in TUI-001.

