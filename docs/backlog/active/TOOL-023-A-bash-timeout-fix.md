# TOOL-023-A: Fix Bash Timeout Defeated by Continuous Output

**Status**: Ready (2026-07-24)
**Priority**: P1
**Parent Epic**: TOOL-023
**Type**: Technical Story (bug fix)
**Depends on**: none

## Problem

`crates/talos-tools/src/bash_tool.rs::run_command` runs this shape:

```rust
let exit_status = loop {
    tokio::select! {
        line = stdout_reader.next_line() => { ... }   // resets loop
        line = stderr_reader.next_line() => { ... }   // resets loop
        status = child.wait() => { break status; }
        _ = tokio::time::sleep(timeout_duration) => { kill; return timeout }
    }
};
```

`tokio::time::sleep(timeout_duration)` is created fresh on every loop iteration.
Whenever a stdout/stderr line arrives, `select!` returns, the loop re-iterates, and
the timer is dropped and restarted. A subprocess that prints a line more frequently
than `timeout_duration` (progress logs, `tail -f`, a retry loop, a chatty hung
network client) **never hits the timeout** and the tool hangs unbounded. This is the
user-reported "shell calls sometimes hang for an unbounded time".

`exec_tool.rs` (lines ~718–733) shows the correct pattern: pipe readers run in
detached `tokio::spawn` tasks and the deadline is a single-shot
`select! { child.wait(), sleep(timeout) }` outside any loop.

## Goal / Value

`bash` enforces its timeout as an absolute wall-clock deadline from spawn,
independent of how often the child writes output. Restores the safety guarantee the
tool already claims to provide.

## Scope

- Restructure `bash_tool.rs::run_command` so the timeout is a single-shot deadline
  measured from spawn, not reset by output. Prefer the `exec` pattern (detached
  bounded readers + one `select!`), or wrap the read loop in `tokio::time::timeout`;
  whichever is chosen must still kill the child and drain already-produced output on
  expiry, preserving the current `[timeout]` marker and drain-after-kill behavior.
- Kill semantics: on Unix the existing `child.kill()` targets the direct `sh` child.
  Document (do not necessarily implement in this story) whether orphaned grandchild
  processes survive; if process-group kill is needed it is recorded as a residual.

## Exclusions

- No timeout default change (that is TOOL-023-B).
- No Windows shell change (that is TOOL-023-C).
- No new dependency; use `tokio` primitives already in the workspace.

## Decision Links And Constraints

- ADR-009 (external dependencies must not crash the process): the kill+drain path
  must degrade gracefully, never panic.

## Uncertainty And Validation Path

Regression must reproduce the hang deterministically: a fake command that emits a
line every N ms and never exits, with a timeout < total runtime, must return the
`[timeout]` error within a bound close to the deadline.

## State/Status Owners

This story file; parent `TOOL-023`; `docs/BOARD.md` mirror.

## User-Facing Documentation

None (behavior returns to what docs already promise). If the README bash timeout
wording implies output resets the timer, correct it; otherwise no doc change.

## Required Reads

- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/src/exec_tool.rs` (reference timeout implementation)

## Acceptance for behavior

- Given a subprocess that prints a line every 100ms and never exits, and a bash tool
  timeout of 1s
  When the model invokes `bash` on that command
  Then the tool returns a `[timeout]` error within a small bound of 1s (not
  unbounded), the child is killed, and output produced before the deadline is
  present in the result.

- Given a subprocess that exits normally in under the timeout while producing output
  When invoked via `bash`
  Then the tool returns the full output and the real exit code (no regression to the
  success path).

## Acceptance for technical work

- [ ] A new test in `bash_tool.rs` reproduces the continuous-output hang and asserts
      the timeout fires (test would fail/hang against the current code).
- [ ] `cargo test -p talos-tools --locked` passes.
- [ ] `cargo clippy --workspace --locked -- -D warnings` clean.
- [ ] Parent `TOOL-023` and Board status synchronized.
- [ ] Grandchild-process kill scope recorded as residual if not addressed.
