# Iteration I273: Bounded Model Decisions and Automatic Protocol Selection

> Document status: Active / Claimed
> Parent: PROMPT-001 / Issue #285

## Objective

Deliver a tested bounded model-decision primitive and integrate it with automatic tool-protocol
selection and protocol-failure diagnosis without changing default tool coverage or replaying unsafe
operations.

## Child slices

1. Capability probe and automatic Native/compatibility selection.
2. Typed bounded invocation primitive with isolated context and dedicated tools.
3. Protocol failure classifier/recovery policy and user-visible diagnostics.
4. Auto-permission consumer integration, followed by independent security review.

## Exclusions

No Dashboard changes, no release/publication work, no I270 harness expansion, and no permission
Deny bypass. Existing manual protocol override remains a diagnostic escape hatch during migration.

## Acceptance

- Provider capability is probed or explicitly unknown; selection is deterministic and cached safely.
- Bounded calls enforce deadline, cancellation, budget, retry and recursion limits.
- Sanitized structured outcomes distinguish correction, fallback, stop and human review.
- Write operations are never replayed when execution state is unknown.
- Focused tests cover Native, compatibility, malformed output, timeout, cancellation and denial.
- User-facing diagnostics identify model-assisted decisions without exposing secrets.

## Dependencies and evidence

Depends on ADR-075 acceptance and the I270/I272 prompt-authority contracts. Implementation requires
an effective Collaboration Claim and exact-head permission/security review. Completion requires an
existing implementation commit; a status-only commit is not evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Provider capability evidence, deterministic automatic protocol selection, protocol failure classification, and fail-closed retry contract primitives |
| Claimed At | 2026-09-14 |
| Source Issue | #285 |
| Governance Claim PR | #554 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim PR #554 merged as eb97905d |
| Implementation PR | Local convergence; not pushed |
| Last Updated | 2026-09-14 |
| Handoff / Release Condition | Exact-head CI and independent permission/security/API review before merge. |
