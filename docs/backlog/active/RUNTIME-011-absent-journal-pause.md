# RUNTIME-011: Absent-Journal Pause Without Initialization

> Document status: Complete

Completion Commit: aa1c477f1a457dc44ad039cc72bf9ed19842bf83

**Type**: Technical / Runtime State Story
**Parent Epic**: None
**Selected Iteration**: I304

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Absent-journal pause no-op under the existing pending-store guard; storage and real Actor/Runtime shutdown regressions and documentation only. Existing-journal durability, admission authority and shutdown policy remain unchanged. |
| Claimed At | 2026-10-15 |
| Source Issue | None |
| Governance Claim PR | #722 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized continuing the intermittent-timeout investigation and grounded repair using GitHub CI. No distinct maintainer reviewer is available; independent Agent storage/custody/API review, exact-head CI, both validators and merge-time CAS are mandatory. |
| Implementation PR | #726 (merged as c869a9afc2871d82c7fe0a18d1bfddf903831b59) |
| Last Updated | 2026-10-16 |
| Handoff / Release Condition | Bounded absence/pause acceptance complete at #726; #705 retains independent timeout investigation. No release or broader latency guarantee. |

## Identity / Goal / Value

Active/Claimed is a finalized atomic proposal in #722, ineffective until merged into main.
No implementation branch/source/test change precedes that gate.

Runtime callers shutting down or interrupting a legacy/new session with no pending journal should
not initialize SQLite solely to pause zero durable submissions. CI3182's successful original-Auto
sample directly measures pause1.5210834s inside its caller1.5214184s and shutdown1.5225249s.
This is a bounded unnecessary-work optimization, not a complete explanation of past watchdogs.

## Scope

Under the existing pending-store guard, permit `pause_unstarted` to return zero for a verified
absent journal without creating directories/database/schema. Reuse the error-preserving absence
classification introduced by #711 only if review and regressions prove its applicability to this
write operation. Existing or ambiguous paths retain the original transaction/schema/UPDATE/commit
and drop/error behavior. Tests cover storage plus actual Actor and Runtime shutdown receipts.

## Exclusions

No admission or reconciliation-policy change, permanent cross-instance admission fence, cache,
connection ownership rewrite, async storage, schema migration, SQL/WAL/FULL change, cancellation
rewrite, dependency/lock/toolchain change, deadline increase, skipped assertions, retry-to-green,
PowerShell/process-hardening change, PSS instrumentation, Search/Git/Skill work, release or #590
reopening. I301 owns recovery reads only; #705 remains diagnostic, I302 owns its self-only evidence.

## Dependencies / Decision Links And Constraints

#711 merge `9cad344b8f7adef855ca6b858e6ccc7723103767` is on main. Preserve ADR-008,
ADR-056 transactional custody, ADR-058 finalization and ADR-063 shutdown authority. No public API or
new native operation is planned; a discovered contract change blocks implementation pending an ADR.
The candidate must keep the guard before checking absence. Absence is an operation snapshot, never
a permanent fence against a later store instance admitting work. Runtime admission closing and
durable receipt authority remain with their original owners; returning zero must not hide an
already acknowledged record or turn failed reconciliation into successful shutdown.

## Acceptance

- Given a genuinely absent journal and missing parent, when pause runs, then zero records change
  and no directories or SQLite files are created; later normal admission still persists and recovers.
- Given accepted_pending, paused_pending and Running records in an existing journal, when pause
  runs, then only accepted_pending transitions, exact changed count and transaction durability hold,
  and restart preserves identities, receipts, payloads, state and order.
- Given concurrent stores, when absence/pause and durable admission interleave, then no acknowledged
  submission disappears: prior admission still in accepted_pending is paused, later admission
  remains recoverable; Running is never relabeled by pause. Exercise
  the Runtime closing/admission barrier as well as the store snapshot; no global fence is claimed.
- Given inaccessible/invalid parents, malformed journals or unsupported schema, when pause runs,
  then original observable errors remain. Final symlinks, dangling ancestors and `..` ambiguity
  must retain conservative #711 behavior, including the missing-intermediate-directory case that
  can reveal an existing acknowledged journal.
- Given a real Actor/Runtime with no journal, when it processes work and shuts down, then original
  terminal/reconciliation receipts remain correct and the journal is still absent AFTER shutdown.
  Nonempty pending and finalization failure cases retain custody and truthful failed outcomes.
- Original Auto15s, cleanup10s, tool120s, evaluator fixture5s and all assertions remain. Compare
  exact repair-head Windows evidence with preserved CI3182; no fragile latency threshold or
  universal timeout guarantee substitutes for structural and durability acceptance.

## Validation / User-Facing Documentation

Pinned Rust1.99.0 and `--locked`; affected session/Agent/Runtime tests; standard
`scripts/release_preflight.sh`; both governance validators; exact-head Windows/macOS full CI;
independent storage/custody/API review of success-return, races, errors, links and unchanged WAL.
Local Unix-socket restrictions, if encountered, remain failed local preflight evidence, with no
waiver or ignored case; capable remote CI must execute the unchanged full suite.
Update README.md, README.zh-CN.md and docs/reference/RUNTIME-SDK-CONTRACT.md to describe the exact
no-journal boundary without promising a cross-process fence or universal latency.

## Required Reads / State Owners

AGENTS.md; collaboration, change-control, iteration and testing SOPs; ADR-008/056/058/063;
pending_submission.rs/storage.rs/tests; Agent session.rs/custody.rs/lifecycle.rs; Runtime shutdown
and admission tests; I301 published scope; #705 evidence comment6184082259. This Story owns
acceptance, I304 execution; indexes/Board are derived only.

## Uncertainty / Residual Destination

Reject or narrow the optimization if returning zero cannot preserve custody/closing semantics.
CI3182's sole baseline failure is a 5s composite evaluator fixture without exact phase attribution;
successful pause spans do not prove that failure or historical10s failures have the same cause.
Those failures, native active-child latency and existing-journal costs remain #705 residuals.
No Completion Commit exists; this is not Complete.

## Implementation Checkpoint — 2026-10-15

Claim #722 became effective at main merge `23e659e2403739b74bc018ec46305346e8e2c7ad`.
The implementation candidate starts from later main `0ef5a532930642d730bcc6d974f2a79605efaaee`,
which adds only #725's separately owned Provider intake. The original proposal wording above is
historical. Delivery is Review / Claimed; the execution and validation record is in I304.
Guard-before-absence, storage regressions and actual Actor/Runtime shutdown tests implement this
slice. Independent exact-head custody/API review, full native CI and post-repair Windows comparison
with preserved CI3182 remain gates. No Completion Commit or timeout-closure claim exists.


## Bounded Acceptance Closeout — 2026-10-16

Current delivery is Complete / Closed. Earlier proposal/pending/Review paragraphs are preserved
history and are superseded only for this bounded pause slice by this closeout.

Implementation #726 exact aa1c477f1a457dc44ad039cc72bf9ed19842bf83 merged as
c869a9afc2871d82c7fe0a18d1bfddf903831b59 after CI3203/run38755790909 all six jobs SUCCESS,
attempt1, independent exact-head storage/custody/API review6195119208 and merge CAS6205402481.
The merge tree1a239eee60205d7a421472b4c66b5b6e8b21d4de equals the reviewed implementation tree.
Windows and macOS executed unchanged full suites; macOS full release preflight passed. The
original local2195pass/1fail Unix-socket EPERM and earlier Plugin admission failure remain
retained in I304; remote success does not explain the Plugin failure or waive local failure.

Acceptance evidence maps to the published requirements:
- Verified absence under the existing guard returns0 with no parent/database/schema creation;
  real later admission/reopen remains durable. Existing Accepted/Paused/Running data, counts,
  receipts/payloads/order and unchanged WAL/FULL/immediate transaction behavior are covered.
- Independent-store admission before/after pause, Runtime closing/start-fence tests, conservative
  ParentDir/link/error/schema behavior and real PermissionDenied keep their original authority.
- Actual Actor absence is checked after shutdown/task join; Runtime empty shutdown preserves
  receipts, post-close RuntimeClosing and no runtime storage. Nonempty and failed reconciliation
  assertions remain unchanged. Session241tests, both SDK executable modes and native CI passed.
- Post-repair #705 exact ff84ec7064edb54ace27ba7c2700f028e5dfaef6 integrates actual #726 main
  with only three temporary diagnostic files; its six production Rust sources match this merge.
  CI3206/run38832452061 all eight jobs SUCCESS attempt1, with both independent fresh-first
  Windows phases60processes/1110pass0fail0ignored each (2220total). All original deadlines and
  assertions remain, no warmup/ignored case/retry-to-green. Both65-file ZIPs are independently
  hashed; all12original/restored raw byte receipts match. Evidence comment6209342583 records
  IDs, SHA256, source/manifest validation and comparison limits; I304 records the full checkpoint.

All60matched original-Auto pause paths classified absence without connection/commit stages.
Pause median0.2475ms/max0.351ms, caller median0.33075ms/max0.527ms and shutdown
median1.07815ms/max1.726ms support the structural optimization against retained CI3182's
successful pause1.5210834s/caller1.5214184s/shutdown1.5225249s. Different production baselines,
runner image/VM and probe overhead prevent a controlled causal speedup or universal guarantee.

README.md, README.zh-CN.md and RUNTIME-SDK-CONTRACT.md were updated in #726. No new API,
persistent-state semantics, custody, permission, dependency or release change is part of this
closeout. #590/I290/I291 stay terminal; I301 stays Review under its own acceptance. Historical
CI3182 evaluator composite5s failure, old10s/15s failures, native active-child latency,
existing-journal costs and prior Plugin uncertainty remain #705 residuals. This completion
neither closes the intermittent-timeout investigation nor authorizes merging temporary #705.


Independent completed-evidence/local closeout review: #705 comment6209379440. Final closeout-head
review/CI, both governance validators and merge CAS remain required before this owner update
reaches main; the existing bounded claim authorizes this owner/derived-view synchronization.
