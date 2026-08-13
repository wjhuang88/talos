# ADR-059: Desktop Renderer, Host, And Motion Quality Boundary

> Status: Proposed
> Date: 2026-08-13 (evidence refresh)
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

The current evidence refresh keeps GPUI as the product direction, but does not authorize selecting
or adding it. The direction is supported by the GPUI source snapshot listed in the companion matrix:
it exposes an AccessKit dependency, explicit Wayland/X11 features, platform crates for macOS,
Windows and Linux, and IME/text handling in the host paths. The same snapshot also contains native
Objective-C/Windows API integration, macOS `bindgen`/`cbindgen` build steps and visible `unsafe`
platform code. Those facts require a separate dependency/security approval before any Cargo change.
Iced is retained as the minimum comparison candidate: its source snapshot exposes winit-backed IME
state and X11/Wayland feature paths, but its wgpu/tiny-skia/rendering and winit dependency closure
also remains unreviewed for Talos. No renderer is therefore authorized by this ADR.

The host responsibilities below are binding requirements for a later implementation decision:

| Host area | Required boundary before implementation |
|---|---|
| macOS | Renderer owns only window/event presentation; Objective-C/CoreText/Metal or equivalent native calls are isolated in the renderer adapter, audited for `unsafe`, and cannot own runtime/session state. |
| Windows | Win32/D3D or equivalent handles, message dispatch, device-loss and shutdown paths stay in the host adapter; failures become an explicit UI error and do not terminate or fabricate domain completion. |
| Linux | Wayland/X11 selection, clipboard, text input and compositor shutdown are host concerns; unsupported display backends fail with a recoverable diagnostic. |
| All hosts | A single host lifecycle owns event-loop startup, cancellation, renderer teardown and process exit. Runtime/session work remains behind an explicit client projection and is never reconstructed in the renderer. |
| Input and IME | Editable controls must preserve composition text, caret rectangle, commit/cancel ordering and mixed CJK/Latin layout; IME state is presentation state and never durable Mission/session state. |
| Accessibility | Every interactive state has keyboard focus/order and an accessibility name/role/state path; motion and colour are supplementary, never the only status signal. |
| Reduced motion | The same state, focus, ordering and completion semantics are reached without non-essential interpolation or looping effects. |

Localization selection is likewise a mechanism criterion, not a dependency authorization. A later
choice must support stable message keys, named interpolation, plural/count formatting, locale-aware
date/number formatting, deterministic `en-US` fallback, missing-key diagnostics, and runtime
`zh-CN`/`en-US` switching without allowing locale into canonical domain identity. User-authored
Mission text, commands, paths, raw evidence and artifacts remain untranslated facts.

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

The local repository and governance boundaries are confirmed. Current primary-source snapshots were
retrieved on 2026-08-13 and are recorded by immutable commit in the companion matrix. They establish
candidate capability and risk facts, not Talos compatibility or authorization. Renderer selection,
locked dependency closure, license/SBOM review, platform test evidence, panic containment and motion
benchmarks remain open. This ADR therefore remains Proposed and no renderer/localization dependency
or implementation is authorized.
