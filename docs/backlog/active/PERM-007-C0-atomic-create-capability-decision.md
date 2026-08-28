# PERM-007-C0: Atomic Create Capability Decision

| Field | Value |
|---|---|
| Story ID | PERM-007-C0 |
| Type | Permission / Security / Architecture Decision |
| Priority | P1 |
| Status | Refinement / Unclaimed |
| Source | [GitHub Issue #188](https://github.com/wjhuang88/talos/issues/188) |
| Selected Iteration | None; proposed I235 |
| Depends On | ADR-064; I234 discovery that std cannot provide directory-handle-relative creation |

## Purpose

Choose and independently review the safe platform primitive required before I234 can expose
ADR-064's positive `AllowOnce` create path. The decision must preserve fail-closed behavior on
unsupported platforms and must not weaken the parent-identity/no-clobber contract.

## Scope And Exclusions

Decision-only work: capability dependency or platform API comparison, support matrix, public API
and migration boundary, adversarial evidence and rollback. No executable behavior, dependency
change, `unsafe`, resolver wiring, WriteTool behavior, Dashboard/Desktop, release or publication.

## Acceptance

An Accepted ADR or amendment names the implementation primitive, platform support, dependency and
unsafe authorization, shared capability injection, race guarantees and later I234 handoff. Until
then I234 must keep automatic positive authorization unavailable.

## Completion Evidence

Completion Commit: Pending.
