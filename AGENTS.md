# Talos — Agent Coding Guide

> Next-generation agent runtime in Rust. Safety-first, minimal core, maximum extensibility.

## Project Overview

Talos is a Rust-based agent runtime that combines the safety of Codex, the extensibility of Pi,
the optimization depth of Claude Code, the openness of OpenCode, and the self-evolution of Hermes.
Starting as a pure CLI tool, evolving into a full agent runtime platform.

**Language**: Rust (stable, edition 2024)
**Workspace**: Cargo workspace with 16 crates under `crates/`
**Architecture**: See `docs/reference/ARCHITECTURE.md`

## Hard Constraints

These are immutable facts that every change must respect:

1. **Rust first.** No arbitrary C/C++ bindings, Python FFI, or Node.js runtime. Approved exceptions are limited to ADR-recorded system/runtime dependencies: OS ABI access via `libc` (ADR-007), bundled SQLite for local storage via `rusqlite/bundled` (ADR-008), and tree-sitter for code analysis via `arborium` (ADR-020).
2. **No `unsafe` without ADR.** Any use of `unsafe` requires a decision record in `docs/decisions/`.
3. **No secrets in build, source, or distribution.** Hardcoded credentials must never be
   committed, baked into the binary, or shipped in default/sample config files. The user's
   local `~/.talos/config.toml` is their own file — they may put an `api_key` (or any
   other credential) there for their own use. `api_key` is persisted normally (not
   `skip_serializing`) so it survives load+save round-trips; display surfaces (CLI
   `config list`/`get`, `Debug` impls) mask it as `***`. See ADR-023 for the full
   boundary. Config also supports `${ENV_VAR}` substitution for users who prefer
   env-var-based credentials.
4. **All write-capable tools gated by permissions.** No tool can modify files without going through the permission pipeline.
5. **Sandbox code requires security review.** All changes to `talos-sandbox`, `talos-permission`, or process-hardening code must be reviewed against escape vectors.
6. **Crate public APIs are semver-bound.** Breaking changes require a decision record and a migration plan.
7. **No speculative features.** Only implement what the current iteration scope defines. Record ideas in `docs/proposals/`.
8. **Tests must pass before merge.** `cargo test --workspace` must exit 0. No `#[ignore]` without a tracking issue.
9. **External dependencies must not crash the process.** Any call into a dependency that involves native/C code (tree-sitter, SQLite, `libc`, process spawning) or that may panic must be wrapped so failures degrade gracefully to a safe fallback, never a silent process exit. `catch_unwind`, timeout guards, and error propagation are mandatory at the integration boundary.

## Coding Behavior

### Think Before Coding

- State assumptions explicitly before implementing. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- Classify constraints: Hard (immutable), Soft (negotiable), Assumption (unvalidated).

### Simplicity First

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" that wasn't requested.
- If you write 200 lines and it could be 50, rewrite it.

### Surgical Changes

- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- Clean up only what your own changes orphan.
- Every changed line should trace to a requirement.

### Goal-Driven Execution

- Define verifiable success criteria before starting.
- For multi-step tasks, state the plan with verification checkpoints.
- Loop until verified, not until "it looks right."
- Treat a committed `Planned` iteration as a published baseline. Preserve its objective,
  dependencies, exclusions, acceptance, validation, and documentation targets; append execution
  facts instead of replacing the plan.
- Before selecting or activating iteration work, inventory every Active, Review, Planned, and
  Blocked iteration and record its disposition. A different objective or acceptance target uses a
  new iteration ID, even when it continues the same product area.
- Every iteration must name a runnable, testable deliverable and affected user-facing
  documentation. Infrastructure-only exceptions must be explicit and cannot claim user behavior.

### Dependency Discipline

- Prefer self-contained capabilities over host environment assumptions. When choosing between
  a Rust-native/library-backed implementation and invoking host utilities, default to the
  self-contained path.
- Host utilities (`git`, `find`, `grep`, shell features, platform tools) may be used as
  compatibility fallbacks, temporary bridges, or explicit escape hatches only when the rationale,
  unavailable-host behavior, and replacement trigger are recorded.
- If a primary implementation depends on host capabilities, classify that as a Soft constraint
  tradeoff and record it in the relevant ADR, backlog story, or iteration note before coding.

## Rust-Specific Rules

- **Error handling**: Use `thiserror` for library crates, `anyhow` for binary crates only. Never `unwrap()` in library code.
- **Async**: All async via `tokio`. No `async-std`, no `smol`. Use `CancellationToken` for graceful shutdown.
- **Traits**: Prefer `impl Trait` for arguments, `dyn Trait` only when dynamic dispatch is required (tool registry, provider abstraction).
- **Types**: Use `serde` + `schemars` for all config/protocol types. JSON Schema validation on load.
- **Crates**: Each crate has a single responsibility. No circular dependencies. `talos-core` depends on nothing; other crates depend on `talos-core`.
- **Testing**: Unit tests in `#[cfg(test)] mod tests`. Integration tests in `tests/`. Property tests with `proptest` for protocol parsing.
- **Documentation**: All public items must have `///` doc comments. No `#[allow(missing_docs)]` on public APIs.

## Git Rules

1. **Review staged diff** before committing: `git diff --cached`
2. **Conventional commits**: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
   Format: `type(scope): description (#story-id) [model:<model-name>]`
   Scope = crate name (`core`, `agent`, `cli`, `tui`, `provider`, `session`, `tools`, etc.) or `workspace`.
   `[model:<model-name>]` is required for Agent-authored or Agent-assisted commits.
3. **One logical change per commit.** No mixed concerns.
4. **Never commit secrets.** Check for API keys, tokens, passwords.
5. **Never force-push to main.**
6. **Commit messages reference iteration/story IDs** when applicable: `feat(agent): implement turn loop (#I1-S3)`

## Task Router

| Task Type | Route To |
|-----------|----------|
| "I want to add a new feature" | `docs/sop/REQUIREMENT-INTAKE.md` → `docs/sop/NEW-FEATURE.md` |
| "Start the next iteration" | `docs/sop/START-ITERATION.md` |
| "How do I implement during an iteration?" | `docs/sop/ITERATION-WORKFLOW.md` |
| "A requirement changed mid-iteration" | `docs/sop/CHANGE-CONTROL.md` |
| "How do I set up local dev?" | `docs/sop/LOCAL-DEV.md` |
| "What's the testing strategy?" | `docs/sop/TESTING.md` |
| "How do I commit my work?" | `docs/sop/GIT-WORKFLOW.md` |
| "Run an unattended / overnight / long-running task" | `docs/sop/LONG-RUNNING-TASK.md` |
| "Where is the architecture documented?" | `docs/reference/ARCHITECTURE.md` |
| "What are the reference projects?" | `docs/reference/REFERENCE-PROJECTS.md` |
| "I have a technical tradeoff to decide" | `docs/decisions/README.md` (then create a new ADR) |
| "I need to fix an architecture/design/security review finding" | `docs/backlog/PRODUCT-BACKLOG.md` → "ARCH: Architecture Review Remediation" (`#ARCH-S1..S4`) |
| "Should we add a global message bus / unified event bus / pub-sub?" | `docs/decisions/006-event-architecture-boundary.md` (decided: no global pub/sub) |
| "Should the splash/logo render inside the viewport / as an overlay?" | `docs/decisions/019-tui-splash-scrollback-boundary.md` (decided: scrollback-only, no viewport overlay) |
| "Where is `unsafe` allowed and why?" | `docs/decisions/007-process-hardening-unsafe.md` |
| "Why is bundled SQLite allowed?" | `docs/decisions/008-sqlite-bundled-storage.md` |
| "What is the inline api_key security boundary?" | `docs/decisions/023-inline-api-key-boundary.md` (persisted in TOML, masked in all display surfaces) |
| "How do I keep docs in sync with code?" | `docs/sop/DOC-CHECK.md` |
| "Governance drift, repair, or skill upgrade" | `docs/sop/DOC-CHECK.md` → refresh audit against current `agent-project-governance` skill, then run `scripts/validate_project_governance.sh .` and `scripts/assess_project_scale.sh .` |
| "A session exposed a reusable lesson, failed validation, or user correction" | `docs/sop/EVOLUTION-FEEDBACK.md` → `EVOLUTION.md` |
| "I have an idea for later" | `docs/proposals/` |
| "What's the implementation plan?" | `docs/roadmap/IMPLEMENTATION-ROADMAP.md` |
| "What work is planned?" | `docs/backlog/PRODUCT-BACKLOG.md` |
| "What is active right now?" | `docs/BOARD.md` (derived view only; verify state in owner docs before editing) |

## Session End Checklist

Before ending a session, verify:

1. **Status sync**: Update backlog story status, iteration progress in `docs/iterations/`.
2. **Verification evidence**: Did tests pass? Did you run `cargo check --workspace`?
3. **Residual work**: Record incomplete items in the backlog or iteration notes.
4. **Lessons**: If you hit a non-obvious problem, failed validation, or user correction, follow `docs/sop/EVOLUTION-FEEDBACK.md` before updating `EVOLUTION.md`.
5. **Decision records**: Did this session make a technical choice affecting Soft/Assumption constraints? If yes, record in `docs/decisions/`.
6. **Commit readiness**: Staged diff reviewed? No secrets? Conventional commit message?
7. **No orphaned changes**: All modified files trace to a requirement.
8. **README sync**: Update `README.md` to reflect any new features, usage changes, or architecture updates from this session. README is a living document, not a one-time setup.
9. **Board sync**: If active/review/paused/next work changed, update `docs/BOARD.md` after the owner docs. The board is a derived view, not a source of truth.
10. **Governance harness**: If governance files changed, run `scripts/validate_project_governance.sh .`; when profile, branch mode, worktree mode, or governance depth is affected, also run `scripts/assess_project_scale.sh .`.
11. **Long task recovery**: If a long-running task record is active, append validation evidence,
    current state, next item, and recovery/resume instructions before stopping.

## Current Known Traps

- **Greenfield**: No existing code to reference for patterns. Every crate is new. Follow the architecture doc strictly.
- **Crate boundary coupling**: It's tempting to put everything in one crate. Resist. Each crate must have a clear single responsibility.
- ** premature async abstraction**: Don't over-abstract async patterns before the core loop works. Get the simplest turn loop working first.
- **Reference project translation**: Patterns from TypeScript projects (Pi, Claude Code, OpenCode) need Rust-idiomatic reimplementation, not literal translation.
