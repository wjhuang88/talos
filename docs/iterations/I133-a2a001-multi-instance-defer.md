# Iteration I133: A2A-001 Multi-Instance Discovery — Defer Decision

> Document status: Complete
> Published plan date: 2026-07-16
> Planned objective: Produce a threat model and ADR or defer/reject for multi-instance discovery and communication.
> MVP deliverable: ADR-044 (Defer) with threat model covering identity, authentication, authorization, discovery, credentials, transcript exposure, and retention/revocation.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `A2A-001` | none | Refinement — ADR-gated | RUNTIME-001, REMOTE-001 (both not implemented) | ADR-044 Defer: no product need; all implementation paths violate P140 non-goals; five threat-model risk categories unresolved. |

### Scope

- Threat model for identity, authentication, authorization, discovery, credentials, transcript exposure, retention/revocation.
- Compare explicit host-managed connections with automatic discovery.
- Produce ADR-044 or explicit defer/reject.

### Acceptance

- ✅ ADR-044 defines the decision and credential/transcript exposure boundary.
- ✅ ADR-044 preserves no-global-event-bus boundary and introduces no implicit authority.
- ✅ Protocol work remains deferred; A2A-001 owner doc updated.

### Non-Goals

- No automatic discovery, network service, protocol, credential transfer, or multi-agent orchestration.

## Verification Evidence

- ADR-044 accepted as Defer with threat model and comparison table.
- No code changes (decision-only package).
- Governance validation passes.

## Variance And Residuals

- A2A-001 capability gap (inter-instance communication) remains Open via [Issue #40](https://github.com/wjhuang88/talos/issues/40) and the ADR-044 reversal trigger.
- REMOTE-001 (prerequisite) remains P4 Research with no implementation.

## Retrospective

- Outcome: met. ADR-044 provides a reviewed Defer with threat model.
- Documentation: A2A-001 owner doc, ADR index, Board, iterations README, execution package, PRODUCT-BACKLOG, Issue #40.
- Lessons: Multi-instance communication introduces severe security risks (identity, credential exposure, discovery attack surface) that cannot be mitigated within Talos's current architecture. The explicit host-managed approach is the only compatible path if a future need arises.
