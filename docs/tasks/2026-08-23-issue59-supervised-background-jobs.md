# Issue #59 Supervised Background Command Jobs Long Task

> Status: Active long task; I222/B, I224/C, I225/D1-A, I226/D1-B and I228/D2 Complete / Closed; I223 remains Planned / Unclaimed.

## Startup Contract

Outcome: close Issue #59 only after TOOL-024-B, C, Windows D1, projection/acceptance D2 and I223
deferred-validation cleanup reach truthful terminal states with pre-existing completion evidence.

In scope: the Accepted ADR-060 ordered chain, child-specific governance, implementation, security
review, exact-head CI, merge-time CAS, user/API documentation, Issue #378 evidence and final
owner-first closeout.

Out of scope: persistent/restart-surviving jobs, scheduling/retry/remote workers, PTY/stdin, shell
syntax inference, `/auto`, Dashboard/I213 authority, PERM-006-D/E implementation, release/version/
tag/publication, Desktop and unrelated backlog work.

Work mode: `Deferred Human Validation`.

Deferred validation tracker: [Issue #378](https://github.com/wjhuang88/talos/issues/378) and planned
evidence-only cleanup iteration I223.

Dependencies and prerequisites: TOOL-024-A/I188, TOOL-023-C, RUNTIME-005 and PERM-006-C/I221 are
Complete; ADR-060 is Accepted. Each later child depends on its predecessor and receives a separate
owner, iteration, effective claim, implementation PR and review. Windows D1 additionally requires
an Accepted Job Object/OS-ABI decision before unsafe implementation.

Artifacts and state owners: child Story and iteration files are authoritative; this task owns
program order; TOOL-024 owns the Epic; Issues #59/#378 own discussion and deferred evidence; Board,
Backlog, iteration index and manifest are derived.

Validation and acceptance evidence: child-focused and workspace locked checks, release preflight,
actual subprocess/runtime tests, exact-head CI, mandatory independent process/permission/unsafe/API
security review, merge-time CAS and Issue #378 rows.

Branch, worktree and checkpoint plan: each child starts from its effective claim merge or later
main in a dedicated `/private/tmp/talos-iNNN-*` worktree; converge locally, push one stable candidate,
close owner-first after implementation merge, then create the next child claim. Never reuse B/C/D
implementation branches or exact-head evidence.

Allowed permissions and external actions: edit/test/commit/push governed repository work, create
child claim/implementation/closeout PRs, post Issue evidence and merge only after applicable exact-
head approval and CAS. No release, publication, deployment or credential changes.

Destructive or irreversible operations: process tests may terminate only their own explicit child
process groups/Job Objects and verified temporary paths. No force-push, tag movement or broad
cleanup. Remote branch deletion occurs only after merge and ancestry verification.

Time, cost and resource limits: repository CI and local toolchain only; no paid external service.
Background tests use bounded jobs/output/deadlines and leave no process or build residue.

Failure, retry and fallback policy: fail closed on permission, platform ownership, process cleanup,
deadline, channel or state uncertainty. Batch corrections locally and push a new stable head. After
three failed design approaches, record the blocker and stop that child; never weaken ADR-060.

Default decisions for foreseeable ambiguity: preserve foreground behavior; choose no spawn over
best-effort cleanup; preserve historical checkpoints; owner-first union updates; no automatic model
continuation; unknown output/state is explicit rather than silently dropped. B supports only
non-daemonizing commands whose descendants remain in Talos's Unix process group. Terminal records
use ADR-060's 32-entry oldest-first cap and session/process-end disposal, with no wall-clock TTL.

Residual-work destination: TASK-001 for durable jobs; PERM-006-D/E for typed/conformance permission
work; new corrective owners for failed Issue #378 rows; release and Desktop owners remain separate.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-23 |
| Work Slice | Coordinate the complete ordered Issue #59 closure chain while granting production authority only through each child owner. This claim activates only I222/TOOL-024-B's recorded Unix core; C, Windows D1 decision/implementation, D2 and I223 require their own later gates. |
| Claimed At | 2026-08-23 |
| Source Issue | #59 |
| Governance Claim PR | #379 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer active goal selects complete Issue #59 delivery; bounded I213/I222-B parallel authorization is recorded in Issue #366 comment `5386904546`. Claim PR #379 exact head `5f0816aa` passed CI `32650593056`, independent Agent-role claim review `5386970071` and CAS `5386973729`, then merged as `48e8ae9b`; every implementation child still requires its own exact-head protected-scope review and CAS. |
| Implementation PR | None; child-specific PRs only |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | Coordination claim grants no production authority; every child requires its own effective claim. |

## Ordered Task Items

| ID | Expected output | Completion gate | Depends on | Fallback |
|---|---|---|---|---|
| G0 | Effective task coordination plus TOOL-024-B/I222 claim and activation | Governance-only PR on main; validators/CI/review/CAS | Current main inventory | Leave all work Unclaimed |
| B | Unix Agent/session supervisor core | Implementation merge plus exact-head security/CI/CAS; V59-B1 queued | G0 | Fail closed; no background spawn |
| B-close | I222/TOOL-024-B owner-first closeout | Pre-existing implementation merge `8671edf4`, exact-head evidence and review | B | Keep B Review only if implementation evidence or protected gates fail |
| C-claim | Separate TOOL-024-C owner/iteration/effective claim | Governance-only merge | B machine/security merge disposition | Keep C Blocked |
| C | Model-readable session-scoped `process` tool | Implementation merge plus exact-head security/CI/CAS; V59-C1 queued | C-claim | Keep B receipts only; no model control |
| D1-decision | Accepted Windows Job Object/OS-ABI ADR under separate owner | Decision PR CI/security review/CAS | C | Windows stays fail closed |
| D1-impl | Assigned-before-exec Job Object ownership and whole-tree cleanup | Implementation merge, Windows CI/security/CAS; V59-D1 queued | D1 decision | Windows stays fail closed |
| D2 | CLI/TUI projection, docs and integrated platform behavior | Implementation merge, CI/security/CAS; V59-D2 queued | C and D1 | Shared APIs remain usable without projection |
| VALIDATE | I223 resolves V59-B1/C1/D1/D2/FINAL | All Issue #378 rows terminal; failures separately owned | B/C/D merged | Keep source owners Review and #59 open |
| CLOSE | Owner-first TOOL-024 and Issue #59 closeout | All children Complete with pre-existing SHAs; validators/review/CAS | VALIDATE | Report Partial with exact residual owner |

## Current Coordination And Overlap Boundary

Maintainer authorization `5386904546` permits only the exact I213/I222-B pair to run concurrently.
I213 retains its existing 17-file Dashboard/CLI/README/owner inventory; B is restricted to
core/agent/tools/runtime plus one narrowly amended `talos-permission` matcher/test seam for the
reserved `background:` Command namespace, and excludes every I213 production file. PR #379 remains
the sole original pairwise-contract source; the permission addition is ineffective until its
change-control PR reaches `main`. Shared derived governance files use union semantics. Before stable
push and merge, compare exact inventories; same production file or authority overlap pauses only the
overlapping work. C/D2 must recompute overlap and cannot reuse this authorization.

## Checkpoint

The following baseline checkpoint records the pre-I224-merge state and is retained for recovery
history; the current state is in the dated checkpoints appended below.

Completed task items: dependency chain through PERM-006-C and validation tracker #378 creation.

Current state and artifacts: PR #379 activated TOOL-024-B/I222, implementation PR #382 merged as
`8671edf45c168612bfa4a4bbb65a9847026e1b96`, and closeout PR #384 merged as `faf7c0e8`.
I222/TOOL-024-B are Complete/Closed through owner-first closeout. I224/TOOL-024-C is now a
governance-only Planned/Unclaimed candidate; its claim is ineffective until target-branch merge.
I223 remains Planned/Unclaimed and D requires a separate claim/decision.

Commands/checks and actual results: repository/worktree/open-PR and nonterminal iteration inventory
completed; governance validation pending finalized Draft claim.

Open risks or deviations: Unix process-group unsafe boundary and Runtime finalizer/API seam require
independent review; self-daemonizing commands are unsupported because a child can escape a Unix
process group; I213/I222-B inventory overlap must remain zero under `5386904546`; Windows remains
fail closed.

Next task item: run final exact-head validators/CI and independent process/permission/API review,
then merge-time CAS and #379 merge before implementation.

Recovery or resume instruction: use this task, TOOL-024-B and I222 owners; implementation starts
from the eventual claim merge commit or later main, never this pre-merge governance branch.

## Claim Activation Checkpoint (2026-08-24)

PR #379 exact head `5f0816aa` passed docs-route CI `32650593056`, independent Agent-role claim
review `5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. This supersedes
the pre-merge recovery instruction above: G0 is complete, I222/TOOL-024-B is Active/Claimed, and B
implementation may start only from `48e8ae9b` or later `main`. The I213/I222-B pairwise boundary in
`5386904546` remains mandatory; C/D and I223 still require their own ordered gates.

## Permission Namespace Change-Control Checkpoint (2026-08-24)

Code inspection proved that current resource-less Execute Allow rules match the proposed
`background:` Command facet, so distinct facet text alone does not enforce ADR-060. B implementation
is paused before permission production edits. The minimum accepted correction is limited to the
reserved background namespace, preserves explicit Deny and exact background authorization, changes
no public schema or other permission behavior, and requires focused tests plus independent
permission/security/API review. The local uncommitted core/agent sketch is not completion evidence.

## Local Convergence Checkpoint (2026-08-24)

I222/TOOL-024-B is locally converged from `main@7fd813e8` after the #381 permission amendment.
The owner-first records were `Review / Claimed` before PR #382; that candidate is now merged and
the closeout records the pre-existing implementation merge as Completion Commit.
Focused locked checks passed for permission (137 tests), Agent (281 tests plus integration
fixtures), Runtime (38 tests), and Unix process-boundary launcher tests. `cargo fmt --all` and the
focused locked check also passed. The candidate changed-file inventory is recorded in I222 and
contains only core/agent/tools/runtime, Cargo lock/dependency metadata and the narrowly amended
permission matcher/tests; it contains no CLI, Dashboard, README, process-tool, Windows,
persistence, `/auto` or release authority.

## I222 Completion Checkpoint (2026-08-24)

PR #382 implementation merged into latest `main` as `8671edf45c168612bfa4a4bbb65a9847026e1b96`.
Exact head `01aa8b6a` passed CI `32690533253` 5/5; independent process/permission/unsafe/API
review and final exact-head governance review approved. B is complete; C-claim is the next
ordered item, while Windows D1/D2 and I223 remain gated and Issue #59 stays open.

Next task item: run full local locked workspace/preflight, both governance validators with explicit
`origin/main`, YAML/diff/EOF/secret/generated-residual audit, then create one stable implementation
candidate. First push still requires fresh exact-head CI, independent process/permission/unsafe/API
review and merge-time CAS. I213 remains independently Review/Claimed and its local duplicate
checkpoint was withdrawn; PR #379 is the sole original pairwise contract source.

## Post-merge Closeout Supersession (2026-08-24)

The preceding local-convergence "next task" instruction is historical and is superseded by the
merged implementation and owner-first closeout above. Do not create another I222 implementation
candidate or repeat its CI/review. The next ordered governance action is a separately claimed
TOOL-024-C owner/iteration; Windows D1/D2 and I223 remain independent gates, and Issue #59 stays
open until all required children reach their own evidence-bearing terminal states.

## I224 Claim Preparation Checkpoint (2026-08-24)

I224/TOOL-024-C owner and iteration were prepared from `main@faf7c0e8` as a governance-only
candidate, with claim PR #385. The slice is limited to the model-visible session-scoped `process`
read/status/list/cancel contract over I222; it excludes Windows Job Object/D1, TUI/D2, I223,
Dashboard/I213, persistence, `/auto`, release and publication. The proposed Active/Claimed record
remains ineffective until #385 merges to `main`; no implementation branch or production code is
authorized before that merge.

This claim-preparation checkpoint is historical and is superseded by the I224 implementation and
closeout checkpoint below; it remains preserved as the pre-merge record.

## I224 Implementation And Closeout Checkpoint (2026-08-24)

I224/TOOL-024-C implementation PR #386 was locally converged from claim merge
`ae009ce68f4f3f5d49803e7d8978a021c2c9d3da`. Exact head
`d42c060d618e61218c4c1efe0651e74830807256` passed CI `32719779528` 5/5 and independent
permission/security/API/process review `5394777902`; merge-time CAS then merged it as
`60b0367cf749397bf1167e189e820e82e32baf03`. The C child is now Complete / Closed with that
pre-existing implementation merge as Completion Commit. The corrective implementation keeps
cancel resources job-unique and preserves the public `BackgroundJobRequest` construction contract.

The ordered long task is not complete: TOOL-024-D1 Windows Job Object decision/implementation,
TOOL-024-D2 CLI/TUI and integrated platform acceptance, and I223/Issue #378 deferred validation
remain separately governed. Windows remains fail-closed, Issue #59 stays open, and no release,
publication, Dashboard/I213, `/auto` or Desktop authority was added by I224.

Next task item: prepare a separate D1 decision/claim from current `main`, after a fresh nonterminal
inventory and overlap check. Do not reuse I224, its implementation branch, or its exact-head evidence.

## I225 / TOOL-024-D1-A Claim Preparation Checkpoint (2026-08-24)

I224 owner-first closeout PR #387 merged as `3cb4eff8a7e70e9b8f2c3ed1b667b2ce58f41fe4`.
A fresh inventory found I164 Paused; I197/I198/I201/I210 Review/Claimed; I206-I208 and I223
Planned/Unclaimed; no Active iteration, D1 owner, ADR-068, competing I225 proposal or non-archival
open implementation PR. I213 is Complete/Closed, so there is no Dashboard concurrency exception to
reuse or reconcile.

I225 and TOOL-024-D1-A now own only the prerequisite Windows Job Object security/OS-ABI
decision after atomic claim PR #388 merged as `2afcdc3e`. The decision phase must
produce an independently reviewed current-path matrix and Accepted ADR-068 covering assigned-before-
exec ownership, handle RAII, kill-on-close, nested Job Objects, fail-closed partial failures,
bounded dependency/`unsafe`, compatibility, migration, rollback and the exact D1-B test/authority
boundary.

No Rust, Cargo, dependency, lockfile, Windows runtime, CLI/TUI, Dashboard/I213, `/auto`, release,
publication or Desktop authority is included. D1-B implementation, D2 projection and I223/#378
remain separately governed; Windows background start stays fail-closed.

## I225 Decision Candidate Checkpoint (2026-08-25)

Decision work started from `main@e6980722`. Proposed ADR-068 and
`docs/reference/I225-WINDOWS-JOB-OBJECT-CURRENT-PATH-2026-08-25.md` reverse-map the current
Windows fail-closed admission, Unix launcher, Agent supervisor, Process tool, Core contract and
Runtime finalizer. The candidate chooses suspended creation, Job assignment before resume,
allowlisted stdio handle inheritance, kill-on-close and fail-closed partial cleanup. It adds no
Rust/Cargo/dependency/unsafe or Windows behavior. I225 remains Active/Claimed / Review-pending
until exact-head decision CI and independent Windows/process/unsafe/API review complete.

## I225 Completion Checkpoint (2026-08-25)

ADR-068 was accepted through PR #391 merge `0021690e` from exact head `fca45c46`. Exact-head CI
`32797375011` passed, independent Windows/process/unsafe/API review `5404361120` approved the
decision, and merge-time CAS passed. I225/D1-A is Complete/Closed with Completion Commit
`fca45c467466cd67b52d4391e88c776abfbea198`. D1-B remains a separate Ready/Unclaimed child and
Windows background admission remains fail-closed until its implementation is reviewed and merged.

## I226 / TOOL-024-D1-B Claim Preparation Checkpoint (2026-08-25)

I225 closeout PR #392 merged as `93ee3253`, making ADR-068 Accepted and D1-B Ready. I226 claim PR
#393 merged as `d1f2a126` after exact-head CI `32810069430` and independent approval `5405428154`.
I226 is now Active / Claimed, limited to the Windows Job Object launcher, allowlisted stdio
inheritance, fail-closed cleanup and real Windows tests. D2 and I223 remain separate and Windows
stays fail-closed until the reviewed implementation merges.

## I226 Implementation Candidate Checkpoint (2026-08-25)

Implementation PR #394 implementation candidate is `70e8b674`. It adds
the Windows-only Job Object launcher, allowlisted stdio inheritance, assigned-before-resume
ownership, kill-on-close cleanup and Bash/Exec integration; Unix behavior and D2 remain outside the
slice.

Local formatting, diff and focused locked check/test pass; release preflight reaches its governance
checks but the full build is interrupted by local `ENOSPC`. Exact-head CI `32820263589` is green
5/5, and its Windows workspace test log records the six I226 launcher tests passing. The candidate
therefore remains Review / Claimed pending independent Windows/process/unsafe/API review, with no merge or Completion Commit;
the next action is local correction followed by fresh exact-head CI and independent Windows/process/
unsafe/API review. I223/#378 remains pending and Issue #59 remains open.

## I226 Candidate Reconciliation Checkpoint (2026-08-25)

The implementation branch advanced to exact head `95740c34` after the Windows launcher candidate
`70e8b674`. The candidate still contains only the I226 Windows Job Object implementation and its
owner/derived synchronization; #395 is registered in the reconciliation snapshot as an unrelated
`Intake / Unclaimed` item with no implementation authority. CI `32821573565` completed
classification, installer fixture, Format/Check/Clippy/Test and Windows workspace successfully;
remote issue reconciliation failed because #395 was absent from that exact head. The corrected
local tree passes the same validator against all 52 open Issues after synchronization comment
`5407182562`; fresh exact-head CI remains required after push. The older CI
`32820263589` and any review bound to `70e8b674` do not transfer to this head. I226 remains
Review / Claimed with no merge or Completion Commit; D2, I223/#378 and Issue #59 closeout remain
ordered after this child.

## I226 Completion Checkpoint (2026-08-26)

PR #394 merged into `main` as `d4d7cb25c9c8418345651024fa2102a83c499659`. The implementation
candidate was exact head `835578635daa1eebc76e79ca893296baeed6b35a`, based on
`07da40fb1723838cec962dcf690d493516b2d724`; exact-head CI `32849330531` passed 5/5 and the
independent Windows/process/security/API review `5410840103` approved it. Merge-time CAS passed
with a stable base/head, green CI, protected review and no overlapping implementation authority.
I226 and TOOL-024-D1-B are now Complete / Closed using the pre-existing implementation merge as
Completion Commit. D2 and I223 remain separately governed and Issue #59 stays open.

The reported `pending_submission::tests::pruned_terminal_payload_retains_permanent_idempotency_identity`
60-second run was not reproducible on current main: the test body completed in about 0.06 seconds
and the complete command in about 0.9 seconds. A future recurrence should capture build state,
CPU/IO pressure and the exact process tree before treating it as a product hang.

## I228 / TOOL-024-D2 Claim Preparation Checkpoint (2026-08-26)

After I226 closeout PR #401 merged as `7dd04afd`, the D2 dependency conjunction is complete:
TOOL-024-B/I222, TOOL-024-C/I224 and TOOL-024-D1-B/I226 all have existing implementation merge
evidence. I228 and the TOOL-024-D2 owner are proposed Active / Claimed in a governance-only
candidate prepared from `main@7dd04afd`; the claim and activation remain ineffective until that
record reaches `main`. The slice is limited to CLI/TUI projection, user/model documentation and
integrated Unix/Windows acceptance. It excludes supervisor, permission, Job Object, persistence,
Dashboard/I213, `/auto`, release, publication, Desktop and I223 authority.

## I228 Activation And Local Candidate Checkpoint (2026-08-26)

Claim PR #402 merged as `da9a79cd`, so I228 is effective on `main`. The local candidate is in
`Review / Claimed` and contains CLI/TUI projection, terminal-event delivery, bounded display-safe
summaries and SDK guidance only. It remains subject to stable implementation PR, exact-head CI,
independent protected-scope review and merge-time CAS. I223 still owns the five deferred Issue #378
validation rows and remains required before closing Issue #59.

## I228 Independent Review Correction Checkpoint (2026-08-26)

Implementation PR #403 exact head `f5dc3415` passed CI `32929839326` 5/5, but independent review
`5420911368` correctly rejected it: the synchronous process result still duplicated terminal
semantics, tests did not traverse the real SessionEvent/UI chain, and a required public
`ToolResultDisplay.is_background` field was source-breaking without migration authority. The next
local candidate restores the public struct shape, carries background intent through an internal
display-name marker, gives `BackgroundJobTerminal` sole terminal-state display authority and adds
platform-neutral production-chain coverage for start/read/status/list/cancel and terminal events.
Old CI and review do not transfer; I228 remains Review / Claimed pending a fresh stable head.

The follow-up independent review also identified a result-pairing defect: interleaved tool results
could be attached to the most recent unresolved call rather than their provider `tool_use_id`. The
candidate correction adds a private identity map in `ConversationEngine` and an out-of-order
background/foreground regression test. This is part of I228's same projection boundary; it changes
no supervisor, permission or process behavior.

Independent review `5421297121` of exact head `4dd1dfd9` confirmed pairing correctness but found
pending identities could survive cancelled, failed or completed turns and capture a later result
when a provider reused the same tool ID. The next candidate clears pending identities at every
authoritative terminal and new-turn boundary, deliberately preserves them across `ToolUse`
continuation, and tests cancel/error/success/end-turn ID reuse. CI `32934838307` and its review do
not transfer to the corrected head.

## I228 / TOOL-024-D2 Completion Checkpoint (2026-08-26)

The final PR #403 head `e65f9b490b0d375926f854076f7576131174c4b1` passed exact-head CI
`32937579899` 5/5, including Windows workspace tests and rebuilt CLI smoke. Independent Agent-role
protected-scope review `5421558305` approved the full CLI/TUI projection, terminal exactly-once,
foreground compatibility, public API and platform coverage. Merge-time CAS confirmed stable
head/base, effective claim, clean mergeability and no overlapping implementation PR before #403
merged as `a5fbc22e71afeb30ff0804ec14bf15187d0fb716`. I228 and TOOL-024-D2 are now
Complete/Closed using that pre-existing implementation merge as Completion Commit. I223 remains
Planned/Unclaimed and is the sole remaining Issue #59 iteration, owning V59-B1/C1/D1/D2/FINAL.
