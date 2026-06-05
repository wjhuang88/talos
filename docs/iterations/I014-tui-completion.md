# I014: TUI Completion

**User can**: Use the TUI as the primary daily interface with clear provenance visibility and
explicit copy/export workflows.

## Status: PLANNED

## Selected Stories

- [ ] #I009-S6: TUI provenance markers + `/plugins` command
- [ ] #I010-S9: TUI clipboard copy/export commands

## Scope

- Finish deferred TUI consumer work for `ToolProvenance`.
- Add `/plugins` and hook/plugin visibility without changing backend plugin semantics.
- Add `/copy last`, `/copy all`, and `/export <path>` using source message text.
- Prefer OSC 52 for clipboard; host clipboard commands remain optional fallback per AGENTS.md
  dependency discipline.

## Non-Goals

- No Guardian auto-approval.
- No exec policy DSL.
- No new provider plugin behavior.

## Acceptance Criteria

- [ ] TUI can show native/MCP provenance consistently in tool call bubbles and plugin status.
- [ ] `/plugins` lists loaded plugins and hook registrations.
- [ ] `/copy last` and `/copy all` extract deterministic source text, not rendered buffers.
- [ ] `/export <path>` does not bypass write permissions.
- [ ] `cargo test -p talos-tui -p talos-cli` passes.
