# 007: `unsafe` in Process Hardening (`talos-sandbox`)

## Status

Accepted

This ADR satisfies **AGENTS.md Hard Constraint #2** ("No `unsafe` without ADR. Any use of `unsafe`
requires a decision record in `docs/decisions/`") for the `talos-sandbox` process-hardening code.
Until this record existed, the `unsafe` blocks in `crates/talos-sandbox/src/hardening.rs` were a
**compliance violation** (code shipped, tests passed, but the required ADR was missing —
acknowledged as a gap in `EVOLUTION.md`).

## Context

`ProcessHardening` (`crates/talos-sandbox/src/hardening.rs`) applies OS-level security measures
before running untrusted commands: environment sanitization (strip `LD_PRELOAD`, `DYLD_*`, …) and
resource limits (CPU, address space, core-dump suppression) via `setrlimit(2)`. These operations
have **no safe Rust equivalent** — they require either `std::env::remove_var` (declared `unsafe` in
edition 2024 due to multithreaded data-race hazards) or direct `libc` syscalls.

There are **four production `unsafe` sites** (test-only `unsafe` for `env::set_var`/`remove_var` in
`#[cfg(test)]` is out of scope — it does not ship):

| # | Location | Call | Purpose |
|---|----------|------|---------|
| 1 | `hardening.rs:254` | `env::remove_var(var)` | Strip a dangerous env var from the current process |
| 2 | `hardening.rs:284` | `libc::setrlimit(RLIMIT_CORE, …)` | Disable core dumps (no memory snapshot leakage) |
| 3 | `hardening.rs:299` | `libc::setrlimit(RLIMIT_CPU, …)` | Cap CPU seconds (runaway prevention) |
| 4 | `hardening.rs:314` | `libc::setrlimit(RLIMIT_AS, …)` | Cap address space (OOM prevention) |

A **fifth, planned** `unsafe` site is pre-authorized by this ADR: `Command::pre_exec` for applying
hardening **in the child process** (see [#ARCH-S3](../backlog/PRODUCT-BACKLOG.md)). `pre_exec` is
`unsafe` because its closure runs post-`fork`/pre-`exec`, where only async-signal-safe operations
are permitted.

I191 adds one further child-side `pre_exec` site in
`crates/talos-tools/src/process_boundary.rs`. It calls `setsid(2)` before `exec` so non-interactive
tool children cannot open the Talos TUI's controlling terminal through `/dev/tty`. This is OS ABI
access under the same hard constraint and post-fork safety boundary.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| No `unsafe` without an ADR | Hard | AGENTS.md #2 | No (this ADR is the required record) |
| No `unwrap()` in library code; errors via `thiserror` | Hard | AGENTS.md | No |
| Sandbox/permission/hardening changes require security review | Hard | AGENTS.md #5 | No |
| "No C/C++ bindings, no Python FFI, no Node.js" | Hard | AGENTS.md #1 | See clarification below |
| Resource limits / env scrubbing must actually take effect on untrusted code | Assumption | I004 security goal | Validated by #ARCH-S3 wiring |

**Clarification on Hard Constraint #1 (`libc` is permitted):** "No C/C++ bindings" prohibits
linking third-party **C/C++ libraries** and C++ runtimes. The `libc` crate is the standard Rust FFI
surface to the **operating system's own syscall ABI** (POSIX `setrlimit`), not a third-party C
library. It is pure-Rust declarations over a stable OS interface, has no C/C++ build step of its
own, and is the only portable way to call `setrlimit`. This is consistent with the project already
depending on OS/std-level FFI. `libc` use is therefore **in-bounds**; this ADR records the
reasoning so the point is not re-litigated.

## Reasoning

**Why `unsafe` is unavoidable here:**

- `std::env::remove_var` is `unsafe` in edition 2024 because mutating the process environment races
  with concurrent readers in other threads. There is no safe wrapper.
- `setrlimit(2)` is exposed only through `libc`; the Rust stdlib offers no resource-limit API.
- Child-process hardening (`pre_exec`) is `unsafe` by construction (post-fork constraints).

**Why each site is sound (the invariants that make `unsafe` safe):**

1. **`env::remove_var` (site 1).** Hardening is documented and intended to run **before** any worker
   threads or child processes are spawned, so no concurrent environment reader exists at call time.
   The function takes `&self` and the call site holds exclusive logical access. *Soundness depends
   on the call-ordering invariant* — see the binding requirement below.
2–4. **`setrlimit` (sites 2–4).** Each call constructs a fully-initialized `libc::rlimit` value and
   passes a valid `&rlim as *const _` pointer; the `RLIMIT_*` constants are well-defined POSIX
   values; the return value is checked and converted to `SandboxError::ExecutionFailed` on `-1`
   (no UB path — failure is an error, not undefined behavior). The struct is stack-local and
   outlives the call. These calls cannot violate memory safety.
5. **`pre_exec` (planned, site 5).** The closure must perform only async-signal-safe work:
   `setrlimit` and `unsetenv`-style operations qualify; allocation, locking, and arbitrary Rust
   must not appear inside it. #ARCH-S3 is bound to this restriction.
6. **`setsid` terminal containment.** `setsid(2)` is async-signal-safe and receives no pointers.
   A `-1` result is converted from the child-local `errno` into the spawn error; success creates a
   new Session without Talos's controlling terminal. The closure performs no allocation, locking,
   formatting or panic-capable work.

**Binding requirement (this is the security review condition, per Hard Constraint #5):**

- Site 1's soundness **requires** that `ProcessHardening::apply()` (the parent-side env scrub) only
  ever runs before threads/children are spawned. The existing module-level `# Safety` note states
  this assumption; any future caller that violates it reopens a data race and **must** be rejected
  in review.
- The current defect (tracked by #ARCH-S3) is that `apply()` is **never wired into the actual child
  execution path** — the `unsafe` code is correct but **inert**, producing a *security illusion*.
  Hardening must be applied to the **child** (bash subprocess) via `pre_exec`, **not** to the parent
  CLI process (applying `RLIMIT_AS`/`RLIMIT_CPU` to the parent would cripple Talos itself).

## Decision

1. **The four production `unsafe` sites in `hardening.rs` (lines 254, 284, 299, 314) are ACCEPTED**
   as the minimal, sound, and only-available means to scrub the environment and set resource limits.

2. **`libc` is an approved dependency** for OS syscall access (`setrlimit`); it does not violate the
   "no C/C++ bindings" constraint (see clarification above).

3. **Child-process `pre_exec` hardening is PRE-AUTHORIZED** (`unsafe`), subject to the
   async-signal-safe restriction, and is to be implemented under
   [#ARCH-S3](../backlog/PRODUCT-BACKLOG.md).

3a. **Child-process terminal detachment via `setsid` is ACCEPTED** for the shared non-interactive
`bash`/`exec` boundary under I191/TOOL-026. Standard input remains null unless an explicit `exec`
pipeline supplies bytes; detachment does not authorize interactive PTY support.

4. **Mandatory code annotations (enforced by #ARCH-S1):** every `unsafe` block listed above MUST
   carry a `// SAFETY:` comment that references this ADR (`docs/decisions/007-…`), and the module
   `# Safety` section MUST link here. This makes the ADR discoverable from the code.

5. **Security-review invariants (Hard Constraint #5):**
   - Parent-side `env::remove_var` only before threads/children spawn.
   - Resource limits apply to the **child**, never the parent CLI.
   - `pre_exec` closures stay async-signal-safe.
   - Any new `unsafe` in `talos-sandbox`/`talos-permission` requires an update to this ADR (or a new
     one) **before** merge.

**Rejected alternatives:**
- *Forbid `unsafe` and drop resource limits* — would silently remove a security control; unacceptable for a hardening module.
- *Shell out to `ulimit`/`prlimit` binaries* — adds process/parse fragility, non-portable, still needs env scrubbing; strictly worse than one checked syscall.
- *Wrap in a third-party "safe rlimit" crate* — adds a dependency that itself wraps the same `libc` `unsafe`; moves the `unsafe`, does not remove it, and dilutes review ownership.

## Reversal Trigger

Revisit this decision if any of the following hold:

- Rust std gains a **safe** resource-limit / environment-mutation API covering these cases — then
  migrate off `libc`/`unsafe` and supersede this ADR.
- The call-ordering invariant for site 1 cannot be guaranteed (e.g., hardening must run after
  threads exist) — then site 1 must be redesigned (apply only in the freshly-forked child via
  `pre_exec`, where no other threads exist) and this ADR amended.
- #ARCH-S3 reveals that child-side hardening needs operations that are **not** async-signal-safe —
  halt and redesign the wiring (e.g., a pre-spawned helper) rather than widening the `pre_exec`
  closure.

## Related

- `crates/talos-sandbox/src/hardening.rs` (the four `unsafe` sites + module `# Safety` note)
- [#ARCH-S1](../backlog/PRODUCT-BACKLOG.md) (annotate `unsafe` blocks with this ADR — closes the compliance gap)
- [#ARCH-S3](../backlog/PRODUCT-BACKLOG.md) (wire hardening into child execution via `pre_exec` — closes #I004-S5 false-complete)
- PRODUCT-BACKLOG.md #I004-S5 (Process hardening basics — originally marked complete)
- AGENTS.md Hard Constraints #1 (no C/C++ bindings), #2 (no `unsafe` without ADR), #5 (sandbox review)
- EVOLUTION.md (records the original missing-ADR gap now closed by this record)
