# RUNTIME-005-A: Bounded Shutdown And Finalizer Contract Decision

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-A |
| Type | Runtime / Architecture Spike |
| Priority | P1 |
| Status | Active / Claimed |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I214 - Active / Claimed |
| Depends On | SESSION-008-A decision output; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-21 |
| Work Slice | Decide only RUNTIME-005-A / I214: current-path characterization plus one shutdown policy, arbitration, admission, deadline, finalizer ordering, durable reconciliation, redacted report and compatibility ADR with B/C boundaries. No Rust/Cargo/API/runtime/Session/persistence, TOOL-024, permission, sandbox, product UI, dependency, release, publication or unsafe change. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #336 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #336 exact head `cc99af9e` passed CI `32435705544`, both governance validators, independent claim review `5364050202` and merge-time CAS, then merged to `main` as `7de582a3`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | I214 is activated from claim merge `7de582a3`. Execute only the current-path matrix and decision ADR; require exact-head independent architecture review and CAS before merge. |

## Identity / Goal / Value

Give embedders and later runtime-owned resources one reviewed shutdown contract before Talos changes
admission, active-turn arbitration, finalizer ordering, deadline accounting, or public shutdown
reports. The output is an accepted decision and current-path matrix, not production behavior.

## Scope

- Characterize the current `RuntimeHandle::shutdown`, Session actor, cancellation, partial-turn
  persistence, task join and caller ownership paths.
- Decide one idempotent shutdown state, caller-selected active-turn policy vocabulary, total
  deadline accounting, concurrent-caller arbitration, admission fence, finalizer ownership/order,
  durable reconciliation boundary, structured redacted report and compatibility wrapper policy.
- Define B/C implementation boundaries, test seams, semver/migration triggers and rollback rules.
- Record how a later TOOL-024 supervisor registers only as a consumer after RUNTIME-005 completes,
  without introducing a dependency cycle or background-process implementation here.

## Exclusions

- No Rust, Cargo, public API, runtime, Session, persistence or shutdown behavior change.
- No background process, process signal, permission, sandbox, Desktop, Dashboard, TUI, release,
  dependency or `unsafe` change.
- No RUNTIME-005-B/C or TOOL-024-B/C/D implementation and no activation of I189/PERM-006-A.

## Dependencies

SESSION-008-A/ADR-058 supplies the interrupt/persist/discard vocabulary; SESSION-008-B is Complete
at implementation `404d7a4b`; RUNTIME-001 is Complete as a pre-1.0 facade. TOOL-024 is downstream
and cannot block this decision.

## Decision Links And Constraints

- ADR-058 and ADR-042 remain authoritative for partial-turn durability and filtering.
- The host owns process lifecycle; Talos exposes bounded runtime finalization rather than signal
  ownership.
- Reports exclude prompts, reasoning, secrets, raw tool arguments and raw output.
- A public or durable-format break requires an explicit migration plan before implementation.

## Uncertainty And Validation Path

The decision must resolve wait/interrupt/abort semantics, deadline partitioning, finalizer
registration and ordering, concurrent and repeated shutdown calls, failure aggregation, admission
races and backward-compatible `shutdown()` behavior. Unresolved safety or compatibility questions
keep RUNTIME-005-B Blocked.

## State / Status Owners

- This file owns RUNTIME-005-A scope, readiness and decision acceptance.
- `RUNTIME-005-bounded-graceful-shutdown.md` owns the parent and A -> B -> C dependency chain.
- I214 owns execution and evidence for the decision-only slice.
- Issue #49 remains the remote request/discussion surface.

## User-Facing Documentation

No user-visible behavior changes in this Spike. The decision must identify later SDK/reference
documentation targets for B/C; it must not present structured shutdown as shipped.

## Required Reads

- `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`
- `docs/backlog/active/SESSION-008-interrupted-turn-partial-persistence.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/TOOL-024-background-command-jobs.md`
- `docs/decisions/039-runtime-event-semantic-single-flow.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/decisions/058-partial-turn-durable-finalization.md`
- `crates/talos-runtime/src/`
- `crates/talos-agent/src/session.rs`
- `crates/talos-agent/src/session/turn.rs`

## Acceptance For Technical / Governance Work

- [ ] A current-path matrix traces admission, active-turn cancellation/completion, persistence,
      actor exit, task join and caller return without asserting behavior not present in code.
- [ ] A Proposed ADR answers every uncertainty above and defines independently runnable B and C.
- [ ] Independent architecture review accepts the exact decision head, including semver, privacy,
      race, deadline and dependency-direction boundaries.
- [ ] Both governance validators, Markdown/link checks and `git diff --check` pass.
- [ ] No Rust/Cargo/runtime behavior changes, and TOOL-024/PERM-006 authority remains unchanged.

## Residual Destination

Implementation belongs only to separately claimed RUNTIME-005-B/C iterations after this decision
is Accepted. Process-specific finalization remains in TOOL-024 after RUNTIME-005 and PERM-006-C
complete.

## 2026-08-21 Activation Checkpoint

Claim PR #336 final head `cc99af9e` passed exact-head CI `32435705544`, both governance
validators, independent claim review `5364050202` and merge-time CAS, then merged to `main` as
`7de582a3`. RUNTIME-005-A and I214 are Active/Claimed from that exact merge point.

Activation authorizes only read-only current-path characterization and the Proposed shutdown
contract ADR defined above. It grants no Rust, Cargo, API, runtime, Session, persistence,
permission, sandbox, TOOL-024, product UI, dependency, release, publication or `unsafe` change.
I189 remains Planned/Claimed and unactivated; I213 remains in the independent Dashboard lane.
