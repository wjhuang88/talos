# Iteration I304: Absent-Journal Pause Without Initialization

> Document status: Active
> Published plan date: 2026-10-15
> Planned objective: avoid initializing SQLite only to pause an absent pending journal.
> MVP deliverable: real Actor/Runtime shutdown preserves receipts without creating an empty journal.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Absent-journal pause no-op under the existing pending-store guard; storage and real Actor/Runtime shutdown regressions and documentation only. Existing-journal durability, admission authority and shutdown policy remain unchanged. |
| Claimed At | 2026-10-15 |
| Source Issue | None |
| Governance Claim PR | #722 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized continuing the intermittent-timeout investigation and grounded repair using GitHub CI. No distinct maintainer reviewer is available; independent Agent storage/custody/API review, exact-head CI, both validators and merge-time CAS are mandatory. |
| Implementation PR | Not started |
| Last Updated | 2026-10-15 |
| Handoff / Release Condition | No implementation branch or committed implementation before finalized claim reaches main; stop if the optimization requires new custody semantics or an ADR change. |

## Published Baseline

Selected Story: [RUNTIME-011](../backlog/active/RUNTIME-011-absent-journal-pause.md), Ready,
no parent. Preserve its scope, exclusions, acceptance, validation and documentation targets.
Depends on merged #711 and accepted ADR-008/056/058/063. This draft does not activate work;
finalized proposed Active/Claimed records become effective only upon target-branch merge.

### Change-Control Decision

This is a different operation and independently acceptable outcome from I301's recovery-read
optimization. Preserve I301 unchanged in Review, do not widen its published baseline or use
#705's bounded CI exception for production. Source/guard/transaction/error and admission review
must prove the write no-op safe before implementation acceptance. No new API/unsafe/dependency.

### Selection Inventory And Dispositions

Baseline main `9b7ba28119c2edeca86f4af5a65e110fe1f4a4f5` includes #720. All iteration
headers inventoried2026-10-15; no Blocked header found (template excluded).

| Owner | State | Disposition |
|---|---|---|
| I293 / I294 | Active / Claimed | Retain Search router/provider ownership and #716; no takeover. |
| I300 | Active / Claimed | Retain Git permission repair. |
| I301 | Review / Claimed | Retain startup acceptance, #711 merged; new scope does not close it. |
| I302 | Active / Claimed | Retain self-only Windows PSS evidence/#717, including #716's independent diagnostic work. |
| I303 | Active / Claimed | #720 merged, #721 Skill implementation; no takeover of Skill/runtime context APIs. |
| I277 | Review / deferred device | Retain deferred validation. |
| I249 | Planned | Retain dependency-pilot deferral, no dependency changes. |
| I164 | Paused | Retain superseded layout, do not reactivate. |
| DEPENDENCY-003-A | Review / Claimed | Retain separately owned native dependency evidence. |

I162 historical Complete and I290/I291/#590 remain terminal. Open PRs checked: #705 diagnosis,
#716 Search plus separately owned I302 test evidence, #721 Skill. No matching RUNTIME-011/I304
owner or overlapping absent-journal pause production candidate found. Continued maintainer
direction and existing explicit non-overlap authorization apply only to this new bounded slice.
Shared Agent/Runtime files require surgical hunks and latest-main integration before publication;
do not overwrite Skill or Search additions. Board/index unions remain batched governance.

### Risks / Rollback

The write operation's successful return feeds durable reconciliation: returning zero is safe only
as a valid empty snapshot, not as a new custody/admission guarantee. Existing acknowledged work,
ParentDir ambiguity, real errors and WAL transaction behavior must be proven preserved. If review
finds a semantic conflict, stop/narrow rather than change shutdown authority under this claim.
Revert only this slice on regression; keep historical failed CI evidence and original budgets.

## Actual Activation And Execution

2026-10-15: governance-only draft; no implementation branch, production or test edit.
CI3182 evidence in #705 comment6184082259 independently recounted2219pass/1fail/0ignored out of
2220 executions. Six ordinary gates passed, baseline diagnostic failed. Successful direct caller
spans justify bounded optimization, not complete timeout attribution. Independent Agent log/source
review confirms the distinction; shared identity is not a distinct human formal approval.

## Verification / Completion Evidence

Finalized atomic proposal #722 is ineffective until main merge. Both validators, exact-head CI,
independent review and merge-time CAS are required.
No implementation or Completion Commit; not Complete. Future execution appends, not replaces, this plan.

## Variance And Residuals

#705 retains evaluator5s composite failure phase, native child latency, old10s attribution and
existing-journal costs. I301 retains its own owner-first closeout; no release or #590 reopening.
