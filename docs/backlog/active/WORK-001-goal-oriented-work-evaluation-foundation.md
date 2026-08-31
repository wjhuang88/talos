# WORK-001: Goal-Oriented Work And Evaluation Foundation

| Field | Value |
|---|---|
| Story ID | WORK-001 |
| Type | Architecture / Domain Epic |
| Priority | P0 |
| Status | In Progress — P0/P1/P2/P3 Complete; P4 is the next separately governed slice |
| Source | DESKTOP-001 refined direction; GitHub Issue #29; three-track development baseline |
| Selected Iteration | None — Epic parents are not selected directly |
| Depends On | RUNTIME-001; TODO-001; TODO-002; VALIDATION-001; ADR-008; ADR-024; ADR-042; ADR-052 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — children require separate non-overlapping claims |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None — Epic parents are not implementation units |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Close every required child through its own owner, iteration, claim, implementation PR, acceptance evidence and independent exact-head review. |

## Identity / Goal / Value

Establish one shared Talos work-state and evaluation foundation that can be consumed by current and
future product surfaces without creating a Desktop-only domain, a second Todo authority or a second
agent runtime.

This Epic converts the future P0-P4 chain in the Desktop direction into separately governed
mainline work. It does not authorize any child merely because the overall direction is recorded.

## Child Summary And Dependency Order

| Child | Outcome | Status | Depends On | Iteration |
|---|---|---|---|---|
| WORK-001-A / P0 | Decide the canonical work-state boundary and publish the Todo migration, compatibility and rollback contract. | Complete / Closed | Current repository and dependency inventory | I196 Complete / Closed |
| WORK-001-B / P1 | Implement the canonical Work Domain and mechanically prove Todo compatibility. | Complete / Closed | WORK-001-A Accepted and Complete | I237 Complete / Closed |
| WORK-001-C / P2 | Implement Completion Claim and revision-bound Evaluation state semantics. | Complete / Closed; Completion Commit `209931e5`, PR #445 merged | WORK-001-B Complete | I238 Complete / Closed |
| WORK-001-D / P3 | Implement an independent evaluator runtime boundary and consume Validation evidence safely. | Complete / Closed | WORK-001-C Complete | I239 Complete / Closed |
| WORK-001-E / P4 | Implement Mission final gating, UI-neutral projection and non-GPUI end-to-end closure. | Active / Claimed (proposed; ineffective until PR #450 merges) | WORK-001-D Complete | I240 Active / Claimed (proposed) |

Completion Commit: `779a4c7116610f07258013e866f74b2a180c5453` (WORK-001-A P0 decision packet; the
Epic remains open while P1-P4 are separately governed).

P1-P4 identifiers reserve dependency boundaries only. Their executable owner documents, iteration
plans and claims must be created separately after the preceding child is accepted; this P0 claim
must not pre-authorize or implement them.

## 2026-08-31 P3 Completion Checkpoint

WORK-001-D/I239 is Complete / Closed through implementation source `2ff8b9f4` merged by PR #448
as `ac4ce436`. Exact-head CI `33359367715` passed all five jobs and the independent
runtime/security Agent-role review recommendation is recorded at review `5063376615`. P4 remains
blocked and separately governed; no Mission gate, UI-neutral projection or end-to-end delivery was
implemented in P3.

## 2026-08-31 P3 Requirement Intake Checkpoint

WORK-001-C/I238 is Complete / Closed on `main` at Completion Commit `209931e5`. WORK-001-D/P3 has
now completed through its bounded I239 owner and implementation. Its scope was limited to fresh
evaluator context, read-only admission, Validation-001 evidence consumption and explicit
fail-closed outcomes. Mission final gating, UI-neutral projection and end-to-end Delivery remain
WORK-001-E/P4 and blocked.

## 2026-08-30 P1 Claim Preparation Checkpoint

WORK-001-A/I196 is Complete / Closed and its completion evidence is present on `main`. The first
eligible child, WORK-001-B/P1, now has an independent requirement owner and I237 iteration with a
bounded Collaboration Claim proposal. The proposed claim is ineffective until its governance PR
merges to `main`; no implementation branch, Rust/Cargo change, persistence change or P2-P4
activation is authorized before that merge. P2, P3 and P4 remain Blocked on their direct child
prerequisites.

## 2026-08-31 P1 Completion Checkpoint

WORK-001-B/I237 is Complete / Closed through Completion Commit
`f2b0b5c7e5f5080c9c36f7b7a1993af4246f6f91`, containing final implementation head
`319a1f1576a2a47a49c987491c2c017376510bf6`. Exact-head CI run `33323509558` attempt 2
passed all five jobs, independent Agent-role review comment `5470154364` approved the same
head/base, and merge-time CAS completed before PR #442 merged. P2 is now the next dependency-order
candidate, but remains unactivated until this closeout reaches `main` and a separate requirement
owner, runnable iteration and effective Collaboration Claim are established. P3 and P4 remain
blocked on their direct predecessors.

## 2026-08-31 P2 Requirement Intake Checkpoint

The I237 closeout merged to `main` as `6ed8a24f1e6b92f8e6e62b58841e27736557065f`, so
WORK-001-C/P2 now has its own API/State Story owner and runnable I238 iteration proposal. P2 is
limited to storage-neutral Completion Claim, Acceptance Criterion and exact-revision Evaluation
state semantics. This proposal does not activate P2 until its finalized atomic claim record merges;
P3/P4 remain blocked and no implementation authority transfers from I237.

## Scope

- Maintain the ordered P0-P4 dependency and completion condition.
- Keep canonical Mission, Goal, WorkUnit, Completion Claim and Evaluation semantics in shared
  mainline ownership.
- Preserve current runtime, session, permission, validation and presentation boundaries until a
  child explicitly changes them through an accepted decision and migration plan.
- Require every child to produce one independently runnable or mechanically testable deliverable.

## Exclusions

- No direct Epic implementation or Epic-wide claim.
- No Desktop, Dashboard, GPUI, localization or native-host implementation.
- No generic workflow scheduler, global event bus or multi-agent framework.
- No child may absorb a later child solely to accelerate Desktop binding.

## Dependencies And Constraints

- `RUNTIME-001` is the current pre-1.0 reusable facade; shared work APIs must not bypass it without
  an accepted boundary decision.
- `TODO-001` and `TODO-002` define shipped persistence, tool, permission and compatibility behavior.
- `VALIDATION-001` is an Evidence producer, not the authority that decides Goal completion.
- `SESSION-009` owns later attach, reconnect and multi-client behavior; a local single-client
  domain decision must remain compatible without claiming SESSION-009 implementation.
- Breaking public APIs require a decision record and migration plan. Persistent storage changes
  require explicit compatibility, rollback and failure-handling evidence.

## Completion Condition

This Epic is Complete only when WORK-001-A through WORK-001-E are each Complete or an explicit
scope decision removes a child, one canonical work-state source of truth exists, Todo compatibility
is mechanically proven, independent criterion-level evaluation gates Mission Delivery, and one
non-GPUI end-to-end walkthrough passes through the shared projection.

## State / Status Owners

- Epic status and child dependency map: this file.
- P0 scope and acceptance: `docs/backlog/active/WORK-001-A-work-domain-decision-migration-contract.md`.
- P0 execution: `docs/iterations/I196-work-domain-decision-migration-contract.md`.
- Product direction: `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Do not present Mission, Work Graph, independent Evaluation, Delivery gating or Desktop binding as
shipped while the applicable children remain incomplete. Each behavior-facing child must name and
update its own user-facing documentation.

## Required Reads

- `docs/proposals/talos-desktop-goal-oriented-workspace.md`, section 20
- `docs/tasks/2026-08-13-three-track-development-baseline.md`
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`

## Residual Destination

Create WORK-001-B through WORK-001-E owner documents only when their direct prerequisite is
accepted and their own requirement intake can define a bounded, testable result.

## 2026-08-31 P4 Requirement Intake Checkpoint

WORK-001-D/I239 is Complete / Closed on `main` with implementation evidence `2ff8b9f4`. The P4
requirement owner `WORK-001-E` and runnable iteration `I240` are now prepared for the exact scope
in DESKTOP-001 section 20.5: Mission final evaluation gating, UI-neutral projection and a non-GPUI
end-to-end walkthrough. The claim is still pending and ineffective; no implementation branch,
Rust/Cargo change, persistence change or Desktop/Dashboard authority is authorized until the
atomic claim+activation record reaches `main`. PR #450 carries the proposed atomic record.
