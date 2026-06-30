# EXT-001: TUI Provenance Markers and `/plugins`

## Outcome

The backend provenance shipped in I009 becomes visible to TUI users.

## Status

**Complete (delivered via I014, 2026-06-06).** Provenance markers, `/plugins` (MCP server snapshot
+ per-provenance call counts), missing-provenance degradation, and TUI rendering tests all landed in
I014. The `/plugins` *command naming* collision (it shows MCP status, not plugins) is a separate
concern tracked by CMD-002; it does not change EXT-001's delivery.

## Priority

P1 because this closes a user-facing gap from completed extensibility work.

## Required Reads

- `docs/iterations/I009-extensible-agent.md`
- `docs/iterations/I014-tui-completion.md`
- `docs/backlog/active/TUI-001-completion.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [x] TUI distinguishes native, native tool-pack, and MCP-remote tools where provenance is available.
- [x] `/plugins` lists plugin/provenance information without changing the agent loop.
- [x] Missing provenance degrades clearly instead of panicking or hiding the tool.
- [x] Tests cover rendering or state construction for provenance-bearing tool events.

## Residual Work Destination

The only residual is the command *naming* (`/plugins` shows MCP/provenance, not plugins). That is
captured by CMD-002 (command taxonomy realignment), blocked on the plugin encapsulation proposal.

