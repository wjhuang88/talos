# ADR-069: Model-First Permission Triage For Bounded Requests

**Status:** Accepted / I242 Complete
**Date:** 2026-09-01
**Owners:** PERM-007-E / I241
**Depends on:** ADR-064, ADR-065, ADR-066, ADR-067

## Context

ADR-064 accepts model assistance only for a narrow atomic file-creation case. That boundary is
safe, but it does not address the repeated prompts caused by routine read-only shell and local
validation requests. Expanding the old resolver in place would silently change an accepted
security contract. This ADR therefore defines a separately reviewed follow-up boundary.

## Decision

Permission requests use a three-stage, fail-closed triage:

1. The authoritative deterministic policy evaluates the request first. An explicit `Deny` always
   wins. Deterministically safe classes may proceed without a model call; the model is never used
   to override policy, sandbox, admission, or execution gates.
2. Only an explicitly enumerated low-risk class is normalized, redacted, and sent to a model
   assessor. The assessor may return only `AllowOnce` or `HumanRequired` for the exact request.
3. All other requests remain human-required (or denied by policy). A model timeout, cancellation,
   provider error, malformed output, insufficient context, stale revision, or circuit failure is
   fail-closed and cannot silently authorize execution.

The first implementation slice is limited to read-only shell commands and local validation
commands whose effects are bounded, reversible or observational. Examples include `pwd`, `ls`,
`rg`, `git status`, `cargo fmt --check`, and test or static-check commands. The allowlist is
structural, not a free-form prompt convention. Existing-file writes, deletion, replacement,
network access, credentials, script interpreters, pipelines, redirection, command substitution,
background execution, sandbox fallback, external paths, and unparseable command structures remain
human-required or denied.

## Normalized Request Contract

The model receives a bounded schema rather than an original command or environment dump. It must
include command structure, working-directory class, resource category, read/write and network
effects, reversibility, impact scope, source surface, permission mode, policy revision, and an
exact request digest. Secrets, credentials, full environment contents, raw reasoning, and
unredacted sensitive paths are excluded.

The response is typed and bound to the digest, policy revision, session, mode, and expiry. The
model cannot create permanent grants, widen a resource scope, select a different command, or
authorize a later request. Admission is rechecked by the existing permission pipeline before
execution.

## Trust And User Experience

An admitted result is at most one `AllowOnce`. Any turn/session/workspace trust affordance must be
created by the existing human-controlled permission state, with explicit scope and expiry; the
model cannot create or extend it. Repeated equivalent requests may be deduplicated only while the
bound revision, scope, and normalized structure remain identical.

Model assessment runs off the interactive rendering path where possible. The user sees a concise
reason and impact summary only when human action is required. Existing CLI, TUI, Runtime, and MCP
surfaces retain equivalent permission semantics and audit classification.

## Safety, Rollback, And Validation

The implementation must add adversarial tests for parser ambiguity, shell metacharacters, path and
revision changes, provider failure, cancellation, timeout, malformed output, and prompt leakage.
The feature is disabled by closing the auto circuit or configuration override without changing
the underlying policy. Any unexpected authorization, classifier ambiguity, or cross-surface
divergence blocks acceptance and rolls back to the existing human-required path.

This proposal does not authorize implementation, dependency changes, schema migration, release,
publication, Desktop work, or changes to ADR-064. Those require an effective I241 Collaboration
Claim and a fresh independent permission/security/API review.

## Reversal Triggers

Supersede or withdraw this ADR if exact request binding cannot be maintained, if a model result can
cross a policy/admission boundary, if secrets reach the assessor or audit log, or if the bounded
allowlist cannot be proven stable and fail-closed across supported surfaces.

## Acceptance Evidence

Decision content was introduced by commit `8c570f84`. I242 claim and decision review merged through
PR #457 as `3e42325916e0299864c109704fe782ca96b04d3d`, with exact-head CI `33464911927` and
independent permission/governance review comments `5488254817` (initial decision review) and
`5488336387` (incremental approval) approving exact head
`1d08c760416cd983ab2c00c56d039938a5424dbe` against base `6501167a5f050920089f3345e29e1e7ed1021b7a`.
Together these reviews verified the threat model, normalized contract, fail-closed boundaries,
exclusions and rollback triggers. This acceptance authorizes no implementation; I241 requires its own effective
implementation claim and independent security review.
