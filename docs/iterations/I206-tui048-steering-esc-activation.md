# Iteration I206: Esc-Cancelled Steering Activation

> Document status: Active / Claimed (proposed; ineffective until claim merge)
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
| Authorization Evidence | Maintainer directed Issue #408 implementation; claim PR #410 becomes effective only after exact-head CI, independent Agent-role review, merge-time CAS and merge to main. |
| Implementation PR | Not started |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Implementation begins from the claim merge or later main and must locally converge before one stable candidate push. |

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

- [ ] Esc cancellation admits accepted steering into the same Session.
- [ ] Exactly one continuation turn becomes runnable; empty and repeated-cancel cases are covered.
- [ ] Focused, locked workspace validation and real-terminal evidence pass at exact head.
- [ ] User-facing steering/cancellation documentation is updated.

## Status

Proposed Active / Claimed. The open governance PR has no ownership or code authorization effect;
implementation starts only after the finalized claim reaches `main`.

## 2026-08-27 Requirement Reconciliation

Issue #408 is not a separate queue-control feature. Its provider-switch failure occurs because Esc
cancellation preserves accepted steering, as required by ADR-049, but the next transactional
submission is not activated. I206 acceptance therefore includes a real sequence proving: active
turn -> queued steering -> Esc -> one next same-Session turn -> queue drained -> provider picker
accepted. Provider mutation remains blocked before queue drain; no guard bypass is authorized.

Inventory disposition at selection: I197/I198/I201/I210 remain Review under their existing
corrective owners; I164 remains Paused and superseded; I207/I208 remain Planned/Unclaimed and are
not activated; no other Active iteration or overlapping open implementation PR exists.
