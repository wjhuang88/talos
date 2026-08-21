# RUNTIME-005-A: Bounded Shutdown And Finalizer Contract Decision

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | RUNTIME-005-A |
| Type | Runtime / Architecture Spike |
| Priority | P1 |
| Status | Complete / Closed |
| Parent Epic | RUNTIME-005 |
| Source | [GitHub Issue #49](https://github.com/wjhuang88/talos/issues/49) |
| Selected Iteration | I214 - Complete / Closed |
| Depends On | SESSION-008-A decision output; SESSION-008-B Complete; RUNTIME-001 Complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-21 |
| Work Slice | Decide only RUNTIME-005-A / I214: current-path characterization plus one shutdown policy, arbitration, admission, deadline, finalizer ordering, durable reconciliation, redacted report and compatibility ADR with B/C boundaries. No Rust/Cargo/API/runtime/Session/persistence, TOOL-024, permission, sandbox, product UI, dependency, release, publication or unsafe change. |
| Claimed At | 2026-08-21 |
| Source Issue | #49 |
| Governance Claim PR | #336 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #336 merged as `7de582a3`. Decision PR #338 exact head `6719c876` passed CI `32449605985`, independent architecture review `5365529351` and merge-time CAS, then merged as `fc70e396`. |
| Implementation PR | #338 |
| Last Updated | 2026-08-21 |
| Handoff / Release Condition | Closed at Completion Commit `6719c876`; RUNTIME-005-B requires its own runnable iteration and effective claim before implementation. |

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

## Decision Execution Evidence

- Current-path matrix: `docs/reference/I214-RUNTIME-SHUTDOWN-CURRENT-PATH.md`, grounded at
  activation merge `14531bbc`.
- Proposed decision: `docs/decisions/063-bounded-runtime-shutdown-finalization.md`.
- Decision content commit: `648a35d3`; implementation PR #338. The later status commit cannot use
  itself as completion evidence.
- The proposal selects first-valid-request arbitration, a shared SDK/actor admission-start arbiter,
  construction-time validated shutdown options and explicit primary/controller Drop semantics,
  `FinishCurrent`/`Interrupt` policies, one absolute deadline, actor-owned durable reconciliation,
  a frozen ordered finalizer registry and a redacted shared report.
- RUNTIME-005-B and C remain separate implementation slices. This evidence changes no production
  behavior and does not authorize TOOL-024 or I189.

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

## 2026-08-21 Architecture Review Correction

Independent review of PR #338 head `0adcd072` found two blocking contract defects: a standalone
actor closing-bit check left a check-to-start race, and a consuming structured shutdown could turn
invalid options into default shutdown through primary-handle Drop. The corrected proposal uses one
SDK/actor admission-start arbiter with an explicit non-await start-commit point, validates options
before coordinator access, makes structured shutdown borrow its handle, and defines primary versus
controller Drop behavior.
RUNTIME-005-A/I214 remains Active pending fresh exact-head review; no B/C implementation authority
is created.

## Completion Evidence

- Completion Commit: `6719c876fe9f190e47fba5ef62f3263e782d6e8b` (pre-existing reviewed
  decision correction; this closeout status commit does not self-certify).
- PR #338 merged as `fc70e3969f5fd1c7f57b386c3ed9859458cd0127` after exact-head CI
  `32449605985`, independent architecture approval `5365529351` and merge-time CAS.
- ADR-063 is Accepted. The matrix and decision remain documentation-only; no runtime behavior is
  claimed.
- RUNTIME-005-B is Ready/Unclaimed and still requires a separate iteration and effective claim;
  C remains Blocked and TOOL-024/PERM-006 authority is unchanged.
