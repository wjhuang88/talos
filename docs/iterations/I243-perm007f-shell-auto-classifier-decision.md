# Iteration I243: Shell Auto Classifier Decision

> Document status: Active / Claimed (proposed; ineffective until claim PR #463 merges)
> Planned objective: decide the classifier context, precedence, trust configuration, exact-request
> binding, rollback, and migration contract needed for Claude-like shell auto mode.
> MVP deliverable: an accepted ADR-070 and security matrix that can authorize a separate runnable
> implementation iteration without changing production behavior.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-F0 | PERM-007-F / Issue #462 | Refinement / Unclaimed | I241, ADR-012, ADR-040, ADR-069 | Decision-only classifier and migration contract |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-09-01 |
| Work Slice | Decision-only ADR-070 classifier context, precedence, exact-request binding, migration/rollback contract, Issue #56/#57 authority reconciliation, and threat matrix. No Rust, Cargo, config schema, runtime behavior, UI, release, or publication authority. |
| Claimed At | 2026-09-01 |
| Source Issue | #462 |
| Governance Claim PR | #463 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer direction in the 2026-09-01 mainline session authorizes a generic model-classifier decision instead of per-command exceptions. Independent exact-head permission/security/API review, CI, validators and merge-time CAS remain mandatory; the proposed claim is ineffective until #463 merges. Shared GitHub identity provides Agent-role separation only, not natural-person identity separation. |
| Implementation PR | None |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | Independent permission/security/API review must accept ADR-070 before I244 claim preparation. |

## Current Nonterminal Inventory And Disposition

| State | Iterations | Disposition |
|---|---|---|
| Active | I241 on the planning baseline | Implementation is merged; finish owner-first closeout PR #461 before selecting I243. Do not resume code work. |
| Review | None | No review work is displaced. |
| Planned | I207, I208 | Preserve unclaimed steering work; no permission overlap. |
| Blocked | None with an iteration-owner status | PERM-006-D/E are adjacent backlog authorities, not silently absorbed. |
| Paused | I164 | Superseded; do not restore. |

I235, I236 and I242 are Complete/Closed. Derived rows that still describe I235/I236 as Active are
drift to repair in this governance stage, not authorization to restart them.

### 2026-09-01 Exact-Base Checkpoint

The planning baseline above is preserved as the pre-closeout inventory. PR #461 subsequently
merged as `93c4377e`, so `main@93c4377e` records I241 and PERM-007-E Complete/Closed with generic
shell behavior explicitly left to Issue #462. At this checkpoint there is no Active or Review
iteration; I207 and I208 remain Planned/Unclaimed, I164 remains Paused/superseded, and I244 remains
Blocked/Unclaimed. I243 may become Active only through its own atomic decision claim merge.

### Proposed Atomic Activation Through PR #463

PR #463 proposes I243 as the sole Active iteration and claims only the decision surface stated
above. The proposal has no target-branch effect before merge. I244 and PERM-007-F remain
Blocked/Unclaimed, and ADR-069 remains the executable authority while ADR-070 is Proposed.

## Decision Questions

- Which deterministic `deny` and explicit `ask` rules run before the classifier?
- Which exact shell/tool inputs, current user intent, workspace/remotes, and trusted-environment
  facts enter the isolated, tool-free classifier context?
- How are likely secrets, credentials, external targets, and protected environments rejected before
  model assessment?
- How do hard-deny, soft-deny, allow-exception, explicit-user-intent, and unknown-result precedence
  interact without letting repository-controlled files add trust?
- How are classifier input, policy revision, session, cwd, environment identity, and the executed
  request bound atomically?
- How do timeout, cancellation, provider error, malformed output, context compaction, and stale
  state fail closed?
- How are existing I241 allowlisted exec behavior and current bash permission templates migrated or
  rolled back?
- Which types remain owned by PERM-006-D / Issue #56, and which cross-surface gates remain owned by
  PERM-006-E / Issue #57, so I244 does not create duplicate public authority?

## Scope Boundary

Decision and threat-model documentation only. Do not modify Rust, Cargo manifests, config schema,
permission behavior, UI, Desktop, release, or publication. ADR-069 remains authoritative until
ADR-070 is accepted.

## Documentation Targets

- ADR-070 and `docs/reference/I243-SHELL-AUTO-CLASSIFIER-THREAT-MATRIX.md` are the decision output.
- README and user-facing `/auto` documentation remain unchanged until I244 implements observable
  behavior.

## Acceptance And Evidence

- The decision explicitly states that AST/effect analysis is advisory evidence, not a proof that an
  arbitrary command is safe.
- The classifier receives enough exact command semantics to make a useful judgment while
  deterministic secret/exfiltration protections remain authoritative.
- Permission `deny` and explicit `ask` precede the classifier; no classifier result can override
  policy, sandbox, grants, or admission.
- The implementation slice, compatibility plan, rollback switch, configuration ownership, and
  adversarial matrix are independently permission/security/API reviewed.
- Governance validators and `git diff --check` pass at exact head.

## Next Step

Prepare ADR-070 and the security matrix, then establish an effective decision-only claim. I244 owns
implementation and requires its own effective claim after ADR acceptance.
