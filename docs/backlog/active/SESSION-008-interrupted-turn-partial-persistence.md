# SESSION-008: Interrupted-Turn Partial Persistence

| Field | Value |
| --- | --- |
| Story ID | SESSION-008 |
| Type | Product / durable-session story |
| Priority | P1 |
| Status | Refinement — current partial coverage inventoried; ADR and durable implementation remain |
| Source | [GitHub Issue #45](https://github.com/wjhuang88/talos/issues/45) |
| Parent Epic | None |
| Selected Iteration | I187 (proposed; claim ineffective until target-branch merge) |
| Depends On | SESSION-002, SESSION-006, ADR-039, ADR-042 |
| Blocks | RUNTIME-005 bounded graceful shutdown |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #45 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Establish the I187 claim before implementation; accept its lifecycle ADR and compatibility decision before SESSION-008-B. |

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

## Current Implementation Baseline (2026-08-09)

- The legacy `Session` error path already persists policy-filtered partial
  messages after a provider error; focused tests prove that an admitted tool
  result and user message survive while a trailing incomplete assistant
  fragment does not.
- Durable embedded sessions still call `abort_turn` on non-success and the
  regression fixture explicitly requires an empty durable transcript after a
  failed turn. This is a remaining gap, not completion evidence.
- Cancellation currently persists a hidden terminal outcome but returns no
  partial messages. It does not yet prove that a completed display-safe tool
  result survives interruption.
- I169 added terminal-outcome evidence and transactional pending-work custody,
  but did not claim or complete SESSION-008.

## Executable Split

| ID | Deliverable | Status | Depends On |
|---|---|---|---|
| SESSION-008-A | Partial-turn lifecycle and durable-format decision | Ready, not selected | Existing ADR-039/ADR-042 and current-path inventory |
| SESSION-008-B | Atomic/idempotent durable partial commit and replay integration | Blocked | SESSION-008-A Accepted |

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

## Residuals

- This Story remains Refinement until SESSION-008-A selects the lifecycle owner,
  durable entry shape and compatibility strategy. Existing legacy error-path
  coverage is retained as a compatibility fixture; it does not authorize or
  substitute for SESSION-008-B.
