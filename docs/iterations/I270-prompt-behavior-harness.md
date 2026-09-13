# Iteration I270: Prompt Model-Behavior Harness

> Document status: Partial / Open

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | Deterministic model-behavior fixtures for prompt authority and protocol parity |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit 224f16bb |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; harness-only scope. |
| Implementation PR | Not started |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Excludes production prompt, Memory, Evolution, SDK, provider, UI, and release changes. |

## Scope

- Add deterministic fixtures for authority conflicts, advisory context, tool/skill data, Todo restraint, and protocol parity.
- Keep fixtures independent from network providers and production prompt behavior.

## Acceptance

- Fixtures produce stable pass/fail outcomes for representative authority cases.
- Harness output identifies the violated precedence contract without relying on string snapshots alone.

## Completion Evidence

- Completion Commit: pending (provider/model fixtures and cross-module scenarios remain)

## Execution Checkpoint

- `33660be2` adds deterministic authority-order and advisory-source fixtures.
- Focused prompt section tests: 3 passed.
- `2c58b52c` adds protocol-surface authority parity coverage; 4 focused tests pass.
- `e04da18b` adds capability-data versus user-authority coverage; 5 focused tests pass.
- Remaining acceptance: provider-independent behavioral scenarios and protocol parity fixtures.
