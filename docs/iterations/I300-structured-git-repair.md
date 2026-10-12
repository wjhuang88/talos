# Iteration I300: Structured Git Permission And Diff Repair

> Document status: Active
> Published plan date: 2026-10-11
> Planned objective: deliver the complete T7 structured Git repair from the session handoff.
> MVP deliverable: approved literal two-file staging through real Agent admission and faithful staged/unstaged built-in diffs.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | T7 structured Git typed effect resources, same-family scope defects and staged/unstaged diff fidelity; associated tests and documentation only. |
| Claimed At | 2026-10-11 |
| Source Issue | None |
| Governance Claim PR | #704 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer authorized serial implementation and gated merges, accepted ADR-089, and permitted non-overlap with I293/I294. No separate maintainer reviewer is available; independent Agent security/API review, exact-head CI, validators and CAS are required. |
| Implementation PR | Not started |
| Last Updated | 2026-10-11 |
| Handoff / Release Condition | No implementation before accepted contract and target-branch claim. |

## Published Baseline

The Active/Claimed values above are proposed atomically by #704 and are ineffective
until its finalized head merges into main. No implementation branch or committed
implementation may start before that merge. PERM-007-H remains Unclaimed.

### Selection And Dependencies

Selected requirement: T7 in the [handoff](../tasks/2026-10-10-session-todo-handoff.md),
under the existing [GIT-001](../backlog/active/GIT-001-embedded-git-tools.md) capability.
Do not reopen historical GIT-001 or TOOL-020 acceptance as though it covered this defect.
I299 is Complete/Closed through implementation #701 and closure #703. ADR-089 is
Accepted. At initial plan publication this was planning only; #704 now proposes
atomic activation, effective only after main merge.

Inventory against main `444a8137`: I293 and I294 Active, retain other-session Search
ownership under the existing explicit non-overlap authorization; I277 Review with
deferred human checks, retain; I249 Planned, retain dependency-pilot deferral;
I164 Paused, retain superseded layout. I299 and I296-I298 remain terminal. No open
PR was found at selection after #696/#702/#703 merged. Recheck before claiming.

### Scope And Acceptance

- Identify the tested executable/build, reproduce the two-file permission error
  through production Agent admission and record actual cause, not only source inference.
  Identify the current tested build; do not pretend an unavailable historical
  executable has been reproduced.
- Implement [ADR-089](../decisions/089-structured-git-effect-authorization.md) after
  acceptance: literal paths and actual repository/index effect binding, approved
  success, denied no-mutation, invalid-resource failure and escape prevention.
- Review and repair same-family Git mutation scope defects without a permission
  bypass, blanket workspace grant, new destructive operation or automatic shell fallback.
- Preserve ordinary configured Git behavior through an explicitly authorized
  execution path; explain and track unsupported configurations without silent bypass.
- Correct staged HEAD/index and unstaged index/worktree diffs using the complete
  ADR regression matrix. Preserve ref comparisons and original content fidelity.
- Run the exact historical two-file operation in an isolated fixture through the
  real production chain, preserving unrelated staged content. Historical publication
  already succeeded via explicitly authorized host Git; verify its existing SHA
  rather than recommitting the historical documents as new implementation evidence.

### Non-Goals

No MODEL-014, compaction, model validation/agile tools, memory-limit parameter,
Search implementation, release, credential/network probing, grant-policy relaxation
or new persistent permission schema. No change to another session's owner.

### Planned Validation And Documentation

- Focused locked Git and real Agent/Runtime integration regressions, including
  approved/denied/invalid resource and the full filesystem/configuration matrix.
- Standard `./scripts/release_preflight.sh`, pinned Rust 1.99.0, `--locked`;
  Windows and Unix CI for platform-specific boundaries. Never skip a failing test
  without a tracked and authorized boundary.
- Independent security/permission/API review, both governance validators and
  exact-head CI plus merge-time CAS.
- Update user-facing built-in Git documentation in README/README.zh-CN and the
  GIT-001 tool reference; document literal-path migration and configured
  execution behavior. Update GIT-001 with bounded corrective evidence, this owner,
  handoff T7, iteration/decision indexes and Board owner-first.
- Native manual acceptance only for a changed UI interaction that automated real
  admission tests cannot establish; do not repeat unchanged I299 observations.

### Risks And Rollback

Index writes are repository metadata effects, not writes to source filenames.
Hooks/filter/config and external linked-worktree metadata cannot be hidden behind
ordinary path approval. Path rechecks alone do not eliminate filesystem races.
Use existing permission/process boundaries and revert only this bounded repair if
authorization fidelity regresses. Preserve user files/index and other-session work.

## Actual Execution And Evidence

2026-10-11: #704 proposes atomic claim and activation after maintainer acceptance
of ADR-089. Current main is `444a8137f054b33802f26bffdb55fb0f83a7a861`;
the inventory above remains applicable. No overlapping PR was present at selection.
Historical Planned baseline above is preserved; activation is effective only on merge.

Read-only source investigation found missing typed facets in add/commit/checkout,
workspace/repo mismatch in pull, unbounded host helper lifetime and incorrect diff
enumeration/baselines. Independent Agent investigation corroborated these findings.
No production reproduction, code fix, effective claim or completion is claimed.

## Residuals

Atomic claim/activation is next. All T7 acceptance remains
pending; all six other numbered handoff requirements retain their existing owners
and serial order. This plan does not close the whole handoff.

### Maintainer scope addition — 2026-10-11

The maintainer accepted ADR-089 and requested Auto approval for all built-in tools,
not only script execution. Track that broader outcome in
[PERM-007-H](../backlog/active/PERM-007-H-all-builtin-auto-review.md), scheduled
immediately after I300 in the handoff. It requires its own iteration/claim and
must reuse I300's truthful Git effects rather than broaden this Git repair silently.
