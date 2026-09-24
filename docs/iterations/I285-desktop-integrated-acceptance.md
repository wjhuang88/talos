# Iteration I285: Desktop Integrated Candidate And Acceptance

> Document status: Active — Claimed (proposed; ineffective until PR #608 merges)
> Plan date: 2026-09-22
> Target window: 2026-10-13 to 2026-10-19
> Planned objective: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A reproducibly built Desktop candidate passes an integrated real-task walkthrough with documented limits and a clean handoff.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | DESKTOP-001-D7 / I285: integrate prior Desktop stages, fix cross-stage acceptance defects including task-page overflow, run reproducible candidate checks and guide/record H1-H6; no new product feature or release |
| Claimed At | 2026-09-24 |
| Source Issue | #29 |
| Governance Claim PR | #608 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer-directed completion of the four-week Desktop delivery; I282-I284 are merged with technical gates; #29 is the existing cycle acceptance tracker |
| Implementation PR | Not started |
| Last Updated | 2026-09-24 |
| Handoff / Release Condition | Claim becomes effective only when the finalized governance PR merges; no release |

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

2026-09-24: I282-I284 implementation stages are merged to main. Governance PR #608 proposes
atomic activation of I285. The proposed claim has no effect until this finalized record merges.
No implementation code is included in the governance slice.

## Verification Evidence

No I285 implementation checks are claimed by this governance-only activation. The I284 Desktop
suite passed 102/102 on 2026-09-24. Planning and inherited human acceptance evidence are recorded
centrally in the four-week task and Issue #29.

## Completion Evidence

Completion Commit: pending.
Only already-existing implementation/evidence commits may close this iteration.

## Variance And Residuals

H1-H6 remain unaccepted until performed on the final integrated candidate. On 2026-09-24, H1
attempt exposed task-page controls below the visible window with no page-level scrolling at the
observed window size; the maintainer later identified the content as below the fold. A local
layout correction is being held outside this governance-only candidate and will be validated in
the I285 implementation stage. This observation is not a provider/runtime failure finding.
