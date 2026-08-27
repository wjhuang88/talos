# Iteration I230: Permission Prompt Composer-Relative Docking

> Document status: Active / Claimed
> Published plan date: 2026-08-27
> Planned objective: close TUI-059/#330 by keeping permission prompts adjacent to the logical composer across non-bottom, narrow and resized layouts.
> Baseline rule: preserve this target; changed objectives require a new iteration ID.
> MVP deliverable: a runnable TUI path and deterministic layout fixtures proving prompt/composer hierarchy without bottom drift or overlap.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline TUI-059 session |
| Work Slice | Implement only TUI-059 composer-relative permission prompt docking: derive placement from the logical composer/layout plan, apply minimum deterministic reflow, restore prior scroll state after resolution, and cover non-bottom, narrow/short, wrapped, repeated and resize paths. Exclude permission semantics, request identity, persistence, provider, release, Dashboard, Desktop and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #330 |
| Governance Claim PR | #416 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, governance validators and independent review before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-27 |
| Handoff / Release Condition | Claim and Active state become effective only after PR #416 merges to `main`; implementation starts from that merge or later. Permission semantic or protected-crate changes require independent security review. |

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-059 | I211 corrective owner / Issue #330 | Ready / Unclaimed | I197 merged anchor state; inline composer ownership; ADR-054 | Composer-relative permission prompt placement with deterministic reflow and state restoration. |

### Scope

- Derive prompt placement from the current logical composer/layout plan, including a new session with composer above the physical terminal bottom.
- Apply only minimum deterministic reflow needed to keep required choices visible.
- Restore follow-tail or anchored-history state after approve, deny, cancel, timeout or error.
- Cover queued prompts, wrapped/multiline content, narrow/short terminals and resize without overlap, cursor artifacts or bottom drift.

### Non-Goals

- No permission policy/default decision/request identity/persistence change.
- No global composer-bottom rule, broad renderer rewrite, dependency, release, publication, Dashboard, Desktop or `/auto` work.

### Acceptance

- Given a non-bottom composer, when a permission prompt opens, then it remains adjacent to that composer rather than the physical terminal bottom.
- Given insufficient height, then minimum reflow keeps every choice visible without overlap.
- After approve, deny, cancel, timeout or error, the prior logical composer and scroll state are restored.
- Repeated prompts, wrapped descriptions, multiline drafts, narrow/short terminals and resize remain stable.
- Permission semantics and request identity remain unchanged.

### Planned Validation

- Focused TUI layout/state tests and real-terminal/PTY evidence for non-bottom, short, narrow, resize and repeated prompt paths.
- `cargo test --locked --workspace`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, release preflight, governance validators and diff check.
- Independent permission/TUI review, exact-head CI and merge-time CAS.

### Documentation To Update

- TUI-059 story, TUI-045/I197 corrective disposition, README behavior reference, Board, Backlog, iterations README, manifest and Issue #330 reconciliation.

### Risks And Rollback

- Risk: prompt reflow hides choices, displaces composer state or changes permission interaction semantics.
- Rollback: retain the prior layout path while preserving permission behavior and transcript state.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-28 | Atomic claim+activation proposal | Prepared from `main@fd84c92e`; I229/TUI-058 is Complete, I197 remains Review, and no overlapping implementation PR exists. PR #416 proposes the single bounded claim and Active state; both remain ineffective until merge. |

## Verification Evidence

- Pending implementation and exact-head evidence.

## Completion Evidence

- Completion Commit: Pending.

## Variance And Residuals

- I197/#125 remains Review until this corrective story closes its layout/docking failure. TUI-060, SKILL-005 and TUI-061 remain separate.
