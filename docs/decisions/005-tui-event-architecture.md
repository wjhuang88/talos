# 005: Canonical TUI Event Architecture (Two-Layer: AppEvent Bus + AppServerSession Seam)

## Status

Accepted

Amends [ADR-004](004-event-loop-architecture.md) (retracts its §355 claim "event loop unchanged / agent spawned inside the TUI loop").

## Context

Three independent run paths in `crates/talos-cli/src/` evolved different concurrency and event
models, and a documentation contradiction left "the canonical TUI architecture" undefined:

| Path | Agent spawn | Evolution (I008) wired | Event model |
|------|-------------|------------------------|-------------|
| `run_print_mode` | `tokio::spawn` in main, direct `broadcast` recv | ✅ Yes | Linear, no event loop |
| `run_interactive_mode` (`event_loop.rs`) | `tokio::spawn(run_agent_turn_inner)` **inside** the event loop | ❌ No | ADR-004 single `mpsc<AppEvent>` + `AppState` |
| `run_tui_mode` (ratatui) | `tokio::spawn` in a separate task | ❌ No | Ad-hoc `mpsc` + `broadcast`, bypasses ADR-004 |

This produced three ways to build an `Agent`, three ways to wire evolution, and three event
models. I008 self-evolution is consequently wired into only one path, and TUI/interactive cannot
be accepted as COMPLETE.

The contradiction:

- **ADR-004 (Accepted)** declares a single `mpsc::unbounded<AppEvent>` bus + explicit `AppState`
  state machine, and its Implementation Plan §355 states the I010 TUI integration only swaps
  `render()` for ratatui with the **"event loop unchanged"** — assuming the agent turn is spawned
  *inside* the TUI loop (as `run_interactive_mode` does today).
- **REFERENCE-PROJECTS.md** (Codex = designated PRIMARY TUI reference): §710 "single mpsc AppEvent
  channel … validates our AgentEvent + broadcast design"; §712 "**TUI never calls agent loop** …
  communicates via `AppServerSession`."
- **ARCHITECTURE.md §95** defines the SQ/EQ async pattern: bounded Submission Queue (cap=512) for
  commands *to* the agent, unbounded Event Queue for streaming *back* to the UI.
- **IMPLEMENTATION-ROADMAP.md §277 / PRODUCT-BACKLOG.md §755 (I010)**: headless (`talos exec`),
  SDK (library embedding), and interactive all "share the same core agent loop via
  `AppServerSession` abstraction (Codex pattern: TUI never calls agent loop directly)."

A decision is needed because the contradiction blocks a coherent TUI roadmap and the unification
point for I008 evolution wiring.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Codex is the PRIMARY TUI reference; "TUI never calls agent loop directly" | Soft | REFERENCE-PROJECTS.md §712 | No (anchors the target UX) |
| SQ bounded (cap=512) / EQ unbounded | Assumption | ARCHITECTURE.md §95 | Maybe (cap is tunable) |
| ADR-004 single-mpsc `AppEvent` bus + `AppState` machine | Soft | ADR-004 | No (validated by Codex `app.rs`) |
| ADR-004 §355 "agent spawned inside TUI loop / event loop unchanged" | Assumption | ADR-004 §355 | Yes (contradicts §712) |
| `talos-core` depends on nothing; others depend on `talos-core`; no circular deps | Hard | AGENTS.md | No |
| Each crate single responsibility; abstractions emerge from implementation | Soft | AGENTS.md | No |
| `cargo test --workspace` exits 0 before merge (currently 450 tests) | Hard | AGENTS.md | No |
| No speculative features; only current iteration scope | Hard | AGENTS.md | No |

## Reasoning

**The simplest correct model is two layers, not one.** ADR-004 and `AppServerSession` are not in
conflict — they describe different layers:

- **Layer 1 — TUI-internal event bus** (ADR-004): a single `mpsc::unbounded<AppEvent>` channel
  feeding an explicit `AppState` state machine. This matches Codex's own `app.rs`/`app_event.rs`
  (single mpsc, 100+ variants). It is sound and is **retained**.
- **Layer 2 — AppServerSession seam**: the boundary between the L1 loop and the agent core. The
  TUI submits commands over a bounded **SQ** (`Op`/`Submission`, cap=512) and consumes streamed
  results over an unbounded **EQ** (`EventMsg`). The TUI **never holds an `Agent` nor calls
  `run_streaming` directly**; it drives a session actor.

**The actual defect** is narrow and concrete: ADR-004 §355's assumption that the agent turn is
spawned *inside* the TUI loop. That direct spawn is exactly the coupling Codex avoids, and it is
why all three paths each rebuild the agent and each must wire evolution separately. Correcting
§355 — not discarding ADR-004 — resolves the contradiction. Hence ADR-004 is **AMENDED**, not
SUPERSEDED: its core decision (single-mpsc bus + state machine) stands; only §355 is retracted.

**One validated caveat (semantic addition, not contradiction):** a bounded SQ (cap=512) introduces
backpressure. When the SQ is full, `send()` may fail/block, so the L1 loop gains one new event
source (EQ recv) and one new action (SQ send with backpressure handling). The `AppState` structure
is unchanged; this is an additive refinement of ADR-004, consistent with the amendment.

**Why converge all three paths on the seam:** it eliminates the three-way duplication and gives
I008 evolution exactly **one** wiring point (the session/EQ) instead of per-path hooks (which would
otherwise risk double-firing). It also unlocks headless (`talos exec`) and SDK reuse of the same
core loop — the I010 goal.

**Why phase it rather than do it now:** AGENTS.md forbids speculative scope, the TUI is untestable
in CI (no TTY), and there are 450 passing tests to protect. The honest minimum *now* is to fix the
documentation defect (§355) and fix the canonical model; the *implementation* migration is I010
scope and is sequenced as a canary-first rollout.

**Crate placement (per dependency rules):** the protocol types that cross the seam
(`Op`/`Submission` for SQ, `EventMsg` for EQ) are protocol types like the existing `AgentEvent`
and belong in **`talos-core`** (depends on nothing). The session *actor implementation* (owns the
bounded SQ receiver, unbounded EQ sender, and runs the agent loop) belongs in **`talos-agent`**
(already owns the turn loop). A dedicated `talos-session-server` crate is rejected as premature —
no second implementation exists yet, so the abstraction has not earned its own crate boundary.
This mirrors Codex: protocol types in `codex-rs/protocol`, the session actor in `codex-rs/core`.

## Decision

1. **Canonical TUI architecture is two layers:**
   - **L1 (retained from ADR-004):** single `mpsc::unbounded<AppEvent>` bus + explicit `AppState`
     state machine, stdin on `std::thread`, signals on `tokio::spawn`, cancel-token tree.
   - **L2 (new):** an `AppServerSession` seam — bounded **SQ** (`Op`/`Submission`, cap=512) for
     UI→core commands, unbounded **EQ** (`EventMsg`) for core→UI streaming. The TUI never spawns
     the agent turn directly; it submits to the session and consumes EQ events into the L1 bus.

2. **ADR-004 is AMENDED:** its single-mpsc bus + `AppState` machine stand; its §355 claim
   ("event loop unchanged / agent spawned inside the TUI loop") is **retracted**. The agent loop
   moves behind the `AppServerSession` boundary.

3. **All three run paths converge on `AppServerSession`** as the single seam; I008 evolution
   attaches once, at the session/EQ, not per-path.

4. **Crate placement:** SQ/EQ protocol types in `talos-core`; session actor implementation in
   `talos-agent`. No new crate.

5. **Migration is phased and deferred to I010** (current iteration only corrects the docs/model):

   | Phase | Work | Risk | Gate |
   |-------|------|------|------|
   | 1 | Define `AppServerSession` SQ/EQ types in `talos-core` (types only) | None | pure types |
   | 2 | Implement session actor in `talos-agent` | Low | unit-testable |
   | 3 | Migrate `run_print_mode` (canary — already CI-tested + evolution wired) | Low | `cargo test` green |
   | 4 | Migrate `run_interactive_mode` (L1 loop kept; `start_agent_turn` → `session.submit`) | Medium | readline-testable |
   | 5 | Migrate `run_tui_mode` (replace ad-hoc spawn with session submit) | Medium-High | manual TTY |
   | 6 | Delete dead `event_loop.rs` variants (`ApprovalRequested`, `ApprovalResolved`, `ToggleSkillSidebar`, `SkillsUpdated`, `ApprovalChoice`) | Low | compiler-clean |

   **Invariant:** never migrate two paths simultaneously; every step keeps `cargo test --workspace`
   green. Evolution hooks move to the session/EQ during Phase 3+ to avoid double-firing.

**Rejected alternatives:**
- *Mark ADR-004 SUPERSEDED* — dishonest; the single-mpsc + state-machine core is correct and Codex-validated.
- *Scope ADR-004 to readline-only* — dishonest; the L1 bus pattern applies to ratatui too.
- *Build the full `AppServerSession` migration now* — violates "no speculative features"; high regression risk against 450 tests with no CI coverage for the TUI.
- *New `talos-session-server` crate* — premature abstraction; no second implementation justifies the boundary.

## Reversal Trigger

Revisit this decision if any of the following hold:

- A bounded SQ (cap=512) proves to materially change L1 loop semantics beyond additive backpressure
  (i.e., forces restructuring `AppState`) — then ADR-004 would warrant SUPERSEDED, not AMENDED.
- A second `AppServerSession` implementation appears (e.g., a remote/RPC session distinct from the
  in-process one) — then extract the seam into its own crate.
- Headless/SDK requirements diverge enough from the interactive session that a single shared core
  loop becomes a forcing constraint rather than a simplification.
- Phase 3 (print-mode canary) cannot be migrated without regressions — halt and re-evaluate whether
  the seam belongs in `talos-agent` vs elsewhere.

## Related

- [ADR-003: TUI Progressive Evolution](003-tui-progressive-evolution.md)
- [ADR-004: Production-Grade Event Loop Architecture](004-event-loop-architecture.md) (amended by this ADR)
- REFERENCE-PROJECTS.md §15 (Codex TUI patterns), §710–712 (AppEvent bus, AppServerSession)
- ARCHITECTURE.md §95 (SQ/EQ async pattern)
- IMPLEMENTATION-ROADMAP.md §266–277 (I010), PRODUCT-BACKLOG.md §755
- I008 (Learning Agent): self-evolution unifies onto the session/EQ seam during migration
