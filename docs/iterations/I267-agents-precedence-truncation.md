# Iteration I267: Scoped Instruction Precedence and Safe Truncation

| Field | Value |
|---|---|
| Status | Partial / Open |
| Source | PROMPT-001 / Issue #285 |
| Deliverable | Implement nearest-scope AGENTS precedence and instruction-aware truncation while preserving runtime invariants and prompt compatibility. |
| Depends On | I266, ADR-074 |
| Excludes | No Evolution/memory policy change, no provider/API change, no UI work. |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | AGENTS precedence and safe truncation |
| Source Issue | #285 |
| Governance Claim PR | Pending |
| Claimed At | 2026-09-13 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Claim-only metadata; implementation remains unauthorized until merge. |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Establish effective claim before implementation; preserve existing loading behavior until tests prove migration. |
| Implementation PR | #552 |

## Acceptance

### 2026-09-13 CI Recovery Checkpoint

The implementation candidate remains unchanged; this governance-only checkpoint exists to
retrigger pull-request validation after the prior workflow ended with `startup_failure`.


- Nearest scoped instructions refine, but cannot weaken, runtime invariants or current user intent.
- Truncation preserves complete authority-bearing rules and records deterministic diagnostics.
- Existing prompt behavior remains compatible outside explicitly tested precedence cases.
- Tests cover ancestor/child conflicts, truncation boundaries and cache effects.

Completion Commit: 90b49f1fef699a2662c7482d91420a0de38d34ab

### Post-merge completion audit (2026-09-13)

The merged implementation provides newline-aware head/tail truncation only. It does not yet
implement semantic nearest-scope precedence, protection of authority-bearing rules, deterministic
precedence diagnostics, or the required ancestor/child and cache-effect tests. The implementation
commit remains recorded as partial evidence; a follow-up governed slice must complete these
acceptance items before I267 can be marked Complete.
