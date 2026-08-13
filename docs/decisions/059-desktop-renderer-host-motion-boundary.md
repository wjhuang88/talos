# ADR-059: Desktop Renderer, Host, And Motion Quality Boundary

> Status: Proposed
> Date: 2026-08-13
> Owner: DESKTOP-001-D0 / I194

## Context

Talos Desktop is a consumer-facing product and must feel responsive, calm, and visually
intentional. The existing design baseline calls for restrained native motion, but the renderer and
host boundary has not yet established testable motion, input, accessibility, or failure constraints.
The GPUI direction remains design input only; this ADR does not authorize a renderer dependency.

Desktop also remains a host/client above `talos-runtime`. Durable session/transcript truth stays in
`talos-session` under ADR-042, and `talos-runtime` remains the supported composition facade under
ADR-052. Motion must therefore present state; it must not create a second work authority or imply
that a visual transition is a runtime event.

## Decision

### Renderer and host boundary

The future Desktop host remains above the supported runtime facade. Renderer and platform-host code
must stay outside `talos-core` and `talos-runtime`; it may consume approved runtime/session
projections through an explicit host/client boundary. TUI and Desktop remain independent renderers.
No renderer, localization, native, build-script, FFI, or `unsafe` dependency is authorized by this
proposed ADR.

Renderer selection remains open pending current primary-source evidence covering macOS, Windows,
Linux, CJK text, Chinese IME, accessibility, reduced motion, native/transitive dependencies,
build-time behavior, panic containment, and shutdown ownership.

### Motion quality policy

Motion is a product-quality constraint, not a decorative layer:

1. Input response and state clarity take priority over animation. A user action must remain
   interruptible and must not wait for a visual transition to complete.
2. Motion must be semantic: use it for focus, insertion/removal, progress, status change, or
   spatial continuity. Do not use perpetual decorative loops, glow, bounce, or attention-stealing
   effects in the primary execution view.
3. Transitions must be short, bounded, cancellable, and replaceable by an immediate final state.
   A newer state supersedes an older transition; queued visual work must not accumulate.
4. Motion must preserve the existing information hierarchy: current Goal and Work state remain the
   visual center, and activity indicators must not compete with errors, approvals, or user input.
5. Reduced-motion mode is an equivalent presentation path, not a hidden disable switch. It removes
   non-essential interpolation and looping motion while preserving state, focus, ordering, and
   completion feedback through immediate changes, opacity/colour cues, or static markers.
6. Motion must respect accessibility and localization: no meaning may depend only on movement,
   timing assumptions, or language-specific geometry; CJK/Latin layout and IME composition cannot be
   animated in a way that obscures text or caret state.

### Performance evidence gate

The first mock-only visual/i18n slice must record, on each supported host class, input-to-visible
feedback latency, frame-time distribution during representative state transitions, cancellation
behavior, and reduced-motion equivalence. The acceptance target is stable interaction under the
renderer-selected host budget, with no long-task or unbounded-animation regression; exact numeric
budgets are to be selected from measured renderer/platform evidence rather than guessed in D0.

## Explicit exclusions

- No Desktop crate, window, widget, renderer, mock UI, localization catalog, or animation code.
- No real Mission, Work Graph, Evaluation, Approval, Artifact, Delivery, reconnect, or durable
  Desktop state.
- No change to ADR-042, ADR-052, SESSION-009, I188, I189, SESSION-008, RUNTIME-005, or
  ARCH-034-R04.
- No claim that GPUI or any other renderer has been selected or shipped.

## Security and failure inputs

The later implementation decision must trace material native/FFI/build/`unsafe` ownership, catch
panic-prone integration boundaries, define shutdown and process-lifetime containment, and preserve
permission/session/runtime authority. Renderer failure must degrade to a safe error state; it must
not silently terminate the host or fabricate a completed work state.

## Reversal triggers

Reopen this proposal if primary-source evidence shows that the candidate renderer cannot provide
acceptable CJK/IME/accessibility behavior, reduced-motion parity, bounded input latency, stable
frame timing, or safe native/panic containment on a supported host. Reopen it if motion materially
obscures state hierarchy, creates unbounded visual work, or requires domain state duplication.

## Next gate

This Proposed ADR may inform a later mock-only visual/i18n child only after independent exact-head
review, current renderer/dependency evidence, the security matrix, and merge-time CAS. That child
requires its own owner, iteration, claim, and worktree. Real Mission/runtime/work/evaluation binding
remains gated by the DESKTOP-001 P0-P4 chain.

## Evidence status

The local repository and governance boundaries are confirmed. Current upstream renderer/dependency
primary sources could not be retrieved in the 2026-08-13 execution environment because its configured
proxy was unavailable. Renderer selection therefore remains open and this ADR remains Proposed.
The companion matrix records the unknowns and the exact retrieval/security/motion validation gate.
