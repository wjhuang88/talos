# RUNTIME-011: Absent-Journal Pause Without Initialization

> Document status: Active

**Type**: Technical / Runtime State Story
**Parent Epic**: None
**Selected Iteration**: I304

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
