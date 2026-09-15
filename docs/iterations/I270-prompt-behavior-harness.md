# Iteration I270: Prompt Model-Behavior Harness

> Document status: Review / Claimed

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

- Completion Commit: `d5c6cd11`, `6264c253`, `87402537` and `face3b2a` (deterministic harness,
  builder integration, warning-free validation, and rendered advisory coverage).

## Execution Checkpoint

- `33660be2` adds deterministic authority-order and advisory-source fixtures.
- Focused prompt section tests: 3 passed.
- `2c58b52c` adds protocol-surface authority parity coverage; 4 focused tests pass.
- `e04da18b` adds capability-data versus user-authority coverage; 5 focused tests pass.
- `40658643` adds provider-independent `AuthorityDecision` fixtures for runtime-over-advisory,
  equal-authority conflict reporting, and structural print/TUI/RPC/MCP parity; 9 focused tests pass.
- `18ad38b7` extends the decision matrix with Todo-as-advisory precedence against current user
  and runtime rules; prompt tests pass (35 tests).
- Remaining acceptance is limited to broader cross-module scenarios and harness output integration;
  this iteration remains Partial until those cases are covered.

## 2026-09-14 Review Checkpoint

- `run_precedence_harness` now consumes the actual assembled `PromptSection` set through
  `SystemPromptBuilder::precedence_report()`, producing deterministic diagnostics for every
  section below the authoritative floor.
- Cross-module rendered Memory/Session Todo coverage is exercised independently of any provider;
  the talos-agent prompt suite passes all 37 tests and the session suite passes all 35 tests
  locally. This records Review readiness; final
  umbrella acceptance and independent review remain outstanding.

## 2026-09-14 Harness Correction Checkpoint

- The initial adjacent-authority rule incorrectly treated the intentional stable-prefix to
  dynamic-suffix boundary (`Skills`/`Context`) as a violation. The harness now accepts that
  cache boundary and reports advisory content following current user intent, while also
  detecting an explicit restrictive runtime rule followed by user text.
- The rendered builder fixture is a positive default-order case; the negative case is an explicit
  `User -> Session Todos` conflict. Focused prompt module tests: 66 passed.
- I270 remains Review pending independent review and broader I285 acceptance.

### 2026-09-15 Current-main independent audit

Agent-role independent audit against `main@c36165a3` confirmed the deterministic harness and
rendered precedence diagnostics are present. I270 remains Review pending umbrella acceptance and
cross-child reconciliation. Shared-account review is Agent-role separation only, not natural-person identity separation.
