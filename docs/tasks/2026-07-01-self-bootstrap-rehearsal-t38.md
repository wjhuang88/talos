# 2026-07-01 Self-Bootstrap Rehearsal: Evidence Record Baseline

**Rehearsal number**: 1
**Plan item**: T38
**Session date**: 2026-07-01
**Runtime**: Talos 0.2.0 on Darwin arm64
**Change type**: documentation-only
**External assistance**: labeled below

## Objective

Create the first self-bootstrap rehearsal evidence record using the T11 template and evaluate how
much of the session was actually performed through Talos as the primary runtime.

## Scope

- **In scope**: Create this evidence record; label external assistance; record validation evidence
  from the current Month-2 recovery session.
- **Out of scope**: Claim REL-002 compliance; commit or push changes; run a real release; implement
  browser, plugin, vector, local-model, or remote-control runtime dependencies.

## Environment

- Talos version: `talos 0.2.0`
- Provider/model used: external Codex coding agent, model GPT-5; Talos was not the primary agent
  runtime for this rehearsal.
- Workspace: `/Users/GHuang/WorkSpace/RustProjects/talos`
- Starting commit: `f6eade47418a6e86249f95b10a22d2b7f4562753`

## Execution Record

| Step | Tool(s) used | Outcome | Notes |
|---|---|---|---|
| 1 | `sed`, `rg`, `git status` | success | Read the four-month plan, current board, I075, and T11 evidence template. |
| 2 | `cargo run -p talos-cli -- --version` | success | Runtime smoke produced `talos 0.2.0`. |
| 3 | `apply_patch` | success | Created this documentation-only evidence record. |
| 4 | validation commands | success | Current session validation evidence recorded below. |

## External Assistance

| Step | What was needed | Who provided it | Why Talos could not |
|---|---|---|---|
| 1-4 | Planning, file edits, and command orchestration for this rehearsal record | External Codex coding agent | The current session was not running inside Talos as the primary agent runtime; Talos CLI was available only as a binary under test. |

## Validation Evidence

- `cargo check --workspace`: pass in current recovery session.
- `cargo test --workspace`: not run for T38; Month-2 closeout T39 owns the full workspace test.
- `scripts/validate_project_governance.sh .`: pass, 0 warnings in current recovery session.
- Other: `cargo run -p talos-cli -- --version` passed and printed `talos 0.2.0`.

## Commit

- Commit SHA: N/A; current changes are not committed.
- Commit message: N/A; no commit was requested in this session.
- Files changed: this evidence record plus T27/T36 code and governance documentation changes in the
  same uncommitted working tree.

## Gaps Exposed

| Gap | Severity | Blocking REL-002? | Recommended fix |
|---|---|---|---|
| Rehearsal was orchestrated by an external Codex agent, not Talos as the primary runtime. | high | yes | Run T52 through Talos itself where possible, at minimum using Talos request-preview/runtime paths to drive a small code or docs slice. |
| Evidence template works for negative evidence, but commit metadata is unavailable before commit. | low | no | Fill commit fields after the final commit, or allow evidence records to explicitly use `N/A` for uncommitted rehearsal checkpoints. |
| Full workspace test was intentionally deferred to T39 closeout. | medium | no | T39 must run `cargo test --workspace` and record counts before Month-2 closeout. |

## Assessment

- **Self-bootstrap coverage**: 10% estimate. Talos provided CLI/runtime smoke validation, but an
  external agent performed planning, editing, and command orchestration.
- **Would this rehearsal satisfy REL-002?**: No. REL-002 requires Talos as the primary development
  runtime; this record is useful gap evidence only.
- **Ready for the next rehearsal level?**: Conditionally. T52 should require a true Talos-driven
  path or explicitly record why Talos cannot yet drive the session.

## Recovery

- To resume: `git checkout f6eade47418a6e86249f95b10a22d2b7f4562753` and read this evidence record.
- Next rehearsal should attempt: a small code or documentation slice driven through Talos runtime
  paths, with external assistance minimized and labeled.
