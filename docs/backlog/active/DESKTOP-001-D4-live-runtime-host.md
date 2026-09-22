# DESKTOP-001-D4: Live Runtime Desktop Host

> Document status: Active — Claimed

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D4 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Active / Claimed — local implementation convergence |
| Selected Iteration | I282 |
| Source | #29; four-week Desktop task |
| Depends On | WORK-001 P0-P4 and I280 complete; ADR-059; explicit host/API readiness map |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | DESKTOP-001-D4 / I282: live Desktop Runtime host, provider output, cancellation and host lifecycle; no tools |
| Claimed At | 2026-09-22 |
| Source Issue | #29 |
| Governance Claim PR | #598 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User authorized continuation; exact-head CI, governance validators and merge-time CAS required; no independent natural-person reviewer available |
| Implementation PR | Local convergence in progress; stable candidate not pushed |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Effective only after #598 reaches main; implementation starts from its merge or later main; no release |

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

## Execution Checkpoint — 2026-09-23

The effective activation is #598 (head `6fb3895611a1ad75eff0ec990607737a75c5f49a`), merged at
`d323ce5d184df57d554f9646cf4a4e639de81995`. Local implementation now contains the live Runtime
host, configured provider adapter, bounded presentation queue, typed completion/error states,
no-tools disclosure, shutdown receipts, and cancellation during provider-backed history
compaction. Focused evidence is 61 Desktop tests and 2 Runtime interrupt tests passing. The
candidate remains local until the full preflight, governance checks, independent review and
exact-head CI are complete. H1/H5 native acceptance remains deferred in #29.

## Completion Evidence

Completion Commit: pending.
No implementation or acceptance evidence exists for this planned child.
