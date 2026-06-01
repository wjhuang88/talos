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
