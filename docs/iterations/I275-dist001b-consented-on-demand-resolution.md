# I275 — DIST-001-B Consented On-Demand Capability Resolution

| Field | Value |
|---|---|
| Status | Complete / Closed |
| Source | Issue #515 / CAP-001 / #466 |
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Explicit consent, bounded verified resolution and separate Bundle activation for one missing optional capability. |
| Implementation PR | #567 (merged) |
| Completion Commit | 8ec6c271 |

## Activation Checkpoint — 2026-09-15

Claim PR #566 (head `d5f58df8`, base `606f4d8f`, CI `34975543686`) merged as `2499b9b8`.
The implementation branch starts from that merge commit; the scope and exclusions above remain
in force.

## Scope

Implement only the consented on-demand Bundle resolution path defined by ADR-077. No startup
network dependency, silent download, marketplace, broad auto-approval or permission bypass.

## Activation Condition

Claim effective after governance merge `2499b9b8`; implementation merged to `main` as `8ec6c271`.

## Acceptance

Auditable request identity and consent; bounded quarantine and verification; rollback;
fail-closed pre/post-operation cancellation, timeout, stale metadata and denial; installed content remains inactive
until existing activation and permission boundaries allow it.

## Validation

Offline/mock fixtures, network failure and cancellation tests, security review, documentation,
locked checks, governance validators and exact changed-file inventory.

## Implementation Checkpoint — 2026-09-15

The local resolver now requires exact consent, checks Bundle identity before installation, preserves
existing installations on cancellation or mismatch, and never activates content. Network acquisition
remains outside this resolver and is not implicit; synchronous installation is bounded by pre/post
operation cancellation and timeout checks.

## Completion Checkpoint — 2026-09-15

Completion Commit: `8ec6c271`

PR #567 merged as `8ec6c271` after exact-head CI `34984742102` (all five jobs successful) and
independent security review bound to head `d9e9c4e0` / base `2499b9b8`. Staging cleanup on guarded
install failure is included. No implicit network acquisition or activation was introduced.
