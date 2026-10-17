# Iteration I305: Windows Known-Resource Acceptance

> Document status: Review
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

CI3216/run38918577883 on5631af7e FAILED remote Issue reconciliation: the new #729 row
contained two owner links where the actual parser requires exactly one. Corrected to TEST-004
as sole owner; I305 remains linked by owner docs. Actual parser now recognizes729 and validates
its source/owner identity. Added the required exact Status reconciliation comment; no validator
weakening, scope change or unrelated reconciliation batch. Prior failure remains recorded.
Fresh exact-head checks/review/CI precede merge; still no implementation.

## Effective Claim And Local Implementation Checkpoint

#730 merged c81782b6d922ec1b8bfa62e9c4a2914d746680bb after exact-head CI3217,
protected Agent review6218119088 and final CAS6218178349. Earlier proposal statements are historical.
Implementation branch was created afterward from that main; separate local baseline merge713bae1d
incorporates old#7160ca03ff1 and preserves both ADR appendices, diagnostics, residual and I301/I304
closed evidence. No production behavior/dependency/lock change. TEST004/I305 are Review /
Claimed; native evidence and fresh exact candidate review/CI are pending, no Completion Commit.

Current test-only candidate wraps the original real cleanup calls with fixed role/generation
observations, successful-output ownership, actual release BOOLs and owned-process termination/wait.
All40 exact faults/count trace remain; mode8 exercises actual normal-operation omissions and guarded
duplicate request, with in-frame explicit recovery before assertions. Full-source inventory and
mutation checks fail on uncovered sites/bypasses. See
[site inventory and limits](../reference/WINDOWS-KNOWN-RESOURCE-ACCEPTANCE.md).
Pinned locked checks/protected source review are running; no native success is claimed.

### Local Review Corrections And Limits

Independent working-source review REQUEST CHANGES on two gaps: omission detectors originally
used injection bookkeeping rather than final acceptance's predicate; termination lacked an
exact-once guard. Both corrected locally. SAME require_release/require_termination predicates
now reject the omitted operation before recovery and run in finish; attempted state is consumed
before termination and repeated requests cannot call native code again. Supplemental failed-
status/timeout/unexpected-wait/no-retry tests support, never replace, real Windows mode8 evidence.
The source-inventory edit's stale array block was caught/fixed before final source review; normalized
CRLF fixture avoids platform checkout noise. Two Windows Clippy style findings were fixed without
waivers. Final Windows coding --tests Clippy-Dwarnings passes. Earlier missing-MinGW check failure
was environment setup; official signed-index package SHA256s verified and compiler restored locally.
Independent corrected-source review approves publication with no blockers; exact final candidate
review and native CI remain required. Historical/local intermediate failures are not native proof.

### Final Local Publication Validation

Final-source pinned1.99 locked release_preflight PASSED:3729primary+1existing exact-selection child,
0failed/ignored, workspace check/Clippy and both external SDK modes. The120suite summaries include
461filtered siblings:1previously authorized local Unix socket filter plus460siblings from the
existing exact-selected child. External wrapper changes only the exact workspace test invocation;
repository scripts/CI remain unfiltered. Final-source Windows coding --tests Clippy(-Dwarnings)
PASSED; earlier --tests cross-check passed and final Clippy compiles that same target. No native
Windows execution is implied. Actual locked Windows tools/coding normal+build graph excludes PSS,
while test/dev includes it. Governance SQLite17/delivery12/claims0warnings, text23/classifier14,
fmt/diff/scale pass. Production launcher/search behavior and manifest/lock/workflow unchanged from prior#716;
I301/I303/I304 owner files match effective main byte-for-byte.

Staged source/site-inventory/owner/mirror diff reviewed. Independent corrected-source review has
no blockers; final exact-commit and remote fingerprint review plus unfiltered native CI remain.
Current Review/Claimed state is candidate truth, not Complete or merge authority. Original CI3213
and other failed receipts remain; #717/#643/#624 stay open. Native receipts expose the all40
232resource/208close/24Delete/16termination+exit proof and4real-negative detection/recovery only
if every actual check passes. No native success or process-wide leak absence is claimed here.

### CI3219 And CRLF Fixture Correction — 2026-10-17

Exact head6bd0ddfb/treec1cd718b CI3219/run38922821235 FAILED Windows462passed/1failed,
0ignored/filtered; other5jobs succeeded. ONLY checkout_crlf_preserves_reviewed_native_inventory
failed. Actual native resource outer test (all40 plus4negative assertions), both audit-unit tests,
real inventory and mutations passed. Successful output is harness-captured: no raw numeric resource
receipt/mode8JSON appears in this job log; success is evidence from passing reviewed assertions.
Original +3 attribution remains outside the known-resource proof; #717/#643/#624 stay OPEN.

Fixture bug: replacing LF with CRLF on already-CRLF input creates CR-CR-LF. Canonicalize input toLF
first, then validate both LF/CRLF synthetic variants. Fingerprint normalization, EXPECTED and all4
native source files remain unchanged. Pinned1.99 standalone std-only inventory suite reproduces
oldCRLF 2pass/1expectedfailure; final corrected LF and CRLF checkouts each pass3/0, including real
inventory/mutation guards. Formatting/diff pass. Workspace maintenance removed old local build
cache/toolchain; minimal pinned compiler/std/formatter restored with official SHA256 verification.
These focused receipts are not a new full-workspace preflight or native Windows pass. Previous
fullpreflight receipts remain historical; fresh exact-head full unfiltered CI/review required.
Review/Claimed remains, no Complete or automatic716merge. Preserve CI3219 failure, no retry-to-green.
