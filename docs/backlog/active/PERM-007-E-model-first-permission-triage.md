# PERM-007-E: Model-First Permission Triage

| Field | Value |
|---|---|
| Story ID | PERM-007-E |
| Type | Permission / Security / Runtime Story |
| Priority | P0 |
| Status | Refinement / Unclaimed |
| Parent Epic | PERM-007 (closed; this is a separately governed follow-up) |
| Source | GitHub Issue #456 |
| Selected Iteration | None |
| Depends On | PERM-006-C, PERM-007-D, ADR-064, ADR-069 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #456 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-09-01 |
| Handoff / Release Condition | Accept the security contract and establish an effective claim before implementation. |

## Goal And Value

Reduce repetitive human approval interruptions by letting a bounded, redacted model assessor triage
normalized low-risk requests, including a constrained shell/exec subset, without becoming a second
permission engine.

## Scope

- Normalize requests and reuse deterministic classifier results before model assessment.
- Allow only explicitly enumerated low-risk, bounded and reversible classes to return `AllowOnce`.
- Keep exact request binding, timeout/cancellation, circuit breaker, audit redaction and cross-surface
  equivalence.
- Start with read-only and local validation shell commands; expand other classes only by change control.

## Exclusions

No blanket shell approval, permanent grants, destructive or network operations, secrets, script
interpreters, pipes/redirection/substitution, background jobs, sandbox fallback expansion, Desktop,
release or publication work.

## Acceptance

- Given a normalized low-risk command accepted by deterministic policy, when the model returns a
  valid high-confidence result bound to the request, then one `AllowOnce` authorization is admitted.
- Given an ambiguous, high-risk, malformed, stale, timed-out or failed assessment, then the request
  remains human-required or is denied; it is never silently allowed.
- Given equivalent requests on supported CLI/TUI/Runtime/MCP surfaces, then authorization semantics
  and audit classification remain equivalent.
- Given a model or provider failure, then the existing permission pipeline remains available and no
  secret or unrestricted environment content is sent to the model or audit log.

## Required Governance

Accept ADR-069, its threat model, normalized shell schema and rollback
plan before implementation. Required review is independent permission/security/API review.

## Required Reads

- docs/decisions/064-bounded-model-assisted-auto-permission.md
- docs/decisions/069-model-first-permission-triage.md
- docs/backlog/active/PERM-007-C-model-assisted-resolver.md
- docs/iterations/I236-perm007d-cross-surface-conformance.md
- docs/reference/I218-AUTO-PERM-THREAT-MATRIX.md
- crates/talos-agent/src/auto_resolver.rs
- crates/talos-agent/src/permission_pipeline.rs
- crates/talos-permission/
- crates/talos-tools/

## Residual Destination

Unclassified shell structures and any request class not admitted by the new ADR remain human-required
and are recorded as follow-up children rather than widened implicitly.
