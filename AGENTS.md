# Talos — Agent Coding Guide

> Next-generation agent runtime in Rust. Safety-first, minimal core, maximum extensibility.

## Project Overview

Talos is a Rust-based agent runtime that combines the safety of Codex, the extensibility of Pi,
the optimization depth of Claude Code, the openness of OpenCode, and the self-evolution of Hermes.
Starting as a pure CLI tool, evolving into a full agent runtime platform.

**Language**: Rust (stable, edition 2024)
**Workspace**: Cargo workspace with 12 crates under `crates/`
**Architecture**: See `docs/reference/ARCHITECTURE.md`

## Hard Constraints

These are immutable facts that every change must respect:

1. **Rust only.** No C/C++ bindings, no Python FFI, no Node.js runtime. All crates are pure Rust.
2. **No `unsafe` without ADR.** Any use of `unsafe` requires a decision record in `docs/decisions/`.
3. **No secrets in code or config.** All credentials via env vars or secret stores. Config supports `${ENV_VAR}` substitution.
4. **All write-capable tools gated by permissions.** No tool can modify files without going through the permission pipeline.
5. **Sandbox code requires security review.** All changes to `talos-sandbox`, `talos-permission`, or process-hardening code must be reviewed against escape vectors.
6. **Crate public APIs are semver-bound.** Breaking changes require a decision record and a migration plan.
7. **No speculative features.** Only implement what the current iteration scope defines. Record ideas in `docs/proposals/`.
8. **Tests must pass before merge.** `cargo test --workspace` must exit 0. No `#[ignore]` without a tracking issue.

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
| "Where is the architecture documented?" | `docs/reference/ARCHITECTURE.md` |
| "What are the reference projects?" | `docs/reference/REFERENCE-PROJECTS.md` |
| "I have a technical tradeoff to decide" | `docs/decisions/README.md` (then create a new ADR) |
| "I need to fix an architecture/design/security review finding" | `docs/backlog/PRODUCT-BACKLOG.md` → "ARCH: Architecture Review Remediation" (`#ARCH-S1..S4`) |
| "Should we add a global message bus / unified event bus / pub-sub?" | `docs/decisions/006-event-architecture-boundary.md` (decided: no global pub/sub) |
| "Where is `unsafe` allowed and why?" | `docs/decisions/007-process-hardening-unsafe.md` |
| "How do I keep docs in sync with code?" | `docs/sop/DOC-CHECK.md` |
| "I have an idea for later" | `docs/proposals/` |
| "What's the implementation plan?" | `docs/roadmap/IMPLEMENTATION-ROADMAP.md` |
| "What work is planned?" | `docs/backlog/PRODUCT-BACKLOG.md` |

## Session End Checklist

Before ending a session, verify:

1. **Status sync**: Update backlog story status, iteration progress in `docs/iterations/`.
2. **Verification evidence**: Did tests pass? Did you run `cargo check --workspace`?
3. **Residual work**: Record incomplete items in the backlog or iteration notes.
4. **Lessons**: If you hit a non-obvious problem, add a lesson to `EVOLUTION.md`.
5. **Decision records**: Did this session make a technical choice affecting Soft/Assumption constraints? If yes, record in `docs/decisions/`.
6. **Commit readiness**: Staged diff reviewed? No secrets? Conventional commit message?
7. **No orphaned changes**: All modified files trace to a requirement.
8. **README sync**: Update `README.md` to reflect any new features, usage changes, or architecture updates from this session. README is a living document, not a one-time setup.

## Current Known Traps

- **Greenfield**: No existing code to reference for patterns. Every crate is new. Follow the architecture doc strictly.
- **Crate boundary coupling**: It's tempting to put everything in one crate. Resist. Each crate must have a clear single responsibility.
- ** premature async abstraction**: Don't over-abstract async patterns before the core loop works. Get the simplest turn loop working first.
- **Reference project translation**: Patterns from TypeScript projects (Pi, Claude Code, OpenCode) need Rust-idiomatic reimplementation, not literal translation.
