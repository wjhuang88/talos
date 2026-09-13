# Iteration I272: Memory And Todo Advisory Authority

> Document status: Active

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | Memory, session Todo, and steering prompt authority boundary |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit b1820a33 |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; advisory-context scope only. |
| Implementation PR | Not started |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Excludes Evolution, SDK hooks/custom_prompt, provider, UI, permissions, and release changes. |

## Scope

- Ensure Memory and Todo/steering content remains advisory and subordinate to current user intent and runtime invariants.
- Add deterministic structural tests for precedence and interruption behavior.

## Acceptance

- Memory, Todo, and steering sections are explicitly identified as advisory context.
- Current user intent outranks stale advisory entries.
- Existing bounded injection and resumability behavior remains compatible.

## Completion Evidence

- Completion Commit: pending
