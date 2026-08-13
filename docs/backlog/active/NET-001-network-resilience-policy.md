# NET-001: Network Resilience Policy

| Field | Value |
|---|---|
| Story ID | NET-001 |
| Source Issue | #199 |
| Status | Intake |
| Priority | P1 |
| Type | Architecture / Reliability / Network Resilience Epic |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #199 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | Refine the inventory and ADR-backed child slices, then establish a non-overlapping claim before implementation. |

## Identity / Goal / Value

Talos needs one explicit resilience boundary for outbound provider, model-probe, first-party network
tool, and Talos-owned remote-client operations. The outcome must combine replay safety, bounded
retry, cancellation-aware backoff, per-target circuit breaking, and redaction-safe diagnostics
without duplicating remote side effects or hiding terminal failure.

This is an intake owner only. It does not authorize runtime, provider, tool, MCP, dependency,
configuration, protocol, or public-API changes.

## Scope

- Inventory every Talos-owned outbound network path, its current timeout/retry owner, streaming
  commit point, target identity, replay safety, idempotency support, and caller-visible failure.
- Decide one ADR-backed retry/circuit composition, failure taxonomy, deadline/cancellation policy,
  breaker scope, half-open behavior, and server-delay-hint policy.
- Decompose implementation into independently testable children for the shared mechanism and each
  provider/tool/protocol adoption surface.
- Define deterministic tests for jitter, deadlines, cancellation, breaker state, stream commit
  points, ambiguous side effects, redaction, and absence of nested retry multiplication.

## Exclusions

- No retry, circuit breaker, HTTP client, provider, tool, MCP, Session, runtime, or TUI code.
- No automatic replay of side-effecting or unknown operations.
- No new end-user tuning surface, dependency, background scheduler, or persistent breaker state.
- No implementation authority for MODEL-011, MODEL-012, PROVIDER-003, SERVER-001, ACP, A2A, or
  another related owner.

## Dependencies And Decision Gates

- Coordinate with MODEL-011/#124, MODEL-012/#146, PROVIDER-003/#132, SERVER-001/#142,
  SESSION-008, RUNTIME-005, and existing provider reliability owners.
- Accept the shared resilience ADR and inventory before selecting implementation children.
- Any public API break requires its own ADR and migration plan; native/C integration remains subject
  to the repository failure-containment boundary.

## Uncertainty And Validation Path

- Current outbound path coverage, nested retry ownership, and protocol-specific replay semantics
  remain unverified until the inventory is reviewed against the workspace.
- Breaker accounting and target partitioning remain design decisions, not accepted behavior.
- Refinement must identify a runnable first child and preserve unrelated owners' authority.

## State / Status Owners

- This owner is authoritative for NET-001 scope and delivery state.
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the issue matrix are derived views.
- GitHub Issue #199 remains the discussion and intake surface.

## User-Facing Documentation

- No user-visible behavior is delivered at Intake.
- Future behavior changes must update configuration, provider/tool, diagnostics, and operational
  documentation actually affected by the selected child.

## Required Reads

- GitHub Issue #199
- `AGENTS.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/MODEL-011-custom-model-capability-probe.md`
- `docs/backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md`
- `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`

## Acceptance For Refinement

- [ ] A reviewed inventory maps every Talos-owned outbound network path and current retry/timeout
      ownership without presenting unknown coverage as complete.
- [ ] An accepted ADR defines replay safety, bounded retry, jitter, cancellation/deadline,
      streaming commit points, breaker scope/state/accounting, redaction, and composition order.
- [ ] Executable children have non-overlapping boundaries, dependencies, tests, documentation
      targets, residual destinations, and effective claims before implementation.
- [ ] Governance validators pass and Issue #199 remains synchronized with this owner.
