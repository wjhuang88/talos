# Iteration I267: Scoped Instruction Precedence and Safe Truncation

| Field | Value |
|---|---|
| Status | Planned / Unclaimed |
| Source | PROMPT-001 / Issue #285 |
| Deliverable | Implement nearest-scope AGENTS precedence and instruction-aware truncation while preserving runtime invariants and prompt compatibility. |
| Depends On | I266, ADR-074 |
| Excludes | No Evolution/memory policy change, no provider/API change, no UI work. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | AGENTS precedence and safe truncation |
| Source Issue | #285 |
| Governance Claim PR | Not applicable |
| Claimed At | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Establish effective claim before implementation; preserve existing loading behavior until tests prove migration. |
| Implementation PR | Not started |

## Acceptance

- Nearest scoped instructions refine, but cannot weaken, runtime invariants or current user intent.
- Truncation preserves complete authority-bearing rules and records deterministic diagnostics.
- Existing prompt behavior remains compatible outside explicitly tested precedence cases.
- Tests cover ancestor/child conflicts, truncation boundaries and cache effects.

Completion Commit: pending
