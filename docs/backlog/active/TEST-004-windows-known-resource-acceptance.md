# TEST-004: Known Windows Native Resource Cleanup Acceptance

**Status**: Ready / Unclaimed; atomic governance preparation
**Type**: Technical Story / bounded test correction
**Parent Epic**: None; related TEST-003/#717 and SEARCH-001-C/#643
**Source Issue**: #729
**Iteration**: [I305](../../iterations/I305-windows-known-resource-acceptance.md)

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | TEST-004 known successful-return Windows resource cleanup acceptance, native call-site coverage and real negative controls; no production launcher/search behavior or I302 PSS attribution. |
| Claimed At | 2026-10-16 |
| Source Issue | #729 |
| Governance Claim PR | Pending |
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
