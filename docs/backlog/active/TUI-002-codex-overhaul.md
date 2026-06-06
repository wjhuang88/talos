# TUI-002: TUI Codex-style modular overhaul

## Outcome

The `talos-tui` crate mirrors the structural depth of the Codex TUI reference
(`docs/reference/REFERENCE-PROJECTS.md` §687-741): modular history cells,
centralized keymap, bottom-pane composer, slash command framework, and the
`tui/` subdir for EventBroker stdin and frame-rate limiting. This is a
structural refactor; user-facing behavior is unchanged from I014.

## Status

Planned. Blocked behind I015 Provider Schema, I016 Portable File/Search, and
I017 Embedded Git (R6-R8) so that the new history cells can render the new
provider/tool/git output formats from day one. Natural iteration slot:
**I022 or later**.

## Priority

P2. TUI-001 (I014, P1) just shipped and is the immediate predecessor.

## Required Reads

- `docs/proposals/tui-codex-overhaul.md` — full proposal, sub-slices A-E,
  alternatives, open questions, scheduling rationale.
- `docs/iterations/I010-polished-agent.md` — R2/R3 Codex-like baseline
  (AppServerSession convergence, inline mode, Nord theme, markdown, diff,
  steering, slash).
- `docs/iterations/I014-tui-completion.md` — provenance markers,
  `/plugins`, `/copy` + `/export` (most recent TUI work).
- `docs/reference/REFERENCE-PROJECTS.md` §687-741 — Codex TUI PRIMARY
  reference (module layout, file-by-file mapping).
- `docs/decisions/003-tui-progressive-evolution.md` — TUI evolution
  anchor.
- `docs/decisions/005-tui-event-architecture.md` — TUI event architecture
  boundary (single-mpsc `AppEvent` bus + `AppServerSession` seam).
- `docs/decisions/006-event-architecture-boundary.md` — single-consumer
  loop rule (refactor must not introduce a global pub/sub bus).
- `docs/iterations/I015-provider-schema.md` (R6) — provider output format
  precondition.
- `docs/iterations/I016-portable-file-search.md` (R7) — tool cell
  rendering precondition.
- `docs/iterations/I017-embedded-git-tools.md` (R8) — diff cell rendering
  precondition.

## Acceptance Criteria

- [ ] `crates/talos-tui/src/` layout mirrors Codex's structural depth:
      `history_cell/` (with submodules for assistant, tool_call, approval,
      diff, user, system), `keymap.rs`, `bottom_pane/`, `slash_command.rs`,
      and `tui/` (with `event_stream.rs` and `frame_requester.rs`).
- [ ] All I014 functionality still works: provenance markers in tool call
      cells, `/plugins`, `/copy last`, `/copy all`, `/export <path>`.
- [ ] I008 hook-based learning still observes the same `HookEvent`
      ordering (verified by `crates/talos-cli/tests/hooks_e2e.rs` and
      `mcp_client_e2e.rs` at `RUST_LOG=debug`).
- [ ] Public API of `talos-tui` is unchanged (or carries a semver bump +
      ADR).
- [ ] `cargo test --workspace` passes with no regressions (baseline
      652+ tests at the time the refactor starts).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on `talos-tui`
      are either unchanged or reduced; no new warnings.
- [ ] An iteration record (e.g. `docs/iterations/I022-tui-codex-overhaul.md`)
      documents sub-slice outcomes and runtime evidence.

## Sub-Slices (Reference)

The proposal (`docs/proposals/tui-codex-overhaul.md`) defines 5 sub-slices
that an iteration may pick independently:

- **A**: history cell modularization (`history_cell/`).
- **B**: keymap system (`keymap.rs`).
- **C**: bottom pane / composer (`bottom_pane/`).
- **D**: slash command framework (`slash_command.rs`).
- **E**: `tui/` subdir (EventBroker stdin + frame_requester).

A single iteration may pick 1-2 sub-slices; the rest can be picked up in
successor iterations.

## Residual Work Destination

Per-cell UX refinements (e.g. syntax highlighting in tool output cells,
animation polish, per-provider theming) stay in this story's notes until
the modular restructure lands. The structural split itself is the DoD
for each sub-slice.
