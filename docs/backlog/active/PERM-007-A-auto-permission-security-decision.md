# PERM-007-A: Auto Permission Security Decision

**Status**: Ready / Unclaimed

| Field | Value |
|---|---|
| Story ID | PERM-007-A |
| Type | Permission / Security Decision |
| Priority | P0 |
| Status | Ready / Unclaimed |
| Parent Epic | PERM-007 |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | I218 - proposed |
| Depends On | Existing ADR-011 and current permission/config/command behavior; PERM-006-A/B/C are implementation prerequisites, not decision prerequisites |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-22 |
| Work Slice | Decide only PERM-007-A / I218: threat model and one ADR revising or superseding ADR-011, including eligible decisions, maximum authority, mode precedence, privacy, validation, audit, deadline, circuit-breaker, migration, rollback and bounded B-D implementation children. No Rust/Cargo/config schema, `/auto`, model request, prompt, grant, approval, runtime, sandbox, TOOL-024, Desktop, release or publication implementation. |
| Claimed At | Not applicable |
| Source Issue | #188 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head independent security review and target-branch merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Claim and activation must reach `main`; the decision head then requires exact-head independent security review before ADR acceptance. Implementation remains blocked until PERM-006-A/B/C close and separate child claims become effective. |

## Identity / Goal / Value

Resolve the security-policy conflict between Issue #188's requested default-on `auto` mode and
ADR-011 before unattended implementation reaches that gate. The deliverable is an independently
reviewed threat model and accepted decision, not permission behavior.

## Scope

- Characterize current permission, config, slash-command, headless and model-call boundaries.
- Define which authoritative `Ask` outcomes, if any, model assistance may resolve and the maximum
  authority for read-only, write-capable, process, network, external-path and secret-bearing work.
- Define global default, persistent config, per-session `/auto`, Goal and headless precedence.
- Define redacted model input, schema-constrained output, injection resistance, uncertainty,
  timeout, cost, circuit-breaker, revocation, audit, migration and rollback behavior.
- Split configuration/command, bounded resolver and cross-surface conformance into separately
  runnable PERM-007-B/C/D children.

## Exclusions

- No executable, Cargo, dependency, config-schema, command, model-request or permission change.
- No automatic grant, sandbox fallback, Deny override or authority enlargement.
- No PERM-006, TOOL-024, Goal/Desktop/Dashboard, release or publication implementation.

## Security Invariants

- Configured Deny, hard boundaries and authoritative policy outcomes cannot be weakened by a
  model response or by enabling `auto`.
- Model assistance may receive only redacted, bounded context and cannot enlarge the original
  resource, action or lifetime.
- Unavailable, malformed, conflicting, injected or uncertain results fail to human confirmation
  or Deny as selected by the ADR; headless behavior remains deterministic and fail-closed.
- Decision execution and implementation review are separate roles. Shared GitHub identity proves
  Agent-role separation only, not natural-person identity separation.

## Required Reads

- `docs/backlog/active/PERM-007-model-assisted-goal-permission-decisions.md`
- `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`
- `docs/backlog/active/PERM-006-A-structured-permission-decisions.md`
- `docs/backlog/active/PERM-006-B-scoped-grant-store.md`
- `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md`
- `docs/decisions/011-guardian-approval-boundary.md`
- `docs/reference/AUTONOMY-PERMISSION-MATRIX-2026-07-04.md`
- `crates/talos-permission/`
- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-config/src/types.rs`
- `crates/talos-conversation/src/command_registry.rs`

## Acceptance

- [ ] A current-path/threat matrix cites the actual policy, execution, config and command seams.
- [ ] A Proposed ADR explicitly accepts, rejects or narrows default-on `auto` per risk class.
- [ ] The ADR fixes authority, precedence, privacy, validation, deadlines, audit, circuit-breaker,
      headless, migration and rollback semantics and defines runnable B-D children.
- [ ] Independent exact-head security review covers every matrix row and accepts the decision.
- [ ] Both governance validators, YAML parsing and `git diff --check` pass with no behavior change.

## Residual Destination

PERM-007-B/C/D own later behavior only after PERM-006-A/B/C complete and each child obtains an
effective claim. General autonomous policy generation and sandbox redesign remain excluded.
