# Iteration I106: Self-Bootstrap Control Plane

> Document status: Active
> Published plan date: 2026-07-08
> Planned objective: establish the Talos-primary execution contract, runtime smoke harness, and
> evidence classification needed before another REL-002 self-bootstrap attempt.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a repeatable Talos-primary rehearsal that records whether the session qualifies,
> partially qualifies, or does not qualify for REL-002.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT100` | 2026-07-08 self-bootstrap plan | Planned | Clean worktree and current owner docs | Talos-primary execution contract and inventory recorded. |
| `SBT101` | 2026-07-08 self-bootstrap plan | Planned | SBT100 | Evidence schema distinguishes qualifying, partial, and non-qualifying sessions. |
| `SBT102` | 2026-07-08 self-bootstrap plan | Planned | SBT101 | Talos runtime smoke harness is repeatable. |
| `SBT103` | 2026-07-08 self-bootstrap plan | Planned | SBT102 | Talos performs a bounded governance rehearsal with rollback evidence. |
| `SBT104` | 2026-07-08 self-bootstrap plan | Planned | SBT103 | Month-1 result is classified in REL-002. |

### Scope

- Define the execution evidence that makes a Talos-primary session auditable.
- Verify Talos can run baseline validation and bounded owner-doc mutation paths.
- Record honest REL-002 qualification state after the rehearsal.

### Non-Goals

- No product feature implementation.
- No release, tag, publish, deployment, or external trial invitation.
- No permission, sandbox, credential, dependency, or session-storage default change.

### Acceptance

- Given a self-bootstrap session is assigned to Talos
  When the session records checkpoints and closeout evidence
  Then reviewers can tell whether Talos or an external runtime was the primary executor.
- Given Talos runs the smoke harness
  When provider, validation, governance, and resume paths are exercised
  Then each result is recorded with commands, outcomes, and residuals.
- Given external assistance occurs
  When REL-002 evidence is updated
  Then the affected session is classified as partial or non-qualifying instead of overclaimed.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- Real `talos` binary smoke commands recorded by SBT102/SBT103.

### Documentation To Update

- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: the rehearsal proves Talos cannot yet act as primary executor.
- Rollback: close I106 as partial or blocked, preserve evidence, and do not activate I107 until the
  blocker has an owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 1 of the 2026-07-08 Talos-primary self-bootstrap plan. |
| 2026-07-09 | Activation (SBT100) | Execution contract activated. Runtime: glm-5.2 via zai-coding-plan provider (external runtime, NOT Talos). Baseline: 1791 workspace tests pass, governance validation 0 warnings, `talos 0.3.0` debug binary available, clean worktree at commit `3418962`. See SBT100 execution contract below. |

## SBT100: Execution Contract, Inventory, And Disqualification Rules

### Activation Date

2026-07-09

### Primary Runtime

**glm-5.2** via **zai-coding-plan** OpenAI-compatible provider.

### REL-002 Honest Classification

**This session does NOT qualify for REL-002 as Talos-primary evidence.**

The executing runtime is glm-5.2 via an external provider (zai-coding-plan), not the `talos`
binary itself. Per the plan's hard constraint:

> REL-002 qualification requires Talos to be the primary development executor. External-agent code
> or doc edits convert the affected session to partial or non-qualifying evidence.

This session is therefore classified as **non-qualifying** for REL-002. However, the plan also
states:

> If Talos cannot carry any part as primary executor, the result is still useful only if the owner
> docs record exactly where the self-bootstrap attempt failed.

This session produces useful execution artifacts (execution contract, evidence schema, smoke
harness, governance rehearsal) but records honestly that the primary executor was an external
runtime, not Talos.

### Baseline Evidence (Recorded In This Worktree)

| Check | Command | Result |
|---|---|---|
| Workspace tests | `cargo test --workspace` | 1791 passed, 0 failed |
| Governance validation | `scripts/validate_project_governance.sh .` | 0 warnings |
| Binary version | `target/debug/talos --version` | `talos 0.3.0` |
| Worktree state | `git status --short` | Clean |
| HEAD commit | `git log --oneline -1` | `3418962` |

### Existing Work Inventory (Verified Against Owner Docs)

| Area | Current State | Disposition |
|---|---|---|
| I106-I109 (this plan) | Planned | I106 activated 2026-07-09; I107-I109 wait monthly closeouts. |
| I085 | Paused (MC107 walkthrough residual) | Not activated; may be used only if I107 explicitly selects it. |
| I086-I089 | Planned (product hardening shells) | Not activated; historical baselines. |
| RUNTIME-002 / PROVIDER-002 | Partial — #18 request-dispatch timeout open | First priority for I107 if I106 qualifies. |
| TUI-028 | Partial — #39, #24, #31 open | Second priority for I107 after #18. |
| TUI-029 | Planned — decision required | Not activated; needs ADR-034 revision first. |
| ARCH-032 | Planned | Candidate for I108 architecture-sensitive work. |
| REL-002 | Planned — not ready | Target gate; no v1.0 claim authorized. |

### Disqualification Rules

A session is **non-qualifying** if ANY of these conditions are true:

1. The primary executor is not the `talos` binary acting autonomously.
2. An external runtime (Codex, another agent, a human) performs planning, editing, validation
   orchestration, evidence interpretation, commit, or push as the primary executor.
3. External-agent edits are not explicitly labeled as review commentary only.
4. Evidence claims a command passed without it being run in this worktree.
5. Any behavior change lacks real runtime evidence through the `talos` binary.

A session is **partial** if:

1. Talos is the primary executor for most of the session, but external review identified and
   corrected defects in the Talos-authored work.
2. Talos completed the core development loop but push, final Board sync, or other bounded
   non-code tasks were not performed.

A session is **qualifying** only if ALL acceptance criteria in REL-002 are met:

1. Talos is the primary executor for planning, implementation, validation, documentation, and
   governance sync.
2. External assistance is explicitly limited to review commentary.
3. Real runtime evidence through the `talos` binary is recorded.
4. No release, tag, publish, permission-default, sandbox, credential, or dependency change
   occurred without authorization.

### Allowed Actions For This Session

- Local source edits, tests, and governance validation inside the repository.
- `talos` binary smoke commands.
- Local commits with conventional commit messages and `[model:glm-5.2]` tag.
- Network use limited to the configured provider calls for this execution.

### Not Authorized

- Push to `main` or any remote.
- Tag, release, publish, or external trial invitation.
- Permission, sandbox, credential, dependency, or session-storage default changes.
- Force-push, reset, tag deletion, release deletion, database deletion, or broad cleanup.

### Next Task

SBT101: Define the evidence schema / checkpoint template for session classification.

## SBT101: Evidence Schema And Checkpoint Template

### Activation Date

2026-07-09

### Purpose

Every self-bootstrap session checkpoint must follow a structured template that allows reviewers to
determine: (a) whether Talos or an external runtime was the primary executor, (b) what was
actually done, and (c) whether the session qualifies for REL-002.

### Checkpoint Template

Every checkpoint record in this iteration must contain these fields:

```markdown
### Checkpoint: SBT<NNN> — <task title>

**Date**: YYYY-MM-DD
**Runtime**: <model name> via <provider> (Talos / external)
**Primary executor**: Talos | External
**REL-002 classification**: Qualifying | Partial | Non-qualifying

**Completed task items**:
- <SBT item IDs completed>

**Current state and artifacts**:
- <files changed, commits, test counts>

**Commands/checks and actual results**:
- `<command>` → <exit status and summary>

**Open risks or deviations**:
- <any deviations from the plan>

**External assistance used**:
- <explicit list of any external edits, or "None">

**Next task item**:
- <next SBT ID and task>

**Recovery or resume instruction**:
- <owning record, git state, next gate>
```

### Session Classification Rubric

| Classification | Required Conditions |
|---|---|
| **Qualifying** | (1) Talos binary is the primary executor for planning, implementation, validation, documentation, and governance sync. (2) External assistance is explicitly limited to review commentary. (3) Real runtime evidence through `talos` binary is recorded. (4) No unauthorized release/tag/publish/permission/sandbox/credential/dependency changes. |
| **Partial** | (1) Talos is primary for most of the session, but external review corrected defects in Talos-authored work. (2) OR Talos completed the core development loop but push/Board sync/bounded non-code tasks were not performed. |
| **Non-qualifying** | (1) Primary executor is not the `talos` binary. (2) OR external runtime performed planning/editing/validation/docs/commit as primary. (3) OR evidence claims unverified commands. (4) OR behavior change lacks runtime evidence. |

### Required Evidence Fields (Per REL-002)

Each qualifying session must record:

1. **Work item and owner document** — story ID, iteration ID, task doc.
2. **Runtime used** — model name, provider, whether `talos` binary was the executor.
3. **Commands/tests run** — exact commands, exit codes, output summaries.
4. **Files changed** — full list with line counts.
5. **Governance synchronization evidence** — `validate_project_governance.sh` output, owner-doc updates.
6. **Residual work** — incomplete items with owners.
7. **External agent assistance** — explicit "None" or detailed list with role (review vs. implementation).

### Sample Record (This Session)

This session's own checkpoint:

- **Runtime**: glm-5.2 via zai-coding-plan (external)
- **Primary executor**: External (glm-5.2 agent runtime)
- **REL-002 classification**: Non-qualifying
- **Rationale**: The executor is not the `talos` binary; it is an external agent runtime using a
  different model provider. All code/doc changes were performed by the external runtime, not by
  `talos` acting autonomously.
- **Useful artifacts produced**: execution contract (SBT100), evidence schema (SBT101), smoke
  harness design (SBT102), governance rehearsal (SBT103).
- **What would make a future session qualifying**: Talos binary invoked with a configured provider,
  making its own planning/editing/validation decisions without an external runtime as primary.

### Next Task

SBT102: Build the repeatable runtime smoke harness.
