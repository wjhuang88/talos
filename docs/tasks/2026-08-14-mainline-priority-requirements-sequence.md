# Mainline Priority Requirements — Ordered Long-Running Task

**Status**: Planned / Unclaimed. Planning only; no child iteration is activated by this record.
**Published plan date**: 2026-08-14
**Prerequisite claim**: I196 / WORK-001-A proposed claim PR #226
**Source requirements**: Issues #59, #125 and #155

This task is the durable execution and recovery ledger requested for the ordered requirements. It
coordinates independently governed iterations; it does not combine their scopes, transfer their
owner authority or make an ineffective child claim executable.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #59 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | The maintainer authorized planning Issues #59, #125 and #155 in order and recording them in one long-running task on 2026-08-14. This planning authorization does not authorize implementation, merge, release, migration, deployment, spending or destructive action. |
| Implementation PR | None; child iterations require separate implementation PRs |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Before this task becomes In Progress, establish its coordination claim on `main`; before each child implementation, establish that child's own effective claim and start from its claim merge or later current `main`. |

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

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Publish this ordered planning packet | Long-task owner, I197/I198 plans and synchronized derived views | PR #226 planning head | Planning PR number backfilled; exact-head governance and diff checks pass | Keep task Unclaimed and report the failed gate | Planned |
| T1 | Establish and deliver I196 / WORK-001-A | Accepted P0 architecture decision, current-state inventory and migration/rollback contract | T0 and effective PR #226 claim | I196 owner acceptance, decision-only implementation PR, exact-head evidence and owner-first closeout | Keep I196 Review/Partial; do not begin P1 or behavior changes | Planned |
| T2 | Reconcile, activate and deliver I188 / TOOL-024-A | Accepted background-job lifecycle/security ADR and runnable current-path characterization | T1 disposition | Existing claim/evidence reconciled; I188 acceptance and independent exact-head security review pass | Keep TOOL-024 children blocked; record #59 residual | Planned |
| T3 | Decide Issue #59 production readiness | Explicit TOOL-024-B/C/D dependency matrix and next runnable iteration selection only if all gates hold | T2 | A accepted and RUNTIME-005, PERM-006-C and TOOL-023-C verified; owner/iteration/claim prepared separately | Record blocked residual and continue to T4 | Planned |
| T4 | Claim and deliver I197 / TUI-045 | Permission prompt anchor correction with focused tests and real-terminal evidence | T3 disposition | Effective I197 claim, owner acceptance, exact-head CI/review/CAS and owner-first closeout | Preserve permission visibility/fail-closed behavior; leave Partial/Blocked | Planned |
| T5 | Claim and deliver I198 / SKILL-004 | Confirmed optional-trigger contract, focused fixtures and skill-author docs | T4 disposition | Effective I198 claim, decision checkpoint, owner acceptance and exact-head CI/review/CAS | Preserve parser behavior; create ADR/migration owner if breaking | Planned |
| T6 | Revisit Issue #59 production slices | Separately numbered runnable TOOL-024 child iteration(s), only when dependency gates are true | T3 and T5 | Each new owner/iteration/claim independently satisfies the collaboration and security gates | Leave #59 open with exact blocked owners; do not hold T7 | Planned |
| T7 | Close the long-running task | Final checkpoint, synchronized owners/views/issues and explicit residual packet | T1-T6 terminal dispositions | Every delivered child has pre-existing commit evidence; every residual has an owner and resume gate | Mark task Partial/Blocked with exact recovery instructions | Planned |

## Checkpoints

| Time | Task Item | Branch / Commit | State And Evidence | Open Risk / Deviation | Next Exact Action / Resume |
|---|---|---|---|---|---|
| 2026-08-14 | Authoring | `docs/mainline-priority-long-task-plan` stacked on PR #226 head `1fffd358742b82e159e18f574764600a2b8c5dbf` | Planning authorized; no implementation code changed; task and I197/I198 claims remain Unclaimed | Stacked plan is not target-branch authority; #59 production gates are not satisfied | Validate the planning diff, publish a Draft stacked PR, backfill its number in this task only, and keep child claims Unclaimed |

## Completion Rule

This task can close only after T0-T7 have terminal evidence, each delivered child owner cites an
already-existing implementation/merge commit, all required exact-head checks and independent reviews
are recorded, derived views and Issues agree with owners, and every deferred production slice has an
explicit owner and resume gate. Planning publication alone does not complete any requirement.
