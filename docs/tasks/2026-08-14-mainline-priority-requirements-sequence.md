# Mainline Priority Requirements — Ordered Long-Running Task

**Status**: In Progress / coordination claim effective through PR #227. Child state remains owned
by each child; this record alone activates none of them.
**Published plan date**: 2026-08-14
**Prerequisite claim**: I196 / WORK-001-A claim effective through PR #226 merge `453d1fba`
**Source requirements**: Issues #59, #69, #79, #111, #125 and #155

This task is the durable execution and recovery ledger requested for the ordered requirements. It
coordinates independently governed iterations; it does not combine their scopes, transfer their
owner authority or make an ineffective child claim executable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session 2026-08-14 |
| Work Slice | Coordination and recovery ledger for I205/GOV-007 → I196 → I188/#59 → I199/#69 → I200/#79 → I197/#125 → I201/#111 → I198/#155, plus a dependency-only TOOL-024-B/C/D readiness disposition. This claim does not implement, activate, merge or close any child and does not replace child claims or implementation PRs. |
| Claimed At | 2026-08-14 |
| Source Issue | #59 |
| Governance Claim PR | #227 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer authorized planning Issues #59, #125 and #155 in order on 2026-08-14, explicitly added Issues #69, #79 and #111, then directed on 2026-08-17 that stale #227/#228 governance residue be digested and PR-flow simplification be scheduled. No independent reviewer is currently available for this planning-only, non-security claim. Single-maintainer merge requires final exact-head CI, both governance validators, merge-time CAS and no unresolved blocking feedback. Planning authorization does not authorize child implementation, release, migration, deployment, spending or destructive action. |
| Implementation PR | None; child iterations require separate implementation PRs |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Close only after every ordered child has a terminal disposition and every residual has an explicit owner/resume gate. Before each child implementation, establish that child's own effective claim and start from its claim merge or later current `main`. |

## Startup Contract

### Outcome

Execute the mainline prerequisite and Issues #59, #125 and #155 in an evidence-preserving order:
I196 P0 decision/migration contract, I188 background-job lifecycle decision, I197 permission-prompt
layout correction, and I198 optional skill-trigger compatibility. Revisit #59 production slices only
after their existing dependency gates are true. Keep at most one child iteration Active or Review
unless a later target-branch claim explicitly authorizes otherwise.

### In Scope

- Coordination and recovery checkpoints for I196, I188, I197 and I198.
- Owner-first state transitions, independent claims, implementation PRs, exact-head review/CI and
  merge-time CAS for each child.
- A dependency disposition for TOOL-024-B/C/D after I188, without claiming they are currently
  runnable.
- Issue synchronization and explicit residual ownership at each child closeout.

### Out Of Scope

- Combining the requirements into one implementation iteration or one implementation PR.
- Activating I188/I189 or any child solely because this plan exists.
- Work Graph, Evaluator, Desktop, Dashboard, release, tag, deployment or persistence migration work.
- TOOL-024-B/C/D implementation before accepted TOOL-024-A and completed RUNTIME-005, PERM-006-C
  and TOOL-023-C gates permit a separately planned runnable iteration.
- Permission-policy changes in I197 or broad ClawHub/skill-routing changes in I198.

### Dependencies And Prerequisites

- `main` and `origin/main` must be re-fetched and exact before every claim/activation gate.
- PR #226 must pass independent exact-head review/CI and merge-time CAS before I196 implementation.
- I188 retains its effective claim merge `02a35588` but remains Planned; before activation, reconcile
  its stale pre-merge evidence wording and obtain the exact-head security review required by its
  published baseline.
- I197 and I198 are Planned/Unclaimed and each needs a separate effective target-branch claim.
- I159-I162 stay Blocked; I164 stays Paused; I189 stays Planned/Claimed and unactivated unless a
  separately authorized workflow changes their owners.
- ARCH-034-R04 remains Partial; RUNTIME-005 remains Refinement/Unclaimed; DESKTOP-001 remains
  Deferred/Unclaimed and ADR-059 remains Proposed.

### Artifacts And State Owners To Update

- I196/WORK-001-A and its decision/reference artifacts for the prerequisite P0 slice.
- I188/TOOL-024-A and TOOL-024 for Issue #59's decision and dependency disposition.
- I197/TUI-045 for Issue #125.
- I198/SKILL-004 for Issue #155.
- This task checkpoint table, then the iteration index, Product Backlog and Board as derived views.
- GitHub Issues #59, #125 and #155 after evidence-bearing state changes; close only when the owning
  requirement is actually delivered or explicitly cancelled.

### Validation And Acceptance Evidence

- Every child uses its owner-defined focused tests and runnable/manual acceptance evidence.
- Workspace checks use the pinned toolchain and `--locked`; release preflight remains mandatory
  before merge under `AGENTS.md` even though no release is authorized.
- Both governance validators and `git diff --check` pass on each finalized exact head.
- Implementation and independent natural-person review roles remain separate; shared-account use
  discloses actual execution/review roles and identity limitations.
- A child closes only with a pre-existing implementation/merge SHA in `Completion Commit:`; a
  status-only commit cannot self-certify.

### Branch, Worktree And Checkpoint Plan

- `main` is the only target/integration branch. Each normal implementation branch is short-lived
  and starts only after its child claim reaches `main`.
- This planning branch is stacked on the exact PR #226 governance head so P0 remains a visible
  predecessor; its planning PR must target the #226 branch until #226 merges, then be rebased or
  retargeted and revalidated against current `main` without changing PR #226's exact head.
- Do not reuse `/private/tmp/talos-i194`, `/private/tmp/talos-i194-closeout` or either historical
  I193 stash; do not restore them as a unit.
- Append one durable checkpoint after planning publication and at every T0-T7 phase boundary,
  including branch/head, commands/results, changed artifacts, deviations and exact resume action.

### Allowed Permissions And External Actions

- Current authorization covers governance planning edits, validation, normal planning commits,
  branch push and a Draft planning PR.
- Later implementation, merge, issue closure/comment, release, migration and deployment actions must
  follow the applicable child claim and explicit repository authorization path.
- Read-only repository/GitHub inspection and normal deterministic local tests are allowed within
  the selected child scope.

### Destructive Or Irreversible Operations

None are authorized. No force-push, stash restore/drop, worktree deletion, tag, release, publish,
deployment, schema/data migration or external destructive action is part of this task.

## 2026-08-17 Queue Gate: Provider Fix And Session Interactivity

The provider UTF-8 emergency fix (`PROVIDER-005`, Issue #270, PR #271) reached Complete/Closed:
implementation `1d31847a`, exact-head CI `32002811484`, independent approval `5313112992`, merge
`89523dbc`, owner closeout `c15da4cf`, and remote synchronization `abf88657`. Its emergency
authorization remains separate from this coordination record.

With #271 closed, the next queued slice is
`I209 / TUI-051 / Issue #272` (`docs/iterations/I209-resumed-session-interactivity.md`). I209 owns
resumed-turn cancellation responsiveness, bounded provider retry status and large-history TUI
projection invalidation. It is Planned / Unclaimed and has no implementation authorization.

Only after I209 reaches its own terminal disposition may this task proceed to `I205 / GOV-007`,
then the previously published child order `I196 -> I188/#59 -> I199/#69 -> I200/#79 ->
I197/#125 -> I201/#111 -> I198/#155`. This checkpoint does not expand the existing coordination
claim, activate any child, or authorize release/publication work.

### Time, Cost And Resource Limits

- No paid service, real provider credential or monetary spend.
- Poll long commands incrementally and issue a progress checkpoint at least every 60 seconds.
- Retry a transient command at most twice after recording the first failure; deterministic failures
  require diagnosis and an in-scope fix rather than blind retry.
- No deadline justifies skipping a claim, independent review, security gate or acceptance evidence.

### Failure, Retry And Fallback Policy

- If a claim is ineffective, exact head changes, CI/review is stale or merge-time CAS fails, leave
  the child Planned/Review and refresh the evidence; do not start or merge implementation.
- If I188 does not accept the lifecycle contract, keep TOOL-024-B/C/D blocked and continue to I197
  only after recording that dependency disposition; the long task need not stall unrelated items.
- If TOOL-024 production prerequisites remain unsatisfied after I188, record #59 as an owned
  residual and proceed to I197/I198. Do not invent a production iteration ID before it is runnable.
- If I197 cannot preserve both security-choice visibility and layout stability, preserve current
  fail-closed behavior and leave the iteration Partial/Blocked.
- If I198's decision checkpoint finds a breaking contract, preserve parser behavior and route the
  change to a separate ADR/migration owner.
- Stop after three consecutive occurrences of the same external blocker when no safe fallback can
  advance another independent task item.

### Default Decisions For Foreseeable Ambiguity

- Preserve current behavior and choose the smaller compatible change.
- Treat public API/format, persistence, permission, security, dependency and `unsafe` ambiguity as
  blocking for implementation, not as implicit authority.
- Order does not mean dependency where none exists: after recording an upstream blocked residual,
  the next independent Planned child may proceed through its own claim gate.
- Existing owner acceptance is authoritative; this task records execution facts and never rewrites
  a published iteration target.

### Residual-Work Destination

- TOOL-024-B/C/D and unresolved Issue #59 production behavior remain in TOOL-024 child owners until
  their prerequisites support new runnable/testable iteration IDs.
- Broader permission UI belongs outside TUI-045; broader skill-format/ClawHub support belongs outside
  SKILL-004.
- P1-P4 Work/Evaluation work remains in WORK-001-B through WORK-001-E and outside this requirements
  sequence unless separately added by change control.

## Change Control — 2026-08-14 Approved Scope Addition

The maintainer explicitly added Issues #69, #79 and #111 after the original planning baseline was
published. This is a scope addition under `docs/sop/CHANGE-CONTROL.md`, not an in-scope correction.
Each requirement has an independently acceptable outcome, so none is folded into I197 or another
published iteration:

| Added Issue | Owner | New Iteration | Readiness At Change | Disposition |
|---|---|---|---|---|
| #69 | TUI-041 | I199 | Refinement / Unclaimed | Refined to a runnable bounded-preview correction; Planned / Unclaimed. |
| #79 | TUI-042 | I200 | Refinement / Unclaimed | Refined to a runnable scroll-transition correction; Planned / Unclaimed. |
| #111 | TUI-043 | I201 | Ready / Unclaimed | Selected as a runnable conditional presentation fix; Planned / Unclaimed. |

The revised technical order is I199/#69 → I200/#79 → I197/#125 → I201/#111. I199 runs first
because its preview-height behavior changes history viewport capacity; I200 then owns the exact
scroll-bound and obsolete-anchor normalization consumed by I197's permission-prompt anchoring.
I201 is independent but follows the anchor work to reduce overlapping TUI review churn. A blocked
predecessor is dispositioned and recorded rather than silently skipped; unrelated later work may
continue through its own claim gate.

The non-terminal inventory remains unchanged by this planning addition: I159-I162 stay Blocked,
I164 stays Paused, I188/I189 stay Planned/Claimed and unactivated, and I196-I201 are Planned with
only I196's and the long task's proposed claims present in open PRs. There is no Active or Review
iteration on target `main`. ARCH-034-R04 remains Partial, RUNTIME-005 remains Refinement/Unclaimed,
DESKTOP-001 remains Deferred/Unclaimed, and ADR-059 remains Proposed.

## Original Ordered Task Items — Published Baseline

The original T0-T7 table is preserved as planning history. The revised table below is the current
execution order after the approved scope addition.

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Publish this ordered planning packet | Long-task owner, I197/I198 plans and synchronized derived views | PR #226 planning head | Planning PR number backfilled; exact-head governance and diff checks pass | Keep task Unclaimed and report the failed gate | Planned |
| T1 | Establish and deliver I196 / WORK-001-A | Accepted P0 architecture decision, current-state inventory and migration/rollback contract | T0 and effective PR #226 claim | I196 owner acceptance, decision-only implementation PR, exact-head evidence and owner-first closeout | Keep I196 Review/Partial; do not begin P1 or behavior changes | Complete |
| T2 | Reconcile, activate and deliver I188 / TOOL-024-A | Accepted background-job lifecycle/security ADR and runnable current-path characterization | T1 disposition | Existing claim/evidence reconciled; I188 acceptance and independent exact-head security review pass | Keep TOOL-024 children blocked; record #59 residual | Complete |
| T3 | Decide Issue #59 production readiness | Explicit TOOL-024-B/C/D dependency matrix and next runnable iteration selection only if all gates hold | T2 | A accepted and RUNTIME-005, PERM-006-C and TOOL-023-C verified; owner/iteration/claim prepared separately | Record blocked residual and continue to T4 | Blocked |
| T4 | Claim and deliver I197 / TUI-045 | Permission prompt anchor correction with focused tests and real-terminal evidence | T3 disposition | Effective I197 claim, owner acceptance, exact-head CI/review/CAS and owner-first closeout | Preserve permission visibility/fail-closed behavior; leave Partial/Blocked | Planned |
| T5 | Claim and deliver I198 / SKILL-004 | Confirmed optional-trigger contract, focused fixtures and skill-author docs | T4 disposition | Effective I198 claim, decision checkpoint, owner acceptance and exact-head CI/review/CAS | Preserve parser behavior; create ADR/migration owner if breaking | Planned |
| T6 | Revisit Issue #59 production slices | Separately numbered runnable TOOL-024 child iteration(s), only when dependency gates are true | T3 and T5 | Each new owner/iteration/claim independently satisfies the collaboration and security gates | Leave #59 open with exact blocked owners; do not hold T7 | Planned |
| T7 | Close the long-running task | Final checkpoint, synchronized owners/views/issues and explicit residual packet | T1-T6 terminal dispositions | Every delivered child has pre-existing commit evidence; every residual has an owner and resume gate | Mark task Partial/Blocked with exact recovery instructions | Planned |

## Current Ordered Task Items After Approved Change

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Publish the revised planning packet | Long-task change record, I199-I201 plans and synchronized derived views | PR #226 planning head | PR #227 contains finalized scope; exact-head governance/CI checks pass | Keep proposed claim ineffective and report the failed gate | Planned |
| T1 | Establish and deliver I196 / WORK-001-A | Accepted P0 architecture decision, inventory and migration/rollback contract | T0 and effective PR #226 claim | I196 owner acceptance, implementation PR, exact-head evidence and owner-first closeout | Keep I196 Review/Partial; do not begin P1 behavior work | Complete |
| T2 | Reconcile, activate and deliver I188 / TOOL-024-A | Accepted background-job lifecycle/security ADR and current-path characterization | T1 disposition | Claim/evidence reconciled; I188 acceptance and independent exact-head security review pass | Keep TOOL-024 children blocked; record #59 residual | Complete |
| T3 | Decide Issue #59 production readiness | TOOL-024-B/C/D dependency matrix and a new runnable iteration only if every gate holds | T2 | A accepted and RUNTIME-005, PERM-006-C and TOOL-023-C verified | Record blocked residual and continue to T4 | Blocked |
| T4 | Claim and deliver I199 / TUI-041 | Bounded multiline preview layout with buffer/layout and native-terminal evidence | T3 disposition | Effective I199 claim, Issue #69 acceptance and exact-head CI/review/CAS | Preserve one-row behavior; leave Review/Partial if layout safety fails | Active |
| T5 | Claim and deliver I200 / TUI-042 | No-op scroll state stability and obsolete-anchor normalization | T4 disposition | Effective I200 claim, Issue #79 acceptance and exact-head CI/review/CAS | Preserve genuine anchor navigation; record I199 interaction residual | Planned |
| T6 | Claim and deliver I197 / TUI-045 | Permission-prompt anchor correction with focused tests and real-terminal evidence | T5 disposition | Effective I197 claim, Issue #125 acceptance and exact-head CI/review/CAS | Preserve permission visibility/fail-closed behavior; leave Review/Partial | Planned |
| T7 | Claim and deliver I201 / TUI-043 | Conditional tool-call marker suppression with negative/order fixtures | T6 disposition | Effective I201 claim, Issue #111 acceptance and exact-head CI/review/CAS | Preserve legitimate text; leave Review/Partial if correlation is unsafe | Planned |
| T8 | Claim and deliver I198 / SKILL-004 | Confirmed optional-trigger contract, fixtures and skill-author docs | T7 disposition | Effective I198 claim, contract checkpoint, acceptance and exact-head CI/review/CAS | Preserve parser behavior; create ADR/migration owner if breaking | Planned |
| T9 | Revisit Issue #59 production slices | Separately numbered runnable TOOL-024 child iteration(s) only when gates are true | T3 and T8 | Every new owner/iteration/claim independently satisfies collaboration and security gates | Leave #59 open with exact blocked owners; do not hold T10 | Planned |
| T10 | Close the long-running task | Final checkpoint, synchronized owners/views/issues and explicit residual packet | T1-T9 terminal dispositions | Delivered children cite pre-existing commit evidence; every residual has an owner/resume gate | Mark task Partial/Blocked with exact recovery instructions | Planned |

## Checkpoints

| Time | Task Item | Branch / Commit | State And Evidence | Open Risk / Deviation | Next Exact Action / Resume |
|---|---|---|---|---|---|
| 2026-08-14 | T0 Draft publication | `docs/mainline-priority-long-task-plan`; plan `d45c82242d6aa199638433e97b041d969771a0c3`; claim backfill `335660d7b9b9ae67f7c56fbbd01a92503f2893b9`; Draft PR #227 stacked on PR #226 head `1fffd358742b82e159e18f574764600a2b8c5dbf` | Planning baseline and proposed coordination claim pushed; repository/skill governance and claim validators reported 0 warnings; diff check passed; no implementation code changed; I197/I198 remain Unclaimed | The #227 coordination claim is ineffective before `main`; #59 production gates are not satisfied | Keep #227 Draft until #226 merges, then reconcile to current `main`, rerun exact-head gates and obtain independent review before merge |
| 2026-08-14 | T2 authorized out-of-order decision execution | I188 implementation `245eddeb`; review-state head `946d9d2f64168644d18abdffa58bed9b9c808162`; PR #228; CI `31771281927` | Maintainer separately directed the mainline session to clear Issue #59 implementation blockers. Effective I188 claim merge `02a35588` allowed the published decision-only slice to run while T1's claim remains pending. ADR-060 and the current-path matrix are now in independent Review; no production, Rust, Cargo, dependency, persistence, Desktop or Dashboard behavior changed. | PR #228 still lacks independent exact-head process/permission security review. RUNTIME-005 and PERM-006 through C remain incomplete; Windows process-tree ownership remains a D gate. | Preserve owner-first ordering: do not mirror I188 Review into this stacked branch's Board/index. After PR #228 lands on `main`, reconcile #227 to that owner truth, record the T1/T2 variance, and rerun exact-head CI/review/CAS before merging this long-task claim. |
| 2026-08-14 | Approved scope addition | `docs/mainline-priority-long-task-plan`; scope commit `c39bf4f535382e2721404d5b0659ffd559881183`; Draft PR #227 | Maintainer added Issues #69/#79/#111; I199/I200/I201 owners and runnable plans published; original task baseline preserved; repository/skill governance and claim validators reported 0 warnings; diff check passed | Revised PR head still requires exact-head CI and independent review; every added child remains Unclaimed | Keep #227 Draft behind #226, update its exact-head description, then after #226 merges reconcile to current `main` and rerun all gates |
| 2026-08-17 | Issue #69 upcoming-pool confirmation | `main@30bf2357`; PR #227 already merged as `3f7db88d` | Maintainer reconfirmed #69 for the upcoming pool. Existing TUI-041/I199 ownership is retained; TUI-041 is Ready and I199 remains Planned/Unclaimed at T4 after the I188/#59 disposition. | No effective I199 claim or implementation authorization exists. | Disposition the current predecessor, then prepare a separate I199 claim from fresh exact `main`; do not create its implementation branch before the claim merges. |

## Completion Rule

This task can close only after T0-T7 have terminal evidence, each delivered child owner cites an
already-existing implementation/merge commit, all required exact-head checks and independent reviews
are recorded, derived views and Issues agree with owners, and every deferred production slice has an
explicit owner and resume gate. Planning publication alone does not complete any requirement.

## Current Exact-Main Reconciliation — 2026-08-17

This checkpoint supersedes only stale current-state instructions; it does not rewrite the published
2026-08-14 planning baseline or its historical checkpoints.

- PR #226 merged as `453d1fba`; I196 remains Planned/Claimed and unactivated after the v0.8.0
  release closure. It needs a fresh exact-main inventory before implementation.
- Current target `main@f46f45d7` has no Active or Review iteration. I164 remains Paused;
  I188/I189/I195/I196 remain Planned/Claimed and unactivated.
- I197-I201 remain Planned/Unclaimed and require independent child claims.
- I188 decision PR #228 remains unmerged and must be refreshed and independently security-reviewed
  on its new exact head; no production background-job implementation is authorized.
- #227 now targets current main directly. Its coordination claim remains ineffective until merge.

## Change Control — 2026-08-17 PR Workflow Simplification

The maintainer added an evidence-based PR workflow simplification Spike after the v0.8.0 delivery
showed that claim, status, synchronization and re-review round trips could exceed the underlying
code and documentation changes. This is a new governance outcome, not an in-scope correction to
I196 or any product child, so GOV-007 / I205 owns it separately.

I205 runs as gate `T0A` before T1 implementation begins. It measures recent delivery chains,
distinguishes mandatory safety evidence from mechanically preventable churn, and selects a smallest
separately claimable process change. It does not itself change SOPs, validators, CI, branch
protection or review requirements.

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0A | Claim and deliver I205 / GOV-007 | Reproducible PR/review audit, target-flow decision, scenario matrix, migration/rollback and smallest implementation slice | T0 planning publication | Effective I205 claim, owner acceptance, exact-head validation/review/CAS and owner-first closeout | Keep current workflow unchanged and record measured blockers | Complete |

### T0A Completion Evidence

Completion Commit: `2e2cf04b7f07fe4744d0cf591c326e4514e346ac` (pre-existing I205 audit artifacts).
PR #287 merged as `0394e26466543ee5dca4e5c02b1d7341d86cb290`; final exact head `d3047ec9` passed
CI `32094384772` 5/5 and independent technical audit comment `5323234878`. This evidence closes
T0A only; the long-running task and all child implementation work remain open or planned.

Hard gates remain effective claim before governed implementation, independent review for protected
security scope, exact-head evidence after content changes, merge-time CAS, owner-first truth and a
pre-existing Completion Commit before Complete. The optimization target is duplicate state/derived
view work and mechanically preventable re-review, not removal of those controls.

## Reconciliation Checkpoint

| Time | Task Item | Branch / Commit | State And Evidence | Open Risk / Deviation | Next Exact Action / Resume |
|---|---|---|---|---|---|
| 2026-08-17 | T0 current-main reconciliation and T0A change control | `docs/mainline-priority-long-task-plan`; PR #227 merged with `main@f46f45d7` | Preserved the published baseline, retained I197-I201 planning, added separate GOV-007/I205 ownership, and changed no product code or executable governance rule. | #227 still needs exact-head validation and independent review; I205 and every product child remain unactivated. | Run both governance validators and CI, obtain exact-head review, then merge #227 before claiming I205. |
| 2026-08-17 | I209 pre-claim reproduction and claim preparation | exact-main baseline `e885d368`; governance PR #276 | TUI Esc entry tests 7/7, CLI bridge cancellation status and agent exact targeted-interrupt tests passed. Source inspection confirms two synchronous full-history projections occur before the TUI polls input again, making input starvation before `UserInput::Cancel` the leading loss point. | The four-boundary resumed structured-turn reproduction and CPU/input-latency evidence remain implementation acceptance; PR #276 is ineffective before merge. | Validate #276 at its exact head, perform merge-time CAS, then after merge activate I209 and create its implementation worktree from the claim merge point or later main. |
| 2026-08-17 | I209 claim effective and activation preparation | claim merge `33b11433`; activation PR #277 | PR #276 passed exact-head CI `32016873512`, both governance validators and merge-time CAS, then established the effective I209 claim. The activation record proposes I209 as the only Active iteration and changes no implementation surface. | Activation remains ineffective until #277 merges; no implementation worktree exists yet. | Validate and merge #277, then create the implementation worktree from that merge point or later main. |
| 2026-08-17 | I209 implementation and retry-progress change control | activation merge `c7380332`; implementation `7b82fea6`/`7d90def8`; PR #279; Issues #272/#278 | Cached unchanged history projection and a 2,000-message reopened-Session integration prove the urgent CPU/input and four-boundary cancellation slice. Source inspection established that truthful retry progress requires a semver-bound provider contract excluded by I209; maintainer authorized transfer to Planned/Unclaimed PROVIDER-006/I210. | I209 remains Review pending exact-head CI, real-terminal CPU/input and restoration evidence, independent review and CAS. I210 has no claim or implementation authority. | Finish PR #279 exact-head review gates and close I209 owner-first; do not claim I210 during this slice. |
| 2026-08-17 | I209 implementation merge and owner-first closeout | Completion Commit `2eff6285`; source implementation `7b82fea6`/`7d90def8`; exact head `6657d14e`; CI `32025371877`; reviews `5316405699`/`5316533941` | A 513,987-byte/2,000-message real-terminal resume remained responsive at approximately 3.6-3.7% idle CPU, input appeared inside 250 ms, supported double-`Ctrl+C` restored terminal modes, and the independent agent audit approved the exact head with the shared-identity limitation disclosed. | Truthful retry progress remains Planned/Unclaimed in PROVIDER-006/I210/#278; I200 and I206 remain separate. No release, provider API or unrelated TUI authority transfers. | After closeout merges and Issue #272 closes, return to GOV-007/I205 only through a fresh inventory, effective claim and separate activation. |
| 2026-08-17 | T2 I188 decision closeout | Completion Commit `245eddebae762d1d0c7ee796baea50d0bb080bd5`; exact head `d7d4fe7a`; CI `31995198205`; independent security review `5312482823`; PR #228 merge `1db1211e` | I188/TOOL-024-A is Complete/Closed and ADR-060 is Accepted as a decision-only contract. The current-path characterization and decision predate this status closeout; no production background process or permission behavior was implemented. | TOOL-024-B remains blocked until RUNTIME-005 and PERM-006-C are Complete; C remains blocked on B and Windows remains fail-closed pending D. Issue #59 stays open. | Resume T0A at GOV-007/I205 through a fresh exact-main inventory and effective governance-only claim; do not activate I196 or any TOOL-024 production child. |
| 2026-08-18 | T0A I205 claim preparation | exact baseline `main@a9cfef02`; governance PR #284 | No Active/Review iteration exists. I189/I195/I196 remain Planned/Claimed; I197-I201, I205-I208 and I210 remain Planned/Unclaimed; I164 remains Paused. #284 proposes only the reproducible evidence/decision Spike. | The claim is ineffective until #284 merges. No audit implementation, SOP, validator, CI, branch-protection or product change is authorized. | Obtain exact-head CI and single-maintainer authorization evidence, repeat CAS, merge #284, then activate I205 separately from that merge or later main. |
| 2026-08-18 | T0A I205 activation | claim head `5af45593`; CI `32046397520`; claim merge `fd1eaad9`; activation governance branch from that merge | I205 became the sole Active iteration for the evidence-only audit; existing hard gates and every product/runtime/release owner remained unchanged. | Audit PR #287 now contains the evidence packet; no executable governance rule is authorized. | Obtain exact-head review/CI and merge-time CAS for #287, then close I205 owner-first as Review/Complete only with pre-existing evidence; create a new bounded claim for atomic activation before resuming I196. |
| 2026-08-18 | T0A I205 audit packet | implementation PR #287; evidence snapshot `docs/reference/I205-PR-WORKFLOW-EVIDENCE.json` | 42 explicit PRs across ten chains: 40 merged, 2 closed unmerged, 37 review rounds, 11 REQUEST CHANGES, 26 approvals and 10 reviewed-head changes. The report selects atomic claim activation and preserves security, exact-head, CAS, owner-first, Completion Commit and release-order gates. Governance validator, Collaboration Claim validator, script compile, JSON assertions and diff check passed. | Independent exact-head review and merge-time CAS remain pending; the audit does not implement the selected follow-up. | Review #287, then merge and close I205 owner-first. Prepare a new child owner/iteration/claim for atomic claim activation; only after that resume I196. |
| 2026-08-18 | Concurrent requirement intake | Issue #285 / PROMPT-001 | Registered the newly opened Prompt Authority Architecture as Refinement/Unclaimed to restore Issue-owner reconciliation. It remains separate from GOV-007/I205 and grants no prompt, SDK, memory, Evolution, plugin or runtime implementation authority. | Architecture decision, behavioral baseline and child decomposition remain unresolved. | Continue only I205 activation; route PROMPT-001 through its own future ADR, child owners and claims. |
| 2026-08-18 | T0A I205 owner-first closeout | Completion Commit `2e2cf04b7f07fe4744d0cf591c326e4514e346ac`; PR #287 merged as `0394e264`; final head `d3047ec9`; CI `32094384772`; independent technical audit `5323234878` | I205/GOV-007 is Complete/Closed. The audit preserves protected review, exact-head, CAS, owner-first, Completion Commit and release-order gates; atomic claim activation remains a separate unclaimed follow-up. | No executable workflow/SOP/validator change is authorized by I205. I196 remains Planned/Claimed and unactivated. | Create/activate only the next separately bounded claim, or refresh exact-main inventory for I196 when its governance gates are satisfied. |
| 2026-08-18 | T1 I196 activation | activation base `main@b59912e36025088e4e3fa76b7b5b4e2aa7a1396c`; effective claim PR #226 merge `453d1fba` | Fresh inventory found I159/I160/I161/I162/I188 Complete, I164 Paused, I189 Planned/Claimed, I195 Active/Claimed and no other Review. I196 is Active for the P0 decision/documentation slice only; its scope is explicitly non-overlapping with Dashboard I195. | Independent exact-head architecture review, decision packet and owner-first closeout remain pending. No Rust/Cargo/persistence/API/product authority. | Build only the decision/migration evidence packet, then submit a decision-only PR for exact-head CI and independent architecture review. |
| 2026-08-18 | T1 I196 P0 closeout | Completion Commit `779a4c7116610f07258013e866f74b2a180c5453`; PR #291 exact head `2128c41c`; merge `1467a561`; CI `32101943484`; independent architecture review approval | WORK-001-A and I196 are Complete / Closed. The packet is documentation-only and preserves P1-P4 separation. | P1 Work Domain implementation remains separately blocked and unclaimed. | Re-inventory current `main` before creating any new P1 claim; no implementation authority transfers from I196. |
| 2026-08-18 | T2/T3 #59 disposition | I188/TOOL-024-A Completion Commit `245eddeb`; PR #228 merge `1db1211e`; CI `31995198205`; independent security review `5312482823` | I188 is Complete/Closed. #59 production readiness is Blocked: RUNTIME-005 is Refinement/Unclaimed, PERM-006-C is Blocked/Unclaimed, and TOOL-023-C is Complete. | No TOOL-024-B/C/D implementation or claim can start until the owner-defined runtime and permission gates are complete. Issue #59 remains open. | Keep T3 Blocked; progress RUNTIME-005 and PERM-006-C only through their own claims/iterations, then re-inventory before creating a runnable #59 child. |
| 2026-08-18 | T4 I199 activation preparation | Activation PR #295 from `main@1e001836`; TUI-041/I199 claim proposed for Issue #69 | I199 is proposed Active/Claimed for the bounded transient preview layout slice. T3/#59 remains Blocked and does not transfer implementation authority. | Claim is ineffective until PR #295 merges; no implementation branch exists. | Obtain exact-head CI, independent claim review and merge-time CAS, then create the I199 implementation branch from the claim merge or later `main`. |
