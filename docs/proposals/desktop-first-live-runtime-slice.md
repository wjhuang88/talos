# Desktop First Live Runtime Slice

Status: Proposed intake; no effective implementation claim.
Scheduling successor: [four-week Desktop task](../tasks/2026-09-22-desktop-four-week-delivery.md)
and I282-I285 divide this initial intake into serial weekly deliverables. Use their owners for
current scope/status; this proposal is retained as the initial rationale, not a competing plan.
Date: 2026-09-22
Source: #29; maintainer requests resuming Desktop after deferring #502 research.
Inspected baseline: `584d243d1d02f96d83592451075ad6063a2c8fde`.

## Runnable Outcome

A local single-client Desktop user can select a workspace, enter a task, submit it to the existing
Talos Runtime with a configured provider, observe real output/tool activity, respond to permission
requests, explicitly cancel execution, and see its actual terminal state. Preserve the accepted
GPUI design and bilingual settings experience. Do not present mock evaluations as real results.

## Existing Work Disposition

- I277: retain Review/Claimed and deferred device/accessibility acceptance. Preserve delivered
  mock and passed manual evidence; do not expand that iteration's published scope.
- I249: retain Planned, deferred while Desktop is selected; #502 remains research-only.
- I164: retain Paused and its recorded supersession.
- WORK-001 P0-P4: complete shared foundation, reuse rather than reimplement.
- MODEL-007: previously selected next priority, defer under the current Desktop scheduling request;
  do not claim its implementation has started.
- DESKTOP-002/#308: retain separate preset/model-role requirements. Do not require full preset
  management or multi-client support before the first single-client slice.

This is a preliminary inventory. Recheck all non-terminal iteration owners and remote overlap
before recording atomic claim/activation for a new iteration.

## Boundaries

- GPUI owns presentation and UI-thread state; existing Tokio-backed Talos Runtime owns business
  execution. Follow ADR-059; never create a runtime per request or synchronously wait on the UI thread.
- Reuse supported Runtime composition, configuration/provider loading, permission and Session
  authority. Do not copy CLI execution logic into a second Desktop engine.
- Use bounded communication and retain task lifetimes. Distinguish observer disposal, explicit
  cancellation request and confirmed execution termination; closing a view is not cancellation.
- Permission prompts must bind to the correct request and expire/cancel safely. Neither Auto nor
  Desktop may bypass Deny. Keep credentials out of UI diagnostics, logs and fixtures.
- Keep mock mode visibly separate. Missing provider configuration produces a useful setup/error
  state, never silent fallback to fabricated execution.
- Do not invent persistence or claim Mission evaluation/Delivery completion from an assistant's
  final message. Shared evaluation integration and durable recovery must be explicitly scoped.

## Acceptance

1. Given valid local configuration, submitting a task displays actual provider output and tool
   activity in the accepted Desktop layout, with UI input remaining responsive.
2. Given a request requiring approval, allow/deny/cancel reaches the existing permission pipeline
   exactly once; stale responses cannot approve a different request.
3. Given running work, explicit cancellation reaches a confirmed terminal state; navigation alone
   does not silently cancel business work. Repeated cancellation remains safe.
4. Given provider failure, channel closure or shutdown, the UI reports the true failure/terminal
   state and releases observation resources without claiming successful execution.
5. Chinese/English switching preserves task identity and business state. Existing input/settings
   behavior remains intact; locale changes do not translate user-authored content.

## Implementation Readiness Checks

Before activating implementation, map the current Runtime builder/session/event/approval/shutdown
APIs to Desktop consumers. Identify which APIs are already sufficient and any missing facade
contract. Any new public API or change outside accepted ADR-059 boundaries requires its own
explicit decision/migration assessment, not an assumed extension of I277.

Use deterministic mock-provider integration tests for event ordering, approval races, cancellation,
and shutdown; native acceptance covers actual input/rendering and a real configured provider.
Local convergence precedes one stable implementation candidate. Security/API boundaries require
independent review under the authorized single-maintainer Agent-role process. No per-test issues.

## Delivery And Residuals

Create a new executable child owner and iteration after readiness checks, establish the effective
claim using the existing atomic activation procedure, then implement. Track scope and acceptance
under existing #29. Update Desktop user-facing launch/configuration/cancellation documentation.
Keep deferred I277 acceptance, #308 presets, multi-client/reconnect, packaging, and broader
artifact/evaluation/delivery requirements visible under their existing owners; this slice does not
close the Desktop parent or authorize a release.

## Required Reads

- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/DESKTOP-002-preset-session-environment-templates.md`
- `docs/decisions/059-desktop-renderer-host-motion-boundary.md`
- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/I18N.md`
- `docs/iterations/I277-desktop-mock-visual-i18n.md`
- `crates/talos-runtime/`
- `crates/talos-desktop/`
