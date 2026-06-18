# I027: ARCH-003 Crate Boundary Cleanup

**Status**: Complete
**Started**: 2026-06-18 (plan opened)
**Closed**: 2026-06-18
**Depends On**: ARCH-002 (complete)

## Outcome

Tighten three crate boundaries identified by the ARCH-002 audit so future RPC/MCP/API surface
does not build on top of accidental coupling. No runtime behavior change.

## Selected Stories

- [x] #ARCH-003-A: Remove dead `talos-mcp -> talos-agent` dependency (XS)
- [x] #ARCH-003-B: Rename `talos_core::message::ToolResult` → `MessageToolResult` (S)
- [x] #ARCH-003-C: Introduce RPC-facing runtime trait; decouple `talos-rpc` from concrete
      `talos_agent::Agent` (M)

## Scope Details

### ARCH-003-A — Remove dead `talos-mcp -> talos-agent` dependency

Delete `talos-agent = { path = "../talos-agent" }` (line 13) from
`crates/talos-mcp/Cargo.toml`. Confirmed zero source usage (`use talos_agent`, `extern crate`,
`talos_agent::`, cfg/feature-gated references all return 0 hits across the crate tree).

### ARCH-003-B — Rename `message::ToolResult` → `MessageToolResult`

Rename the data/protocol `ToolResult` in `crates/talos-core/src/message.rs:39` so it no longer
collides conceptually with the execution-layer `talos_core::tool::ToolResult`. 10 files to
touch (see ARCH-003 exploration notes). `talos-agent/src/lib.rs` already aliases
`ToolResult as MessageToolResult`, so the alias collapses to a direct import after rename.

### ARCH-003-C — Decouple `talos-rpc` from concrete `Agent`

`MethodContext { agent: Arc<Agent> }` in `crates/talos-rpc/src/methods/mod.rs:24-30` exposes the
concrete `talos_agent::Agent`. Introduce a narrow RPC-owned `Runtime` trait with the 2 methods
RPC actually calls (`run`, `run_streaming`); change `MethodContext.agent` to `Arc<dyn Runtime>`;
move the concrete `Agent` wiring to `talos-cli`. `talos-rpc/Cargo.toml` should no longer declare
`talos-agent` as a direct dependency.

## Risks

- **R1 (ARCH-003-C)**: `AgentError` and `AgentResult<T>` leak via the Agent method signatures.
  The trait must abstract the error type (e.g., `Box<dyn Error + Send + Sync>` or a new
  `talos-core::RuntimeError` enum) so `talos-rpc` does not re-introduce `talos-agent` as a
  dependency through error types.
- **R2 (ARCH-003-C)**: `run_streaming` currently returns `(String, Vec<Message>)` but RPC
  discards the `Vec<Message>` part. The trait can simplify the return to `String`, but the
  `Agent` impl must keep the full return type internally and the adapter drops the messages.
- **R3 (ARCH-003-B)**: `AgentEvent::ToolResult` is an enum variant name and is intentionally
  NOT in scope; only the `message::ToolResult` struct is renamed.

## Acceptance Criteria

- [x] `crates/talos-mcp/Cargo.toml` no longer depends on `talos-agent`.
- [x] `crates/talos-rpc/Cargo.toml` no longer depends on `talos-agent`.
- [x] `talos-rpc` public API no longer names `talos_agent::Agent`.
- [x] `talos_core::message::ToolResult` is renamed to `MessageToolResult`; no remaining
      references to the old name in the workspace.
- [x] `cargo check --workspace` passes.
- [x] `cargo test -p talos-rpc -p talos-mcp -p talos-core` passes.
- [x] `cargo test --workspace` passes.
- [x] `cargo clippy --workspace -- -D warnings` passes.

## Commit Strategy

Three atomic commits, one per story (matches ARCH-003 backlog acceptance):

1. `refactor(mcp): remove dead talos-agent dependency (#ARCH-003-A) [model:glm-5.2]`
2. `refactor(core): rename message::ToolResult to MessageToolResult (#ARCH-003-B) [model:glm-5.2]`
3. `refactor(rpc): introduce Runtime trait and decouple from concrete Agent (#ARCH-003-C) [model:glm-5.2]`

## Verification Log

- 2026-06-18: `cargo check --workspace` — passed (1.22s after all three stories landed).
- 2026-06-18: `cargo test --workspace` — all suites green (0 failures, 1 pre-existing
  ignored timing-sensitive test retained).
- 2026-06-18: `cargo clippy --workspace -- -D warnings` — passed (19.74s).
- 2026-06-18: `cargo fmt --all` — no changes.
- 2026-06-18: `scripts/validate_project_governance.sh .` — 0 warnings.
- 2026-06-18: targeted grep audits:
  - `grep -c "talos-agent" crates/talos-mcp/Cargo.toml` → 0
  - `grep -c "talos-agent" crates/talos-rpc/Cargo.toml` → 0
  - `grep -rn "message::ToolResult\b" crates/ --include="*.rs"` (excluding `MessageToolResult`
    and `AgentEvent::ToolResult` variant) → 0 matches
  - `AgentEvent::ToolResult` variant preserved (not renamed); only its field type changed to
    `MessageToolResult`.
  - `talos_core::tool::ToolResult` (execution-layer) untouched.

### Risk Realizations

- **R1 (AgentError leak)**: handled by mapping `AgentError → anyhow::Error → RuntimeError` at
  the CLI composition root. `talos-rpc` never names `AgentError`.
- **R2 (run_streaming tuple return)**: handled — `Runtime::run_streaming` returns `String`
  directly; `AgentRuntime` adapter destructures `(text, _messages)` internally.
- **R3 (AgentEvent::ToolResult variant)**: preserved as designed — only the struct was renamed.

### Parallel Execution Note

ARCH-003-B and ARCH-003-C were delegated to two background agents running simultaneously.
During their runs, the C agent performed a `git stash` / `git stash pop` cycle to isolate its
verification, which accidentally reverted the ARCH-003-A edit to `talos-mcp/Cargo.toml`. This
was caught during post-merge verification and re-applied before the final workspace check.
Future parallel delegations that touch overlapping files should either sequence commit-time
verification or use worktree isolation to avoid stash-related cross-contamination.
