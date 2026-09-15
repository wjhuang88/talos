# Iteration I272: Memory And Todo Advisory Authority

> Document status: Complete / Closed

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | @wjhuang88 |
| Work Slice | Memory, session Todo, and steering prompt authority boundary |
| Claimed At | 2026-09-13 |
| Source Issue | #285 |
| Governance Claim PR | Direct commit b1820a33 |
| Authorization Mode | Direct commit |
| Authorization Evidence | User-authorized continuation; advisory-context scope only. |
| Implementation PR | Direct commit `f5cacdd7` |
| Last Updated | 2026-09-13 |
| Handoff / Release Condition | Complete; advisory Memory/Todo boundary and resumability evidence merged and audited. |

## Scope

- Ensure Memory and Todo/steering content remains advisory and subordinate to current user intent and runtime invariants.
- Add deterministic structural tests for precedence and interruption behavior.

## Acceptance

- Memory, Todo, and steering sections are explicitly identified as advisory context.
- Current user intent outranks stale advisory entries.
- Existing bounded injection and resumability behavior remains compatible.

## Completion Evidence

- Completion Commit: `f5cacdd7`, `d8c04a4f`, and `face3b2a` (advisory classification/rendering,
  precedence report integration) plus existing session boundary implementation evidence.

## Execution Checkpoint

- `f5cacdd7` adds advisory classification for Memory and Session Todo sections with six focused tests.
- `d8c04a4f` renders explicit `(advisory)` labels for Memory and Session Todos; 34 prompt tests pass.
- Steering interruption/resumability is covered by the existing session boundary harness
  (`test_concurrent_submit_and_interrupt` and the steering handoff matrix in
  `crates/talos-agent/src/session/tests.rs`): accepted steering is injected after the
  preceding tool result, before the next response, without starting a second outer turn,
  and retains the original turn identity across cancellation/error paths.
- This evidence confirms runtime ordering compatibility, but does not yet provide the
  provider-independent prompt behavior harness is owned by I270; I272 remains Review pending
  final umbrella reconciliation.
- Focused prompt and session regression suites were re-run on `dev/prompt-285-convergence`; advisory Memory/Todo
  rendering remains dynamic and steering submissions preserve resumability without a second
  outer turn. No implementation change is claimed by this checkpoint.
- Exact focused evidence: `cargo test -p talos-agent session::tests::test_concurrent_submit_and_interrupt --locked`
  passed (1 test). This confirms the representative interruption path, while broader runtime
  rendering/resumability coverage remains explicitly outstanding.
- Session regression suite rerun: `cargo test -p talos-agent session::tests --lib --locked` passed
  (35 tests), including durable cancellation, failed continuation, persistence recovery, and
  steering interruption cases.

## 2026-09-14 Review Checkpoint

- `SystemPromptBuilder::precedence_report()` now runs the deterministic provider-independent
  harness over the actual rendered section set; Memory and Session Todos are reported below the
  authoritative floor without relying on provider or string snapshots.
- Focused prompt and session evidence passed locally: 1 rendered advisory test and 35 session
  boundary tests. This checkpoint records Review readiness; umbrella PROMPT-001 acceptance and
  independent review remain outstanding.

### 2026-09-15 Current-main independent audit

Agent-role audit against `main@c36165a3` confirmed advisory Memory/Session Todo rendering and
session-boundary tests. Remaining umbrella reconciliation and final owner closeout are still
required; this does not mark I272 Complete.
