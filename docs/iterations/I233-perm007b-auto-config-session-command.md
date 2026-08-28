# Iteration I233: Auto Configuration And Session Command

> Document status: Complete / Closed
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
| Implementation PR | #428 (merged as `c536e190e63ec7a3aed3c54c726ca6d82d054d75`) |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Complete; PERM-007-C remains separately gated and no model/permission-result authority was transferred. |

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
| I233 | Complete / Closed | Claim #426 merged as `7f47f9c3`; implementation PR #428 merged as `c536e190`. |

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

PERM-007-C must separately define and review the model-assessment seam before any permission
request can trigger model judgment: the temporary context must be minimal, redacted and bound to
the exact normalized request, policy/mode generation and session; conversation, credentials,
untrusted raw tool input and provider reasoning are excluded. A model result is only a bounded
one-shot suggestion after deterministic eligibility and independent schema/digest validation; it
cannot authorize itself, create grants or replace the authoritative permission pipeline.

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
| 2026-08-28 | Claim effective | PR #426 exact head `996a63f8` passed CI `33153127031`, independent Agent-role review and CAS, then merged as `7f47f9c3`; implementation starts from that merge. |
| 2026-08-28 | Local implementation | Added default-on `auto.enabled`, CLI config get/set, session-only `/auto` status/on/off, TUI registry wiring and lifecycle-by-rebuild reset. No model request, resolver, permission decision or execution path changed. |
| 2026-08-28 | Lifecycle correction | Independent review found the long-lived conversation engine did not clear session override after runtime replacement. The implementation now clears it only after an authoritative new command sender/generation is published, with a regression test; failed transitions retain the prior session state. |

## Verification Evidence

- `cargo test -p talos-config -p talos-conversation --locked`: 225 + 171 tests passed.
- `cargo test -p talos-cli --locked`: 358 unit tests and all CLI integration tests passed.
- Session state is owned only by each `ConversationEngine`; new/resume/fork runtime rebuilds create a
  fresh engine from current config, so no override is serialized or inherited.
- Public `Config` exhaustive literals must add `auto: AutoConfig::default()` or use
  `..Config::default()`. This source migration is recorded for the next compatible workspace
  version; release/version/tag/publication remain outside I233.

## Completion Evidence

- Completion Commit: `c536e190e63ec7a3aed3c54c726ca6d82d054d75` (implementation merge for PR #428).

## Variance And Residuals

- PERM-007-C and D remain separately blocked and unclaimed; this iteration grants no model or permission-result authority.
