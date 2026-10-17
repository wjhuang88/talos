# TEST-004: Known Windows Native Resource Cleanup Acceptance

**Status**: Review / Claimed; effective #730 merge c81782b6; candidate #716
**Type**: Technical Story / bounded test correction
**Parent Epic**: None; related TEST-003/#717 and SEARCH-001-C/#643
**Source Issue**: #729
**Iteration**: [I305](../../iterations/I305-windows-known-resource-acceptance.md)

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

## Goal And Evidence

Operators need a test that detects actual cleanup failures of enumerated Talos-owned successful-
return resources. CI3213/run38886975683 on #716 head0ca03ff1 FAILED Windows456/1 while the direct
Win32 control reproduced73/74/76/76 without PSS or Talos launcher bookkeeping. Modes1–7 succeeded;
original mode0/all40/+2 remained failed. This does not establish harmless caches or leak absence.
Maintainer accepted the narrower criterion, actual-operation evidence and real negative controls
in #717 comment6217718821 on2026-10-16 at22:39 Asia/Shanghai. A ledger alone is insufficient.

## Scope And Acceptance

- Retain all40 existing fault attempts and exact role/stage expectations, the original count trace,
  direct Win32 controls and every failed run. Only after this claim/amendment reaches main may
  +2 stop being a correctness oracle; it remains historical failed evidence/count diagnostic.
- Enumerate every native acquisition, transfer, close, termination and initialized-attribute delete
  site. Capture real successful outputs/statuses automatically at the integration boundary;
  failed-call outputs are never adopted. Use role+acquisition generation, not numeric handle alone.
- Verify each known returned resource is released exactly once, all required cleanup is attempted
  despite failures, and only successfully initialized attribute lists are deleted. Attribute
  storage has a distinct lifetime; void Delete proves invocation, not allocator reclamation.
- Enforce a reviewable native call-site inventory that fails on new/uncovered acquisitions or
  releases/bypasses. Independent protected source review must inspect the inventory's completeness.
- Real isolated negative controls omit an actual close, reject duplicate release BEFORE another
  native call, omit actual termination, and omit initialized-attribute deletion. Remaining ownership
  and backing storage stay retained until explicit recovery. Receipt-only mutation is supplemental.
- Termination uses existing WaitForSingleObject on an already-owned returned process handle:
  positive bounded wait must observe signalled exit before close; wait timeout/failure/unexpected
  status fails but cannot short-circuit later cleanup. Omitted termination uses timeout0 and must
  observe WAIT_TIMEOUT, then recover TerminateProcess/bounded wait/close. No PID lookup/OpenProcess.
- Negative fixtures remain under assigned kill-on-close outer Job; never resume an unassigned
  child. Preserve bounded output/deadlines, native-boundary panic containment and checked releases.
- Native positive/all40 and real negative sensitivity/cleanup evidence, pinned locked full checks,
  actual production/test feature graphs, exact-head protected review/full CI/CAS precede acceptance.

## Exclusions And Residual

No production behavior/API/permission change, dependency/version/lock, new observer API/privilege,
ObjectName/memory/otherPID probe, search activation, provider admission, #705 takeover or release.
The test establishes only enumerated Talos-owned successful-return cleanup and termination. It
cannot prove all-process/native leak absence, OS-private ownership/legal lifetime, mapped DLLs or
indefinite reclamation. TEST-003/#717 remains OPEN for unaccounted resources and future untracked
native acquisitions; #643/#624 remain open. Do not describe gate correction as proving +3 harmless.

## Dependencies / Documentation / Rollback

Effective amended ADR-091 and this claim must reach main before implementation; read ADR-068,
AGENTS, collaboration/start/change-control/testing SOPs and I302/#716 evidence. Runtime deliverable
is the Windows fault/negative fixture and bounded receipts; infrastructure-only exception, no user
feature claim. Update this owner/I305/ADR091, Board/index/matrix and Issue. Revert test-only acceptance
instrumentation if native proof fails; original historical failure remains available. No completion
commit exists. Governance-only draft, not implementation or effective activation.

## Final Atomic Proposal — #730

This Active/Claimed record and ADR-091 amendment are proposed together in governance-only #730.
They become effective only upon reviewed exact-head CI/validator/CAS main merge. No implementation
branch/code/test/native/production/dependency change exists in this candidate. The maintainer's
22:39 acceptance authorizes this bounded change; no second confirmation is required. Independent
protected Agent contract review is mandatory and is not distinct natural-person approval.

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
[site inventory and limits](../../reference/WINDOWS-KNOWN-RESOURCE-ACCEPTANCE.md).
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
