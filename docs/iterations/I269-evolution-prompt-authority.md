# Iteration I269: Evolution Learned-Pattern Authority

> Document status: Partial / Open
> Published plan date: 2026-09-13
> Planned objective: Keep learned Evolution patterns advisory and provenance-bearing when contributed to prompts.
> MVP deliverable: Tested Evolution prompt contribution boundary that cannot present learned history as runtime authority.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | Evolution learned-pattern prompt authority and provenance boundary |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit 8d4bd334c46aa4f0212b6229eb70788d6f72374a |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; scope is limited to Evolution prompt contributions. |
| Implementation PR | Direct commit `a0a958e5` |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Excludes Memory, Todo/steering, SDK custom_prompt, provider, UI, and release work. |

## Scope

- Separate learned-pattern confidence from prompt authority and scope.
- Render Evolution contributions as bounded advisory context with provenance.
- Preserve existing hook compatibility and stable prompt/cache behavior outside this boundary.

## Non-Goals

- No Memory, Todo, steering, SDK, provider, UI, public API, or persistence redesign.

## Acceptance

- Learned patterns cannot be rendered as runtime/core instructions.
- Every injected pattern identifies advisory authority and provenance.
- Structural and representative behavior tests cover imperative-pattern demotion and hook bounds.

## Planned Validation

- Focused `talos-evolution` and `talos-agent` prompt tests.
- Locked workspace validation, governance validators, and diff checks after local convergence.

## Completion Evidence

- Completion Commit: pending (behavior harness and hook-boundary tests remain)

## Execution Checkpoint

- `a0a958e5` adds explicit advisory wording and bounded UTF-8-safe output truncation.
- Focused `talos-evolution` tests: 63 passed.
- Remaining acceptance: representative behavior harness and explicit provenance fields.

## Residuals

- Remaining #285 authority boundaries require separate child iterations.
