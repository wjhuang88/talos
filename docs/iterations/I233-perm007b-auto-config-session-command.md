# Iteration I233: Auto Configuration And Session Command

> Document status: Active / Claimed (proposed; effective only after PR #426 merges)
> Published plan date: 2026-08-28
> Planned objective: implement the ADR-064 configuration and active-session `/auto` control surface without changing permission decisions.
> Baseline rule: once committed, preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: a runnable CLI/TUI `/auto` command and persisted `auto.enabled` default that report deterministic session state without invoking a model.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline session |
| Work Slice | Implement only PERM-007-B/I233: `auto.enabled` config default/migration and non-persistent session `/auto` status/on/off control. Exclude model requests/resolver/eligibility/grants/permission results/execution, PERM-007-C/D, PERM-006-D/E, sandbox, Dashboard, Desktop, release, publication and dependency changes. |
| Claimed At | 2026-08-28 |
| Source Issue | #188 |
| Governance Claim PR | #426 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; exact-base validators, CI and independent review required before implementation. |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim/activation must reach `main` before implementation; ADR-064 remains normative and no model/permission-result authority is transferred. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-B | PERM-007 / Issue #188 | Ready / Unclaimed | PERM-007-A/I218, ADR-064, PERM-006-A/B/C | Configured default and session `/auto` control are runnable and testable with no model or permission-result change. |

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I233 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I197, I201, I210 | Review / Claimed | Preserve corrective owners and deferred acceptance; no authority transfer. |
| I198 | Terminal / Complete | Preserve the completed Skill compatibility owner and its original/corrective evidence; no authority transfer. |
| I207, I208 | Planned / Unclaimed | Preserve the ordered steering sequence; do not activate. |
| I213 | Terminal / independent | Dashboard lane remains independent; no overlap. |
| I233 | Active / Claimed proposed by #426 | Claim/activation is ineffective until PR #426 merges; implementation starts only from merge or later. |

PRs #120/#121 remain archival Drafts and are not to be resumed. No other open PR owns PERM-007-B;
PERM-007-C/D remain Blocked / Unclaimed. This inventory is a current checkpoint, not a rewrite of
historical Published Baselines.

### Scope

- Versioned `auto.enabled` config, serde default `true`, old TOML compatibility and migration docs.
- Non-persistent session override with `/auto`, `/auto on`, `/auto off` and bounded status output.
- Deterministic lifecycle reset for new/resumed/forked sessions and explicit disabled fallback.

### Non-Goals

- No evaluator/model call, eligibility, grant, permission policy/result, execution, resolver or audit behavior.
- No PERM-007-C/D, PERM-006-D/E, sandbox, Dashboard, Desktop, release, version/tag/publication or dependency work.

### Acceptance

- Omitted config defaults to enabled attempted assistance, not Allow, and existing permission behavior remains unchanged.
- Session override wins over config for the active session only and is not persisted to TOML/transcript.
- `/auto` reports effective state/source and bounded metadata; invalid arguments do not mutate state.
- New/resumed/forked sessions do not inherit stale overrides.

### Planned Validation

- Focused config, CLI/TUI command and session lifecycle tests; schema/migration checks.
- `cargo fmt --all --check`, locked focused/workspace checks and tests, strict Clippy.
- `scripts/validate_project_governance.sh .`, `bash scripts/validate_collaboration_claims.sh .`, `git diff --check`.
- Exact-head CI and independent permission/API/security review after stable candidate push.

### Documentation To Update

- README command/config docs, PERM-007 and PERM-007-B owners, Board, backlog, iteration index, manifest and Issue #188.

### Risks And Rollback

- Risk: default-on is misread as unconditional authorization or session state leaks into persistence.
- Rollback: set `auto.enabled = false` or remove session override while preserving the existing approval path.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-28 | Selection | PERM-006-A/B/C are complete and ADR-064 is Accepted; I233 is selected as the sole PERM-007-B governance/implementation child. |

## Verification Evidence

- Pending effective claim and implementation candidate.

## Completion Evidence

- Completion Commit: Pending.

## Variance And Residuals

- PERM-007-C and D remain separately blocked and unclaimed; this iteration grants no model or permission-result authority.
