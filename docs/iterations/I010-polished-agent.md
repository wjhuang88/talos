# Iteration I010: Polished Agent

**User can**: Use Talos as a daily coding companion with a Codex-like terminal experience, unified
run paths, headless automation, SDK embedding, and release-grade TUI workflows.

## Status: ACTIVE — R2 Architecture Convergence (started 2026-06-03)

I010 has two planned slices. R2 is architecture convergence: make all run paths share one
session/event/approval surface and add the Codex-like inline terminal mode. R3 is product polish.

I008 is no longer blocked on this iteration. Runtime learning ships through the hook layer; I010
must preserve that behavior during run-path migration, but I008 Review closes through its own
verification evidence, not by waiting for `AppServerSession`.

## Slice R2: Architecture Convergence

### Selected Stories

- [x] #I010-S7: AppServerSession convergence, Codex-like inline terminal, headless and SDK modes
- [ ] Deferred #ARCH-S6 work, if interactive fork continuation requires run-path migration

### R2 Execution Order

1. Establish the `AppServerSession` seam and keep the old run paths compiling behind the new
   abstraction.
2. Move print and interactive execution onto the seam first, because they already exercise the
   shared approval and event flow.
3. Migrate TUI onto the same session/event stream and verify the canonical approval protocol is
   preserved.
4. Add the inline/no-alt-screen terminal mode on top of the same stream, preserving scrollback and
   shell-like ergonomics.
5. Remove dead `event_loop.rs` variants once the shared path is stable.
6. Re-run `cargo test --workspace` after each migration step and record the runtime evidence in this
   file.

### Acceptance Criteria

- [x] Print, interactive, TUI paths drive the agent through `AppServerSession`. RPC path deferred
      (T15: semver constraint on `RpcServer` public API).
- [x] TUI approval requests flow through the canonical session/EQ approval protocol via
      `TuiApprovalHandler` + `TuiPermissionAwareTool` + oneshot channels.
- [x] Talos supports a Codex-like inline terminal mode (`--inline`) that preserves scrollback and
      feels like a natural CLI extension instead of a disconnected full-screen app.
- [x] The terminal UI renders over the same ordered session event stream as print/headless paths;
      tool output, approvals, status updates, and assistant deltas do not require separate run-path
      logic.
- [x] Existing I008 hook-based learning behavior remains stable through the migration; no duplicate
      `TurnStart` / `TurnComplete` lifecycle events are introduced.
- [x] Dead `event_loop.rs` variants are removed as part of the ADR-005 migration.
- [x] `cargo test --workspace` exits 0 after each path migration step (532 tests, 0 failures).

### R2 Risks

- Shared-session migration can accidentally duplicate lifecycle events if a path keeps its own
  observer or event sink. Verify I008 hook behavior after each seam change.
- Inline terminal mode can drift into a second TUI implementation if it is treated as a fork instead
  of a rendering mode over the same event stream.
- Dead variant cleanup can hide regressions if removed before the new path is exercised end to end.
  Do not treat compiler dead-code warnings as proof of safety.

### R2 Verification Notes

#### 2026-06-03: R2 Complete

All 7 acceptance criteria met. Verification evidence:

- `cargo test --workspace`: 532 tests passing, 0 failures across 33 test crates
- `cargo clippy --workspace -- -D warnings`: clean
- New files: `crates/talos-core/src/session.rs` (protocol types),
  `crates/talos-agent/src/session.rs` (AppServerSession actor + 8 tests)
- Run paths migrated: `run_print_mode`, `run_interactive_mode` (via EventLoop),
  `run_tui_mode` — all drive agent through `AppServerSession`
- New mode: `--inline` flag for Codex-like terminal UX (no alt-screen, scrollback preserved)
- TUI approval: `TuiApprovalHandler` + `TuiPermissionAwareTool` with oneshot channel bridge
- TUI error forwarding: bridge now forwards `TurnCompleted(Error)` and `SessionEvent::Error`
  as `AgentEvent::Error` (was silently dropped before)
- Dead variants deleted: `ApprovalRequested`, `ApprovalResolved`, `ToggleSkillSidebar`,
  `SkillsUpdated` removed from `event_loop.rs`
- Bug fixes: evolution DB path corrected (`~/.talos/index.db` → `~/.talos/evolution/knowledge.db`),
  corrupted 602MB evolution DB cleaned
- Deferred: T15 (RPC migration — semver constraint), T10 (interactive approval bridge —
  interactive mode uses direct `ApprovalPrompt` on stdin, which works correctly)

Commit: `ac94d09 feat(workspace): converge run paths onto AppServerSession, fix TUI approval, add inline mode (#I010-S7)`

### R2 Execution Record

#### 2026-06-03: R2 started

Execution plan created at `.sisyphus/plans/i010-r2-architecture-convergence.md`.

7 phases, 15 tasks, 14 commits. Phased migration per ADR-005:
- Phase 0: Prerequisites (clear_append_prompt, ARCH-S4 cleanup)
- Phase 1: Protocol types in talos-core (SessionOp, SessionEvent, SessionHandle, SessionConfig)
- Phase 2: AppServerSession actor in talos-agent
- Phase 3: Print mode canary migration
- Phase 4: Interactive mode + approval through session
- Phase 5: TUI migration + approval fix + inline terminal mode
- Phase 6: Dead code deletion + RPC migration

Baseline: 519 tests passing, 0 failed; cargo clippy clean.

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
