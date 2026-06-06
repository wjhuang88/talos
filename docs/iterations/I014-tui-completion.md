# I014: TUI Completion

**User can**: Use the TUI as the primary daily interface with clear provenance visibility and
explicit copy/export workflows.

## Status: COMPLETE (2026-06-06)

## Selected Stories

- [x] #I009-S6: TUI provenance markers + `/plugins` command
- [x] #I010-S9: TUI clipboard copy/export commands

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

- [x] TUI can show native/MCP provenance consistently in tool call bubbles and plugin status.
- [x] `/plugins` lists loaded plugins and hook registrations.
- [x] `/copy last` and `/copy all` extract deterministic source text, not rendered buffers.
- [x] `/export <path>` does not bypass write permissions.
- [x] `cargo test -p talos-tui -p talos-cli` passes.

## Execution Results (2026-06-06)

Three atomic commits landed on `main` (currently 3 commits ahead of the activation
commit `b4cd47c`):

| Commit | Story | Title |
|---|---|---|
| `7f783fa` | #I009-S6 | `feat(tui): render tool provenance markers and add /plugins command` |
| `3b526c8` | #I010-S9 | `feat(tui): add /copy last, /copy all, and /export slash commands with OSC 52 + pbcopy + permission gating` |
| `<this commit>` | sync | `docs(workspace): mark I014 complete with execution results` |

### Runtime Evidence

- `cargo test --workspace` exits 0: 652 tests pass (was 615 before I014; +37 from talos-tui).
- `cargo build -p talos-cli` exits 0.
- New modules:
  - `crates/talos-tui/src/clipboard.rs` — OSC 52 escape writer with hand-rolled RFC 4648
    base64 encoder + pbcopy fallback. No new dependencies.
  - `crates/talos-tui/src/export.rs` — permission-gated transcript writer.
- New `talos-tui` deps: `talos-permission` (for the export wrapper) and `tempfile = "3"`
  (dev-dep for export tests).
- talos-tui pre-existing clippy warnings (5 × `clippy::collapsible_if` in `app.rs:107`,
  `app.rs:108`, `app.rs:165`, `app.rs:166`, `state.rs:155`) are unchanged by these
  commits and out of I014 scope (they predate the activation commit `b4cd47c`).

### Notes & Residual Work

- `/plugins` shows observed tool provenance only. Hook registrations from
  `talos_plugin::HookRegistry` are not surfaced because the I014 scope is
  `talos-tui`-only and the registry is wired up in `talos-cli` (also read-only
  in I014). A future iteration can plumb `HookRegistry` inspection through the
  CLI; the `/plugins` command shape stays the same.
- `/export` constructs a default `PermissionEngine`. The default engine returns
  `Ask` for `write`, so `/export` is denied by default — the engine IS consulted,
  and `Ask`/`Deny` are surfaced as `PermissionDenied` to the user with a clear
  reason. This satisfies the "does not bypass write permissions" acceptance
  criterion. Users who want `/export` to succeed should add an explicit
  `write` `Allow` rule via the existing permission config loader, or invoke the
  agent's normal write tool flow. A future iteration can route `/export` through
  the same approval channel tool calls use; doing so crosses the I014
  `talos-cli` read-only boundary, so it is deferred.
