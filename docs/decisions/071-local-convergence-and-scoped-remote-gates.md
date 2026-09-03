# ADR-071: Local Convergence And Scoped Remote Gates

## Status

**Accepted for governance workflow** (2026-09-03)

## Context

Remote review is valuable as a stage gate, but it is a poor edit loop. A repository-wide Issue
reconciliation check recently blocked a converged implementation because unrelated Issues were
registered in a separate governance change, forcing a rebase and making valid exact-head evidence
stale without changing the implementation scope.

## Decision

1. **Local convergence is the default loop.** Complete design, implementation, tests, affected
   documentation, owner synchronization, residuals, and staged-diff review locally before the
   first stable push. Intermediate local commits are allowed.
2. **Remote gates are scope-based.** Implementation candidates require exact-head CI and the
   authorization-specific independent review required by their Work Slice. Permission, sandbox,
   process-hardening, release, public API, and security changes retain their applicable hard gates.
   Governance-only candidates run checks relevant to their files.
3. **Global reconciliation is batched.** Open-Issue registration, global owner matrices, Board
   mirrors, and unrelated metadata are separate governance work. They must not block or invalidate
   an unrelated implementation candidate; temporary drift is a separately owned residual.
4. **Evidence invalidation is substantive.** Code, behavior, contract, security scope, acceptance,
   validation evidence, or a relevant target-base change requires fresh exact-head evidence.
   Comments, labels, Issue synchronization, unrelated reconciliation, and Board/matrix batches that
   leave the candidate and relevant base unchanged do not.
5. **One stable candidate, batched corrections.** Fix routine findings locally and submit them
   together. Do not create a new PR for each compile, wording, owner-sync, or reviewer finding.
   A substantive correction produces one new stable candidate and fresh gates.

## Enforcement And Migration

The `remote-governance-reconciliation` job is advisory for implementation PRs while changed-file
routing is being implemented. It remains required evidence for dedicated governance,
release, and closeout reconciliation batches. The PR body must record its result and any residual.
This ADR does not waive relevant security review, exact-head validation, merge-time CAS, or
Completion Commit evidence.

## Consequences

Local iteration is faster and remote rounds are fewer. Global metadata may be temporarily stale,
but the residual and owner are explicit and are closed by a bounded governance batch.
