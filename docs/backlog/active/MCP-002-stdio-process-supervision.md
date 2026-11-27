# MCP-002: Supervised stdio MCP process cleanup on macOS

**Status**: Intake / Unclaimed
**Type**: Architecture / API / Process lifecycle Story
**Parent Epic**: None
**Source Issue**: #1033
**Selected Iteration**: None

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #1033 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-27 |
| Handoff / Release Condition | Refine process ownership, API and security boundary; select native macOS acceptance and obtain effective claim before implementation. |

## Identity / Goal / Value

An embedding desktop host needs supported connection-lifetime cleanup for local stdio MCP subprocesses, including helpers that enter a separate process group. Release resources without terminating another connection's processes.

## Scope and exclusions

Issue #1033 reports direct-child cleanup in talos-mcp 0.10.1 and main a40a077d. Source inspection at c7153ab1d1e0a7cd946e49e37c24ddcc5768768d confirms spawn_stdio_transport uses kill_on_drop(true). Evaluate Talos-owned supervision versus host-provided supervision with a guard retained across startup, cancellation, normal exit and shutdown; no approach selected at intake. Define ownership, guarantees, prerequisites, bounded cleanup, failure reporting and separate-group descendant isolation. No code, dependency, default, API or permission change, broad process enumeration or unrelated-process termination is authorized. MCP-001/I034 remains Complete; I337 HTTP fixtures are separate.

## Dependencies and decisions

Inspect talos-mcp transport/client ownership and host integration; consider existing process primitives and ADR-060 ownership/permission constraints as applicable. Background job supervision is a separate lifecycle owner, not automatic authority for reuse. Refine safe process identity, containment, races, cancellation and compatibility. Independent process/security/API review required before protected implementation.

## Acceptance for behavior (requested, not verified)

Given a stdio fixture whose service and detached helper listen on separate loopback ports, verify both ports reachable before each trigger on native macOS:
- Successful handshake and tool call, followed by connection release.
- Rejected initialization handshake.
- Initialization timeout or cancellation.
- Normal service exit while caller retains the connection.

Both ports must stop accepting connections within a defined bounded time. With two connections, closing one leaves the other service/helper reachable. Confirm owned process identity and isolation; document prerequisites, guarantees and failure limits. The reporter's Linux host-side experience is not macOS evidence. Native macOS acceptance remains pending.

## State/status and documentation

Intake only; no claim or selected iteration. Refinement defines native fixture commands and evidence, API/compatibility and security decision, then a separately activated iteration. Document selected host integration and supported cleanup guarantee in user-facing MCP/SDK material. Keep #1033 open until owner-first completion.

## Required Reads

- [Issue #1033](https://github.com/wjhuang88/talos/issues/1033)
- docs/sop/REQUIREMENT-INTAKE.md
- docs/sop/AGENT-COLLABORATION.md
- docs/backlog/active/MCP-001-session-mcp-integration.md
- docs/decisions/060-supervised-background-command-jobs.md
- crates/talos-mcp/src/client/transport.rs
- Relevant host composition and platform process safety contracts (identify during refinement).
