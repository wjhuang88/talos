# Iteration I206: Esc-Cancelled Steering Activation

> Document status: Review / Claimed
> Planned date: 2026-08-17
> Objective: implement TUI-048 so accepted steering becomes a runnable Session turn when `Esc`
> cancels the active turn, without changing the accepted I169 custody contract.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-048 session |
| Work Slice | Implement only TUI-048/I206: transactionally activate already-accepted steering as exactly one same-Session next turn after active-turn Esc cancellation; prove queue custody/drain and provider-switch recovery without bypassing session mutation guards. Preserve I169, ADR-049 and ADR-056 identity/FIFO/rollback semantics. Exclude queue deletion/editing, TUI-049/TUI-050, provider lifecycle redesign, permission, persistence schema, release, Dashboard and Desktop work. |
| Claimed At | 2026-08-27 |
| Source Issue | #267, #408 |
| Governance Claim PR | #410 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer directed Issue #408 implementation; claim PR #410 exact head `11861083` passed CI `33036746350`, independent Agent-role approval `5434254388` and merge-time CAS, then merged as `c3a121f0`. |
| Implementation PR | #411 |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Claim PR #410 merged as `c3a121f0`; implementation must pass exact-head CI, independent review, real-terminal acceptance and merge-time CAS. |

## Selected Story

- `TUI-048` — `docs/backlog/active/TUI-048-steering-esc-activation.md`

## Activation Gate

- Current-main inventory records all non-terminal iterations and open PRs/issues.
- An effective Collaboration Claim exists on target `main`.
- A fresh implementation branch starts from the claim merge point.

## Runnable Deliverable

Lifecycle implementation and focused tests proving cancellation, Session admission and exactly one
subsequent turn, plus real-terminal evidence.

## Exclusions

No TUI-049/TUI-050 implementation, queue redesign, permission policy change or release work.

## Acceptance

- [x] Esc cancellation admits accepted steering into the same Session.
- [x] Exactly one continuation turn becomes runnable; empty and repeated-cancel cases are covered.
- [ ] Focused, locked workspace validation and real-terminal evidence pass at exact head.
- [x] User-facing steering/cancellation documentation is updated.

## Status

Review / Claimed. Implementation locally converged from claim merge `c3a121f0`; remote candidate
and exact-head evidence remain pending.

## 2026-08-27 Requirement Reconciliation

Issue #408 is not a separate queue-control feature. Its provider-switch failure occurs because Esc
cancellation preserves accepted steering, as required by ADR-049, but the next transactional
submission is not activated. I206 acceptance therefore includes a real sequence proving: active
turn -> queued steering -> Esc -> one next same-Session turn -> queue drained -> provider picker
accepted. Provider mutation remains blocked before queue drain; no guard bypass is authorized.

Inventory disposition at selection: I197/I198/I201/I210 remain Review under their existing
corrective owners; I164 remains Paused and superseded; I207/I208 remain Planned/Unclaimed and are
not activated; no other Active iteration or overlapping open implementation PR exists.

## 2026-08-27 Local Execution Checkpoint

- Structured completion dispatches Engine-owned steering after Success or an explicitly requested
  cancellation; unrequested cancellation and Error remain paused.
- Legacy compatibility dispatches after `LegacyCancelling + Cancelled`, without making arbitrary
  legacy Cancelled events runnable.
- `esc_cancel_activates_queued_turn_before_provider_switch_is_allowed` drives the bridge protocol:
  provider switch rejected before drain, generation-bound Esc interrupt, exactly one queued
  continuation submission, terminal success, then provider switch accepted.
- `cargo test --locked -p talos-cli`: 355 unit tests plus all integration suites passed, 0 failures.
- `cargo clippy --locked -p talos-cli --all-targets -- -D warnings`: passed.
- User documentation updated in `README.md`, `README.zh-CN.md` and architecture reference.
- Real-terminal evidence: rebuilt `target/debug/talos --no-init --tui --no-context` was driven in a
  120x40 PTY against a temporary loopback provider. Session
  `d4385b54-5d37-455b-b065-5634ebf87e4c` recorded turn 1 `cancelled`, automatically admitted user
  text `queued continuation`, received `queued-complete`, and recorded turn 2 `success` in the same
  session. No second submit was issued. The bridge integration fixture separately proves provider
  switching is fenced before queue drain and accepted after the continuation succeeds.
- Exact-head CI and independent review: Pending.

## Completion Evidence

- Completion Commit: Pending.
- A status-only documentation commit cannot self-certify completion.
