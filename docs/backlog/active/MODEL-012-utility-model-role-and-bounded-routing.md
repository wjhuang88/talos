# MODEL-012: Utility Model Role And Bounded Routing

| Field | Value |
|---|---|
| Story ID | MODEL-012 |
| Source Issue | #146 |
| Status | Intake |
| Priority | P2 |
| Type | Architecture / Model Routing / Product Interaction Epic |

## Disposition

Register the optional Utility Model request for ADR, task-boundary, compatibility, and evaluation
refinement. The owner must preserve the Primary Model as final decision authority and route all
model calls through canonical provider/configuration and permission boundaries. This intake record
does not authorize a second Agent runtime, hidden sub-agent, or automatic router implementation.

## Required follow-up

- Additive role-reference configuration with Primary-only backward compatibility.
- Explicit task eligibility, capability admission, upward fallback, and bounded cost policy.
- Reusable provider/model factory without duplicated routing or credential ownership.
- Menu-first TUI configuration that cannot accidentally switch the Primary Model.
- Evaluation corpus and rollout gate before enabling additional consumers.

## Dependencies

Coordinate with MODEL-007/MODEL-011, MEM-003, SERVER-001 (#142), AGENT-003, and PERM-006 (#52).
Keep this scope separate from R02 CLI/TUI bridge decomposition.
