# Mainline Priority Requirements — Ordered Long-Running Task

**Status**: In Progress / coordination claim effective through PR #227. Child state remains owned
by each child; this record alone activates none of them.
**Published plan date**: 2026-08-14
**Prerequisite claim**: I196 / WORK-001-A claim effective through PR #226 merge `453d1fba`
**Source requirements**: Issues #59, #69, #79, #111, #125, #155, #278, #312 and
deferred-validation tracker #302

This task is the durable execution and recovery ledger requested for the ordered requirements. It
coordinates independently governed iterations; it does not combine their scopes, transfer their
owner authority or make an ineffective child claim executable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline planning session 2026-08-14 |
| Work Slice | Coordination and recovery ledger for I205/GOV-007 → I196 → I188/#59 → I199/#69 → I200/#79 → I197/#125 → I201/#111 → I212/#312 → I210/#278 → I198/#155 → I211/VALIDATION-002/#302, plus a dependency-only TOOL-024-B/C/D readiness disposition. This claim does not implement, activate, merge or close any child and does not replace child claims or implementation PRs. |
| Claimed At | 2026-08-14 |
| Source Issue | #59 |
| Governance Claim PR | #303 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | The maintainer authorized planning Issues #59, #125 and #155 in order on 2026-08-14, explicitly added Issues #69, #79 and #111, then directed on 2026-08-17 that stale #227/#228 governance residue be digested and PR-flow simplification be scheduled. On 2026-08-18 the maintainer selected Deferred Human Validation Mode; PR #303 passed full preflight, exact-head CI, remote Issue reconciliation and merge-time CAS, then merged to `main` as `99645e78`. No independent reviewer was available for this planning-only, non-security amendment. Planning authorization does not authorize child implementation, release, migration, deployment, spending or destructive action; each child still requires its own effective claim and gates. |
| Implementation PR | None; child iterations require separate implementation PRs |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Close only after every ordered child has a terminal disposition, every residual has an explicit owner/resume gate, and I211 resolves every required Issue #302 row. Before each child implementation, establish that child's own effective claim and start from its claim merge or later current `main`. |

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
- Under the 2026-08-18 Deferred Human Validation Mode, per-child exact-head CI, independent Agent
  technical review, applicable security review, governance checks and CAS stay local merge gates.
  Explicit natural-person/manual rows move to Issue #302 and I211; source owners remain Review and
  the long task cannot close until the batch resolves them.

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
| T4 | Claim and deliver I199 / TUI-041 | Bounded multiline preview layout with buffer/layout and native-terminal evidence | T3 disposition | Effective I199 claim, Issue #69 acceptance and exact-head CI/review/CAS | Preserve one-row behavior; leave Review/Partial if layout safety fails | Complete |
| T5 | Claim and deliver I200 / TUI-042 | No-op scroll state stability and obsolete-anchor normalization | T4 disposition | Effective I200 claim, Issue #79 acceptance and exact-head CI/review/CAS | Preserve genuine anchor navigation; record I199 interaction residual | Review |
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
| 2026-08-18 | T4 I199 implementation review | Claim merge `8127fa57`; implementation commit `938c9edb9b3336e81a3b90232a69e0993574bc69`; PR #297 | Shared preview planning, display-width wrapping, bounded six-row tail clipping, constrained compression and cleanup are implemented. Focused tests, Clippy, fmt, diff check and isolated PTY coverage passed. | Exact-head CI completion, maintainer native-terminal acceptance and independent exact-head review remain pending; agent PTY evidence does not replace the maintainer gate. | Keep I199 in Review, finish PR #297 gates and CAS, then merge and close owner-first before claiming I200. |
| 2026-08-18 | T4 I199 implementation merge and owner-first closeout | Completion Commits `938c9edb`/`558b76d3`/`14bf4e60`/`add84074`/`de24bffd`; PR #297 head `4434bc83`; merge `5fc814b5`; CI `32138003207`; acceptance `5328282375`; review `5328531254` | I199/TUI-041 is Complete/Closed. The live preview uses one fixed title plus nine rolling body rows, display-width wrapping, prefix-only clipping marker and raw newest thinking tail; required-panel/composer priority is preserved. | TUI-056/#298 owns completed-history folding. I200/#79 remains Planned/Unclaimed and receives no implementation authority from I199. | Close Issue #69 after this owner closeout reaches main, reconcile it into the closed-Issue matrix, then prepare a separate I200 claim from fresh exact main. |
| 2026-08-18 | T5 I200 claim preparation | exact base `main@4acb896e`; I200/TUI-042; Issue #79 OPEN | I199/T4 and Issue #69 are closed. No Active/Review iteration or overlapping open PR exists; I189 stays Planned/Claimed and unactivated, I164 stays Paused, and every later long-task child stays Planned/Unclaimed. | Claim remains ineffective while this governance branch is open. No Rust/Cargo, implementation branch, TUI-045/TUI-043, provider, persistence or release authority. | Open the governance-only Draft PR, backfill its number, finalize the bounded Claimed record, run exact-head CI/review/CAS, then implement only from the claim merge or later `main`. |
| 2026-08-18 | T5 I200 finalized claim proposal | governance PR #300; preparation `cbf2bb3a` | I200/TUI-042 is proposed Active/Claimed for the published no-op scroll transition and obsolete-anchor normalization slice. #69 is closed and reconciled; #79 remains OPEN. | Claim is ineffective until #300 merges. Single-maintainer claim authorization does not waive final natural-person exact-head implementation review or maintainer mouse/touchpad acceptance. No implementation branch exists. | Obtain exact-head CI and independent agent technical audit, repeat CAS, merge #300, then create the implementation branch from that merge or later exact `main`. |
| 2026-08-18 | T5 I200 implementation review | claim merge `356dc3c5`; implementation `3afeeb2859a441ef7e1b7628ff4b5b83b974210d`; PR #301 | Rendering-derived bounds now produce centralized Noop/Anchored/FollowTail outcomes; full-frame tests cover short/exact/overflow boundaries, repeated bursts, multiline input state, height/CJK reflow and I199 preview shrink. Package tests, Clippy, fmt, diff check and full release preflight pass. | Exact-head CI, independent technical review, merge-time CAS, independent natural-person exact-head review and maintainer mouse/touchpad walkthrough remain pending. No later child authority transfers. | Finish PR #301 machine/review gates, merge only after CAS, then keep I200 Review until the two published human gates are recorded; do not claim I197 early. |

## 2026-08-18 Deferred Human Validation Mode Change Control

The maintainer directed that unavailable natural-person review must not idle the ordered long task.
This is a validation-timing and priority change, not an acceptance reduction. Issue #302 is the
tracker and VALIDATION-002/I211 is the evidence-only cleanup iteration. The original child
baselines remain unchanged. PR #303 merged as `99645e78`, so the scheduling amendment is effective;
I211 remains Planned/Unclaimed until the ordered implementation children have terminal dispositions.

Per child, exact-head CI, locked checks, independent Agent technical review with identity limits,
applicable security review, both governance validators and merge-time CAS remain merge gates. Only
the explicitly tracked natural-person/manual rows may be deferred. Each source owner remains Review
until I211 records its row; the next child may proceed after the current table records a terminal
implementation/deferred-validation disposition and its own claim becomes effective.

### Current Ordered Task Items After Validation-Scheduling Change

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Publish the revised planning packet | Long-task change record, I199-I201 plans and synchronized derived views | PR #226 planning head | PR #227 finalized and reached `main` | Preserve ineffective claim and stop | Complete |
| T1 | Establish and deliver I196 / WORK-001-A | Accepted P0 decision, inventory and migration/rollback contract | T0 | Recorded Completion Commit and reviewed closeout | Keep Review/Partial | Complete |
| T2 | Reconcile and deliver I188 / TOOL-024-A | Accepted lifecycle/security decision and characterization | T1 disposition | Recorded Completion Commit and security review | Keep TOOL-024 children blocked | Complete |
| T3 | Decide Issue #59 production readiness | TOOL-024-B/C/D dependency matrix | T2 | All owner-defined runtime/permission gates true | Keep exact blocked residual and continue | Blocked |
| T4 | Deliver I199 / TUI-041 | Bounded multiline preview behavior | T3 disposition | Recorded Completion Commit and acceptance | Preserve one-row behavior | Complete |
| T5 | Deliver I200 / TUI-042 implementation | No-op scroll stability and obsolete-anchor normalization in `main` | T4 | PR #301 exact-head CI, Agent technical review and CAS; add human rows to #302 | Keep I200 Review and do not claim acceptance | Complete / Completion Commit `3afeeb28` |
| T6 | Claim and deliver I197 / TUI-045 implementation | Permission-prompt anchor correction without permission semantic changes | T5 implementation/deferred-validation disposition | Effective claim, exact-head CI, Agent technical review, applicable security gate and CAS; add eligible human rows to #302 | Preserve permission visibility/fail-closed behavior; leave Review/Blocked | Review / Implementation merged; human validation deferred to #302 / I211 |
| T7 | Claim and deliver I201 / TUI-043 implementation | Conditional tool-call marker suppression with negative/order fixtures | T6 implementation/deferred-validation disposition | Effective claim, exact-head CI, Agent technical review and CAS; add natural-person row to #302 | Preserve legitimate text; leave Review/Partial | Review / Implementation merged; human validation deferred to #302 / I211 |
| T7A | Claim and deliver I212 / MODEL-013 implementation | Conservative local catalog context-window inference with explicit precedence/provenance | T7 implementation/deferred-validation disposition | Effective claim, exact-head CI, Agent technical review and CAS; add custom-provider walkthrough row to #302 | Preserve unknown fallback; reject ambiguous matches | Complete / Completion Commit `5a1709cb` |
| T7B | Claim and deliver I210 / PROVIDER-006 implementation | Typed provider progress with `Connecting…` then truthful `Reconnecting… (attempt n/m)` | T7A implementation/deferred-validation disposition | Accepted ADR, effective claim, exact-head CI, Agent technical review and CAS; add live retry-status row to #302 | Preserve static connecting behavior and retry policy; do not fabricate progress | Review / Claimed; implementation merged as `9d5c8a71`, human row deferred |
| T8 | Claim and deliver I198 / SKILL-004 implementation | Confirmed optional-trigger contract, fixtures and skill-author docs | T7B implementation/deferred-validation disposition | Effective claim, contract checkpoint, exact-head CI, Agent technical review and CAS; add natural-person row to #302 | Preserve parser behavior; create ADR/migration owner if breaking | Review / Claimed; PR #325 merged as `15a3d424`, human row deferred |
| T9 | Execute I211 / VALIDATION-002 | One human review/manual evidence packet for every Issue #302 row | T5-T8 implementation dispositions, including T7A/T7B | Effective evidence-only claim; all rows pass or have corrective owners; source owners synchronized first | Keep failed source owners Review and long task Partial | Complete / Completion Commits `b7d55a0d`/`7c333d98` |
| T10 | Revisit Issue #59 production slices | Separately numbered runnable TOOL-024 child iteration(s) only when gates are true | T3 and T9 | Every new owner/iteration/claim independently satisfies collaboration and security gates | Leave #59 open with exact blocked owners | Blocked / reassessed; RUNTIME-005 and PERM-006-C incomplete |
| T11 | Close the long-running task | Final checkpoint, synchronized owners/views/issues and explicit residual packet | T1-T10 terminal dispositions | Delivered children cite pre-existing evidence; Issue #302 rows resolved; every residual has an owner | Mark task Partial/Blocked with exact recovery instructions | Planned |

### Change-Control Checkpoint

| Time | Task Item | Branch / Commit | State And Evidence | Open Risk / Deviation | Next Exact Action / Resume |
|---|---|---|---|---|---|
| 2026-08-18 | T5 merge and human-validation scheduling change | PR #301 head `8a58cb2d`; merge `9628e183`; CI `32149762367`; Agent technical review `5330234992`; Issue #302; VALIDATION-002/I211 | I200 implementation reached `main` after machine/technical/CAS gates. The maintainer selected Deferred Human Validation Mode and created one later cleanup phase for I200/I197/I201/I198 human rows. | I200 natural-person exact-head review and mouse/touchpad matrix remain unpassed. The mode cannot defer protected security review or justify Complete. | Merge #303 after exact-head validation/CAS, then re-inventory and prepare only the separately scoped I197 claim from current `main`. |
| 2026-08-19 | T6 I197 claim preparation | PR #303 merge `99645e78`; governance branch `docs/i197-claim-governance` from exact `main` | #303 is effective; I197/TUI-045 is proposed Active/Claimed for the layout-only permission-prompt anchor slice. No implementation branch or permission-policy authority exists. | Claim #304 is ineffective until merge; I200 human rows remain open in #302/I211. Any protected permission/security change remains non-deferrable. | Validate and merge #304 with exact-head CI, both governance validators and CAS; only then create the I197 implementation branch from the claim merge or later `main`. |

## 2026-08-19 Work Mode Record

The active long task uses the `Deferred Human Validation` work mode defined by
`docs/sop/LONG-RUNNING-TASK.md`. Its validation tracker is GitHub Issue #302 and its planned
evidence-only cleanup iteration is I211 / VALIDATION-002. The tracker is a deferred-validation
queue: an open natural-person or device-dependent row does not block a later non-overlapping child
once that child's dependencies, claim, machine checks, technical review, applicable security review
and merge-time CAS are terminal. Source owners remain `Review` until I211 records a pass; protected
security-review gates are never deferred; the long task cannot close while a required tracker row is
open.

## 2026-08-19 I197 Implementation Merge Checkpoint

I197/TUI-045 implementation PR #305 merged to `main` as `d98f37e742e32313a9b670837b5c45129cf4e700`.
The implementation head was `9fce4f13c6f1e598025f661494f67af73b60fcd3`, based on effective claim
merge `0db92cf9`. Exact-head CI run `32204974418` passed all five jobs and independent Agent review
`5336592072` approved the exact head with shared-account identity limits disclosed. The changed
runtime scope remains TUI layout/anchor behavior only; no permission policy, request identity,
protected crate, persistence or release authority changed. I197 remains `Review` with
`Completion Commit: Pending`; its natural-person and terminal rows are tracked in Issue #302 / I211.
The next non-overlapping child may proceed only after its own effective claim and current-main
inventory gates.

## 2026-08-19 I201 Claim Activation Checkpoint

I201/TUI-043 claim PR #306 final head `153e470f` merged to `main` as `78cb1ddd` from exact base
`8069ea6a`. Exact-head CI run `32209314843` passed every routed job, both governance validators
reported 0 warnings, independent Agent review `5336890794` approved the exact head with
shared-account identity limits disclosed, and merge-time CAS passed. The claim is now effective;
implementation has not started and must branch only from `78cb1ddd` or later current `main`. The
authorized scope remains the published TUI-visible placeholder suppression boundary and excludes
provider protocol, core Message, tool execution, permission, persistence, broad renderer and
release changes.

## 2026-08-19 I201 Implementation Review Checkpoint

Implementation commits `68f4fb7b` and `d1fef291` are published through PR #309 from exact
activation merge `25fe1f0c`. Fourteen focused state/event tests and the full `talos-tui` suite prove
conditional suppression, fallback visibility, direct result/approval non-confirmation, multi-tool
ordering and no blank replacement row; strict package
Clippy, formatting, both governance validators, `git diff --check` and release preflight passed.
The change touches only TUI ordered-content presentation. I201 remains `Review` with `Completion
Commit: Pending`; exact-head CI, independent Agent technical review, merge-time CAS and the Issue
#302 / I211 human suppression-safety row remain open.

## 2026-08-19 I201 Merge And I212 Priority Checkpoint

PR #309 final head `d8d414ce3f2d65c6859fa4f30566efb3ac94196c` passed exact-head CI
`32220300200`, independent Agent technical review `5338185591`, both governance validators and
merge-time CAS, then merged to `main` as `7f5a6df2122d9b5ed70e55e59281e3e4e127f18c`. I201 remains
Review with its natural-person row open in Issue #302/I211.

The maintainer then advanced Issue #312 ahead of I198. MODEL-013 is refined into the runnable I212
local catalog-inference slice. This is a priority change only: I212 remains Planned/Unclaimed until
its separate claim reaches `main`; it does not inherit I201 authority, and it adds no network probe,
capability inference, config migration or Rust/Cargo authorization during claim preparation. After
I212 receives an implementation/deferred-validation disposition, execution returns to I198/#155.

## 2026-08-19 I212 Activation And I210 UI Requirement Checkpoint

I212 claim PR #314 final head `ec5c6920` passed exact-head CI `32223903534`, both governance
validators and merge-time CAS, then merged as `a62f448b`. The independent reviewer attempt
disconnected without a conclusion, so the planning-only claim used the SOP single-maintainer path
with disclosure `5338629524`. I212 is Active/Claimed and implementation must branch from that merge
or later current `main`.

The maintainer also clarified Issue #278/I210: the existing model-request activity row must start as
`Connecting…`, then show `Reconnecting… (attempt n/m)` from actual structured retry/timeout facts,
and clear on success, terminal failure or cancellation. This is part of the long task after I212 and
before I198, but I210 remains Planned/Unclaimed and still requires its ADR and separate effective
claim. It cannot be implemented inside I212 or inferred from error text/timers.

## 2026-08-19 Human Validation And I212 Implementation Checkpoint

The maintainer executed the available Issue #302 real-terminal matrix on integrated
`main@ec794515` under macOS. Plain Unicode/ASCII markers, streamed multiline and long-sentence
content, ordered read-only tools, denied and approved permission recovery, tool failure recovery,
prompt Esc cancellation within about one second, the following turn, and session resume all
completed. The approved write executed exactly once and its content was verified; the temporary
artifact was removed afterward.

The same matrix found that a permission-mediated tool sequence can retain `Calling tools…`, an
unnamed `approved` row and the named structured tool row. This is a failing I201/#111 sequence
(`marker -> approval -> ToolCall`), so I201 remains Review and the corrective requirement stays in
Issue #111/#302 rather than being called accepted. I197/#125 also remains Review: the permission
selector and running-tool status occupy opposite sides of the composer and the full resize,
small-terminal and queued-prompt matrix is incomplete. Esc cancellation was responsive, but resumed
history lacked an explicit cancelled terminal row; Issue #45 owns that observation. Retry/timeout
and direct internal result/approval events were not exercised. Resumed `out` counters reset to zero;
Issue #302 records this as an unresolved semantics question, not a confirmed defect.

I212 implementation commit `3cb1a801` now provides the bounded local catalog inference slice; its
implementation PR is #318.
Config tests passed 224/224 plus one doctest with isolated HOME; 28 CLI lifecycle tests, strict
Clippy, formatting and diff checks passed. Full release preflight also passed outside the outer
execution sandbox, including macOS seatbelt tests. No Cargo/default-feature/dependency/persistence/
network surface changed. Issue #316 now owns the separate HOME-mutating test isolation defect. I212
still requires exact-head CI, independent Agent review, CAS and its custom-provider Issue #302 row
before closeout. I210 remains the next ordered child and has no implementation authority until its
separate ADR and claim land.

## 2026-08-20 I212 Review Correction Checkpoint

Independent Agent review `5349780559` verified I212 implementation behavior at superseded head
`7f6838a0` but requested changes because remote owner reconciliation lacked #316/#317 and MODEL-013
still showed the already-effective claim gate unchecked. Governance PR #319 reconciled those Issues
without adding I212 product scope and merged as `8d0d3166`. The I212 branch is rebased onto that
merge, the claim fact is corrected, and all exact-head CI/review evidence must now be regenerated.
I212 remains the active T7A item; no I210/I198 authority transfers before its implementation merge
and deferred-validation disposition.

## 2026-08-20 I212 Merge And T7B Resume Checkpoint

PR #318 exact head `a2466c55641cc893ae5cf9248519af8b1ca4f093` passed exact-head CI
`32319297491` (5/5), independent Agent approval `5349952979`, both governance validators and
merge-time CAS, then merged as `5a1709cbcdb4ec1960fae637bfe48cd93e817d87`. I212/MODEL-013 is
Review / Claimed with Completion Commit pending; its natural-person exact/explicit/ambiguous custom
provider walkthrough is now an explicit Issue #302/I211 row.

This is the terminal implementation/deferred-validation disposition required to resume T7B. The
next exact action is I210/PROVIDER-006 ADR and claim preparation from fresh `main`; I210 remains
Planned/Unclaimed and no provider API, retry-policy or implementation authority exists yet.

## 2026-08-20 I210 ADR And Claim Proposal Checkpoint

Fresh inventory at `main@7c5cc8b7b4d75d7a71d2f632e6696d9023588396` found no Active
iteration; I197/I200/I201/I212 remain Review under Issue #302/I211, I189 remains
Planned/Claimed and unactivated, I198/I206-I208/I210/I211 remain Planned/Unclaimed, and I164
remains Paused. Open PRs #120/#121 are archival Drafts and no open PR overlaps Issue #278.

Governance PR #321 proposes the bounded I210 claim and ADR-062. The decision uses a defaulted
additive provider progress entrypoint, real request-local retry/backoff/first-packet facts, the
existing ordered non-exhaustive Agent/session progress path and a distinct reconnecting phase. It
preserves retry/timeout/backoff policy, third-party providers that do not opt into progress,
persistence, dependencies and release state. The public conversation enum addition is explicitly
gated to a future pre-1.0 minor release, not a patch.

The proposal remains ineffective while #321 is open. Independent exact-head architecture/claim
review, CI, both governance validators and merge-time CAS must pass before merge. No implementation
branch, Rust/Cargo change, version, tag or publication action is authorized before that target-branch
merge.

## 2026-08-20 I210 Claim Activation Checkpoint

PR #321 exact head `4d45f1ba890fa7cb1ea6f6f058ecb0f0916eb639` passed CI `32322271343`, independent
governance/architecture review `5350249740`, both governance validators, `git diff --check` and
merge-time CAS. It merged to `main` as `e58fbd399a7071aad7ad8fd846a82f2745611fa0`. ADR-062 is
Accepted and PROVIDER-006/I210 is now Active/Claimed. No implementation, release, tag or
publication action was included.

The next exact action is to create an I210 implementation worktree from `main@e58fbd39`, verify the
owner/claim and current non-terminal inventory once more, then implement only the accepted provider
progress Work Slice. I212 remains Review pending its Issue #302 natural-person walkthrough.

## 2026-08-20 I210 Implementation Checkpoint

I210 implementation commit `6efee2b8` is now available for its separately governed implementation
PR. The commit stays within the accepted provider-progress slice: typed dispatch/retry/backoff/
first-packet facts, runtime projection, reconnecting presentation, cancellation tests and user
documentation. It does not alter retry policy, dependencies, persistence, version/tag state or
release/publication authority.

Locked affected-crate tests, the complete `talos-cli` suite under an isolated writable `HOME`,
strict affected-crate Clippy, formatting and `git diff --check` passed. The earlier CLI error was
reproduced as an outer-sandbox configuration-I/O restriction and did not recur with the isolated
test home. I210 remains Review/Claimed until exact-head CI, independent technical review,
merge-time CAS and its Issue #302 live retry-status row are complete; no Completion Commit is
claimed yet. I212 and the other deferred rows remain unchanged.

## 2026-08-20 I210 Merge And I198 Claim-Preparation Checkpoint

I210 PR #323 final head `c984ec483aaba5f6d4d1e96d288cfcb874b0f239` passed CI
`32333116774`, independent Agent technical re-review `5351610613`, both governance validators and
merge-time CAS, then merged as `9d5c8a71718b44d424092a45a75d3da0d593547d`. Issue #302 comment
`5351796088` records its open natural-person live retry-status row. I210 remains Review/Claimed;
its implementation disposition is terminal for scheduling but is not a human-acceptance or
completion claim.

Current `main`/`origin/main` match at `9d5c8a71`; no iteration is Active. I197, I200, I201, I210 and
I212 remain Review for Issue #302; I189 stays Planned/Claimed and unactivated; I198, I206, I207,
I208 and I211 remain Planned/Unclaimed; I164 stays Paused; no current iteration document is Blocked.
Open PRs #120/#121 are archival Drafts and no open branch/PR overlaps I198.

Read-only I198 characterization confirms omission fails only because `SkillFrontmatter.triggers`
has no serde default, while explicit empty/non-empty lists and malformed-type rejection already
have distinct deterministic behavior. The proposed missing-field default is an additive input
extension with no struct-shape, routing, permission, dependency or persistence change. The I198
claim preparation remains ineffective until its actual PR number, exact-head CI/review and CAS are
complete; no implementation branch or parser edit is authorized yet.

## 2026-08-20 I198 Finalized Claim Proposal

Draft PR #324 supplied the governance identifier. Its finalized exact head now proposes one
Active/Claimed I198 Work Slice limited to omitted-`triggers` defaulting, focused compatibility
fixtures and bilingual skill-author documentation. The compatibility checkpoint remains additive;
explicit lists and malformed-value rejection are hard acceptance boundaries.

The proposal has no target-branch effect while #324 is open. Independent exact-head claim review,
CI, both governance validators and merge-time CAS must pass before merge. No implementation branch,
Rust/Cargo change, dependency, release, tag or publication action is authorized before that merge.

## 2026-08-20 I198 Claim Activation And Characterization Correction

PR #324 exact head `a06e34a51dabd33a3204d2e96e749f2342545438` passed CI `32337065552`,
independent Agent claim review `5351981686`, both governance validators and merge-time CAS, then
merged as `ea6686855de971df42de0311333617090c30de47`. The I198 implementation branch starts from
that exact claim merge.

Test-first characterization corrected one over-broad review statement: `yaml_serde` coerces numeric
sequence scalars to strings. I198 preserves that historical explicit-list behavior and limits the
change to omitted-field defaulting; rejected container/mapping shapes remain covered. The correction
is published in PR #324 comment `5352103571` and Issue #155 comment `5352103804`.

## 2026-08-20 I198 Implementation Validation Checkpoint

The bounded implementation and binary runtime fixture passed focused locked tests, strict package
Clippy, both governance validators, manifest parsing, whitespace checks and the complete release
preflight. I198 is `Review / Claimed`; exact-head implementation CI, independent Agent technical
review, merge-time CAS and its Issue #302 natural-person row remain open. T9/I211 is not activated
by this checkpoint.

## 2026-08-20 I198 Implementation PR Checkpoint

Implementation commit `f719ed913d36ad7ad00f5a99d3d990b414dbbd5d` is published in PR #325.
The owner records now identify that PR; the next head requires fresh exact-head CI and independent
Agent technical review before merge-time CAS. This evidence backfill does not complete I198 or
authorize I211.

## 2026-08-20 I198 Merge And I211 Claim-Preparation Checkpoint

PR #325 final head `b2d5adaf0cdbc57906f37661dfe42762c7deead6` passed CI run
`32340432185` attempt 2, independent Agent review `5352638953` and merge-time CAS, then merged as
`15a3d4248d13d3951c823628454a2629398a9d48`. Issue #302 comment `5352702000` records the final
I198 disposition; its natural-person row remains open.

All six I211 child implementation dispositions are now terminal. Current iteration inventory has
no Active item; I197/I198/I200/I201/I210/I212 remain Review for #302; I189 remains
Planned/Claimed and unactivated; I206-I208 and I211 remain Planned/Unclaimed; I164 remains Paused;
no current iteration document is Blocked. Open PRs #120/#121 are archival Drafts and do not overlap
I211. PR #326 proposes the governance-only evidence claim; it grants no product repair, release or
publication authority before target-branch merge.

## 2026-08-20 I211 Claim Review Correction Checkpoint

Independent Agent claim review `5353122975` bound to PR #326 head `229b9754` accepted the
evidence-only scope, dependency inventory and baseline preservation but requested the missing I200
final disposition in Issue #302. Comment `5353130091` now records PR #301 final head `8a58cb2d`,
implementation `3afeeb28`, CI `32149762367`, Agent technical review `5330234992`, merge-time CAS
and merge `9628e183`; I200 remains Review and its natural-person rows remain open.

The correction resolves the sole review blocker without changing product code or acceptance
truth. Open PR #327 is an unrelated Dashboard claim. Re-run both exact-base governance
validators, CI and independent review on the new #326 head before merge-time CAS; only the claim
merge may authorize a later I211 activation branch.

## 2026-08-20 I211 Claim Merge And Activation Proposal

PR #326 exact head `d51d5721` passed CI `32347993402`, independent Agent approval `5353284891`,
both governance validators and merge-time CAS, then merged as `285fc3c7`. The I211 claim is
effective and this activation branch starts exactly at that merge.

The non-terminal inventory remains six Review children, I189 Planned/Claimed but unactivated,
I206-I208 Planned/Unclaimed and I164 Paused; no other iteration is Active. Open PR #327 is a
separate Dashboard claim and does not overlap I211. The activation proposal remains ineffective
until PR #328 merges and grants no product repair, release or publication authority.

## 2026-08-20 I211 Activation Merge And Evidence Disposition

PR #328 final head `bb501862` passed CI `32349317758` attempt 3, independent Agent approval
`5353504113` and merge-time CAS, then merged as `a2f43248`. I211 is the sole Active iteration.

Issue #302 checkpoint `5341637918` is partial: several ordinary runtime observations passed, but
the permission-mediated marker/unnamed outcome path failed and is now owned by TUI-058 / Issue
#329; permission prompt hierarchy/composer docking failed or remained incomplete and is owned by
TUI-059 / Issue #330. Both corrective Stories are Ready/Unclaimed and authorize no implementation.
I200, I212, I210, I198 and the remaining direct-event rows stay pending; the long task remains In
Progress and cannot close.

## 2026-08-20 I211 Integrated Validation Checkpoint

Integrated locked tests now cover I198 omitted-trigger runtime reachability and parser compatibility,
I212 catalog resolution/provenance, I210 provider-to-TUI retry projection and terminal cleanup, and
I201 direct-event negative cases. Maintainer mock-provider validation confirmed truthful
`Reconnecting... (attempt 1/2)` and cleanup, but found `Connecting...` too brief to observe and the
first idle submission falsely labeled as queued. Ready/Unclaimed TUI-060 / Issue #332 owns those
defects without authorizing implementation. Remaining human/device work is I200, I212 and I198;
the long task and I210 remain open.

## 2026-08-20 I198, I212 And I200 Human Validation Checkpoint

Maintainer real-binary validation on integrated `main@a2f43248` closed I212/MODEL-013: exact and
one-prefix custom identities showed catalog provenance, explicit `777K` remained authoritative,
ambiguous/unknown identities stayed manual with conservative fallback, no inferred value was
persisted and no model request was sent. I212 is Complete/Closed at pre-existing mainline
implementation merge `5a1709cb`.

I198 passed omitted/empty/list activation and body projection but failed the required real-CLI
malformed-`triggers` diagnostic; Ready/Unclaimed SKILL-005 / Issue #333 owns the correction. I200's
macOS touchpad short/exact/overflow/multiline/resize/CJK matrix passed. No physical mouse was
available or executed; the maintainer explicitly accepted touchpad evidence as the native
scrolling-device substitute. Reflow exposed a separate ordinary-history continuation-prefix
regression owned by Ready/Unclaimed TUI-061 / Issue #334. I200 is Complete/Closed at pre-existing
implementation commit `3afeeb28`; I198 remains Review.

Every Issue #302 row now passes or has a separately governed corrective owner. I211 moves to
Review pending rolling evidence PR #331 exact-head CI, independent review and merge-time CAS. A
later status-only closeout must cite the already-existing #331 evidence commit and then reassess
Issue #59; this evidence update cannot self-certify I211 completion.

## 2026-08-21 I211 Evidence Merge And Closeout Checkpoint

PR #331 final head `7c333d98` passed exact-head CI `32372514265`, independent Agent approval
`5356366597`, both governance validators, remote owner reconciliation and merge-time CAS, then
merged as `97dbf35f`. I211/VALIDATION-002 is Complete/Closed at pre-existing evidence commits
`b7d55a0d` and `7c333d98`; this status-only closeout does not self-certify completion.

I200 and I212 are Complete. I197, I198, I201 and I210 remain Review with corrective destinations
TUI-059/#330, SKILL-005/#333, TUI-058/#329 and TUI-060/#332; TUI-061/#334 remains the separate
padding regression owner. T9 is terminal. The next exact action is T10: re-fetch current main and
reassess Issue #59's RUNTIME-005, PERM-006-C and TOOL-023-C gates before selecting any new child.

## 2026-08-21 T10 Issue #59 Gate Reassessment

Current `main@5301b8c2` contains the I211 closeout; Issue #302 is remotely closed after evidence
commits `b7d55a0d`/`7c333d98`, #331 merge `97dbf35f` and #335 closeout merge `5301b8c2`.

Issue #59 is not production-ready. TOOL-024-A/I188 and TOOL-023-C are Complete, but TOOL-024-B
still requires all of RUNTIME-005 and PERM-006-C. RUNTIME-005-A is Ready because SESSION-008-A/B
and RUNTIME-001 are Complete; B/C remain blocked in order. PERM-006-A/I189 remains
Planned/Claimed and deliberately unactivated, while PERM-006-B/C remain blocked in order. No
TOOL-024-B/C/D owner, iteration, claim or implementation branch is created.

T10 therefore receives a terminal Blocked disposition under its published fallback. The smallest
independent gate-clearing follow-up is the decision-only RUNTIME-005-A / I214 claim preparation;
it changes no Rust/Cargo/runtime behavior and transfers no I211, I189 or TOOL-024 authority. T11
remains pending until that separate claim proposal is recorded and the long-task residual packet is
synchronized.

PR #336 now carries that governance-only proposal. Its Claimed records remain ineffective while
open; exact-head CI, both governance validators, independent architecture/claim review and
merge-time CAS are still required. No I214 decision execution or implementation branch may start
before the claim reaches `main` and a later activation record is established.

## 2026-08-21 I214 Claim Merge And Activation

PR #336 final head `cc99af9e` passed exact-head CI `32435705544`, both governance validators,
independent claim review `5364050202` and merge-time CAS, then merged to `main` as `7de582a3`.
RUNTIME-005-A/I214 is now Active/Claimed from that claim merge for the current-path matrix and
Proposed shutdown-contract ADR only.

The non-terminal inventory remains explicit: I197, I198, I201 and I210 stay Review; I206-I208
stay Planned/Unclaimed; I189 stays Planned/Claimed and unactivated; I213 stays Planned/Claimed in
the independent Dashboard lane; I164 stays Paused/superseded. RUNTIME-005-B/C, PERM-006-B/C and
TOOL-024-B/C/D remain blocked or unauthorized in their existing order. No Rust, Cargo, API,
runtime, persistence, permission, sandbox, product UI, dependency, release, publication or
`unsafe` change is authorized by this activation.

## 2026-08-21 I214 Decision Execution

From activation merge `14531bbc`, the decision-only branch prepared a code-grounded runtime
shutdown matrix and Proposed ADR-063. The proposal retains the current Session/ADR-058 finalizer,
adds no implementation, and defines B as coordinator/admission/active-turn/deadline/report work
and C as ordered-finalizer/durable-reconciliation/compatibility work.

I214 remains Active until its exact decision head passes CI and independent architecture review.
RUNTIME-005-B/C, I189/PERM-006 and TOOL-024-B/C/D remain blocked or unactivated exactly as before.
Decision content commit `648a35d3` is submitted through PR #338; that existing content commit, not
a later status-only commit, is the candidate completion evidence after acceptance.

## 2026-08-21 Governance Workflow Repair Interruption

The maintainer explicitly paused further product/runtime development to correct the delivery
workflow measured by GOV-007/I205. Issue #339, GOV-008 and I215 own a non-overlapping governance
slice: atomic claim+activation, local design/implementation/test/documentation convergence, one
stable remote stage candidate, scenario fixtures and an EVOLUTION lesson. The claim+activation PR
has no effect until merge and contains no SOP, validator, Rust/Cargo or product implementation.

I214 remains Active/Claimed with decision PR #338 waiting for valid independent exact-head review;
no new I214 edits are scheduled during the governance repair. After I215 closes, finish the already
started I214 flow, then govern RUNTIME-005-B/C and later long-task work using the revised process.
I197/I198/I201/I210 remain Review, I189/I213 remain Planned/Claimed and unactivated, I206-I208
remain Planned/Unclaimed, and I164 remains Paused.

## 2026-08-21 I215 Atomic Activation And Local Convergence

PR #340 head `1e00249b` passed exact-head CI `32439457491`, both validators, manifest parsing, scale
assessment and merge-time CAS, then merged as `e66d039c`. GOV-008/I215 became Claimed and Active in
that single merge; no separate activation PR was created.

The implementation worktree began at the activation merge. The AGENTS/SOP changes, POSIX and
PowerShell harness integration, 12 ordinary/protected/release/bounded-maintenance scenarios and
EVOLUTION lesson converged locally without intermediate pushes. I215 is Review pending one stable
stage candidate, not Complete.

The final local convergence passed the full release preflight, both governance validators,
PowerShell strict parse and execution, workflow and SQLite scenario harnesses, manifest/scale/site/
installer/classifier checks, workspace check/Clippy/tests/doctests and diff/scope hygiene. This loop
caught a PowerShell wiring defect that had omitted the SQLite self-test; it was repaired and all
affected gates were rerun before the first push.

Meanwhile #338 review identified two blocking decision defects: the proposed closing-bit check does
not atomically pair actor admission with turn start, leaving a check-to-start race; and consuming
`shutdown_with(self, invalid_options)` conflicts with Drop-triggered default shutdown. I214 remains
Active/Claimed. After I215 closes, batch both corrections locally on #338 and obtain fresh exact-head
review; do not convert either finding into a separate micro-PR.

## 2026-08-21 I215 Stable-Stage Completion

PR #341 exact head `06e61e3c` passed CI `32442052401`, independent Agent-role review
`5365129718` and merge-time CAS, then merged as `81a603b4`. GOV-008/I215 is Complete/Closed at
Completion Commit `06e61e3c`; this status synchronization does not cite itself. Issue #339 closed
with the implementation merge and receives the same owner evidence.

The next mainline action returns to already Active/Claimed I214/#338. Batch its admission
check-to-start race and invalid-options/Drop contract corrections locally, rerun the complete local
checkpoint, then submit one new stable #338 head for fresh exact-head architecture review.

## 2026-08-21 I214 Batched Architecture Corrections

After I215 closed, #338 was synchronized locally with `main@457895c5`. The two architecture
findings are corrected in one local convergence cycle: ADR-063 now uses one SDK/actor
admission-start arbiter whose non-await start commit shares the shutdown fence, and structured
shutdown borrows its handle and accepts only construction-time validated options, with explicit
primary/controller Drop semantics. I214 remains Active/Claimed and ADR-063 remains Proposed pending
local validation, one stable push and fresh exact-head independent architecture review. No
Rust/Cargo/runtime behavior, RUNTIME-005-B/C, I189, TOOL-024, release or publication authority is
added.

## 2026-08-21 I214 Decision Acceptance

PR #338 corrected exact head `6719c876` passed CI `32449605985`, independent architecture review
`5365529351` and merge-time CAS, then merged as `fc70e396`. ADR-063 is Accepted and
RUNTIME-005-A/I214 is Complete/Closed at the pre-existing decision commit; the status closeout does
not self-certify.

RUNTIME-005-B is now Ready/Unclaimed but remains unselected and unactivated until a new runnable
iteration and effective claim reach `main`. C remains Blocked on B. T10/Issue #59 remains Blocked
until all RUNTIME-005 and PERM-006-C complete; I189 stays Planned/Claimed and unactivated, and no
TOOL-024, Rust/Cargo/runtime, release or publication authority transfers. The next mainline planning
step is the smallest RUNTIME-005-B iteration/claim, not implementation on the I214 branch.

## 2026-08-21 I216 RUNTIME-005-B Claim Preparation

Current `main@3c98f315` contains the I214 closeout. RUNTIME-005-B now has a separate child owner and
I216 runnable plan for only the coordinator, validated options, shared SDK/actor admission-start
arbiter, active-turn policies, one deadline, cached redacted report, Drop/legacy behavior and
deterministic fixtures fixed by ADR-063.

This governance-only proposal remains Planned/Unclaimed until its actual PR number is backfilled and
the finalized exact head passes CI, both validators, independent runtime architecture review and
merge-time CAS. Parent RUNTIME-005 remains Unclaimed; C, PERM-006-B/C and TOOL-024-B/C/D remain
blocked or unauthorized. No implementation branch, Rust/Cargo change, version, release or
publication action is allowed before the claim reaches `main`.
