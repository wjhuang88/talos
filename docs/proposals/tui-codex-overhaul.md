# TUI Codex-style modular overhaul

## Status

Proposal. Captured 2026-06-06 as the follow-up to the Codex-like TUI baseline
established by I010 R2/R3 and the TUI-completion work in I014.

This proposal is not sufficient as an implementation authority. Before code lands,
record an ADR or an iteration plan that:

- Lays out the module split in `crates/talos-tui/src/` (file-by-file).
- Confirms the public API of `talos-tui` is unchanged or carries a semver bump.
- Re-validates that the I008 hook-based learning path still observes the same
  event ordering after the refactor.
- Verifies that I014's I009-S6 / I010-S9 functionality (provenance markers,
  `/plugins`, `/copy last`, `/copy all`, `/export <path>`) survives the move.
- Re-runs `cargo test --workspace` after every module split; refactors that
  silently break the I008 hook observer are out of policy per ADR-006.

## Motivation

The current `talos-tui` crate is a single-screen, full-screen TUI with a
Nord-themed look, markdown rendering, diff display, slash commands, tool
provenance markers, and OSC 52 clipboard support. It is functionally
Codex-aligned and is the result of two completed iterations:

- **I010 R2 (2026-06-03)**: `AppServerSession` convergence, `--inline` mode,
  canonical approval protocol.
- **I010 R3 (2026-06-04)**: Nord theme, markdown, diff, steering queues,
  slash commands.
- **I014 (2026-06-06)**: provenance markers, `/plugins`, `/copy` + `/export`.

The Codex reference in `docs/reference/REFERENCE-PROJECTS.md` §687-741
(Codex TUI marked as **PRIMARY REFERENCE** for `talos-tui`) describes a deeper
structural decomposition that we have not adopted. The Codex TUI ships as
80+ source modules in `codex-rs/tui/src/`, with explicit subsystems for chat
composition, key binding, history cells, frame rate limiting, and EventBroker
stdin integration. Our TUI is currently ~10 source files, with most of these
subsystems bundled into `state.rs`, `app.rs`, and `widgets.rs`.

The functional surface matches Codex; the structural depth does not. Further
TUI features (planned future cells, multi-line composer ergonomics, animation,
non-TTY stdin piping) will compound the monolith. A one-time structural
refactor before more TUI work lands will keep each new feature diffable
against a stable module boundary.

## Reference: Codex TUI module layout

Per `docs/reference/REFERENCE-PROJECTS.md` §687-741 (TUI PRIMARY reference):

| Codex module | Responsibility | Talos current home |
|---|---|---|
| `chatwidget.rs` | Top-level chat UI orchestrator | `app.rs` + `state.rs` |
| `bottom_pane/mod.rs` | Composer + hints + footer | inline in `state.rs` |
| `keymap.rs` | Centralized keybinding system | inline in `app.rs`/`state.rs` |
| `slash_command.rs` | Slash command framework | `handle_slash_command` branches in `state.rs` |
| `history_cell/mod.rs` | Modular cell type registry | `widgets.rs` (flat) |
| `history_cell/exec.rs` | Tool execution cells | `widgets.rs` |
| `history_cell/approvals.rs` | Approval prompt cells | `widgets.rs` |
| `history_cell/patches.rs` | Patch / diff cells | `widgets.rs` |
| `tui/event_stream.rs` | EventBroker stdin for non-TTY | not present |
| `tui/frame_requester.rs` | Frame rate limiting | not present |
| `markdown_render.rs` | Markdown rendering | `widgets.rs` (markdown submodule) |
| `diff_render.rs` | Diff display | `widgets.rs` (diff submodule) |
| `app_event.rs` | AppEvent variant module | inline in `app.rs` |
| `app.rs` | Core event loop | `app.rs` |

## Proposed Approach

A pure structural refactor of `talos-tui`, not a feature change. The refactor
is split into 5 sub-slices; each sub-slice is independently shippable and
preserves the I010 R2/R3 + I014 user-facing behavior.

### Sub-slice A: history cell modularization

- Create `crates/talos-tui/src/history_cell/mod.rs` with a `HistoryCell` trait
  + cell-type enum dispatch.
- Move per-cell rendering code out of `widgets.rs` into:
  - `history_cell/assistant.rs` (text + markdown)
  - `history_cell/tool_call.rs` (tool call bubble + provenance marker)
  - `history_cell/approval.rs` (approval prompt rendering)
  - `history_cell/diff.rs` (diff display)
  - `history_cell/user.rs` (user input rendering)
  - `history_cell/system.rs` (system messages)
- `widgets.rs` keeps shared primitives (gauge, badge, key hint) and re-exports
  cell submodules.
- Add `HistoryCell` unit tests for each cell type (snapshot-style).

### Sub-slice B: keymap system

- Create `crates/talos-tui/src/keymap.rs` with a `Keymap` struct that maps
  `KeyEvent` → `KeyAction` enum.
- Move inline key matching from `app.rs` / `state.rs` into `keymap.rs`.
- Add a `Keymap` builder so the main `app.rs` can register
  global / context-scoped bindings in one place.
- All existing key bindings (slash, vim, ctrl-c, ctrl-d, etc.) keep their
  current behavior; this slice only relocates the code.

### Sub-slice C: bottom pane / composer

- Create `crates/talos-tui/src/bottom_pane/mod.rs` with:
  - `composer.rs` (multi-line input + history)
  - `footer.rs` (status bar + key hints)
  - `hint.rs` (slash-command hint strip)
- Move input handling from `state.rs` to `bottom_pane/composer.rs`.
- The existing `handle_slash_command` stays in `state.rs` (slash commands
  are state mutations, not view concerns) but gets a `&[SlashCommand]`
  descriptor list exposed via the new `slash_command.rs` framework.

### Sub-slice D: slash command framework

- Create `crates/talos-tui/src/slash_command.rs` with a `SlashCommand` trait
  + descriptor (name, summary, usage).
- Convert the current `SLASH_COMMANDS` static list into trait-object-based
  registration so new commands can be added without touching
  `handle_slash_command`.
- I014 commands (`/plugins`, `/copy`, `/export`) get the trait treatment.

### Sub-slice E: tui/ subdir (EventBroker stdin + frame_requester)

- Create `crates/talos-tui/src/tui/mod.rs` to host terminal-level primitives.
- `tui/event_stream.rs`: stdin reader that pumps bytes through a
  `mpsc::Sender<TerminalEvent>` for non-TTY piping. Mirrors
  `codex-rs/tui/src/tui/event_stream.rs` §733.
- `tui/frame_requester.rs`: frame-rate limiter that coalesces redraw requests
  to a target FPS. Mirrors `codex-rs/tui/src/tui/frame_requester.rs` §734.
- These are prerequisites for the eventual headless / scripted TUI mode
  proposed in `unified-event-stream.md`.

## Non-Goals

- No new TUI features (no new slash commands, no new cells, no new key bindings).
- No user-visible behavior changes.
- No public API changes to `talos-tui` (or, if any, a semver bump + ADR).
- No migration of `/copy` / `/export` / `/plugins` semantics from I014.
- No changes to I010 R2 run-path convergence (AppServerSession stays).

## Alternatives Considered

- **Leave the monolith as-is**. Rejected: future TUI work will compound
  `state.rs` past the point where diffs stay reviewable. The refactor is a
  one-time cost; deferring it multiplies the per-feature cost forever.
- **Adopt a third-party TUI framework (e.g. `tui-realm`)**. Rejected: we
  use ratatui + crossterm directly per `docs/reference/REFERENCE-PROJECTS.md`
  §956 ("ratatui + crossterm for chat TUI"). A framework would add an
  external dependency that goes against AGENTS.md rule #1
  (self-contained capabilities).
- **Adopt Codex's modules wholesale by porting `codex-rs/tui` files
  directly**. Rejected: the I010 R2 event architecture (single-mpsc
  `AppEvent` + `AppServerSession` seam) is the right boundary for Talos.
  Direct porting would either duplicate the agent loop or break the
  `AppServerSession` seam (per ADR-005). We adopt the *layout*, not the
  *implementation*.

## Open Questions

- Should the `HistoryCell` trait take `&TuiState` or a narrower read-only
  view type? (Prefer the narrower view to keep cells decoupled from
  `TuiState` internals.)
- Should `frame_requester` use a target FPS from config or hard-code 60?
  Hard-code first; config knob is a follow-up.
- Is the `SlashCommand` trait object registry or a typed enum dispatcher
  better for I014's command count? Object registry is more flexible but
  costs an allocation per command at startup (negligible).

## Dependencies

- **I015 Provider Schema** (R6): provider output format may affect how
  `history_cell/assistant.rs` renders. Should land first.
- **I016 Portable File/Search** (R7): tool call cells in
  `history_cell/tool_call.rs` will need to render results from the new
  native tools. Should land first.
- **I017 Embedded Git** (R8): patch / diff cells will need to render
  `gix` output. Should land first.
- **ADR-003** (TUI progressive evolution) — anchor for the migration.
- **ADR-005** (TUI event architecture) — boundary on event flow.
- **ADR-006** (event architecture boundary) — single-mpsc bus contract
  must not be violated.
- **I008** (Learning Agent): hook-based learning observes the same event
  stream. The refactor must preserve I008's event ordering. Verified by
  `crates/talos-cli/tests/hooks_e2e.rs` after each sub-slice.

## Scheduling

- Blocked behind I015-I017 (R6-R8) landing.
- Natural iteration slot: **I022 or later**.
- Each sub-slice is independently shippable. I022 may pick 1-2 sub-slices;
  the rest can be picked up in I023+.

## Acceptance Criteria (Iteration-Level)

- [ ] `talos-tui/src/` layout mirrors Codex's structural depth
      (`history_cell/`, `keymap.rs`, `bottom_pane/`, `slash_command.rs`,
      `tui/event_stream.rs`, `tui/frame_requester.rs`).
- [ ] All I014 functionality (provenance markers, `/plugins`, `/copy last`,
      `/copy all`, `/export <path>`) still works.
- [ ] I008 hook-based learning still observes the same `HookEvent`
      ordering.
- [ ] Public API of `talos-tui` is unchanged (or carries a semver bump +
      ADR).
- [ ] `cargo test --workspace` passes with no regressions (652+ tests).
- [ ] Pre-existing 5 × `clippy::collapsible_if` warnings on talos-tui are
      either unchanged or reduced by the refactor (no new warnings).
- [ ] `docs/iterations/I022-tui-codex-overhaul.md` (or successor) records
      sub-slice outcomes and runtime evidence.

## Risks

- **Event ordering drift** during cell refactor can silently break the
  I008 hook observer. Mitigation: run `hooks_e2e` + `mcp_client_e2e` after
  every sub-slice; they assert on event strings in stderr at
  `RUST_LOG=debug`.
- **Public API churn**: even a "pure" refactor tends to expose types.
  Mitigation: gate the refactor behind `cargo doc` review + semver check.
- **I014 regression**: `/copy` and `/export` use `TuiState` private methods
  (`last_assistant_text`, `transcript_plain_text`, `transcript_markdown`).
  These are `pub(crate)`; cell refactor must not need to expose them more
  widely. Mitigation: keep the same access pattern; do not move transcript
  state to a public type.
- **Talos-CLI integration**: `talos-cli/src/main.rs:1096` registers
  `LoggingHandler::new()` directly. The TUI refactor does not touch
  `talos-cli`, but `talos-cli` tests that depend on TUI module paths
  would need updating. Mitigation: keep `talos-tui` module paths
  backward-compatible where possible (e.g. `mod history_cell` re-exports
  the old `widgets::*` symbols as deprecated for one cycle).
