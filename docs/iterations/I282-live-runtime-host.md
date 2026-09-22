# Iteration I282: Live Runtime Desktop Host

> Document status: Active — Claimed
> Plan date: 2026-09-22
> Target window: 2026-09-22 to 2026-09-28
> Planned objective: A runnable live Desktop task streams real configured-provider output and can be cancelled without blocking the UI.
> Baseline rule: once committed, preserve scope and append execution facts; changed targets use a new ID.
> MVP deliverable: A runnable live Desktop task streams real configured-provider output and can be cancelled without blocking the UI.

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

## Published Baseline

### Selected Story And Dependencies

[DESKTOP-001-D4](../backlog/active/DESKTOP-001-D4-live-runtime-host.md), parent DESKTOP-001;
Claimed / Active proposed in #598; both states remain ineffective until merge.
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

2026-09-22: atomic claim/activation merged through #598. Claim PR #598 head
`6fb3895611a1ad75eff0ec990607737a75c5f49a` reached `main` at activation merge
`d323ce5d184df57d554f9646cf4a4e639de81995`; implementation proceeds from that effective base.

2026-09-23 checkpoint: local I282 implementation adds the live Runtime host, configured provider
construction, bounded presentation buffering, typed terminal outcomes, no-tools disclosure,
shutdown receipts, and cancellation through provider-backed history compaction. Focused Desktop
tests: 61 passed; Runtime interrupt tests: 2 passed. The stable implementation candidate is not
yet pushed. Remaining gates are full local preflight, governance synchronization, independent
technical/security review, exact-head CI, and deferred native rows H1/H5 in #29.

## Verification Evidence

Implementation checks: `cargo test -p talos-desktop --locked --features desktop-ui --bin
talos-desktop-mock` (61 passed); `cargo test -p talos-runtime --locked interrupt` (2 passed).
Planning checks are recorded centrally in the four-week task. Full preflight and stable-candidate
remote evidence remain pending.

## Completion Evidence

Completion Commit: pending.
Only already-existing implementation/evidence commits may close this iteration.

## Variance And Residuals

None yet. Carry eligible human rows to #29 / I285 without transferring protected security gates.
