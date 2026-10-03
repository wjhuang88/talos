# Iteration I289: TUI History Return To Bottom

> Document status: Review
> Published plan date: 2026-09-30
> MVP deliverable: A visible return-to-bottom hint and `Ctrl+Down` action for anchored history.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | TUI-062 history return-to-bottom prompt and Ctrl+Down routing. |
| Claimed At | 2026-09-30 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not authorized; pending governance review |
| Authorization Evidence | No effective target-branch claim or Direct-commit authorization verified; see historical deviation below. |
| Implementation PR | #628 (existing candidate; blocked on effective claim) |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | Do not mark Complete until implementation evidence exists. |

## Scope

- Add a concise hint in the existing tips row when history is anchored.
- Add `Ctrl+Down` to restore `HistoryScrollMode::FollowTail`.

## Non-Goals

- No changes to Esc semantics, mouse scrolling, selection, approvals, task cancellation, or transcript persistence.

## Acceptance

- Anchored history displays the hint.
- `Ctrl+Down` returns to the tail.
- FollowTail keeps the current default hint.

## Planned Validation

- `cargo test -p talos-tui --locked --offline`
- `bash scripts/validate_collaboration_claims.sh .`
- `bash scripts/validate_project_governance.sh .`

## Completion Evidence

- Completion Commit: d6d90412 (candidate; native acceptance and target-branch review remain)

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-09-30 | Direct claim and local convergence | Claim recorded in 44abce9f; implementation d6d90412; locked offline TUI tests and governance validators passed. |

## Governance Recovery — 2026-10-03

Independent Agent review of PR #628 head `aa00a6e0` against target `fa76b103`
found that local claim commit `44abce9f` never reached `origin/main` before
implementation `d6d90412`. The former Direct-commit authorization description was
unsupported: a user request to implement in a worktree did not authorize bypassing
the target-branch claim order. Earlier execution records are provenance, not proof
of effective ownership or activation. No emergency override is claimed.

Delivery remains Review because implementation exists, while collaboration is
Unclaimed because no effective target-branch claim was verified. PR #628 must not
merge until a separate governance-only claim establishes ownership, followed by
fresh implementation validation and independent review. Native terminal acceptance
also remains pending; this recovery is not completion evidence.

### Governance-Only Claim Candidate

Create a fresh governance branch from the current target, containing only this
Story/iteration scope, claim, inventory and derived-view records; exclude Rust,
implementation tests and user-facing implementation changes. Preserve published
scope/acceptance and identify PR #628 as the historical implementation candidate.
Before its Draft PR number exists, use Unclaimed with Governance Claim PR Pending.
After backfilling the actual number, propose Claimed and Active, explicitly
ineffective until target-branch merge. Record a real permitted authorization path,
run both governance validators and exact-head checks, obtain review, and repeat
merge-time CAS. Do not copy the unsupported Direct-commit authorization forward.
After that merge, rebuild/rebase the implementation candidate from the effective
claim target and retain this historical deviation record.

## Retrospective Selection Inventory — 2026-10-03

This is a recovery-time inventory against target `fa76b103`, not evidence that the
required selection preflight happened on 2026-09-30. Current iteration owner
headers were inspected across `docs/iterations/I*.md`; historical execution-log
mentions of Review/Active do not supersede terminal owner headers.

| Iteration | Current owner state | Disposition for TUI-062 recovery |
|---|---|---|
| I249 | Planned / Unclaimed | Keep deferred; dependency-upgrade pilot is outside this presentation-only slice. |
| I277 | Review; human acceptance Deferred | Retain Review and existing #29 deferred device/human checks; no Desktop implementation is selected here. |
| I288 | Active / Claimed proposed; ineffective until #627 merges | Retain architecture-only SEARCH scope and its own activation gate; TUI-062 does not change search architecture or claim it. |
| I289 | Review / Unclaimed | Recover the missing effective claim; no implementation merge or Complete authorization yet. |
| I164 | Paused | Retain the published superseded target; do not reactivate it through this shortcut change. |

No other non-terminal Active, Review, Planned or Blocked iteration was found in
current owner status headers at this snapshot. I162 is Complete with a recorded
Review outcome, not a non-terminal Review iteration. Effective dependency work
DEP-ENDENCY-003-A has a separate Story owner and must remain non-overlapping;
no unrelated dependency, Desktop or search-owner state is changed here.

## User-Facing Documentation Target

- `README.md` and `README.zh-CN.md`: document the history return shortcut after
  effective claim activation, alongside existing TUI keyboard controls.
