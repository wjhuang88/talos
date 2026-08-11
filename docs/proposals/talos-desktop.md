# Talos Desktop

> Status: Refined proposal entry — **no implementation authorization**
>
> Product/architecture baseline: [Talos Desktop Goal-Oriented Workspace Design Baseline](talos-desktop-goal-oriented-workspace.md)
>
> Source: [GitHub Issue #29](https://github.com/wjhuang88/talos/issues/29)

## Problem

Talos currently serves terminal-first workflows. A desktop product is only worthwhile if it creates
a distinct interaction model rather than reproducing the TUI inside a native window.

The refined direction is therefore a **goal-oriented Mission workspace** for shaping outcomes,
supervising execution, reviewing artifacts, independently evaluating completion, and receiving a
structured Delivery.

The TUI remains conversation-first and optimized for dense, immediate expert interaction. Desktop
is intended to be state-centric and visual.

## Current Direction

The current renderer direction is **GPUI with a Rust-native product host**. The earlier Tauri/WebView
candidate recorded in Issue #29 is retained as historical context but is no longer the selected
technical route.

Desktop must reuse Talos runtime, permission, sandbox, session, provider, tool, Skill, plugin, and
validation boundaries rather than creating a second execution engine.

The refined product model is:

```text
Mission
  -> Work Graph
       -> Goal
       -> WorkUnit
  -> Execution
  -> Independent Evaluation
  -> Artifact / Evidence
  -> Delivery
```

The existing Todo domain should evolve into the canonical Work Graph rather than coexist with a
parallel Desktop Goal store.

## Implementation Gate

No Desktop implementation is authorized by this proposal.

Before the first GPUI Desktop implementation PR, a **separate prerequisite implementation PR** must
establish the shared Work Graph and independent Goal-evaluation foundation. Its required actions,
acceptance, and explicit exclusions are defined in
[`talos-desktop-goal-oriented-workspace.md`](talos-desktop-goal-oriented-workspace.md).

That prerequisite PR is a future action only. This documentation change does not create it.

Normal requirement intake, ADR, iteration selection, Collaboration Claim, dependency/security
review, and validation rules remain mandatory.

## Dependencies

- `RUNTIME-001` embeddable runtime facade.
- Existing `talos-conversation` UI-independent projection, to be reconciled before inventing a new
  presentation crate.
- Existing Todo persistence/tool semantics as migration input for the future Work Graph.
- `VALIDATION-001` shared validation service as an evidence producer for independent evaluation.
- `SESSION-009` for later multi-client/reconnect semantics; it should not block a local,
  single-client embedded Desktop vertical slice.
- Permission, sandbox, credential-display, durable-session, and distribution decisions.

## Open Questions

Implementation/ADR questions are tracked in the refined baseline, including persistence migration,
workspace revision identity, evaluator policy, exact shared presentation boundaries, and Desktop
packaging/security review.

## Historical Alternatives

The original proposal discussed:

- Tauri + Web frontend;
- pure Rust GUI frameworks such as egui/iced;
- hybrid WebView/native rendering;
- continuing TUI-first.

Those alternatives remain useful historical context, but the current product/renderer baseline is
Goal-first Desktop + GPUI. Any change to that direction should be recorded explicitly rather than
silently reviving the earlier candidate recommendation.
