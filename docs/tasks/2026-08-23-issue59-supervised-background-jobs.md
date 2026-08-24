# Issue #59 Supervised Background Command Jobs Long Task

> Status: Active long task; I222/B and I224/C Complete; D1-A/I225 Active/Claimed proposed by #388 but ineffective until merge; D1-B/D2 and I223 remain.

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

I225 and TOOL-024-D1-A now propose only the prerequisite Windows Job Object security/OS-ABI
decision through atomic claim PR #388. Their Active/Claimed record is ineffective until merge. The decision phase must
produce an independently reviewed current-path matrix and Accepted ADR-068 covering assigned-before-
exec ownership, handle RAII, kill-on-close, nested Job Objects, fail-closed partial failures,
bounded dependency/`unsafe`, compatibility, migration, rollback and the exact D1-B test/authority
boundary.

No Rust, Cargo, dependency, lockfile, Windows runtime, CLI/TUI, Dashboard/I213, `/auto`, release,
publication or Desktop authority is included. D1-B implementation, D2 projection and I223/#378
remain separately governed; Windows background start stays fail-closed.
