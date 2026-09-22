# Iteration I282: Live Runtime Desktop Host

> Document status: Planned
> Plan date: 2026-09-22
> Target window: 2026-09-22 to 2026-09-28
> Planned objective: A runnable live Desktop task streams real configured-provider output and can be cancelled without blocking the UI.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A runnable live Desktop task streams real configured-provider output and can be cancelled without blocking the UI.

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

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D4](../backlog/active/DESKTOP-001-D4-live-runtime-host.md), parent DESKTOP-001;
Planned / Unclaimed, not yet Ready or activated.
Prerequisites: WORK-001 P0-P4 and I280 complete; ADR-059; explicit host/API readiness map.
Follow [the four-week task](../tasks/2026-09-22-desktop-four-week-delivery.md) for
serial scheduling, inventory, resource limits, authorization and deferred acceptance. Dates do not
activate implementation. Resolve readiness, then use one atomic claim/activation transition.

### Scope

- Add an explicit live launch path alongside the existing visibly labelled mock; reuse the accepted layout and bilingual Settings.
- Map provider/config loading, RuntimeHandle ownership, event buffering and shutdown. Reuse one application-owned Tokio execution domain; no per-request runtime and no blocking UI calls.
- Select workspace and submit a task; display connecting/streaming/error/cancelled/finished from authoritative events. Preserve task identity during navigation and locale changes.
- In this first slice register no executable tools. A requested tool is explicitly unavailable; no simulated successful output or hidden execution.
- Keep business lifetime separate from view observation. Define window-close shutdown with bounded errors and explicit outcomes.

### Non-Goals

Tool execution/approval integration (I283), durable navigation (I284), new Runtime engine, full presets, new permissions or release.

### Acceptance

- Given valid configuration, when a task is submitted, the window displays actual provider content while input/navigation remains responsive.
- Given no credentials or an unavailable provider, show actionable setup/error state without exposing secrets or substituting mock output.
- Given streaming work, explicit cancellation and host shutdown reach observable terminal states; view disposal alone does not cancel business work.
- Given Chinese IME or locale switching, input and task identity remain correct under the accepted visual baseline.
- Before coding, the API map resolves command/event borrowing, queue saturation and Tokio ownership; any uncovered public API decision has an accepted migration path.

### Planned Validation

Fake-provider native-host E2E for success/error/cancel/shutdown, queue saturation and view teardown; focused Desktop/Runtime locked tests; real-provider row H1 and bilingual row H5.

Use the pinned toolchain and --locked. Before a stable code candidate, run focused checks,
./scripts/release_preflight.sh, both governance validators, diff/secret review and applicable
independent Agent-role review. Exact-head remote CI validates the stable stage. Planning-only
changes do not require Rust builds. Required human rows remain Review until accepted.

### Documentation To Update

- Desktop crate README with delivered launch/behavior instructions (create in I282).
- README.md and README.zh-CN.md; site English/Chinese capability instructions where affected.
- Child owner, this iteration, four-week task, parent summary, indexes and #29 acceptance evidence.

### Risks And Rollback

A facade ownership gap may require a bounded decision before coding. Keep mock launch independent and surface unsupported live state; never detach business work as a workaround.
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
