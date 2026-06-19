# TOOL-005: Bash Tool Streaming Output

**Status**: Planned
**Priority**: P2
**Source**: User request 2026-06-19
**Depends on**: None (self-contained bash tool change)

## Problem

The bash tool currently captures all stdout/stderr into a buffer and returns
the complete output as a single `ToolResult`.  For long-running commands
(such as builds, installs, or large file processing), the user sees no output
until the command completes — making it difficult to distinguish slow
commands from hung ones, and reducing the interactive feel of the TUI.

## Scope

Enhance the bash tool to emit output progressively instead of buffering it
all at once.

### Required behavior

1. **Print the command itself first**: Before any output, emit a line showing
   the command that was executed, prefixed with `$ ` (e.g. `$ cargo build`).
   This gives context to the output that follows.

2. **Stream stdout line-by-line**: As the child process produces output on
   stdout, emit each line to the tool result stream without waiting for the
   process to exit.

3. **Stream stderr line-by-line**: Same as stdout — emit stderr lines
   progressively, preserving interleaving with stdout.

4. **Preserve timeout behavior**: The existing timeout logic (kill process
   after `timeout_secs`) must still work; any output collected before the
   timeout fires must be emitted.

5. **Preserve exit code**: Emit the final exit status (e.g. `[exit 0]` or
   `[exit 1]`) after the process completes.

### Non-goals

- Do not change the bash tool's input schema (`BashInput`) or API.
- Do not change how other tools interact with the bash tool.
- Do not add a TTY/PTY mode (pseudo-terminal allocation remains out of scope).

## Acceptance Criteria

- [ ] Executing a bash command emits `$ <command>` as the first line of output.
- [ ] stdout lines appear in the output as the process produces them.
- [ ] stderr lines appear interleaved with stdout as produced.
- [ ] The existing timeout still kills the process and emits collected output.
- [ ] The final exit code is appended to the output.
- [ ] `cargo test -p talos-tools` passes.
- [ ] Existing bash tool tests still pass (echo, timeout, invalid command, etc.).

## Implementation Notes

The current implementation uses `tokio::process::Command` and captures
`output.stdout` + `output.stderr` after `wait_with_output()`.  Streaming
requires switching to:
- `tokio::process::Command::stdout(Stdio::piped())` / `stderr(Stdio::piped())`
- `child.stdout.take()` as `BufReader` for line-by-line reading
- `tokio::select!` between the stdout reader, stderr reader, and timeout
- Accumulating lines into a `Vec<String>` and joining at the end for the
  `ToolResult`

## Required Reads

- `crates/talos-tools/src/bash_tool.rs`
- `crates/talos-tools/Cargo.toml`
