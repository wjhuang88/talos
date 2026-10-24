# Iteration I307: Bubblewrap Workspace Visibility

> Document status: Active
> Published plan date: 2026-10-24
> Planned objective: restore the authorized Linux workspace after temporary mounts without expanding writable scope.
> Baseline rule: preserve this bounded repair; broader sandbox redesign requires a separate owner.
> MVP deliverable: sandboxed commands in a /tmp child workspace run there while host temporary siblings remain isolated.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | SANDBOX-001: Linux Bubblewrap workspace mount order, explicit cwd, isolation tests and docs; no Git admission, fallback policy, dependencies or API changes. |
| Claimed At | 2026-10-24 |
| Source Issue | #757 |
| Governance Claim PR | #758 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested session-handoff and failing-CI closure; no distinct maintainer reviewer available. Independent Agent security review, exact-head CI, both validators and merge-time CAS remain mandatory. Ineffective until merged to main. |
| Implementation PR | Not started |
| Last Updated | 2026-10-24 |
| Handoff / Release Condition | Governance-only preparation; implement only after effective claim. No release authorization. |

## Published Baseline

Selected [SANDBOX-001](../backlog/active/SANDBOX-001-bubblewrap-workspace-visibility.md),
source #757. This independent prerequisite repairs a general sandbox defect uncovered
by I300 validation; it does not replace or narrow I300 acceptance.

### Selection Inventory And Disposition

Target main `d5e0d0627`, refreshed 2026-10-24. Owner status inventory:

- I249 Planned: retain dependency upgrade pilot; no overlap.
- I277 Review: retain deferred human/device acceptance.
- I293 and I294 Active: retain Search owners and their implementation.
- I300 Active: retain Git work; isolated dependency repair precedes its final Linux gate.
- I302 Review: retain Windows resource evidence follow-up.
- I306 Active: retain provider protocol work and #749.
- I162 Complete with recorded Review outcome is not an open iteration.
- No Blocked owner or other nonterminal iteration found owning this repair.

Open PRs #735 (Git), #749 (provider) and #705 (Windows diagnostic) do not own
Linux mount-order repair. Issue #245 and its comments concern completed I161
review assignment, not this new defect; I161 is not reopened.
I307 is serial prerequisite work by this actor, not a second autonomous stream.

### Scope And Acceptance

1. Preserve read-only host root, private /tmp, network policy and owned cleanup.
2. Bind only the authorized workspace after temporary/device mounts, preserve extra
   read paths' later read-only precedence, and select sandbox cwd explicitly.
3. Real Linux Bubblewrap reads an existing marker, creates a workspace file and
   reports exact cwd for a workspace below /tmp.
4. Host /tmp siblings remain invisible; a same-named sandbox temporary write must
   not mutate the host sentinel. Existing outside-workspace file writes fail.
5. An explicitly exposed /tmp extra-read file remains readable and not writable.
6. Missing workspace fails without executing the payload in HOME or another cwd.
7. Re-run Linux Desktop Auto allow-once and both cancellation readiness failures;
   available bwrap with namespace failure must not be misreported as test success.

### Non-Goals

No broad /tmp binding, unsandboxed fallback, new backend, dependency, unsafe code,
policy relaxation, public API change, process-supervisor rewrite or Git feature work.

### Validation And Documentation

Use pinned Rust 1.99.0 and --locked. Run real Linux bwrap regressions with
bwrap_installed, original Desktop failure cases, `./scripts/release_preflight.sh`,
exact-head CI and independent escape-vector review. Run both governance validators
and project-scale assessment for the isolated worktree. Update sandbox README,
this owner, backlog, Board and issue records. Rollback is a reviewed revert;
no test disablement or writable-scope enlargement.

## Evidence And Residuals

CI 40089763867 at f30b4df6f, Linux job 120859903922: current order produces
cwd /home/runner and absent marker; tmpfs-first/workspace-last plus --chdir
produces exact workspace cwd and visible marker. This proves the probe, not
a product fix. Independent Agent scope/security review requires the acceptance
above and separate effective claim before coding. No implementation or completion
is claimed; I300 and the full handoff remain incomplete.

## Proposed Atomic Activation

Claim and Active state above are proposed by #758 and become effective only
when its finalized reviewed head merges to main. Both validators passed before
PR creation; repeat them and target-branch CAS on the finalized candidate.
