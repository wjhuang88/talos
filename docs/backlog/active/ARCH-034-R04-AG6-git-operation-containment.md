# ARCH-034-R04-AG6: Git Operation Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | I181 AG-6 / gix and host-git operation boundaries |
| Status | Refinement — caller authority map and enforceable adapter decision required |
| Priority | P1 |
| Selected Iteration | None |
| Preserved behavior | Git tool schemas/results, permission ownership, repository discovery, TUI status fallback/cache, and write-operation semantics |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Independent security review required |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Finish the caller/authority map and accept a bounded adapter decision before code changes. |

## Confirmed Baseline

In-process `gix` discovery/status/revision/walk/reference work runs synchronously
inside async tools without a panic boundary or enforceable operation deadline.
TUI status catches panics and safely returns no summary, but its 500 ms cache is
not a timeout. Non-interactive host-git write paths lack a consistent deadline.

## Scope And Acceptance

- Map every `gix` and host-git caller to read/write authority, permission gate,
  blocking context, timeout, output bound and fallback.
- Select one narrow dependency adapter; do not treat `spawn_blocking` cancellation
  as proof that in-process native work stopped.
- Preserve existing tool errors and TUI no-summary fallback on corrupt/hostile
  repositories, dependency panic and operation timeout.
- Add corrupt ref/object, large walk, injected panic, spawn-not-found, nonzero and
  bounded host-git timeout fixtures.
- Give write-capable host-git paths an operation deadline without bypassing the
  existing permission pipeline.

## Exclusions And Residuals

No replacement of `gix`, global Git service, sorting/result redesign, permission
broadening or background job runtime. Other subprocess families remain evidence
work until separately classified.

## Minimum Validation

Focused tools/TUI Git tests, locked release preflight, Unix/Windows CI, permission
regressions and independent security/native-boundary review.
