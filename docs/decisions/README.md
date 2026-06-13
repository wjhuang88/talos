# Architecture Decision Records

## Purpose

Record significant technical decisions that affect Soft or Assumption constraints. Not for
routine implementation choices that follow established patterns.

## Naming Convention

```
docs/decisions/
├── README.md           (this file)
├── 001-<slug>.md       (decision record)
├── 002-<slug>.md
└── ...
```

## Template

```markdown
# [Decision Title]

## Context
[Why a decision is needed]

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| [constraint] | Hard / Soft / Assumption | [source] | No / Yes / Maybe |

## Reasoning
[What is the simplest approach satisfying Hard constraints?
Why deviate if we chose to?
Which Assumptions need validation?]

## Decision
[What was chosen and what was rejected]

## Reversal Trigger
[Under what conditions should this be revisited?]
```

## When to Write

| Trigger | Example |
| --- | --- |
| Choosing between approaches satisfying Hard constraints | Async runtime choice |
| Proceeding based on unvalidated Assumption | "WASM is fast enough for plugins" |
| Overriding a Soft constraint | "Using dynamic dispatch despite preferring static" |
| A Hard constraint forces an unpopular choice | "No unsafe without ADR" |

## Current Decisions

1. [001: Self-Evolution as Runtime Primitive](001-runtime-self-evolution.md) — Evolution is a first-class runtime capability (Observe → Learn → Adapt), not just a skill system feature.
2. [002: Local Storage Architecture](002-local-storage-architecture.md) — Progressive storage strategy: pure files first, SQLite introduced only where query patterns (FTS, aggregation) demand it.
3. [003: TUI Progressive Evolution](003-tui-progressive-evolution.md) — Accepted. TUI grows incrementally from I005 onward rather than landing all at once in a final polish iteration.
4. [004: Production-Grade Event Loop Architecture](004-event-loop-architecture.md) — Accepted (amended by ADR-005). Single-mpsc `AppEvent` bus + explicit `AppState` state machine for the TUI-internal event loop.
5. [005: Canonical TUI Event Architecture](005-tui-event-architecture.md) — Accepted. Two-layer model: retain ADR-004's L1 mpsc bus; add an `AppServerSession` L2 seam (bounded SQ / unbounded EQ) so the TUI never spawns the agent loop directly. Phased migration deferred to I010.
6. [006: Event Architecture Boundary](006-event-architecture-boundary.md) — Accepted. Adopt the single-consumer event loop (A, ADR-004) and the `AppServerSession` session seam (B, ADR-005); **reject** a global publish/subscribe event bus (C) on Simplicity-First, security-auditability, and hidden-coupling grounds. Guardrail for implementers.
7. [007: `unsafe` in Process Hardening](007-process-hardening-unsafe.md) — Accepted. Records and justifies the four production `unsafe` sites in `talos-sandbox/hardening.rs` (`env::remove_var` + 3× `libc::setrlimit`), approves `libc` for OS syscalls, and pre-authorizes child-process `pre_exec` hardening. Satisfies Hard Constraint #2.
8. [008: Bundled SQLite for Local Storage](008-sqlite-bundled-storage.md) — Accepted. Approves `rusqlite/bundled` as a scoped exception to the no-C/C++-bindings rule for local storage only; SQLite is statically linked so Talos does not require a system SQLite installation.
9. [009: Tool Provenance Tracking](009-tool-provenance.md) — Accepted. Adds typed provenance for native and MCP-remote tools so TUI/RPC/plugin consumers can distinguish tool sources without changing the agent loop.
10. [010: Git and Search Tool Dependency Boundary](010-git-search-tool-dependency-boundary.md) — Accepted. Rejects `git2`/libgit2 for the first I012 search/Git slices; search starts Rust-native, read-only Git tools target `gix`, and host `git` is fallback/temporary bridge only.
11. [011: Guardian Approval Boundary](011-guardian-approval-boundary.md) — Accepted. Keeps Guardian AI inside the existing permission pipeline, disabled by default, and forbids first-slice write-capable auto-approval.
12. [012: Exec Policy DSL Boundary](012-exec-policy-dsl-boundary.md) — Accepted. Defines the policy DSL as typed permission input, not a shell parser; complex shell features fail back to Ask.
13. [013: Provider Config Schema Boundary](013-provider-config-schema-boundary.md) — Accepted. Limits provider openness to schema/config in #I011-S2 and defers dynamic provider loading to a future ADR.
14. [014: Log Retention and Rotation Boundary](014-log-retention-and-rotation.md) — Accepted. Requires bounded local log files and in-process rotation/cleanup for #ARCH-S8 R2.
15. [015: Embedded Prompt Asset Boundary](015-embedded-prompt-assets.md) — Accepted. Extracts built-in prompts into standalone files embedded at compile time.
16. [016: Layered Agent Memory Architecture](016-layered-memory-architecture.md) — Accepted for architecture. Defines working, episodic, semantic, and procedural memory with explicit consolidation.
17. [017: Exploration and Library Storage Architecture](017-exploration-library-storage.md) — Accepted for direction. Starts research-library storage on SQLite/FTS with vector/graph stores gated by Spike.
18. [018: `unsafe` in TUI Job Control](018-tui-job-control-unsafe.md) — Accepted (drafted for I022). Records and justifies the single `unsafe` site in `talos-tui/src/tui/job_control.rs` (`libc::raise(SIGTSTP)`) for foreground suspend on Ctrl+Z. Follow-on to [ADR-007](007-process-hardening-unsafe.md); reuses the same `libc` FFI discipline in a different module, with no new top-level dependencies.
19. [019: TUI Splash Scrollback-Only Boundary](019-tui-splash-scrollback-boundary.md) — Accepted (2026-06-13). Adopts the scrollback-only splash (Phase 1) and rejects the viewport overlay (Phase 3) on Simplicity-First, no-speculative-features, and rendering-timing-coupling grounds. Guardrail for implementers.
