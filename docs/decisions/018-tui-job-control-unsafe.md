# 018: `unsafe` in TUI Job Control (`talos-tui`)

## Status

Accepted (drafted 2026-06-07 for I022 TUI inline-by-default refactor, sub-slice A)

This ADR satisfies **AGENTS.md Hard Constraint #2** ("No `unsafe` without ADR. Any use of `unsafe`
requires a decision record in `docs/decisions/`") for the SIGTSTP job-control code that lands in
I022 sub-slice A. It is a follow-on to [ADR-007](007-process-hardening-unsafe.md) (which records
the same `libc` FFI pattern for the sandbox hardening module) and applies the same review
discipline in a different domain.

## Context

I022 sub-slice A (`docs/iterations/I022-tui-inline-default.md`) introduces
`crates/talos-tui/src/tui/job_control.rs` — the Codex-pattern SIGTSTP handler that lets the
user press `Ctrl+Z` to suspend the foreground TUI and resume it later with `fg`. The codex
reference is `codex-rs/tui/src/tui/job_control.rs` (read in full 2026-06-06; see
`docs/reference/codex-tui-architecture.md` §4 for the verified pattern).

Sending `SIGTSTP` to the current process requires an OS call. There is **no safe Rust API**
for "raise a signal to the current PID":

- `std::process` exposes `exit`, `abort`, `id`, and child handling — none of them raise a
  specific POSIX signal to self.
- `tokio::signal::unix::signal(SignalKind::tstp())` is a **listener** for SIGTSTP arriving
  from outside, not a sender; it does not satisfy "TUI suspends itself on user request".
- The Rust standard library has no plan to add a safe `raise(3)` wrapper; the function mutates
  global process state and is generally considered to belong in an `unsafe` boundary.

Codex's `codex-rs/tui/src/tui/job_control.rs` solves this with one line:

```rust
// (Codex, codex-rs/tui/src/tui/job_control.rs)
unsafe { libc::raise(libc::SIGTSTP) };
```

— a single `unsafe` site that calls the C library's `raise(3)` POSIX function. We adopt the
same pattern.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| No `unsafe` without an ADR | Hard | AGENTS.md #2 | No (this ADR is the required record) |
| "No C/C++ bindings, no Python FFI, no Node.js" | Hard | AGENTS.md #1 | See clarification in ADR-007 §46-52 (`libc` is OS-ABI FFI, in-bounds) |
| No new dependencies without explicit Soft-constraint justification | Soft | AGENTS.md "Dependency Discipline" | Avoid `nix` / `signal-hook` crates; reuse the `libc` we already depend on transitively |
| Sandbox/permission code requires security review | Hard | AGENTS.md #5 | TUI job control is **not** sandbox/permission code, but AGENTS.md #5's review discipline still applies (single unsafe site, single function, clear `# Safety` note) |
| Codex is the PRIMARY TUI reference; module layout mirrors Codex | Soft | REFERENCE-PROJECTS.md §687-741 | Job-control feature is part of the Codex parity surface; dropping it would be a UX regression vs the reference |
| Each crate has a single responsibility | Soft | AGENTS.md | `tui/job_control.rs` is the TUI's only file concerned with OS signal dispatch for foreground/background transitions |
| `cargo test --workspace` exits 0 before merge (currently 652 tests) | Hard | AGENTS.md | I022 must not regress |
| I008 hook-based learning observes the same `HookEvent` ordering | Hard | ADR-006 | I022 must not break `hooks_e2e` / `mcp_client_e2e` at `RUST_LOG=debug` |
| Public API of `talos-tui` is semver-bound | Hard | AGENTS.md #6 | Job-control is internal (`pub(crate)`); no public API change |

**Clarification on `libc` reuse:** `libc` is already a transitive dependency of `talos-tui`
(via `crossterm` and `ratatui`). The I022 commit may either pull `libc` into
`crates/talos-tui/Cargo.toml` directly or rely on the transitive dep — the latter is
preferred to keep `Cargo.toml` clean. Either way, no new top-level dependency is added
(the `nix` and `signal-hook` crates would be).

## Reasoning

**Why `unsafe` is unavoidable here:**

- `raise(3)` is a C library function that mutates the process's signal mask and pending
  signals — observable side effects on process state, which is the textbook reason to gate
  behind `unsafe`.
- The Rust standard library exposes no safe alternative. `std::process::exit` aborts the
  process, not suspends it. `tokio::signal` listens, not sends.
- Wrapping the call in a third-party "safe raise" crate (e.g. `nix::sys::signal::raise`)
  moves the `unsafe` block from us into the wrapper and adds a top-level dependency, with
  no net safety improvement.

**Why the single site is sound (the invariants that make `unsafe` safe):**

1. **The call site is single-purpose.** `tui/job_control.rs::suspend(alt_screen_active)` is
   the only function in the entire `talos-tui` crate that raises a signal. The `unsafe`
   block is one expression: `libc::raise(libc::SIGTSTP)`.
2. **No pointer is constructed.** `libc::raise` takes a `c_int` (the signal number); no
   pointer arithmetic, no lifetime entanglement, no FFI struct marshalling. The C signature
   is `int raise(int sig)`.
3. **The return value is checked.** `raise(3)` returns 0 on success and -1 on error (with
   `errno` set). The implementation must check the return value and convert a -1 into a
   `JobControlError::RaiseFailed` (no UB path — failure is an error, not undefined
   behavior).
4. **Signal-handling context.** SIGTSTP is a job-control signal; its default action is to
   stop the process, which is exactly what we want. We do not install a custom handler for
   it (we do not call `sigaction`); we are not racing with any other code path that
   could be holding a lock. The Codex pattern documents this carefully — see
   `codex-rs/tui/src/tui/job_control.rs` `SuspendContext` and the `alt_screen_active:
   &Arc<AtomicBool>` parameter, which makes the suspend observable to the alt-screen
   toggling code.

**Binding requirement (this is the security review condition):**

- The `unsafe` block in `tui/job_control.rs` MUST carry a `// SAFETY:` comment that
  references this ADR (`docs/decisions/018-…`), and the module `# Safety` section MUST
  link here. This makes the ADR discoverable from the code.
- The `unsafe` block MUST call `libc::raise(libc::SIGTSTP)` and nothing else. Any other
  signal-raise, signal-mask manipulation, or `sigaction` use is a **separate concern**
  and requires an amendment to this ADR (or a new one) before merge.
- The return value MUST be checked and a `-1` return converted to a typed error.
- The function MUST be called from a single keyboard handler (the Ctrl+Z keybinding in
  `tui/keyboard_modes.rs` or equivalent), not from arbitrary event-loop paths.

**Why we don't need a new top-level dependency:**

- The `nix` crate is the standard safe wrapper around POSIX `libc` calls, but its
  `sys::signal::raise` is itself a thin wrapper around `libc::raise` — it does not
  provide additional safety, just a typed signature. We already have `libc` transitively.
- The `signal-hook` crate is for installing signal handlers (`sigaction` wrappers), not
  for raising signals to self. It does not apply.
- Adding either crate would be a net new top-level dependency on `talos-tui`, which is
  out of scope for I022 (per AGENTS.md "No speculative features").

## Decision

1. **A single production `unsafe` site is AUTHORIZED** in
   `crates/talos-tui/src/tui/job_control.rs`, calling `libc::raise(libc::SIGTSTP)` to
   suspend the foreground TUI on user Ctrl+Z request.

2. **`libc` is an approved dependency for OS signal dispatch** in `talos-tui` (it is
   already a transitive dependency via `crossterm` and `ratatui`; if the I022 implementation
   references `libc::raise` directly, the crate should add `libc` to its direct deps for
   clarity, not to gain a new capability). This is consistent with ADR-007's §46-52
   clarification that `libc` is OS-ABI FFI, not a third-party C library.

3. **No new top-level dependencies** (`nix`, `signal-hook`, etc.) are introduced for job
   control. The `unsafe` site is the minimal primitive.

4. **Mandatory code annotations (enforced by review):**
   - Every `unsafe` block in `tui/job_control.rs` MUST carry a `// SAFETY:` comment that
     references this ADR.
   - The module-level `# Safety` section MUST link to this ADR.
   - The function signature MUST be `fn suspend(alt_screen_active: &Arc<AtomicBool>) -> Result<(), JobControlError>`.

5. **Behavioral invariants (Hard Constraint #5 review discipline):**
   - The function suspends the process; it does not install a signal handler.
   - The function returns a typed `Result`; the `-1` error from `libc::raise` is mapped
     to `JobControlError::RaiseFailed(std::io::Error::last_os_error())`.
   - The function is called from the Ctrl+Z keybinding handler only; it is not exposed
     as a public API.
   - Any new `unsafe` in `talos-tui/src/tui/` requires an update to this ADR (or a new
     one) **before** merge.

6. **When SIGTSTP support is intentionally out of scope** (e.g., a CI environment where
   the test harness cannot suspend the process), the `unsafe` site is still present
   (the function is part of the TUI feature surface) but the keybinding may be gated
   by a `feature = "job-control"` cfg flag in a follow-up iteration. For I022, the
   keybinding is **always** active.

**Rejected alternatives:**

- *Add the `nix` crate as a safe wrapper* — adds a new top-level dependency, moves the
  `unsafe` into the wrapper, and does not improve safety. The `unsafe` site would be
  smaller in our code, not absent. Net loss.
- *Use `tokio::signal::unix::signal(SignalKind::tstp())` to "send" SIGTSTP* — this is
  semantic confusion: `tokio::signal::*` installs a *handler* for incoming signals, it
  does not send signals. Cannot satisfy the use case.
- *Shell out to `kill -TSTP $$` or `raise(1)`* — adds process-spawn fragility, non-portable
  across shells, parse-fragile, and still depends on shell semantics. Strictly worse
  than one checked libc call.
- *Drop SIGTSTP support and document it as a limitation* — the Codex reference supports
  Ctrl+Z suspension; dropping it would be a UX regression vs the primary reference and
  violate the user's "Codex-like experience" intent. Out of scope.
- *Use `std::process::Command::kill()`* — sends SIGKILL to a child, not SIGTSTP to self.
  Not applicable.
- *Wrap in a Talos-internal "safe raise" helper that hides the `unsafe`* — the helper
  would still be `unsafe`; the `unsafe` surface does not shrink, it just moves. Review
  ownership gets harder, not easier.

## Reversal Trigger

Revisit this decision if any of the following hold:

- Rust std gains a **safe** signal-raise API covering `SIGTSTP` — then migrate off `libc`/`unsafe`
  and supersede this ADR. (No such API is on the stdlib roadmap as of 2026-06-07.)
- The job-control requirements change (e.g., we need to *catch* SIGTSTP from another
  source, or we need to send a different signal) — then this ADR is amended or
  superseded; a new unsafe site would be a separate decision.
- We adopt a signal-handling crate (e.g. `signal-hook`) for other reasons and want to
  consolidate — then this ADR is amended to document the consolidated surface, and
  the `unsafe` site is moved into the wrapper or kept here as the minimal primitive.
- The I022 implementation discovers that the function needs additional unsafe operations
  (e.g. `sigaction`, `sigprocmask`) — then **halt and redesign**, do not silently widen
  this ADR. Each new unsafe call is a separate decision.

## Related

- `docs/iterations/I022-tui-inline-default.md` — the iteration plan that introduces
  `tui/job_control.rs` and triggers this ADR.
- `docs/proposals/tui-codex-overhaul.md` — sub-slice A scope (the `tui/` subdir
  plumbing, including `job_control.rs`).
- `docs/reference/codex-tui-architecture.md` §4 — verified Codex source for the SIGTSTP
  handler pattern (with file:line evidence).
- [ADR-007](007-process-hardening-unsafe.md) — parent ADR for the `libc` FFI pattern in
  a different module (`talos-sandbox`). This ADR is a sibling, not a child.
- [ADR-003](003-tui-progressive-evolution.md) — TUI evolution anchor; I022 is the next
  TUI iteration.
- `crates/talos-tui/src/app.rs:50-71` (current `EnterAlternateScreen` call) and
  `crates/talos-tui/src/app.rs:614-625` (current `LeaveAlternateScreen` call) — the
  two sites that I022 sub-slice A removes, completing the inline-by-default flip.
- AGENTS.md Hard Constraints #1 (no C/C++ bindings), #2 (no `unsafe` without ADR),
  #5 (sandbox review), #6 (semver-bound public APIs).
