# Iteration I305: Windows Known-Resource Acceptance

> Document status: Active
> Published plan date: 2026-10-16
> Planned objective: replace an unqualified process-total gate with tested known-resource cleanup acceptance.
> Baseline rule: preserve this scope; a different acceptance target uses another iteration.
> MVP deliverable: executable Windows all40 fault and real negative-control resource cleanup receipts.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | TEST-004 known successful-return Windows resource cleanup acceptance, native call-site coverage and real negative controls; no production launcher/search behavior or I302 PSS attribution. |
| Claimed At | 2026-10-16 |
| Source Issue | #729 |
| Governance Claim PR | #730 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly accepted #717 proposal6217718821 on2026-10-16 at22:39 Asia/Shanghai and continued GitHub delivery. No distinct maintainer reviewer available; independent protected Agent security/technical review, exact-head CI, validators and CAS remain mandatory. |
| Implementation PR | #716 after effective claim merge and baseline refresh; separate I305 commits |
| Last Updated | 2026-10-16 |
| Handoff / Release Condition | No implementation before finalized claim and ADR-091 amendment reach main; no automatic Complete or #716 merge. |

## Published Baseline

Selected [TEST-004](../backlog/active/TEST-004-windows-known-resource-acceptance.md), Ready / Unclaimed,
Source Issue#729; no parent. Preserve all acceptance/exclusions/validation/residuals in that owner.
ADR-091 is amended before implementation; ADR-068 lifecycle semantics stay unchanged.

### Selection Inventory And Disposition

Selection main3bc38d780804799133f20cfdede583901fb5ada4. Full iteration headers inventoried:
I293/I294/I300 Active retain Search/provider/Git ownership; I302 Active on main (Review diagnostic
candidate #716) retains PSS and unresolved attribution. I249 Planned remains deferred dependency
pilot. No Blocked header found. I277 deferred review and I164 paused disposition remain unchanged;
I162 historical terminal wording does not reactivate it. I301/I303/I304 Complete/Closed stay closed.
Open #705 retains independent latency diagnostics, #716 existing form/PSS candidate may carry
separate I305 commits only after this new claim reaches main and its baseline is refreshed.
New acceptance target uses I305 rather than rewriting I302's published diagnostic objective.
Maintainer expressly selected this bounded adjacent correction; no ownership transfer.

### Scope / Non-Goals / Acceptance

The runnable deliverable is a cfg(test,windows) resource acceptance fixture: all40 original fault
attempts, actual call-boundary ownership/release results and enforced source inventory, plus real
omitted-close/duplicate-guard/omitted-termination/omitted-attribute-delete sensitivity with recovery.
Retain counts/direct controls/failures. No warmup/threshold increase/skip, production launcher or
search behavior change. Known-resource proof is narrower than all-process leak absence; #717 stays
open as its declared residual. The exact native lifecycle, waits, error handling and resource limits
are in TEST-004 and the amended ADR091. Stable local convergence precedes publication.

### Planned Validation

Pinned1.99 locked full release_preflight, Windows test-target check/Clippy, real Windows positive
and negative receipts, source native-call inventory sensitivity, normal/build vs test feature graphs,
both governance validators/text/classifier/fmt/diff/scale and independent protected exact-head review.
Only the previously authorized exact local Unix socket fixture may be filtered externally;
remote CI remains unfiltered. No new ignore or waiver. Positive wait must observe exit; a negative
fixture must detect the actual intended defect and recover resources, not merely edit a receipt.

### Documentation / Risks / Rollback

Operator-facing docs are TEST-004/I305/ADR091 and bounded native receipts; infrastructure-only,
no README user feature. Update Board/index/matrix/Issue owner-first. Retained resources during
negative fixtures need explicit recovery; use no double-close of reused values. Limits and outer
Jobs do not prove every OS-private lifetime. Revert acceptance code rather than silently pass an
unverified gate. I302 residual must not be marked Complete by this work.

## Actual Activation And Execution

2026-10-16: maintainer accepted concrete criterion change at22:39 Asia/Shanghai. This governance-only
proposal has no implementation/claim/activation effect until finalized PR reaches main. No new
code, unsafe, feature/dependency or production change. No Completion Commit exists.

## Verification / Completion / Residuals

Draft validators/protected contract review/exact-head applicable CI/CAS pending. Implementation and
native positive/negative evidence pending. Keep new owner Active/Review only after effective claim,
and Complete only with existing implementation evidence. #717/#643/#624 remain open.

## Final Atomic Proposal — #730

This Active/Claimed record and ADR-091 amendment are proposed together in governance-only #730.
They become effective only upon reviewed exact-head CI/validator/CAS main merge. No implementation
branch/code/test/native/production/dependency change exists in this candidate. The maintainer's
22:39 acceptance authorizes this bounded change; no second confirmation is required. Independent
protected Agent contract review is mandatory and is not distinct natural-person approval.

### Local Governance Checkpoint

Pinned1.99 actual locked dependency metadata governance passed SQLite17/delivery12 with0warnings;
claims0warnings/text23/classifier14/diff passed. Independent protected nine-file working-tree
contract review APPROVES governance proposal, not effectiveness/implementation/native execution.
Authorization is the maintainer's22:39 session acceptance of proposal6217718821, not the older
proposal comment's content. Fresh exact-remote review, applicable CI and final CAS remain.
