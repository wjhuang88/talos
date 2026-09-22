# DESKTOP-001-D7: Desktop Integrated Candidate And Acceptance

> Document status: Planned

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D7 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Selected Iteration | I285 |
| Source | #29; four-week Desktop task |
| Depends On | I282-I284 implementation stages merged with technical gates; acceptance ledger in #29 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | DESKTOP-001-D7 / I285: Desktop Integrated Candidate And Acceptance; planned scope only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested four-week Desktop planning on 2026-09-22; no implementation activation yet |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Resolve readiness and establish effective serial child claim before implementation; no release |

## Outcome

A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.

## Scope

- Converge cross-stage defects locally: focus/IME, scrolling, tool/result identity, permission placement, cancellation, recovery and stale evaluation.
- Reserve three days for fixes and two for acceptance/documentation; do not add new product features.
- Run a reproducible native candidate build and consolidated smoke/acceptance flow against the exact integrated head.
- Guide human checks one at a time, reusing prior valid evidence when code has not affected it; record exact head/device and remaining unsupported platform cases.
- Update launch/configuration/tool/approval/resume/evidence instructions and capability limits; synchronize owner-first completion and clean only verified merged branches/task-owned artifacts.

## Acceptance

- Given the candidate, the user can create, execute, approve/deny, cancel, restart/resume and inspect results with no mock data presented as live.
- All required H1-H6 acceptance rows have passing evidence or the cycle remains Partial/Review; unavailable pre-existing I277 rows remain explicitly separate residuals.
- The integrated current head passes applicable CI and independent technical/security/API review; documentation reflects actual capabilities and deferred limitations.
- A clean checkout can reproduce the documented build/launch; no signing, distribution, release or publication is implied.
- Completion records cite existing implementation commits and preserve earlier baselines; no open protected blocker is hidden in cleanup notes.

## Validation And Risks

Full integrated deterministic E2E and locked release_preflight, Desktop feature builds in applicable CI, native H1-H6 ledger, bilingual docs/link checks, independent Agent-role integration review.

Device availability can delay manual closure. Record Partial with exact outstanding rows rather than marking Complete. Corrections that expand scope require explicit replanning.

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
New features, retroactive claims of Windows/Linux native or physical latency acceptance, new installers/auto-update, release/version/tag/crates publication.

## Completion Evidence

Completion Commit: pending.
No implementation or acceptance evidence exists for this planned child.
