# 019: TUI Splash Scrollback-Only Boundary (Adopt Scrollback Splash; Reject Viewport Overlay)

## Status

Accepted (2026-06-13)

This ADR draws the outer boundary for the TUI startup splash: it decides where the splash
renders and what it will **not** grow into. It is a follow-on to the inline-by-default
model (I022) and ADR-018 (no alt-screen), applying the same Simplicity-First discipline
that ADR-006 applied to the event architecture.

## Context

TUI-005 (`docs/backlog/active/TUI-005-logo-splash.md`) originally envisioned a two-phase
splash:

| Phase | Description | Where it renders | Status |
|-------|-------------|------------------|--------|
| **Phase 1** | Styled ANSI splash (wordmark + gradient + subtitle + badges + version) printed to terminal scrollback before raw mode | Native terminal scrollback (outside viewport) | **Shipped** (2026-06-13) |
| **Phase 3** | Viewport overlay showing subsystem readiness badges (`[✓] Agent Runtime`, etc.) with 2-second auto-dismiss or first-keypress dismiss | Fixed viewport area (inside raw mode) | **Not implemented** |

Phase 1 is complete and verified. Phase 3 was explicitly marked out-of-scope in the
TUI-005 Scope Boundary ("Startup subsystem check animation … defer until I023 + runtime
integration"). This ADR makes the deferral permanent and records the architectural
reasoning.

A decision is needed because:

1. The original TUI-005 design left Phase 3 as a "future enhancement," which invites
   repeated re-proposal.
2. The original TUI-005 design described a reusable viewport `LogoWidget`, but the
   accepted implementation does not need one. Adding it now would violate AGENTS.md
   "No speculative features" unless a real viewport consumer exists.
3. The two-phase design conflates two distinct rendering models (scrollback append vs.
   viewport overlay with lifecycle), which would be easy to let drift if both paths
   existed without a shared model.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Inline-by-default TUI model (viewport at cursor, finalized content pushes to scrollback) | Hard | I022, `docs/iterations/I022-tui-inline-default.md` | No (core UX model) |
| No alt-screen for splash | Hard | ADR-018, TUI-005 Architecture Constraint | No |
| Simplicity First — no abstraction without a present, requested need | Hard | AGENTS.md | No |
| No speculative features; only current iteration scope | Hard | AGENTS.md | No |
| Viewport is fixed/cursor-anchored; finalized content pushes to native scrollback | Hard | I022 architecture | No (architectural fact) |
| No real runtime readiness state exists (subsystem badges require state source) | Assumption | TUI-005 Scope Boundary | Maybe (if I023 + runtime integration lands) |
| `talos-conversation` owns business state; `talos-tui` owns pure UI state | Hard | I023, ADR-005 | No |
| Each crate has a single responsibility | Soft | AGENTS.md | No |

## Reasoning

**Phase 1 (scrollback splash) already satisfies the product need.** The splash is a
one-time brand greeting: "Talos is starting." The correct destination for such a greeting is
terminal scrollback — like banners from `git`, `cargo`, or any CLI tool. It appears
once, stays in the history, and the user can scroll up to see it. This requires **zero
state, zero lifecycle, zero rendering-timing coupling**. It uses the inline-by-default
model's existing capability: finalized content pushes to native scrollback.

**Phase 3 (viewport overlay) is rejected on four grounds:**

1. **Simplicity First (Hard).** A viewport overlay is an abstraction with no present
   consumer. The "subsystem readiness badges" it would display require a real runtime
   readiness state (e.g., `[✓] Agent Runtime`, `[✓] Plugin Manager`), but TUI-005
   explicitly marked this as out-of-scope ("defer until I023 + runtime integration").
   The overlay would be built "in case" such a state exists later — the textbook
   speculative feature the project forbids.

2. **Rendering-timing coupling (Hard).** The inline-by-default model has a fragile
   rendering sequence: the viewport is established at a fixed cursor position, finalized
   content pushes to scrollback, and the viewport redraws. A viewport overlay that
   "auto-dismisses after 2s or first keypress" introduces a **state machine**
   (displaying → dismissed → normal viewport) that must coordinate with:
   - First-frame viewport establishment (handoff checklist #5 warns: "Flushing history
     before the viewport exists can erase restored lines or create apparent logo spacing
     bugs")
   - Scrollback flush timing
   - Cursor sync
   - Raw mode entry/exit

   This is a disproportionate complexity increase for a purely decorative feature. The
   scrollback splash avoids this entirely.

3. **Semantic mismatch (Soft).** The viewport is semantically a "real-time status panel"
   (fixed area, dynamic refresh, may disappear). The splash is semantically a "one-time
   brand greeting." Putting a static logo in a dynamic-status surface is a semantic
   mismatch. If a real runtime readiness state existed, the overlay would be justified;
   absent that state, the overlay is a decorative misuse of the viewport surface.

4. **Two rendering paths drift (Soft).** Maintaining "viewport can also draw the logo"
   would mean long-term maintenance of two implementations of the same visual. If they
   consume separate data, the layouts will drift; if they consume a shared model, that
   shared model is premature until a real viewport consumer exists. Both paths violate
   the current scope.

**The duplication problem is solved by avoiding the second path, not abstraction.** The
real risk — two rendering paths for the same visual — is resolved by converging on the
scrollback path. This is the same reasoning ADR-006 applied to the event architecture:
the duplication was solved by converging on the `AppServerSession` seam, not by adding
a global bus. Here, the "duplication" is solved by not adding a viewport logo renderer.

**If a real runtime readiness state genuinely appears** (e.g., I023 + runtime
integration lands a `SubsystemReadiness` API), the correct move is to **extract the
splash content into a shared pure data model** (wordmark rows, gradient, subtitle,
badges, version) consumed by both the scrollback printer and the viewport overlay. This
is the same pattern ADR-006 prescribes for event fan-out: deterministic, typed, named
consumers of a shared source. But building that abstraction now, without a real
consumer, is speculative.

## Decision

1. **Adopt Phase 1 (scrollback-only splash)** — the styled ANSI splash printed to
   terminal scrollback before raw mode is the canonical TUI startup behavior. It is
   shipped in `crates/talos-tui/src/splash.rs::print_splash_scrollback()`.

2. **Reject Phase 3 (viewport overlay)** — Talos will **not** introduce a viewport
   overlay for the splash. No `LogoWidget` ratatui `Widget`, no "subsystem readiness
   badges," no 2-second auto-dismiss state machine.

3. **Guardrail for implementers (read before adding any viewport splash rendering):**
   - Do **not** add a ratatui `Widget` that renders the logo/splash inside the viewport.
   - Do **not** add a state machine for "splash displaying → dismissed → normal
     viewport."
   - Do **not** add "subsystem readiness badges" unless a real runtime readiness state
     exists (see Reversal Trigger).
   - The splash is scrollback-only; it appears once, stays in history, and does not
     occupy viewport space.

4. **Do not add a dead viewport path** — `LogoWidget`, `impl Widget`,
   `build_badge_line`, and their associated tests are speculative unless a future ADR
   reopens viewport rendering with a concrete consumer. If such code appears without
   that decision, delete it.

**Rejected alternatives:**

- *Viewport overlay with subsystem badges + auto-dismiss* — speculative (no real state
  source), introduces rendering-timing coupling, semantic mismatch with the viewport surface.
- *Shared pure data model now (wordmark/gradient/badges consumed by both renderers)* —
  premature abstraction; violates "No speculative features" absent a real viewport
  overlay consumer.
- *Keep `LogoWidget` as "future-proofing"* — violates "No speculative features"; the
  widget would be dead code until a concrete viewport consumer exists.

## Reversal Trigger

Revisit this decision only if **all** of the following hold simultaneously:

- A concrete, present (not hypothetical) **runtime readiness state** exists (e.g.,
  `SubsystemReadiness` API from I023 + runtime integration) that the product genuinely
  needs to display at startup; **and**
- The product requirement is for a **persistent status panel** (not a one-time brand
  greeting), justifying viewport occupancy; **and**
- The implementation extracts the splash content into a **shared pure data model**
  (wordmark rows, gradient, subtitle, badges, version) consumed by both the scrollback
  printer and the viewport overlay, avoiding renderer drift.

Even then, the remedy is a **scoped, typed, shared-content-model** approach — never a
speculative `Widget` built "in case" it's needed later.

## Related

- [ADR-006: Event Architecture Boundary](006-event-architecture-boundary.md) — same
  Simplicity-First reasoning applied to the event architecture (reject global pub/sub).
- [ADR-018: `unsafe` in TUI Job Control](018-tui-job-control-unsafe.md) — drafted for
  I022; establishes the inline-by-default model and no-alt-screen constraint.
- `docs/iterations/I022-tui-inline-default.md` — the iteration that established the
  inline-by-default model (viewport at cursor, finalized content pushes to scrollback).
- `docs/backlog/active/TUI-005-logo-splash.md` — the backlog story that originally
  envisioned Phase 3; its Scope Boundary explicitly defers the overlay.
- `crates/talos-tui/src/splash.rs` — the shipped Phase 1 implementation
  (`print_splash_scrollback()`).
- AGENTS.md Hard Constraints #7 (no speculative features), Simplicity First principle.
