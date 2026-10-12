# Iteration I301: Missing Pending-Sidecar Startup Recovery

> Document status: Review
> Published plan date: 2026-10-12
> Planned objective: eliminate database initialization during recovery of an absent pending sidecar.
> MVP deliverable: new/legacy Runtime admits commands without a recovery-only SQLite side effect.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Missing pending-sidecar recovery read fast path, storage/Actor regressions and startup documentation only; no existing-store durability or custody change. |
| Claimed At | 2026-10-12 |
| Source Issue | None |
| Governance Claim PR | #710 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer repeatedly authorized continuing the intermittent-timeout investigation and repair through GitHub CI. No separate maintainer reviewer is available; independent Agent storage/API review, exact-head CI, both validators and merge-time CAS remain required. |
| Implementation PR | Not opened (local candidate) |
| Last Updated | 2026-10-12 |
| Handoff / Release Condition | No implementation before this atomic claim and activation merges into main; release only after evidence-bearing closeout or explicit handoff. |

## Published Baseline

Selected Story: [RUNTIME-010](../backlog/active/RUNTIME-010-missing-pending-sidecar-startup.md),
Ready; no parent. Preserve all scope, exclusions, Given/When/Then acceptance, validation and
documentation targets in that owner. Accepted ADR-008/056/058 and this claim merge are dependencies.
The final Active/Claimed proposal becomes effective only when merged into main.

### Selection Inventory And Dispositions

Selection baseline: main `f036111f6e969feea47ad784f3cb3effd7e20a33` (includes #708).
All iteration document headers were inventoried on 2026-10-12:

| Owner | Current state | Disposition |
|---|---|---|
| I293 | Active / Claimed | Retain other-session Search router scope; no transfer or overlap. |
| I294 | Active / Claimed | Retain other-session Search provider evidence; no transfer or overlap. |
| I300 | Active / Claimed (#704 merged) | Retain T7 Git permission/diff repair; no overlap in behavior or implementation files. |
| I277 | Review | Retain deferred human/device acceptance. |
| I249 | Planned | Retain dependency-pilot deferral; no dependency version change here. |
| I164 | Paused | Retain superseded layout target; no reactivation. |
| DEPENDENCY-003-A | Review / Claimed | Retain separately owned native dependency evidence and platform gates. |

No Blocked iteration header was found. I290/I291/#590 and I296-I299 remain terminal.
Existing explicit non-overlap authorization plus the maintainer's continued direction governs this
independent Runtime repair; this record does not activate another owner's work.
Only open PR found at selection was #705, whose bounded diagnostic exception excludes production
storage changes. RUNTIME-008/009 and SESSION-011 are unclaimed future semantic/API work; this repair
preserves their contracts and does not claim them. Recheck inventory/overlap before merge.

### Risks And Rollback

Filesystem absence must preserve real I/O errors; concurrent creation cannot turn a read snapshot
into custody authority. Existing journal schema validation and WAL/FULL remain unchanged. Revert only
this repair if regression evidence shows loss, hidden error or altered Actor recovery; retain all
historical failing CI evidence. Windows runner timing is variable and cannot prove universal latency.

## Actual Activation And Execution

2026-10-12: governance-only draft prepared; no production/test/dependency edit or implementation
branch. Official Rust 1.99.0 toolchain is available locally. Final PR number, exact-head checks,
independent review and merge-time CAS must be recorded before claim becomes effective.

## Verification Evidence / Completion Evidence

Pending governance checks and implementation. No Completion Commit exists; not Complete.

## Variance And Residuals

#705 retains PowerShell latency, old-failure attribution and existing-store startup cost. This
iteration does not claim the entire intermittent-timeout investigation is closed.

## Atomic Claim Proposal — #710

The Active/Claimed record is a proposal only until finalized #710 merges into main.
No production implementation or implementation branch exists. Both validators, relevant exact-head
CI, independent Agent storage/API review and merge-time CAS are required before merge.

## Implementation Checkpoint — 2026-10-12

Claim #710 merged as `42b861125d78aab0707528fc5fdec5e14ede0d19` after CI3156,
independent Agent review6127973689 and merge-time CAS6128063256. Implementation branch starts
from that main commit, including other-session #709. Claim is now effective; proposal paragraphs
above are preserved publication history. Delivery remains Review, not Complete.

The candidate adds an error-preserving absence check under the existing store guard. An existing
final directory entry takes the original SQLite route. Absent entries require a verified existing
parent directory; dangling ancestors, loops and permission failures remain errors. A negative read
is a snapshot only; normal durable admissions and subsequent acknowledged-work recovery are unchanged.

Conservative path exception: any `..` component with an unavailable final entry takes the original
SQLite/directory-initialization route. A real regression proved that `missing/../session.tlog`
otherwise returns zero records despite an acknowledged journal at the canonical target (0 vs 1).
Creating the missing intermediate directory can reveal that existing journal. Preserve this legacy
path behavior rather than normalize paths or claim zero side effects for this ambiguous case.

Validation so far: 231 session tests passed before the final ParentDir regressions; one exact real
Actor Submit/Success/no-sidecar test passed. Final storage, full preflight and exact-head remote
Windows/Unix CI are pending. Initial Actor fixture type errors were corrected locally before
publication. Default-debug baseline full preflight ran out of the local 32GB disk during test
linking; only owned generated cache was cleared, interrupted test writes restored from Git, and
local builds now use environment-only debug=0/incremental=0/jobs=2. No dependency/lock/pin/assertion
or remote CI policy change. Nonprivileged `runuser` verification was blocked by unavailable group
switch capability; the local permission fixture exercised privileged NotFound, not PermissionDenied.
Unix CI must execute the nonprivileged error branch. Full baseline/preflight failures and the
pre-correction ParentDir test failure are retained, never replaced with retry-to-green evidence.

Source/test/doc review and final checks remain required. #705 owns PowerShell active-child latency
and attribution of earlier failed samples; no overall timeout-risk closure or release is claimed.

Final storage-path validation checkpoint: the corrected ParentDir candidate passed all 233
session unit/integration tests (206 unit + 27 integration), zero failures/ignored. Both prior
ParentDir failure runs remain retained. The existing macOS full-validation CI chain now repeats
only the exact permission fixture with ErrorKind-only output and requires one passed test plus
PermissionDenied. No new workflow/job/permission/deadline or dependency is introduced; this is
associated storage-regression validation, not production policy. Final full preflight and remote
implementation review/CI remain pending.

Local environment boundary: full preflight reached workspace tests but failed the unchanged
`talos-runtime::tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries` at its Unix
socket fixture (`Operation not permitted`). This sandbox does not permit Unix socket creation.
No assertion was removed/ignored and no success is claimed for that run. Final locked format,
workspace check/Clippy/governance and affected-crate tests are rerun after source convergence;
remote exact-head full CI must execute every workspace test, including this fixture, before merge.

Stable source evidence: `cargo test --locked -p talos-session -p talos-agent` passed 733
unit/integration/doctests, zero failed/ignored, including all 11 new storage regressions and the
real Actor case. Standard stable-source preflight passed fmt, locked workspace check/Clippy,
project/claim validators and CI-classifier tests; workspace tests again failed solely at the
unchanged Unix-socket fixture above (64 other runtime unit tests passed). It is still a failed
full preflight, with no skipped test or local waiver. Remote exact-head full workspace CI is the
required environment-capable gate. Runtime SDK external fixture validation is in progress.

Runtime SDK fixture completed: `python3 scripts/validate_runtime_sdk_fixture.py` passed both
independently locked consumer modes (default and coding), with one direct Talos dependency.
Root and fixture lockfiles/manifests remain unchanged. Final owner/derived diffs were staged and
reviewed; project and collaboration validators passed with zero warnings after Review synchronization.
Candidate publication is for exact-head CI/review, not a local full-preflight success claim.
