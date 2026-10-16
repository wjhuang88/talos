# RUNTIME-010: Missing Pending-Sidecar Startup Recovery

> Document status: Complete

Completion Commit: 98fa04cbdfdb7b30ff4a32d06036427872a41703

**Type**: Technical / Runtime State Story
**Parent Epic**: None
**Selected Iteration**: I301

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Missing pending-sidecar recovery read fast path, storage/Actor regressions and startup documentation only; no existing-store durability or custody change. |
| Claimed At | 2026-10-12 |
| Source Issue | None |
| Governance Claim PR | #710 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer repeatedly authorized continuing the intermittent-timeout investigation and repair through GitHub CI. No separate maintainer reviewer is available; independent Agent storage/API review, exact-head CI, both validators and merge-time CAS remain required. |
| Implementation PR | #711 (merged as 9cad344b8f7adef855ca6b858e6ccc7723103767) |
| Last Updated | 2026-10-16 |
| Handoff / Release Condition | Bounded startup storage acceptance complete; #705 retains broader timeout investigation. No release. |

## Identity / Goal / Value

A new or legacy Runtime caller with no pending journal should reach command admission without
creating SQLite solely to discover an empty inbox. Windows diagnostic #705 measured successful
startup samples spending seconds in PRAGMA/schema work before Submit reception. This is a bounded
unnecessary-work repair, not proof of the cause of every historical watchdog failure.

## Scope

Under the store's existing guard, distinguish a genuinely absent pending-sidecar file using an
error-preserving filesystem query. Return empty recovery without directory/database/schema creation
only for absence. Retain the existing connection/schema/query path for an existing journal.
Affected code: talos-session pending_submission recovery and its tests; talos-agent startup tests
only as needed to prove command admission. No new dependency or public API is required.

## Exclusions

No connection cache, async ownership rewrite, schema migration, WAL/FULL change, recovery bypass for
existing journals, permission/path-policy change, retry or deadline increase, process hardening,
PowerShell repair, provider/Search/Git work, release or reopening #590/I290/I291.

## Dependencies / Decision Links And Constraints

ADR-008 bundled SQLite and ADR-056 custody are accepted. ADR-056 explicitly treats legacy Sessions
without a sidecar as an empty inbox. Preserve ADR-058 durable finalization and current native error
propagation. The claim merge is the implementation gate; #705 remains an open diagnostic owner.
Concurrent instances use existing SQLite serialization; a missing-file observation is a read snapshot,
not a guarantee that another instance cannot later create or admit work. Do not weaken acknowledgements.

## Acceptance

- Given an absent sidecar and absent parent directory, when recovery reads run, then they return no
  work and create neither directories nor database; normal submission still creates durable storage.
- Given an existing journal containing pending and Running submissions, when recovery runs, then
  identity/state/order, Running reconciliation and restart custody remain unchanged.
- Given an inaccessible/malformed path or unsupported schema, when recovery runs, then a real error
  remains visible; absence handling must not hide I/O or schema failures.
- Given concurrent store instances, when reads and admissions interleave, then acknowledged work is
  durable/recoverable and existing idempotency/custody guarantees hold.
- Original Auto flow and ordinary workspace tests retain assertions, 15s watchdog, 10s cleanup and
  tool deadlines. Compare exact repair-head Windows startup evidence with #705's preserved baseline;
  successful timing samples are not a universal latency guarantee or a closed PowerShell risk.

## Uncertainty And Validation Path

Review filesystem absence/race semantics before implementation. Reject the fast path if it cannot
preserve the existing snapshot and custody contract. Operation timings include scheduling; no
Defender/disk/OS root cause is inferred. Avoid fragile millisecond unit-test thresholds: assert
structural absence and real persisted recovery, then use bounded Windows CI for timing evidence.

## Planned Validation / Documentation

Focused locked talos-session and real Actor/Runtime regressions; repository-pinned Rust 1.99.0;
standard release_preflight.sh, both governance validators, exact-head Windows/Unix CI and independent
Agent storage/API review. No ignored assertions or retry-to-green. Update user-facing Runtime session
storage documentation in README.md / README.zh-CN.md and relevant runtime SDK contract, then owner,
iteration, backlog and Board in that order. No release is authorized.

## State/Status Owners / Required Reads

This Story owns acceptance; I301 owns execution/evidence; indexes and Board mirror them.
Read AGENTS.md, collaboration/iteration/testing SOPs, ADR-008/056/058, pending_submission.rs and
pending_submission/storage.rs, Agent session.rs/custody.rs and #705 checkpoint
https://github.com/wjhuang88/talos/pull/705#issuecomment-6121305365.

## Residual Destination

PowerShell low-CPU active-child latency and attribution of previous failures remain in #705 and
WINDOWS-AUTO-TIMEOUT-DIAGNOSIS.md. Existing-store initialization latency is outside this first slice.
No Complete state without an already-existing implementation SHA and required evidence.

## Atomic Claim Proposal — #710

The Active/Claimed record is a proposal only until finalized #710 merges into main.
No production implementation or implementation branch exists. Both validators, relevant exact-head
CI, independent Agent storage/API review and merge-time CAS are required before merge.

## Implementation Checkpoint — 2026-10-12

Claim #710 is effective at merge `42b861125d78aab0707528fc5fdec5e14ede0d19`.
Delivery remains Review / Claimed. Source, validation, failed attempts and residual evidence are
owned by [I301](../../iterations/I301-missing-pending-sidecar-startup.md); no Completion Commit exists.

Conservative acceptance boundary: paths containing `..` retain original directory/database
initialization. A missing intermediate directory can otherwise hide an already acknowledged journal
at the resolved target. Preserve that custody behavior instead of claiming the zero-creation fast
path for an ambiguous path. Final symlinks also retain original SQLite behavior; other missing paths
require a verified parent and preserve real I/O errors.

Local locked session/Agent tests passed 733 cases including 11 storage regressions and actual Actor
Submit. Full local preflight is blocked by the unchanged Unix-socket fixture; remote exact-head
full CI, explicit nonprivileged PermissionDenied evidence and independent storage/API review are
mandatory before merge. #705 still owns PowerShell latency and historical-failure attribution.


## Bounded Acceptance Closeout — 2026-10-16

Current delivery is Complete / Closed for this existing storage slice. Earlier proposal/Review/
pending paragraphs are retained execution history. Completion names the already-existing
implementation98fa04cbdfdb7b30ff4a32d06036427872a41703, not this status update.

#711 merged as9cad344b8f7adef855ca6b858e6ccc7723103767 after CI3161/run38283574722 exact98fa04cb,
attempt1/all6jobsSUCCESS, independent storage/API review6129237320 and CAS6129422889.
Completed Windows/macOS logs verify actual fresh Actor admission, absence/later admission,
existing Running/Pending/schema/error/concurrent-store and ParentDir regression cases. macOS
full release preflight includes the original Unix-socket fixture and both external SDK executable
modes; the permission gate explicitly observes PermissionDenied. Retained local failed full
preflight and CI3160's200pass/2fail Windows fixture attempt remain failures, not replaced results.

Acceptance mapping:
- Missing-parent recovery returns empty and creates no directory/database; later admission and
  reopened recovery preserve the receipt/submission. Actual Actor Submit reaches success without
  a recovery-created journal. Tests run against production store/Actor paths, not a substitute.
- Existing Pending/Running/state/order/restart custody and invalid-parent/schema/error/link cases
  retain their original SQLite behavior. Cross-instance acknowledged work remains recoverable.
  ParentDir and final symlinks keep the conservative route; absence is a snapshot, never a fence.
- The published post-merge Windows comparison is fulfilled by retained #705 CI3163 plus CI3206.
  CI3163 first-recovery60target samples median0.2307ms/max69.1622ms, but that matrix remainsRED
  2205pass/15fail. CI3182 remainsRED2219pass/1fail. Neither is relabeled by subsequent passes.
  CI3206 exactff84ec7 integrates actual #726 main and unchanged I301 recovery logic:120processes/
  2220pass0fail0ignored, attempt1/all8jobsSUCCESS, same original15s/10s/120s/5s budgets and assertions.
  Independent evidence checks cover exact sources, two ZIPs and12raw restoration receipts.
- Current60first recovery spans have median0.19485ms/max136.6005ms. All180recovery spans (three
  per target) median0.137ms/max140.7649ms are a separate population. These timers do not classify
  absence; structural regressions and source semantics establish the no-initialization boundary.
  Different production baselines/VM/probes prevent a controlled speedup or universal guarantee.

README.md, README.zh-CN.md and runtime SDK contract changes shipped in #711. SOP3a scope:
this is an SDK/library-only pending-store/Actor startup repair with no new CLI capability or
binary-entry acceptance claim. Real external SDK executables and Actor/Runtime execution establish
this layer; Auto tests are budget comparisons, not actual talos CLI walkthroughs. Parent Epic is
None; broader CLI/PowerShell timeout/integration behavior remains an open residual in #705.

I304 is separately Complete via #726/#727. No I301 scope absorbs its pause operation. Historical
15s/10s failures, evaluator5s phase uncertainty, native-child waiting, existing-journal costs and
prior Plugin uncertainty remain #705 residuals. #590/I290/I291 and other owner states are unchanged;
no release or whole-intermittent-timeout closure is claimed. I301 records full evidence/limits.


Fresh root reconciliation is recorded at #711 comment6211705932; independent completed-evidence
and local closeout review6211716953 APPROVE is retained in that PR conversation. Both governance validators
pass0warnings locally; published exact-head review/CI and merge CAS remain required before this
owner update reaches main. The existing#710 claim covers this owner/derived synchronization.
