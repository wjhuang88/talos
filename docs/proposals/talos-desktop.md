# Talos Desktop

> Status: refined product/architecture proposal; **no implementation authorization**
>
> Owner: `DESKTOP-001`
>
> Consolidated design: [`talos-desktop-goal-oriented-workspace.md`](talos-desktop-goal-oriented-workspace.md)
>
> Visual design: [`../design/talos-desktop/DESIGN.md`](../design/talos-desktop/DESIGN.md)
>
> Internationalization: [`../design/talos-desktop/I18N.md`](../design/talos-desktop/I18N.md)

## Current Direction

Talos Desktop should not be a graphical reproduction of the TUI. The current product direction is a
**goal-oriented Mission workspace** for shaping outcomes, supervising execution, reviewing changes,
independently evaluating completion, and receiving a durable Delivery.

The renderer direction is **GPUI** with a Rust-native host. The earlier Tauri/WebView recommendation
from GitHub Issue #29 is retained only as historical context and is not the current selected route.

The initial Desktop UI must also be internationalized from the first visible implementation slice.
The initial supported product locales are Simplified Chinese (`zh-CN`) and English (`en-US`). Locale
is client/presentation state and must not change canonical Mission, Goal, Evaluation, Evidence,
Artifact, or Delivery identity.

## Product Split

- **TUI:** conversation-first, immediate, dense, keyboard-driven expert workflow.
- **Desktop:** goal-first, state-centric, visual workspace for longer-running and structured work.

The intended lifecycle is roughly:

```text
Intake
  -> Shaping
  -> Baselined
  -> Executing
  -> Independent Goal Evaluation
  -> Rework or Goal Completion
  -> Independent Mission Evaluation
  -> Delivery
```

The executor may claim readiness for evaluation but does not have authority to self-certify a Goal
as completed.

## Shared Work Model

Desktop should not introduce a Goal store parallel to the existing Todo system. The proposed
long-term shared model is a canonical Work Graph containing:

- Mission;
- Goal;
- WorkUnit;
- containment and dependency edges;
- acceptance criteria;
- Completion Claims;
- evaluation reports;
- artifact/evidence references.

Existing Todo semantics should evolve into WorkUnit compatibility over the shared Work Graph rather
than remain a second planning source of truth.

The exact design, migration direction, evaluator boundary, Delivery model, and future prerequisite PR
scope are documented in `talos-desktop-goal-oriented-workspace.md`.

## Desktop Experience Direction

The execution experience should not default to transcript/tool-log reproduction. It should emphasize:

- the current Goal;
- the current Work Unit;
- the current position in the Mission;
- a semantic activity stream;
- concise Artifact/change state;
- independent evaluation status;
- final evaluated Delivery.

Detailed logs, raw tool calls, stdout/stderr, full Goal Graph, full diff, and diagnostics remain
available through drill-down views.

The visual direction is light-first, Nord-derived, low-density, and focused on one dominant state
narrative per page. `docs/design/talos-desktop/DESIGN.md` is the visual baseline.

The localization direction is defined by `docs/design/talos-desktop/I18N.md`. The first GPUI
execution spike must be validated in both `zh-CN` and `en-US`, including layout/wrapping, Chinese IME,
locale selection/fallback, and preservation of locale-neutral domain identity.

## Technology Direction

### Selected: GPUI

GPUI is the current Desktop renderer direction because Talos is expected to become a text- and
workspace-heavy native application with Markdown, code, diffs, file navigation, command palette,
large lists, panels, IME, terminal/log views, and long-running Agent state.

### Long-term experiment: Makepad

Makepad remains a potential future visual/shader/generated-UI experiment. It is not the initial
Desktop product framework.

### Historical candidate: Tauri/WebView

Tauri/WebView was recommended by the original Issue #29 discussion, but later Desktop design work
selected GPUI. Tauri therefore remains useful historical tradeoff material, not implementation
authority.

## Architecture Constraints

```text
                    talos-runtime
                         |
               shared product state
                         |
                  /              \
                 /                \
          talos-tui          talos-desktop
       Ratatui/Crossterm          GPUI
```

The exact shared projection/text crate names are not fixed here. Current `talos-conversation` must be
reconciled before inventing a parallel `talos-presentation` abstraction.

Hard boundaries:

- Desktop is a host/client above existing runtime/security boundaries, not a second Agent engine;
- no GPUI dependency may flow into `talos-core` or `talos-runtime`;
- TUI and Desktop do not depend on each other;
- Work Graph/evaluation semantics are shared Talos domain state, not GPUI-local state;
- locale is Desktop presentation state, not Work Graph/runtime semantic state;
- localized labels must not become protocol, persistence, command, or enum identity;
- permission, sandbox, credential, durable-session, and evaluation/evidence boundaries remain
  authoritative.

## Future Implementation Sequence

This proposal does not authorize implementation.

Before the first GPUI Desktop implementation PR, a **separate governed prerequisite implementation
PR** must establish the Work Graph / Todo migration / independent evaluation foundation described in
the consolidated design document.

That prerequisite PR must not add GPUI or Desktop UI code.

After it is merged, a separately selected and claimed GPUI Desktop implementation slice may begin.
The first visible GPUI slice must establish the localization foundation and bilingual coverage for
its selected UI scope rather than shipping a hard-coded single-language prototype.

Implementation remains subject to requirement intake, required ADRs, iteration selection,
Collaboration Claim governance, independent review, CI, and merge-time checks.

## References

- GitHub Issue #29 — original Desktop proposal discussion.
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md` — governed owner.
- `docs/proposals/talos-desktop-goal-oriented-workspace.md` — consolidated product/architecture
  design baseline.
- `docs/design/talos-desktop/DESIGN.md` — visual design baseline and execution reference image.
- `docs/design/talos-desktop/I18N.md` — initial `zh-CN` / `en-US` internationalization contract.
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md` — reusable runtime boundary.
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md` — later multi-client model.
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md` — validation/evidence service.
- `docs/decisions/052-sdk-publication-and-composition-boundary.md` — SDK/composition boundary.
