# Iteration I010: Polished Agent

**User can**: Use Talos as a daily coding companion with a Codex-like terminal experience, unified
run paths, headless automation, SDK embedding, and higher-level approval policy controls.

## Status: PLANNED

I010 has two planned slices. The architecture slice may be pulled forward if required to close I008
Review status, because I008 TUI/interactive evolution wiring must attach at the shared
AppServerSession seam.

## Slice R2: Architecture Convergence

### Selected Stories

- [ ] #I010-S7: Headless and SDK modes
- [ ] Deferred #ARCH-S6 work, if interactive fork continuation requires run-path migration

### Acceptance Criteria

- [ ] Print, interactive, TUI, headless, and SDK paths drive the agent through `AppServerSession`.
- [ ] TUI approval requests flow through the canonical session/EQ approval protocol.
- [ ] Talos supports a Codex-like inline terminal mode that preserves scrollback and feels like a
      natural CLI extension instead of a disconnected full-screen app.
- [ ] The terminal UI renders over the same ordered session event stream as print/headless paths;
      tool output, approvals, status updates, and assistant deltas do not require separate run-path
      logic.
- [ ] I008 `TurnObserver` / `BehaviorAdapter` attach once at the session/EQ seam.
- [ ] I008 can move from Review to Complete after TUI/interactive runtime evidence is recorded.
- [ ] Dead `event_loop.rs` variants are removed as part of the ADR-005 migration.
- [ ] `cargo test --workspace` exits 0 after each path migration step.

## Slice R3: Product Polish

### Selected Stories

- [ ] #I010-S1: Nord theme application
- [ ] #I010-S2: Markdown rendering in assistant messages
- [ ] #I010-S3: Diff display for file changes
- [ ] #I010-S4: Steering and follow-up queues
- [ ] #I010-S5: Slash commands with fuzzy filtering
- [ ] #I010-S6: Guardian AI sub-agent
- [ ] #I010-S8: Exec policy DSL rules

### Acceptance Criteria

- [ ] Daily TUI workflow is visually coherent and verified end-to-end in both full-screen and
      inline/no-alt-screen terminal modes.
- [ ] Assistant markdown, code blocks, diffs, and command output render without layout overlap.
- [ ] Slash commands cover common session, model, status, compact, fork, diff, and quit workflows.
- [ ] Guardian can auto-approve low-risk tool calls with a circuit breaker.
- [ ] Exec policy DSL can load trusted/forbidden rules from configured rule files.
- [ ] `cargo test --workspace` exits 0.

## Out of Scope

- Desktop app.
- Web UI.
- Mobile UI.
- Multi-agent side threads.

## Verification Notes

Append TUI screenshots or terminal recordings, command outputs, and SDK/headless examples here during
execution. I010 should not move to Review until the architecture slice and product polish slice both
have user-visible evidence.
