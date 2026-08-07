# ARCH-034-R08: Agent Session Custody Decomposition

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F24 |
| Status | Ready |
| Priority | P2 |
| Selected Iteration | I177 (Planned; Claim PR pending) |
| Preserved behavior | Actor ordering, generation fences, receipts, recovery, pause/cancel, and archive |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | None |

## Problem And Boundary

`talos-agent/src/session.rs` is a 1,386-line, high-change actor that combines run-loop dispatch,
durable custody/reconciliation, rejection/receipt projection, turn start/finish, and archiving.

## Scope

- Extract private custody/reconciliation helpers while keeping `AppServerSession` as the actor.
- Preserve the one-writer state machine, queue ordering, and generation checks.

## Exclusions

- No channel, persistence protocol, scheduler, retry, receipt, or public API change.

## Acceptance And Validation

- Extracted helpers cannot independently mutate actor state outside explicit parameters/results.
- Structured submission, recovery, pause/cancel, generation, and shutdown tests remain identical.
- Locked workspace, governance, and diff checks pass.

## Rollback / Residual

Revert if actor ordering/custody equivalence cannot be proven. Actor redesign requires an ADR.
