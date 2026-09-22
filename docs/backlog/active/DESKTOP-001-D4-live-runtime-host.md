# DESKTOP-001-D4: Live Runtime Desktop Host

> Document status: Planned

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D4 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Selected Iteration | I282 |
| Source | #29; four-week Desktop task |
| Depends On | WORK-001 P0-P4 and I280 complete; ADR-059; explicit host/API readiness map |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | DESKTOP-001-D4 / I282: Live Runtime Desktop Host; planned scope only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested four-week Desktop planning on 2026-09-22; no implementation activation yet |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Resolve readiness and establish effective serial child claim before implementation; no release |

## Outcome

A runnable live Desktop task streams real configured-provider output and can be cancelled without blocking the UI.

## Scope

- Add an explicit live launch path alongside the existing visibly labelled mock; reuse the accepted layout and bilingual Settings.
- Map provider/config loading, RuntimeHandle ownership, event buffering and shutdown. Reuse one application-owned Tokio execution domain; no per-request runtime and no blocking UI calls.
- Select workspace and submit a task; display connecting/streaming/error/cancelled/finished from authoritative events. Preserve task identity during navigation and locale changes.
- In this first slice register no executable tools. A requested tool is explicitly unavailable; no simulated successful output or hidden execution.
- Keep business lifetime separate from view observation. Define window-close shutdown with bounded errors and explicit outcomes.

## Acceptance

- Given valid configuration, when a task is submitted, the window displays actual provider content while input/navigation remains responsive.
- Given no credentials or an unavailable provider, show actionable setup/error state without exposing secrets or substituting mock output.
- Given streaming work, explicit cancellation and host shutdown reach observable terminal states; view disposal alone does not cancel business work.
- Given Chinese IME or locale switching, input and task identity remain correct under the accepted visual baseline.
- Before coding, the API map resolves command/event borrowing, queue saturation and Tokio ownership; any uncovered public API decision has an accepted migration path.

## Validation And Risks

Fake-provider native-host E2E for success/error/cancel/shutdown, queue saturation and view teardown; focused Desktop/Runtime locked tests; real-provider row H1 and bilingual row H5.

A facade ownership gap may require a bounded decision before coding. Keep mock launch independent and surface unsupported live state; never detach business work as a workaround.

## Required Reads

- [Four-week task](../../tasks/2026-09-22-desktop-four-week-delivery.md)
- [Desktop parent](DESKTOP-001-desktop-product-direction.md)
- [Renderer boundary](../../decisions/059-desktop-renderer-host-motion-boundary.md)
- [Visual baseline](../../design/talos-desktop/DESIGN.md)
- [Localization](../../design/talos-desktop/I18N.md)
- [Shared work foundation](WORK-001-goal-oriented-work-evaluation-foundation.md)
- [Runtime SDK migration](../../reference/I280-RUNTIME-FACADE-MIGRATION.md)

## Documentation And Residuals

Update the Desktop crate README (created in I282), bilingual user-facing instructions as behavior
lands, and this owner before derived views. Preserve #29 and the four-week task acceptance ledger.
Tool execution/approval integration (I283), durable navigation (I284), new Runtime engine, full presets, new permissions or release.

## Completion Evidence

Completion Commit: pending.
No implementation or acceptance evidence exists for this planned child.
