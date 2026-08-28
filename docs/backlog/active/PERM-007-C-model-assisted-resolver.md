# PERM-007-C: Bounded Model-Assisted Permission Resolver

| Field | Value |
|---|---|
| Story ID | PERM-007-C |
| Type | Permission / Security / Runtime Story |
| Priority | P1 |
| Status | Active / Claimed (proposed; ineffective until claim merge) |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | I234 |
| Depends On | ADR-064 Accepted; PERM-006-A/B/C and PERM-007-B complete |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline permission session |
| Work Slice | Deterministic ADR-064 create-only eligibility, isolated redacted evaluator, closed output validation, one-shot admission binding, audit and circuit breaker. |
| Claimed At | 2026-08-28 |
| Source Issue | #188 |
| Governance Claim PR | #431 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-064 `c129d4a5`; protected permission/security scope requires exact-head independent review. |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | No implementation merge without exact-head permission/security review, CI and CAS. |

## Contract

The resolver runs only after the authoritative PERM-006-C pipeline returns `Ask`. Deterministic code,
not the model, establishes eligibility. The first allow class is one native workspace-local Write
creating one absent structured text file in an isolated non-`main` managed worktree. The request is
bound to a trusted open parent capability, normalized path, absent target, policy revision, mode
generation and session. Creation must be atomic and no-clobber.

The evaluator is tool-free, non-recursive, single-call and bounded. Its redacted closed output can
only be `AllowOnce` or `HumanRequired`; high confidence and the exact allow reason are required.
Every other result, including timeout, provider failure, schema mismatch, injection indicator,
uncertainty or stale digest, falls back to human approval or headless Deny. No model output can
create grants, alter resources or override Deny.

## Acceptance And Exclusions

Use the acceptance and exclusions in I234 and ADR-064 as normative. Add adversarial tests for Deny
precedence, scope mutation, secrets, replay, races, circuit thresholds and headless behavior. Keep
PERM-007-D, existing-file modification, sandbox fallback, Execute/Network, Dashboard/Desktop and
release/publication outside this story.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as implementation evidence.
