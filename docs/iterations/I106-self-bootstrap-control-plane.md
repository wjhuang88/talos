# Iteration I106: Self-Bootstrap Control Plane

> Document status: Review
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

## SBT102: Repeatable Runtime Smoke Harness

### Activation Date

2026-07-09

### Deliverable

`scripts/talos_smoke.sh` — a repeatable, non-mutating smoke harness for Talos development sessions.

### Smoke Coverage

| # | Surface | Command | Result |
|---|---|---|---|
| 1 | Version | `talos --version` | `talos 0.3.0` |
| 2 | Validation plan (read-only) | `talos validate plan` | Lists `cargo fmt`, `cargo check`, `cargo test --workspace`, and governance validation |
| 3 | Validation run (governance profile) | `talos validate run --profile governance` | `governance_validation` passed, exit_status: 0 |
| 4 | Governance status | `talos --governance-status` | Manifest/profile/board/validation/Git state present; I106 shown as active |
| 5 | Governance iteration-record preview (dry run) | `talos governance iteration-record preview ...` | Mutation preview shown; **no file modification** verified via `git diff` |
| 6 | Mock provider (print mode) | `talos -p --mock --no-init --no-context "Say hello"` | Mock LLM produced output |
| 7 | Session list (resume evidence) | `talos --list --limit 3` | Session list with IDs, message counts, timestamps |
| 8 | Config list (secret masking) | `talos --config-list` | Secrets masked as `***` |

### Harness Run Evidence

```
scripts/talos_smoke.sh
```

Output (2026-07-09):
```
Passed: 9
Failed: 0
Skipped: 0
```

### Provider Failure Coverage

Provider failure paths are covered by the checked-in integration test suite (no real API key
needed for smoke):

| Smoke Surface | Test Command | Result |
|---|---|---|
| Provider error after tool result | `cargo test -p talos-cli --bin talos -- conversation_loop_clears_processing_on_provider_error_after_tool_result` | ✅ Clears `is_processing`, emits terminal status |
| Provider error visible signals | `cargo test -p talos-cli --bin talos -- conversation_loop_emits_visible_error_signals_on_terminal_failure` | ✅ Emits `Tip`, `Stream`, and terminal `Status` |
| SSE error chunk terminal error | `cargo test -p talos-provider openai::tests::parse_sse_stream_error_chunk_emits_terminal_error` | ✅ Error chunk produces terminal `AgentEvent::Error` |

### Session Resume Coverage

| Smoke Surface | Test Command | Result |
|---|---|---|
| Resume invalid/nonexistent sessions | `cargo test -p talos-cli --bin talos -- session_manager_resume` | ✅ Rejects invalid sessions |
| History hydration on resume | `cargo test -p talos-agent session::tests::test_initial_history_from_jsonl_resume` | ✅ Hydrates persisted history |
| Model metadata preserved on resume | `cargo test -p talos-cli --bin talos -- session_model_metadata_overrides_config_on_resume` | ✅ Session model metadata overrides config |

### Design Properties

- **Non-mutating**: no files written, no commits, no push, no real API calls (mock provider).
- **Repeatable**: deterministic output; safe to run any number of times.
- **Self-contained**: all checks use only the `talos` binary and local repo state.
- **No external dependencies**: no network, no API keys, no paid resources.
- **Secret-safe**: config list masking verified.

### Next Task

SBT103: Perform bounded governance rehearsal with rollback evidence.

## SBT103: Bounded Governance Rehearsal With Rollback Evidence

### Activation Date

2026-07-09

### Purpose

Prove that the `talos governance iteration-record write` path can safely mutate an owner doc,
that post-write governance validation runs, and that a rollback restores the pre-write state
cleanly.

### Rehearsal Steps And Evidence

#### Step 1: Preview (read-only)

```
target/debug/talos governance iteration-record preview \
  --iteration I106 \
  --date 2026-07-09 \
  --record-type execution \
  --record "SBT103 governance rehearsal: exercising iteration-record write path"
```

Result: `Mutation Preview` shown, row formatted correctly, no file modified.

#### Step 2: Write (mutation)

```
target/debug/talos governance iteration-record write \
  --iteration I106 \
  --date 2026-07-09 \
  --record-type execution \
  --record "SBT103 governance rehearsal: exercising iteration-record write path" \
  --confirm-preview
```

Result: `Write: applied`, `Validation: passed`.

Diff produced:
```
+| 2026-07-09 | Execution | SBT103 governance rehearsal: exercising iteration-record write path |
```

The row was correctly appended to the top of the Actual Activation And Execution table in
`docs/iterations/I106-self-bootstrap-control-plane.md`.

#### Step 3: Post-write governance validation

```
scripts/validate_project_governance.sh .
```

Result: `Governance validation passed: 0 warning(s).`

#### Step 4: Rollback

```
cp /tmp/I106-before-sbt103-write.md docs/iterations/I106-self-bootstrap-control-plane.md
```

Result: `git diff --stat` shows no changes; `git status --short` clean. The file was restored to
its pre-write state.

#### Step 5: Post-rollback governance validation

```
scripts/validate_project_governance.sh .
```

Result: `Governance validation passed: 0 warning(s).`

### Findings

1. **Write path is correct**: the governance tool appends an execution row to the correct table
   in the correct owner doc, identified by iteration ID.
2. **Post-write validation is automatic**: the tool runs governance validation after writing and
   reports the result. This is a safety gate — if validation failed, the tool should have rolled
   back automatically (per I096 design).
3. **Manual rollback is clean**: restoring the pre-write file state produces a clean worktree with
   no governance validation warnings.
4. **The rehearsal row was intentionally rolled back**: it was a dry-run exercise, not a permanent
   record. The real SBT103 evidence is this section, added through normal file editing and
   committed via git.

### REL-002 Classification Note

This rehearsal proved the governance mutation mechanism works. However, the rehearsal itself was
driven by an external runtime (glm-5.2), not by the `talos` binary acting autonomously. The
evidence is therefore non-qualifying for REL-002 but demonstrates that the mechanism is ready for
a Talos-primary session to use.

### Next Task

SBT104: Month-1 closeout classified for REL-002.

## SBT104: Month-1 Closeout Classified For REL-002

### Activation Date

2026-07-09

### Validation Matrix (Run In This Worktree)

| Check | Command | Result |
|---|---|---|
| `cargo fmt --all -- --check` | format check | ✅ Clean (after fixing pre-existing `bash_tool.rs` formatting) |
| `cargo check --workspace` | workspace compilation | ✅ Passed |
| `cargo test --workspace` | workspace tests | ✅ 1791 passed, 0 failed |
| `cargo clippy --workspace -- -D warnings` | lint check | ✅ Passed, no warnings |
| `scripts/validate_project_governance.sh .` | governance validation | ✅ 0 warnings |
| `git diff --check` | whitespace/conflict check | ✅ Clean |
| `scripts/talos_smoke.sh` | runtime smoke harness | ✅ 9/9 checks passed |

### Commits In This Session

| Commit | Task | Description |
|---|---|---|
| `81854af` | SBT100 | Execution contract, inventory, disqualification rules. I106 activated. |
| `66a46a8` | SBT101 | Evidence schema and checkpoint template for session classification. |
| `785e4af` | SBT102 | Repeatable runtime smoke harness (`scripts/talos_smoke.sh`). |
| `d7e4964` | SBT103 | Governance rehearsal with rollback evidence. |
| *(this commit)* | SBT104 | Month-1 closeout, REL-002 classification, pre-existing fmt fix. |

### REL-002 Classification

**Classification: Non-qualifying.**

**Rationale**: The primary executor for this session was glm-5.2 via zai-coding-plan (external
runtime), not the `talos` binary itself. All planning, file editing, validation orchestration,
evidence interpretation, and commits were performed by the external runtime. The `talos` binary
was used as a validation subject (smoke commands, governance commands) but not as the autonomous
primary executor.

**What was proven**:
1. The execution evidence harness can distinguish qualifying, partial, and non-qualifying sessions
   (SBT100/SBT101).
2. The `talos` binary has a repeatable smoke harness covering version, validation, governance
   preview/write, provider mock, session list, and secret masking (SBT102).
3. The governance mutation path works correctly: preview → write → post-write validation →
   rollback (SBT103).
4. The validation matrix (fmt, check, test, clippy, governance, diff-check) passes cleanly.

**What was NOT proven**:
1. Talos did not act as the primary development executor.
2. No Talos-autonomous planning, implementation, or evidence interpretation occurred.
3. The REL-002 acceptance criteria remain unmet.

**Useful artifacts for future qualifying sessions**:
- `scripts/talos_smoke.sh` — ready for any Talos-primary session to run.
- Evidence schema and checkpoint template — ready for any Talos-primary session to use.
- Governance mutation path — proven safe and rollback-capable.

### Pre-Existing Fix

`crates/talos-tools/src/bash_tool.rs:585-588` had a pre-existing `cargo fmt` violation (multi-line
condition that rustfmt wanted on a single line). Fixed during closeout to make the validation
matrix fully green. No behavior change.

### Iteration Status

I106 moves to **Review**. All SBT100-SBT104 tasks are complete. The deliverable — execution
contract, evidence schema, smoke harness, and governance rehearsal — is verified. The REL-002
classification is non-qualifying, recorded honestly.

### Next Iteration

I107 (Talos-Primary Feature Polish) may be activated after I106 Review is accepted. Per the plan's
default decisions, I107 must first address #18 request-dispatch timeout before lower-priority
polish. However, I107 activation requires a Talos-primary runtime; if the runtime remains external,
I107 will also be non-qualifying.

## Verification Evidence

- SBT100-SBT104 complete; validation matrix green; REL-002 classification recorded as
  non-qualifying.

## Variance And Residuals

- Pre-existing `bash_tool.rs` fmt violation fixed during closeout (not a session variance).
- Push to remote not performed (not authorized).
- I107-I109 remain Planned pending I106 Review acceptance and Talos-primary runtime availability.

## Retrospective

**What worked**:
- The execution contract and evidence schema provide a clear, honest framework for classifying
  self-bootstrap sessions. The non-qualifying classification was immediate and unambiguous because
  the runtime was external.
- The smoke harness is lightweight (9 checks, <30 seconds) and non-mutating, making it safe to run
  repeatedly.
- The governance mutation path is well-designed: preview → confirm → write → validate → rollback
  is a clean safety chain.

**What didn't work**:
- The session cannot qualify for REL-002 because the runtime is external. This is a structural
  limitation, not a process failure — the plan correctly anticipated this and requires honest
  classification.
- A pre-existing `cargo fmt` violation was discovered during closeout validation. This suggests the
  previous session's closeout validation may not have included `cargo fmt --check`, or the violation
  was introduced after the last fmt pass.

**Lessons for future sessions**:
- Always run `cargo fmt --all -- --check` as the first validation step, not just at closeout.
- The smoke harness should be run at the start of every self-bootstrap session to establish a
  runtime baseline.
- A Talos-primary session requires the `talos` binary to be invoked with a configured provider,
  making its own autonomous decisions through the full development loop.
