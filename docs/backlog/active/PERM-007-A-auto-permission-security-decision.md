# PERM-007-A: Auto Permission Security Decision

**Status**: Complete / Closed

| Field | Value |
|---|---|
| Story ID | PERM-007-A |
| Type | Permission / Security Decision |
| Priority | P0 |
| Status | Complete / Closed |
| Parent Epic | PERM-007 |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | I218 - Complete / Closed |
| Depends On | Existing ADR-011 and current permission/config/command behavior; PERM-006-A/B/C are implementation prerequisites, not decision prerequisites |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-22 |
| Work Slice | Decide only PERM-007-A / I218: threat model and one ADR revising or superseding ADR-011, including eligible decisions, maximum authority, mode precedence, privacy, validation, audit, deadline, circuit-breaker, migration, rollback and bounded B-D implementation children. No Rust/Cargo/config schema, `/auto`, model request, prompt, grant, approval, runtime, sandbox, TOOL-024, Desktop, release or publication implementation. |
| Claimed At | 2026-08-22 |
| Source Issue | #188 |
| Governance Claim PR | #352 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #352 merged as `ca30081a`. Decision PR #353 exact head `a289a07f` passed CI `32505438495`, independent Agent-role security review `5372825090` and merge-time CAS, then merged as `c129d4a5`. |
| Implementation PR | #353 (decision documentation only) |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Closed at Completion Commit `a289a07f`; ADR-064 is Accepted and supersedes ADR-011. PERM-007-B/C/D remain blocked until PERM-006-A/B/C close and separate child claims become effective. |

## Claim Activation Checkpoint — 2026-08-22

PR #352 proposes one atomic claim and activation for decision-only PERM-007-A/I218. Before that PR
reaches `main`, this Claimed/Active record is proposal metadata and grants no authority. After merge,
the executing Agent may create only the threat matrix and Proposed ADR described by the Work Slice.
No executable permission, configuration, command, model-call or grant behavior is authorized.

## Claim Effective Checkpoint — 2026-08-22

PR #352 exact head `13ecbdfa` passed CI `32503441611`, independent Agent-role security/governance
review `5372605087` and merge-time CAS, then merged to `main` as `ca30081a`. PERM-007-A/I218 is
Active/Claimed from that merge. Authority remains limited to the threat matrix and Proposed ADR.

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

- [x] A current-path/threat matrix cites the actual policy, execution, config and command seams.
- [x] A Proposed ADR explicitly accepts, rejects or narrows default-on `auto` per risk class.
- [x] The ADR fixes authority, precedence, privacy, validation, deadlines, audit, circuit-breaker,
      headless, migration and rollback semantics and defines runnable B-D children.
- [x] Independent exact-head security review covers every matrix row and accepts the decision.
- [x] Both governance validators, YAML parsing and `git diff --check` pass with no behavior change.

## Decision Execution Evidence

- Current-path and threat matrix:
  `docs/reference/I218-AUTO-PERMISSION-THREAT-MATRIX.md`.
- Accepted decision: `docs/decisions/064-bounded-model-assisted-auto-permission.md`.
- Decision documentation PR #353 exact head `a289a07f` passed CI `32505438495`, independent
  security review `5372825090` and CAS, then merged as `c129d4a5`.
- ADR-064 proposes default-on *attempted* assistance, not default Allow; only one-shot atomic
  no-clobber creation of a new structured text file under a typed managed-workspace lease is
  initially eligible. Existing-file modification remains human-mediated.
- Execute, Network, external paths, secrets, destructive/binary mutations, sandbox fallback,
  plugin/MCP calls, persistent grants and unmanaged/user-dirty work remain human/headless-Deny.
- This documentation changes no executable behavior and does not authorize PERM-007-B/C/D.

## Completion Evidence

- Completion Commit: `a289a07ff97746d877f3a422d15f8044bbf50ab6`
- The Completion Commit predates this status-only closeout and contains the independently reviewed
  create-only capability-relative no-clobber correction to ADR-064 and its threat matrix.
- PR #353 merged as `c129d4a54e021f50aa7df8ab2040f9abcd8edba7` after exact-head CI
  `32505438495`, independent Agent-role security approval `5372825090` and merge-time CAS.
- ADR-064 is Accepted and ADR-011 is Superseded. No executable behavior is claimed.

## Residual Destination

PERM-007-B/C/D own later behavior only after PERM-006-A/B/C complete and each child obtains an
effective claim. General autonomous policy generation and sandbox redesign remain excluded.
