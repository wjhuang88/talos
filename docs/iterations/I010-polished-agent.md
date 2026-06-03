# Iteration I010: Polished Agent

**User can**: Use Talos as a daily coding companion with a Codex-like terminal experience, unified
run paths, headless automation, SDK embedding, and release-grade TUI workflows.

## Status: PLANNED

I010 has two planned slices. R2 is architecture convergence: make all run paths share one
session/event/approval surface and add the Codex-like inline terminal mode. R3 is product polish.

I008 is no longer blocked on this iteration. Runtime learning ships through the hook layer; I010
must preserve that behavior during run-path migration, but I008 Review closes through its own
verification evidence, not by waiting for `AppServerSession`.

## Slice R2: Architecture Convergence

### Selected Stories

- [ ] #I010-S7: AppServerSession convergence, Codex-like inline terminal, headless and SDK modes
- [ ] Deferred #ARCH-S6 work, if interactive fork continuation requires run-path migration

### Acceptance Criteria

- [ ] Print, interactive, TUI, headless, and SDK paths drive the agent through `AppServerSession`.
- [ ] TUI approval requests flow through the canonical session/EQ approval protocol.
- [ ] Talos supports a Codex-like inline terminal mode that preserves scrollback and feels like a
      natural CLI extension instead of a disconnected full-screen app.
- [ ] The terminal UI renders over the same ordered session event stream as print/headless paths;
      tool output, approvals, status updates, and assistant deltas do not require separate run-path
      logic.
- [ ] Existing I008 hook-based learning behavior remains stable through the migration; no duplicate
      `TurnStart` / `TurnComplete` lifecycle events are introduced.
- [ ] Dead `event_loop.rs` variants are removed as part of the ADR-005 migration.
- [ ] `cargo test --workspace` exits 0 after each path migration step.

### R2 Non-Goals

- Nord theme, markdown rendering, diff display, steering queues, slash command polish.
- Guardian auto-approval and exec policy DSL.

## Slice R3: Product Polish

### Selected Stories

- [ ] #I010-S1: Nord theme application
- [ ] #I010-S2: Markdown rendering in assistant messages
- [ ] #I010-S3: Diff display for file changes
- [ ] #I010-S4: Steering and follow-up queues
- [ ] #I010-S5: Slash commands with fuzzy filtering

### Acceptance Criteria

- [ ] Daily TUI workflow is visually coherent and verified end-to-end in both full-screen and
      inline/no-alt-screen terminal modes.
- [ ] Assistant markdown, code blocks, diffs, and command output render without layout overlap.
- [ ] Slash commands cover common session, model, status, compact, fork, diff, and quit workflows.
- [ ] `cargo test --workspace` exits 0.

### Deferred Product Follow-Up

- `#I010-S6` Guardian AI sub-agent remains a backlog story, but is not part of the first R3 polish
  slice unless explicitly activated through change control.
- `#I010-S8` exec policy DSL remains a backlog story, but is not part of the first R3 polish slice
  unless explicitly activated through change control.

## Out of Scope

- Desktop app.
- Web UI.
- Mobile UI.
- Multi-agent side threads.

## Verification Notes

Append TUI screenshots or terminal recordings, command outputs, and SDK/headless examples here during
execution. I010 should not move to Review until the architecture slice and product polish slice both
have user-visible evidence.
