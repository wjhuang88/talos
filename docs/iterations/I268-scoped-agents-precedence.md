# Iteration I268: Scoped AGENTS Precedence And Authority-Safe Truncation

> Document status: Active
> Published plan date: 2026-09-13
> Planned objective: Complete the semantic precedence and authority-safe truncation residual from I267.
> MVP deliverable: A deterministic, tested prompt-context projection that preserves nearest-scope rules and never emits ambiguous authority fragments.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | Scoped AGENTS precedence and authority-safe truncation |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit 9b9ace72213c0e6e213c959ec626e8024b3ea131 |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; claim is limited to this slice. |
| Implementation PR | Not started |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Keep Evolution, memory, provider, UI, and public API changes out of scope. |

## Published Baseline

### Scope

- Resolve nearest applicable AGENTS scope deterministically and preserve runtime/user authority.
- Truncate instruction context only at complete rule boundaries, with deterministic diagnostics.

### Non-Goals

- No Evolution or memory policy changes, provider/API changes, UI work, or prompt customization redesign.

### Acceptance

- Given conflicting ancestor and child instructions, the nearest applicable scope is explicit and wins.
- Given an authority-bearing rule, truncation never emits a partial rule or silently drops its authority.
- Given an oversized context, diagnostics identify deterministic retained and omitted sections.
- Existing prompt compatibility and cache partition remain unchanged outside these cases.

### Planned Validation

- Focused `talos-agent` context and prompt tests, including ancestor/child conflicts, long rules, diagnostics, and cache effects.
- Locked workspace validation after local convergence.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-13 | Direct claim activation | I268 claim established on main; implementation remains not started. |

## Completion Evidence

- Completion Commit: pending

## Variance And Residuals

- I267 remains Partial and supplies newline-aware truncation evidence only.
