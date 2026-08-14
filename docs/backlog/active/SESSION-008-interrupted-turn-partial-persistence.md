# SESSION-008: Interrupted-Turn Partial Persistence

| Field | Value |
| --- | --- |
| Story ID | SESSION-008 |
| Type | Product / durable-session story |
| Priority | P1 |
| Status | Complete — SESSION-008-A and SESSION-008-B complete |
| Source | [GitHub Issue #45](https://github.com/wjhuang88/talos/issues/45) |
| Parent Epic | None |
| Selected Iteration | I193 Complete; implementation merged in `1b5461cd` |
| Depends On | SESSION-002, SESSION-006, ADR-039, ADR-042 |
| Blocks | RUNTIME-005 bounded graceful shutdown |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Released |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned - completed SESSION-008-A and SESSION-008-B claims released |
| Claimed At | 2026-08-11 |
| Source Issue | #45 |
| Governance Claim PR | #194 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | A: claim `5bb83f80`, decision `e288afb5`, review `5261130488`. B: claim `fb5a1f62`, implementation PR #216, exact-head CI `31691761892`, disclosed role audits `5287961007`/`5287989820`, merge `1b5461cd`. |
| Implementation PR | #195 (A); #216 (B) |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Complete; RUNTIME-005 retains its independently owned A/B/C gates. |

Completion Commit: `e288afb5d97026f7ccb3ce0f519a4a81f99fe104` (A decision),
`404d7a4bf5b9c7dedeae479fe91fa5400b42d411` (B implementation).

## Identity / Goal / Value

Preserve the display-safe facts already produced by a turn when the user
interrupts it or it fails after tools have executed. After restart, users must
see the same safe tool-result facts they saw live, the reconstructed model
context must not contradict side effects that already happened, and the
transcript must identify that the turn did not complete normally.

## Scope

- Define one durable, atomic, and idempotent partial-turn commit operation for
  interrupted and eligible error paths.
- Persist only messages admitted by the existing `PersistencePolicy` filtering
  boundary; never persist raw provider/tool payloads merely because the turn is
  partial.
- Record a deterministic incomplete/interrupted status that replay and host
  projection can distinguish from a successful turn.
- Preserve existing `turn_id` idempotency so retrying the same partial commit
  cannot duplicate durable entries.
- Rebuild runtime/model context from the same canonical display-safe entries
  used by restart rendering.
- Define ordering and race behavior when cancellation competes with normal
  completion, provider failure, or a second persistence attempt.
- Cover both embedded durable sessions and the Talos-owned session path without
  changing successful-turn behavior.

## Exclusions

- No persistence of hidden reasoning, credentials, unredacted tool arguments,
  raw tool output, or provider-private protocol data.
- No automatic provider retry, undo of completed tool side effects, or claim
  that an interrupted turn is successful.
- No implementation inside I164/TUI-038.
- No breaking public session API or durable-format change without an accepted
  ADR and migration/compatibility plan.
- No change to transcript export semantics beyond the separately approved
  representation of an incomplete durable turn.

## Decision Links And Constraints

- ADR-039 defines the session integrity and lifecycle boundary. Cancellation,
  completion, and persistence must have one deterministic winner.
- ADR-042 defines embedded durable session reconstruction and display-safe
  filtering. Realtime and restart replay must use the same canonical safe
  projection.
- SESSION-006 closed provider-error persistence for the prior session path; this
  story must reconcile that behavior with durable embedded sessions rather than
  create a second incompatible error model.
- The requested API shape (`commit_partial_turn`, status-bearing commit, or an
  expanded abort operation) is not preselected. The ADR must choose one owner
  and state transition.

## Current Implementation Baseline (2026-08-09, superseded by I193 on 2026-08-14)

- The legacy `Session` error path already persists policy-filtered partial
  messages after a provider error; focused tests prove that an admitted tool
  result and user message survive while a trailing incomplete assistant
  fragment does not.
- Before I193, durable embedded sessions called `abort_turn` on non-success and cancellation
  returned no partial messages; those facts remain the historical pre-implementation baseline.
- I169 added terminal-outcome evidence and transactional pending-work custody,
  but did not claim or complete SESSION-008.

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| SESSION-008-A | Partial-turn lifecycle and durable-format decision | Complete in I187; Completion Commit `e288afb5d97026f7ccb3ce0f519a4a81f99fe104` | Existing ADR-039/ADR-042 and current-path inventory |
| SESSION-008-B | Atomic/idempotent durable partial commit and replay integration | Complete in I193; Completion Commit `404d7a4bf5b9c7dedeae479fe91fa5400b42d411` | SESSION-008-A Complete; ADR-058 Accepted |

Only one child may be selected at a time. The parent becomes Complete only
after both children have existing completion evidence and the Issue #45
acceptance matrix is reconciled.

## Uncertainty And Validation Path

Before Ready, inventory every cancel/error path and identify where partial
messages are still owned when cancellation wins. Decide the durable entry
shape, incomplete marker/status representation, replay visibility, model-context
admission, and compatibility behavior for existing TLOG data. Prove that the
chosen operation can be atomic and idempotent without persisting a fabricated
empty turn. If the public API or durable schema changes, accept an ADR and
migration plan before implementation.

## SESSION-008-A Decision Evidence

- Effective claim merge: `5bb83f80b7dd7216ed83ee69fd4de0ef954c32f7` (PR #194).
- Accepted decision: `docs/decisions/058-partial-turn-durable-finalization.md`.
- Current-path evidence: `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md`.
- ADR-058 was Accepted by the reviewed I185-I187 closeout merged as `6c7e11cc44fdd8c7b48a2d2bf6d5438db036f432`;
  it defines the target contract but does not describe current runtime behavior before B lands.

## SESSION-008-B Claim Proposal

- [I193](../../iterations/I193-session008b-durable-partial-finalization.md) and PR #216 delivered
  the separately governed B implementation. The claim became effective in `fb5a1f62`; the source
  implementation is `404d7a4b` and the merge is `1b5461cd`.
- **SESSION-008-R1 — current-versus-target truth linkage.** Before B reached `main`, the
  [I187 characterization](../../reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md) was
  the released-behavior truth source and ADR-058 was target-only; after the I193 merge, I187 is
  retained as historical baseline and ADR-058 is implemented behavior on `main`.
- **SESSION-008-R2 — transient test diagnosis.** This remains conditional diagnostic evidence: if
  the seven transient `talos-session` failures recur, capture disk bytes, inode availability,
  temporary paths, complete stderr and the default-parallel result. No concurrency defect or
  ENOSPC root cause is confirmed.

## State / Status Owners

- Durable commit, TLOG compatibility, and replay reconstruction:
  `talos-session`.
- Turn cancellation/completion arbitration and partial message ownership:
  `talos-agent`.
- CLI/embedded runtime integration: `talos-cli` and `talos-runtime`.
- Host-facing display-safe projection: existing durable transcript contract.
- Story status: this document.

## User-Facing Documentation

- Document that completed tool results may survive an interrupted turn and are
  shown with an explicit incomplete/interrupted state after resume.
- Document the redaction boundary and that Talos does not imply the assistant's
  final response completed.
- Record host migration requirements if the durable transcript representation
  changes.

## Required Reads

- [GitHub Issue #45](https://github.com/wjhuang88/talos/issues/45)
- `docs/backlog/active/SESSION-002-session-integrity-lifecycle-hardening.md`
- `docs/backlog/active/SESSION-006-session-error-path-persistence.md`
- `docs/decisions/039-session-integrity-and-lifecycle-semantics.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `crates/talos-agent/src/session/turn.rs`
- `crates/talos-session/src/durable.rs`
- `crates/talos-runtime/src/`

## Acceptance

- Given a display-safe tool result exists before user interruption, when the
  session is restarted, then that result is present exactly once and the turn
  is visibly marked incomplete/interrupted.
- Given partial messages contain reasoning, credentials, raw arguments, or
  other policy-rejected data, when partial persistence runs, then the same
  filtering policy as successful durable commits excludes it.
- Given the same `turn_id` is committed more than once through cancellation,
  error, or retry races, then the durable transcript contains no duplicate
  entries and reports a deterministic final status.
- Given cancellation happens before any persistable fact exists, when the turn
  stops, then Talos does not fabricate a durable tool result or empty completed
  turn.
- Given a provider error occurs after a tool result, when durable and in-memory
  sessions resume, then both reconstruct the same display-safe conversation
  facts and incomplete status.
- Existing successful-turn commit, resume, fork, filtering, and replay tests
  remain unchanged and green.

## Minimum Validation

```bash
cargo test --locked -p talos-session
cargo test --locked -p talos-agent session
cargo test --locked -p talos-runtime
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
```

## Completion Evidence

- Completion Commit: `404d7a4bf5b9c7dedeae479fe91fa5400b42d411` (SESSION-008-B implementation).
- PR #216 merged as `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`; exact-head CI `31691761892` passed.
- I193 owner records the full acceptance, R1/R2, role disclosure and validator evidence.

## Residuals

- RUNTIME-005 remains Refinement / Unclaimed with A Ready/not selected, B Blocked until its other
  gate (RUNTIME-005-A Accepted) is satisfied, and C blocked on B. I188/I189 remain Planned/Claimed
  and unactivated; Issues #45/#49/#59 remain open.
