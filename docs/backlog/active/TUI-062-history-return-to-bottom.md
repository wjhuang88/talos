# TUI-062: History Return To Bottom

> Document status: Active

| Field | Value |
|---|---|
| Story ID | TUI-062 |
| Type | TUI interaction enhancement |
| Priority | P1 |
| Status | Active |
| Source | User request, 2026-09-30 |
| Selected Iteration | I289 |
| Depends On | Existing history scroll projection and follow-tail state |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | Presentation-only history return-to-bottom affordance: prompt while anchored and Ctrl+Down to restore FollowTail. |
| Claimed At | 2026-09-30 |
| Source Issue | None |
| Governance Claim PR | #637 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User requested correction on 2026-10-03. No independent maintainer is available in this session; exact-head CI, Agent review, validators and CAS required. Proposed claim is ineffective until #637 merges. |
| Implementation PR | #628 (existing candidate; blocked on effective claim) |
| Last Updated | 2026-10-03 |
| Handoff / Release Condition | Activate claim on target branch before stable implementation submission. |

## Scope

- Show a transient bottom-area hint while history is manually anchored.
- Make `Ctrl+Down` restore follow-tail mode and clear the hint on the next frame.
- Preserve existing Esc cancellation, approval, menu, selection, and PageUp/PageDown behavior.

## Acceptance

- Given the history is anchored above the tail, the bottom hint says `Ctrl+Down to return to bottom`.
- When `Ctrl+Down` is pressed, history returns to FollowTail and newly appended content is visible.
- Given FollowTail mode, the existing default tips remain unchanged.

## Validation

- Focused `talos-tui` tests for key routing and rendered tip state.
- `cargo test -p talos-tui --locked --offline`.
- Governance validators before candidate submission.

## Verification Evidence

- `cargo test -p talos-tui --locked --offline`: 592 unit, 2 integration, and 2 doc tests passed.
- `bash scripts/validate_collaboration_claims.sh .`: passed with 0 warnings.
- `bash scripts/validate_project_governance.sh .`: passed with 0 warnings.

## Documentation To Update

- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- `README.md` and `README.zh-CN.md` (TUI shortcut reference)

## Completion Evidence

- Completion Commit: d6d90412 (candidate; owner remains Review pending native acceptance and target-branch review)

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
