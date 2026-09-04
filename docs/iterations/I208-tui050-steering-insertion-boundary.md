# Iteration I208: Steering Boundary Insertion

> Document status: Active / Claimed
> Planned date: 2026-08-17
> Objective: implement TUI-050 so steering is inserted at an explicit model-response or tool-call
> boundary rather than only after the outer turn completes.

## Selected Story

- `TUI-050` — `docs/backlog/active/TUI-050-steering-insertion-boundary.md`

## Activation Gate

- TUI-048 and TUI-049 contracts are accepted or their interaction is explicitly resolved.
- Current-main inventory and an effective Collaboration Claim are recorded before activation.
- The implementation branch starts from the effective claim merge point.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I208 / TUI-050 only: insert accepted steering at explicit model-response or tool-call boundaries, preserving FIFO, Session/generation identity, exactly-once custody and existing transcript semantics. Excludes layout/padding, arbitrary token preemption, parallel model execution, global event bus, persistent cross-Session queues, permission, release and CAP-001 text seam work. |
| Claimed At | 2026-09-04 |
| Source Issue | #267 |
| Governance Claim PR | #487 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | I207/TUI-049 is Complete / Closed on main at `2edb914f`; maintainer directed serial execution of I207, I208 and I246. |
| Implementation PR | #488 |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Claim and activation became effective when governance PR #487 merged as `75ca8057`; implementation starts from that merge or a later `main`; independent exact-head review remains required. |

## Activation Checkpoint — 2026-09-05

I208 claim and activation are effective on `main` after governance PR #487 merged as
`75ca80571a42f2d026f507fdf84624f5a103b873`. The claim candidate was reviewed at exact head
`d3b1d94e` with CI `33894155189`; this checkpoint records activation only and is not implementation
evidence. The implementation branch starts at the merge commit above. Published Baseline remains
unchanged.

## Runnable Deliverable

An event-boundary implementation with deterministic ordering tests, error/cancel/restart coverage,
and real-terminal timing evidence.

## Exclusions

No arbitrary token preemption, parallel model execution, global event bus, or release work.

## Acceptance

- [ ] Steering is inserted at the selected model/tool boundary with published ordering semantics.
- [ ] Multiple boundaries, late arrivals, errors, cancellation and restart reconcile exactly once.
- [ ] Locked validation and real-terminal evidence pass at exact head.
- [ ] User-facing steering timing documentation is updated.

## Status

Review / Claimed. Implementation authorization is now limited to the Work Slice above; no release,
permission-policy, CAP-001, Dashboard or Desktop work is authorized.

## Local Convergence Checkpoint — 2026-09-05

- Implemented the first boundary slice locally: a `ToolUse` model-response boundary transfers one
  prepared FIFO batch into the Session command route while the current structured turn remains the
  execution authority; durable receipt custody is tracked separately and adopted at completion.
- Added a focused boundary transfer test and preserved existing queue/continuation tests.
- `cargo test -p talos-cli --locked` passed (360 tests); `cargo clippy -p talos-cli --locked -- -D warnings`
  passed; `cargo test --workspace --locked` passed, including doctests.
- Governance validators and `git diff --check` passed with zero warnings/errors.
- This is a local checkpoint only. Implementation PR, exact-head CI, independent review, terminal
  evidence and completion evidence remain pending.
