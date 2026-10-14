# Iteration I302: Windows Current-Process Handle Evidence

> Document status: Planned
> Published plan date: 2026-10-14
> Planned objective: qualify cold-worker resource growth with bounded self-only native metadata.
> MVP deliverable: reproducible operator-visible handle type/cleanup evidence or explicit uncertainty.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #717 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer accepted self-only diagnostic scope 2026-10-14 at14:50 Asia/Shanghai; finalized ADR/claim/CI/protected review/CAS required. |
| Implementation PR | Not started; later #716 follow-up after effective main refresh |
| Last Updated | 2026-10-14 |
| Handoff / Release Condition | No PSS code before this finalized claim/ADR reaches main; diagnostic completion does not close the original failure or permit #716 merge. |

## Published Baseline

Selected Story [TEST-003](../backlog/active/TEST-003-windows-owned-handle-evidence.md), Ready,
no parent. Preserve all scope/exclusions/acceptance/validation/rollback in that owner. ADR-091 is
the new native/unsafe contract; ADR-007/068 ownership remains. The final Active/Claimed/Accepted
proposal is ineffective until atomic governance merge. No native implementation is in this PR.

### Selection Inventory And Disposition

Selection main d66e380ad144329dbea49634122505a5a3075e95. All iteration headers inventoried:

| Owner | State | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain Search scope; #716 form and existing mechanical checks remain I293; I302 owns only new self-snapshot diagnostics. |
| I294 | Active / Claimed | Retain provider evidence; E/F gated, no takeover. |
| I300 | Active / Claimed | Retain independent Git scope; no transfer. |
| I301 | Review / Claimed | Retain published startup/storage acceptance and residuals. |
| I277 | Review / deferred device acceptance | Retain, not a diagnostic blocker. |
| I249 | Planned | Retain dependency-pilot deferral; no version upgrade. |
| I164 | Paused | Retain superseded layout disposition. |
| I162 | Terminal with historical Review wording | Retain publication disposition; do not reactivate from text matching. |

No Blocked iteration header found; other terminal owners remain terminal. DEPENDENCY-003-A retains
its independent native evidence. Open PRs #705/#716: #705 workflow/latency diagnostics remain
independent; #716 is the existing search/fixture candidate and may carry new I302 diagnostic
commits only after claim merge and refreshed base/head. Maintainer's explicit accepted adjacent
diagnostic authorizes this bounded selection without pausing or claiming another active slice.

### Dual-Owner Delivery Contract

No rule requires one implementation PR per Story. If #716 carries this deliverable, preserve
separate logical commits/work slices and synchronize both owners. I302 PSS work starts in a
worktree derived from the effective claim merge or later main; refresh #716 to include that main
before publishing PSS. Obtain a complete fresh protected review/CI of the combined head.
Diagnostic-native evidence is not I293 Search acceptance or original-defect repair.

### Risks / Rollback / Documentation

PSS can allocate/perturb observations; names are collected internally under the bounded authorized
flag but excluded from copied output. Numeric handles can be reused. Neither differences nor
stable totals prove object identity or lifetime. ABI strings require bounded copies under marker
ownership, checked releases and no double free. Retain an uninstrumented cold worker and original
failure after observation. Revert/remove helpers or explicitly review their retention. Operator
docs are ADR-091/this owner/TEST-003; no changed user feature or release docs.

## Actual Activation And Execution

2026-10-14: accepted proposal selected for governance preparation. Draft claim/ADR are not
effective; PR number backfill, validators, exact-head protected review/CI and CAS remain.
No PSS code, new dependency feature or unsafe has been implemented.

## Verification / Completion / Residuals

Governance checks pending. Implementation/native evidence Pending; no Completion Commit exists.
Keep I302 Active/Review as appropriate until diagnostic acceptance and existing evidence are
recorded. Keep #643/#624 open and original #716 failure unresolved until actual repair gates pass.
