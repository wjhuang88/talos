# Iteration I285: Desktop Integrated Candidate And Acceptance

> Document status: Planned
> Plan date: 2026-09-22
> Target window: 2026-10-13 to 2026-10-19
> Planned objective: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.

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

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D7](../backlog/active/DESKTOP-001-D7-desktop-integrated-acceptance.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: I282-I284 implementation stages merged with technical gates; acceptance ledger in #29.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Converge cross-stage defects locally: focus/IME, scrolling, tool/result identity, permission placement, cancellation, recovery and stale evaluation.
- Reserve three days for fixes and two for acceptance/documentation; do not add new product features.
- Run a reproducible native candidate build and consolidated smoke/acceptance flow against the exact integrated head.
- Guide human checks one at a time, reusing prior valid evidence when code has not affected it; record exact head/device and remaining unsupported platform cases.
- Update launch/configuration/tool/approval/resume/evidence instructions and capability limits; synchronize owner-first completion and clean only verified merged branches/task-owned artifacts.

### Non-Goals

New features, retroactive claims of Windows/Linux native or physical latency acceptance, new installers/auto-update, release/version/tag/crates publication.

### Acceptance

- Given the candidate, the user can create, execute, approve/deny, cancel, restart/resume and inspect results with no mock data presented as live.
- All required H1-H6 acceptance rows have passing evidence or the cycle remains Partial/Review; unavailable pre-existing I277 rows remain explicitly separate residuals.
- The integrated current head passes applicable CI and independent technical/security/API review; documentation reflects actual capabilities and deferred limitations.
- A clean checkout can reproduce the documented build/launch; no signing, distribution, release or publication is implied.
- Completion records cite existing implementation commits and preserve earlier baselines; no open protected blocker is hidden in cleanup notes.

### Planned Validation

Full integrated deterministic E2E and locked release_preflight, Desktop feature builds in applicable CI, native H1-H6 ledger, bilingual docs/link checks, independent Agent-role integration review.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

Device availability can delay manual closure. Record Partial with exact outstanding rows rather than marking Complete. Corrections that expand scope require explicit replanning.
Preserve working mock/preceding stage. Disable incomplete live features rather than weakening
safety. Roll back only the failing slice; no destructive cleanup of user sessions or workspaces.

## Actual Activation And Execution

2026-09-22: planning only. Claim, implementation PR and remote evidence not yet established.

## Verification Evidence

Implementation checks: not run; no implementation exists for this child.
Planning checks are recorded centrally in the four-week task.

## Completion Evidence

Completion Commit: pending.
Only already-existing implementation/evidence commits may close this iteration.

## Variance And Residuals

None yet. Carry eligible human rows to #29 / I285 without transferring protected security gates.
