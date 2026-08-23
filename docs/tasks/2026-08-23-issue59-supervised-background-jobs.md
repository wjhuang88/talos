# Issue #59 Supervised Background Command Jobs Long Task

> Status: Claimed coordination and I222 Active/Claimed proposed by PR #379; ineffective until the
> finalized atomic claim PR reaches `main`.

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
| Authorization Evidence | Maintainer active goal selects complete Issue #59 delivery; bounded I213/I222-B parallel authorization is recorded in Issue #366 comment `5386904546`. Exact-head governance/process-security review and CAS remain required. |
| Implementation PR | None; child-specific PRs only |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Coordination claim grants no production authority; every child requires its own effective claim. |

## Ordered Task Items

| ID | Expected output | Completion gate | Depends on | Fallback |
|---|---|---|---|---|
| G0 | Effective task coordination plus TOOL-024-B/I222 claim and activation | Governance-only PR on main; validators/CI/review/CAS | Current main inventory | Leave all work Unclaimed |
| B | Unix Agent/session supervisor core | Implementation merge plus exact-head security/CI/CAS; V59-B1 queued | G0 | Fail closed; no background spawn |
| B-close | I222/TOOL-024-B owner-first Review disposition | Pre-existing implementation SHA and Issue #378 head binding | B | Keep Review; do not start C if machine/security gates fail |
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
core/agent/tools/runtime and excludes every I213 production file. Shared derived governance files
use union semantics. Before stable push and merge, compare exact inventories; same production file
or authority overlap pauses only the overlapping work. C/D2 must recompute overlap and cannot reuse
this authorization.

## Checkpoint

Completed task items: dependency chain through PERM-006-C and validation tracker #378 creation.

Current state and artifacts: PR #379 proposes TOOL-024-B/I222 Active/Claimed from
`main@e1c375e6`; the proposal is ineffective until merge. I223 remains Planned/Unclaimed.

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
