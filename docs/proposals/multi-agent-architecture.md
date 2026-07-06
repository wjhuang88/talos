# Multi-Agent Architecture

## Problem

Single-agent execution concentrates context, tool use, and planning in one loop. Complex work may
benefit from bounded worker agents for search, review, implementation, or validation, but an
uncontrolled multi-agent design risks violating Talos's event-boundary and permission principles.

## Proposed Approach

Keep multi-agent work as a proposal until architecture and permission boundaries are explicit. The
initial candidate is a hierarchical orchestrator/worker model with bounded child contexts, explicit
tool subsets, clear cancellation, and session-visible provenance.

## Alternatives Considered

- Peer-to-peer agent collaboration.
- Shared blackboard coordination.
- Hybrid orchestrator plus shared artifact store.

## Open Questions

- Whether child agents can create further child agents.
- How token/cost budgets are enforced and displayed per child agent.
- How permissions are inherited or narrowed.
- How cancellation and failure propagate.
- How child-agent messages are stored and resumed.
- How the design avoids a global event bus under ADR-006.

## Dependencies

- `RUNTIME-001` runtime facade.
- `PERM-004` workspace trust sandbox.
- `ARCH-032` single-data-flow audit.
- A new ADR before implementation.

## Source

- [GitHub Issue #30](https://github.com/wjhuang88/talos/issues/30)

