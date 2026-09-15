# Iteration I273: Bounded Model Decisions and Automatic Protocol Selection

> Document status: Review / Claimed (implementation #559 merged; closeout pending)
> Parent: PROMPT-001 / Issue #285

## Objective

Deliver a tested bounded model-decision primitive and integrate it with automatic tool-protocol
selection and protocol-failure diagnosis without changing default tool coverage or replaying unsafe
operations.

## Child slices

1. Capability probe and automatic Native/compatibility selection.
2. Typed bounded invocation primitive with isolated context and dedicated tools.
3. Protocol failure classifier/recovery policy and user-visible diagnostics.
4. Auto-permission consumer integration, followed by independent security review.

## Exclusions

No Dashboard changes, no release/publication work, no I270 harness expansion, and no permission
Deny bypass. Existing manual protocol override remains a diagnostic escape hatch during migration.

## Acceptance

- Provider capability is probed or explicitly unknown; selection is deterministic and cached safely.
- Bounded calls enforce deadline, cancellation, budget, retry and recursion limits.
- Sanitized structured outcomes distinguish correction, fallback, stop and human review.
- Write operations are never replayed when execution state is unknown.
- Focused tests cover Native, compatibility, malformed output, timeout, cancellation and denial.
- User-facing diagnostics identify model-assisted decisions without exposing secrets.

## Dependencies and evidence

Depends on ADR-075 acceptance and the I270/I272 prompt-authority contracts. Implementation requires
an effective Collaboration Claim and exact-head permission/security review. Completion requires an
existing implementation commit; a status-only commit is not evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex |
| Work Slice | Endpoint/model capability evidence and automatic Native/validated compatibility selection; typed isolated bounded model decisions; model-assisted protocol recovery with tool-execution replay protection; migration of the existing auto-permission consumer; tests and user/API documentation under ADR-075. |
| Claimed At | 2026-09-14 |
| Source Issue | #285 |
| Governance Claim PR | #558 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested full development-cycle convergence on 2026-09-14; #558 repairs the ineffective #554 record. Proposed authorization becomes effective only on target-branch merge; independent Agent-role permission/security/API review remains mandatory. |
| Implementation PR | #555 (merged initial primitive); #559 (merged as `e4173caf`) |
| Last Updated | 2026-09-14 |
| Handoff / Release Condition | Implementation #559 is merged. No Dashboard, release, Deny bypass or I270 harness expansion. Final closeout requires owner-first evidence synchronization and umbrella acceptance audit. |

## 2026-09-14 Implementation Merge Checkpoint

- Implementation PR #559 merged to `main` as `e4173caf` from exact head
  `e4820e1e5b5e0922c2fb7ca8c119a5f4e008a809`.
- Exact-head CI run `34838903222` passed all five jobs, including Windows Rust workspace.
- Independent Agent-role permission/security/API review approved the same exact head; shared
  account constraints mean this records Agent-role separation, not natural-person separation.
- I273 remains `Review / Claimed` until closeout acceptance is audited; this checkpoint does not
  claim PROMPT-001/#285 completion or close I264/I267/I270/I272 residuals.

## 2026-09-15 Automatic Protocol Selection Merge Checkpoint

- Implementation PR #562 merged to `main` as `edc0b09a` from exact implementation head
  `b4e5336e3202a488598c7acd01cbdb7a786b5aa0`.
- Exact-head CI run `34918446844` passed all five jobs, including the Windows Rust workspace.
- Independent Agent-role permission/security/API review approved the same exact head; shared
  account constraints mean this records Agent-role separation, not natural-person separation.
- This checkpoint records the capability-probe and automatic Native/Compat selection evidence only;
  it does not claim protocol recovery, umbrella completion, or child closeout.

## 2026-09-15 Post-merge Acceptance Audit

- Current `main` contains the bounded decision primitive, protocol-failure recovery path,
  execution-ledger replay guard, and automatic capability selection. Focused bounded-model,
  request-plan, capability-evidence, compatibility-frame, decision-limit, and stream-cancellation
  suites pass on the merged tree.
- The implementation evidence is now sufficient for the I273 code slice, but the owner remains
  `Review / Claimed` because umbrella PROMPT-001 acceptance still requires requirement-by-
  requirement reconciliation, user/API documentation review, and closeout of residual child
  owners. This audit does not mark I273 or PROMPT-001 complete.

## 2026-09-14 Development-Cycle Convergence Checkpoint

The maintainer requested that this development cycle close both its original work and the
additional handling requirements, leaving a clean subsequent starting point. This checkpoint
preserves the Objective through Dependencies and evidence above; it does not narrow acceptance.

### Verified starting facts and authorization repair

- Target baseline: `4a219fd52eb261c97949b69827b4deb4ea1ffba7`.
- #554 merged as `eb97905d11eb7c8168118cbe5ce4599b8cb7d8b5`, but its actual tree left this
  owner Planned/Unclaimed. Earlier assertions that this merge established a valid claim were
  incorrect. Neither #555's merge nor #557's proposed owner record repairs target-branch truth.
- #555 delivered initial code. #557 at `a31f7472390040a96709001d34058696fbdbab86` has successful
  CI `34806525595`, but unresolved implementation findings; it is not merge-ready.
- Repair the claim through a governance-only candidate based on current main. Do not push the
  divergent local main or retroactively describe earlier implementation as claim-compliant.
  After the repair merges, resume implementation from that merge or later main; preserve useful
  existing code as reviewed input, not as evidence that the ordering requirement was met.
- Final code still requires locked local validation, exact-head CI, independent Agent-role
  permission/security/API review and merge-time CAS. Single-maintainer operation is not a waiver.

### Non-terminal inventory and scheduling

Inventory of current owner headers at the target baseline (legacy empty/obsolete headers are not
new activation authority): I164 Paused; I249 Planned; I264 Planned/Unclaimed; I267 Partial/Open;
I270 Partial/Open; I272 Partial/Open; I273 Planned/Unclaimed. No other current owner header declares
Active, Review, Planned or Blocked without already declaring Complete.

- I164 and I249 remain outside this cycle and are not activated.
- I264, I267, I270 and I272 remain part of the #285 completion audit; inspect their actual
  acceptance and evidence, do not infer completion from I273 or silently drop their residuals.
- I273 is the immediate repair/implementation lane. Its dependencies are existing prompt
  contracts, not a claim that I270/I272's outstanding acceptance is complete. Do not expand
  I273 into the harness or the other child owners.
- The only open PR observed is #557, owned by this same lane. No independent concurrent
  implementation is scheduled by this record.

### Cycle closure ledger

| Item | Required result before cycle handoff | Current evidence / remaining work |
|---|---|---|
| #45 interrupted results | Verify merged persistence, restart parity and display-safe outcome documentation | #546 merge `239a01cc`, implementation `0f021a6f`; cancellation/restart fixture passed on 2026-09-14; audit remaining public documentation and owner consistency |
| #285 original scope | Match every umbrella acceptance to code, behavior evidence and user/API docs | I264/I267/I270/I272 and parent records still need requirement-by-requirement reconciliation; no umbrella Complete claim |
| I273 selection | Endpoint/model evidence controls actual request serialization and parser selection, with safe cache and explicit override | Current setters only rewrite prompts; unconditional adapter capability declarations are insufficient |
| I273 bounded decisions | Isolated explicit context, dedicated tools, shared deadline, cancellation, budgets, retry/recursion limits and typed outcomes | Initial helper exists; local dispatch-deadline tests pass, full contract remains incomplete |
| I273 recovery and auto | Real model-assisted bounded recovery and migrated auto consumer; Deny and execution-state replay guard enforced | Typed error guardrail is local; request-attempt integration, true tool-execution boundary and end-to-end tests remain |
| Windows validation cost | Correct smoke binary provenance and no redundant builds; retain complete Windows test coverage | Local commits `776d5460`/`cfaf3b63` are unshipped and must be independently checked before reuse |
| Live CI waiting | Live queued/running jobs remain waits, not stalled because two polls are unchanged | Reject unshipped `8f50aca6` rule; polling failure is not terminal evidence; do not cancel a live run without cause |
| Documentation-only follow-ups | Avoid redundant Rust work only with trusted matching code/build/workflow/base evidence; validate new docs head | Safe evidence reuse is not implemented; never replace this with a last-commit-only diff shortcut |
| Git and disk hygiene | Synchronize main, dispose of superseded branches/worktrees and disposable builds after preserving unique work | Main diverges 11 ahead/13 behind; its dirty owner and all three existing worktrees must be compared before cleanup |

All rows stay within this cycle's local ledger; do not create an Issue per repair or test iteration.
The clean starting point requires merged accepted implementation, truthful owner/derived records,
closed or explicitly resolved cycle PRs, synchronized clean main, and no unaccounted local work.
Do not delete unmerged unique changes merely to obtain a clean status.

### Validation and resume

The current local corrections pass two bounded deadline tests and four core protocol tests.
These are narrow guardrail evidence, not end-to-end automatic protocol acceptance. Existing local
code remains preserved in `impl/i273-capability-selection`; branch identity is the durable resume
pointer, not a temporary directory path. Finish authorization repair, then local implementation
convergence and independent review before another stable code push. No release is in this cycle.
