# 006: Event Architecture Boundary (Adopt Single-Consumer Loop + Session Seam; Reject Global Pub/Sub)

## Status

Accepted

Builds on [ADR-004](004-event-loop-architecture.md) (L1 event loop) and [ADR-005](005-tui-event-architecture.md) (L2 session seam). This ADR draws the *outer boundary*: it decides what event architecture Talos will **not** grow into.

## Context

The phrase "unified message bus" surfaced as a proposed architecture. On inspection it is
ambiguous — it conflates three distinct designs, two of which Talos already has and one of which
it does not:

| # | "Message bus" interpretation | Where it lives today | Status |
|---|------------------------------|----------------------|--------|
| **A** | **Single-consumer event loop** — one `mpsc::unbounded<AppEvent>` channel feeding one `AppState` state machine; many producers (stdin thread, signal task, agent stream), exactly **one** consumer (the UI loop). | ADR-004 (L1) | Adopted |
| **B** | **UI↔core session seam** — bounded **SQ** (`Op`/`Submission`, cap=512) for commands to the agent, unbounded **EQ** (`EventMsg`) for streamed results back; the UI never spawns the agent turn directly. | ADR-005 (L2, `AppServerSession`) | Adopted |
| **C** | **Global publish/subscribe bus** — an app-wide broadcast hub with **many publishers and many independent subscribers**; any component can publish any event and any component can subscribe to it. | — | **Not present** |

A and B are not "a bus" in the pub/sub sense — they are point-to-point channels with a single
consumer / single seam. C is the thing people usually mean by "message bus," and it is the one
being implicitly proposed. A decision is needed because adopting C would be a large, hard-to-reverse
architectural commitment that touches the security-critical tool/permission path, and because the
existing three-path duplication (see ADR-005) makes "just add a global bus to tie it together" look
deceptively attractive.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|------------|------|--------|-------------|
| Single-mpsc `AppEvent` bus + `AppState` machine (one consumer) | Soft | ADR-004 | No (Codex-validated) |
| `AppServerSession` SQ/EQ seam; UI never spawns the agent turn | Soft | ADR-005, REFERENCE-PROJECTS.md §712 | No (anchors target UX) |
| Simplicity First — no abstraction without a present, requested need | Hard | AGENTS.md | No |
| No speculative features; only current iteration scope | Hard | AGENTS.md | No |
| All write-capable tools gated by permissions; tool/permission events must stay auditable | Hard | AGENTS.md (Hard Constraint #4) | No |
| `talos-core` depends on nothing; no circular deps; one responsibility per crate | Hard | AGENTS.md | No |
| I008 evolution attaches at exactly one wiring point (the EQ seam), not per-path | Soft | ADR-005 | No (avoids double-firing) |

## Reasoning

**A + B already satisfy every concrete need we have.** Every event flow that exists in Talos is
either (1) many-producer → one-consumer (the UI loop: interpretation A) or (2) UI → agent-core and
back (interpretation B). There is currently **no** flow that requires multiple independent
subscribers of the same event. The need that would justify C does not exist.

**C (global pub/sub) is rejected on three grounds:**

1. **Simplicity First (Hard).** A global bus is an abstraction with no present consumer. It would
   be built "in case" something needs to subscribe later — the textbook speculative feature the
   project forbids. A and B are the *simplest* structures that carry today's traffic.

2. **Security auditability (Hard Constraint #4).** Tool calls and permission decisions flow through
   the event path. With A/B, the set of observers is statically known and singular: the UI loop
   consumes the EQ; evolution attaches at exactly one seam. With a global pub/sub bus, **any**
   component can silently subscribe to tool/permission events — every subscriber becomes an
   implicit, hard-to-audit data sink. That directly undermines the "all write-capable tools gated
   and auditable" guarantee. A broadcast fan-out is the wrong shape for a security-sensitive event
   stream.

3. **Hidden coupling.** Pub/sub trades explicit wiring for implicit, action-at-a-distance coupling.
   It makes the question "who reacts to this event?" unanswerable from the type system. That
   contradicts the single-responsibility, no-circular-deps crate discipline and makes the codebase
   harder to reason about, not easier.

**The duplication problem is already solved by B, not by C.** The real pain — three run paths each
rebuilding the agent and each wiring evolution separately (ADR-005) — is resolved by converging on
the `AppServerSession` seam, where evolution attaches **once** at the EQ. A global bus would
*re-introduce* the multi-subscriber ambiguity (which path's evolution hook fired?) that B was
designed to eliminate.

**If a second observer genuinely appears** (e.g., a metrics exporter or a remote session mirror),
the correct move is to **fan out deterministically from the single EQ consumer** (the UI loop
forwards to a typed, named downstream), not to invert control into a global broadcast. Fan-out from
one consumer keeps the observer set explicit and auditable; pub/sub does not.

## Decision

1. **Adopt A** — the single-consumer `mpsc<AppEvent>` event loop (ADR-004 L1) is the canonical
   intra-UI event model. One channel, one consumer, one `AppState`.

2. **Adopt B** — the `AppServerSession` SQ/EQ seam (ADR-005 L2) is the canonical UI↔core boundary.
   I008 evolution attaches at the EQ, once.

3. **Reject C** — Talos will **not** introduce a global publish/subscribe / app-wide broadcast event
   bus. No "EventBus" abstraction with open many-to-many subscription.

4. **Guardrail for implementers (read before adding any event plumbing):**
   - Do **not** add a `broadcast`-based, app-wide, openly-subscribable event hub.
   - Do **not** route tool-call or permission-decision events to more than one consumer via
     implicit subscription.
   - New event flows MUST be either (A) producer→single-consumer mpsc, or (B) over the SQ/EQ seam.
   - If you believe a second subscriber is needed, **stop** and update this ADR (see Reversal
     Trigger) before writing code — fan-out from the single EQ consumer is the default answer.

**Rejected alternatives:**
- *Global pub/sub "EventBus"* — speculative, harms security auditability, introduces hidden coupling.
- *Per-path evolution hooks tied together by a bus* — re-introduces the double-firing ambiguity ADR-005 eliminates.
- *Untyped/`Any`-typed broadcast channel* — defeats the typed-protocol discipline of `AgentEvent`/`EventMsg`.

## Reversal Trigger

Revisit this decision only if **all** of the following hold simultaneously:

- A concrete, present (not hypothetical) need exists for **two or more independent subscribers** of
  the *same* event stream that genuinely cannot be served by deterministic fan-out from the single
  EQ consumer; **and**
- The subscriber set is **statically enumerable and typed** (no open `subscribe(Any)` surface); **and**
- Tool-call and permission-decision events remain routed through an **auditable, gated** path (a
  global bus must never become a side channel around the permission pipeline).

Even then, the remedy is a **scoped, typed, named-subscriber** fan-out — never an untyped global
broadcast.

## Related

- [ADR-004: Production-Grade Event Loop Architecture](004-event-loop-architecture.md) (interpretation A / L1)
- [ADR-005: Canonical TUI Event Architecture](005-tui-event-architecture.md) (interpretation B / L2 seam)
- REFERENCE-PROJECTS.md §710–712 (Codex single mpsc bus; "TUI never calls agent loop"; `AppServerSession`)
- ARCHITECTURE.md §95 (SQ/EQ async pattern)
- PRODUCT-BACKLOG.md §755 (#I010-S7 — convergence on `AppServerSession`)
- AGENTS.md Hard Constraints #4 (permission gating), #7 (no speculative features)
