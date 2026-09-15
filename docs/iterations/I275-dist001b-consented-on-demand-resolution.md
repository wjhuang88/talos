# I275 — DIST-001-B Consented On-Demand Capability Resolution

| Field | Value |
|---|---|
| Status | Planned / Claimed |
| Source | Issue #515 / CAP-001 / #466 |
| Claim State | Claimed (pending governance merge) |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | Explicit consent, bounded verified resolution and separate Bundle activation for one missing optional capability. |
| Implementation PR | Not started |

## Scope

Implement only the consented on-demand Bundle resolution path defined by ADR-077. No startup
network dependency, silent download, marketplace, broad auto-approval or permission bypass.

## Activation Condition

This claim is ineffective until its governance record merges to `main`; implementation starts
from that merge or a later main commit.

## Acceptance

Auditable request identity and consent; bounded download/quarantine and verification; rollback;
fail-closed cancellation, timeout, stale metadata and denial; installed content remains inactive
until existing activation and permission boundaries allow it.

## Validation

Offline/mock fixtures, network failure and cancellation tests, security review, documentation,
locked checks, governance validators and exact changed-file inventory.
