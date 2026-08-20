# Iteration I198: Optional Skill Triggers Compatibility

> Document status: Active / Claimed via governance PR #324; ineffective before target-branch merge
> Published plan date: 2026-08-14
> Planned objective: decide and implement the smallest compatible omitted-`triggers` contract for
> otherwise-valid `SKILL.md` files without weakening malformed-value validation or changing skill
> discovery and invocation policy.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a maintainer can load a minimal otherwise-valid skill with omitted `triggers`,
> observe the documented deterministic result, and run compatibility fixtures proving explicit
> triggers and malformed values retain their intended behavior.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I198/SKILL-004 only: confirm the additive omitted-`triggers` contract; default an omitted field to an empty list; preserve explicit empty/non-empty lists and malformed-value rejection; add focused parser/runtime fixtures; update English and Chinese skill-author documentation. Excludes discovery, linked-skill policy, trigger routing, permissions, registry/ClawHub clients, dependencies, persistence, release/publication and unrelated I211 work. |
| Claimed At | 2026-08-20 |
| Source Issue | #155 |
| Governance Claim PR | #324 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #324 must pass independent exact-head claim review, exact-head CI, both governance validators and merge-time CAS. The proposed claim remains ineffective while the PR is open. |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | PR #324 exact-head review, CI and CAS must pass and the claim must reach `main`. Only then branch from that merge or later current `main`; implementation CI/review/CAS and Issue #302 human validation remain required. |

## Published Baseline

### Selected Story

| Story | Status At Selection | Depends On | Outcome |
|---|---|---|---|
| SKILL-004 / Issue #155 | Ready | Current `talos-skill` parser and SKILL-001 through SKILL-003 contracts | One runnable additive compatibility slice with parser fixtures and skill-author documentation |

### Decision Checkpoint

The planned compatibility target is: omitted `triggers` behaves like explicit `triggers: []`,
explicit lists retain current semantics, and malformed values still fail. Before production edits,
the iteration must verify that this is additive against current public docs, serde/schema behavior
and fixtures. If it is breaking or conflicts with a published invariant, stop implementation and
create the required ADR/migration owner; do not silently choose a different contract.

### Scope

- Characterize current frontmatter deserialization, validation and diagnostics for omitted, empty,
  non-empty and malformed `triggers`.
- Implement only the confirmed omitted-field compatibility behavior.
- Add focused fixtures/tests for all four cases and preserve current repository skill fixtures.
- Update the affected skill-author documentation and schema examples.

### Non-Goals

- No linked-skill discovery policy, trigger matching/routing semantics, permission, remote registry,
  ClawHub client, unrelated frontmatter field or conversation-engine change.
- No broad external-format compatibility claim and no weakening of invalid-input diagnostics.
- No new dependency, public API break or format migration without a separate accepted decision.

### Acceptance

- Minimal valid frontmatter with omitted `triggers` produces the confirmed documented result.
- Explicit empty and non-empty trigger lists remain deterministic and backward compatible.
- Malformed trigger values fail with actionable diagnostics.
- Existing skill fixtures plus focused parser/schema tests and locked workspace validation pass.
- User-facing skill-author docs match the shipped contract.

### Planned Validation

- Focused `talos-skill` parser and schema tests with omitted/empty/non-empty/malformed fixtures.
- Relevant crate tests and `cargo test --workspace --locked`.
- Repository release preflight, both governance validators and `git diff --check`.
- Independent natural-person exact-head compatibility review; shared-account identity and role
  separation must be disclosed.

### Risks And Fallback

- Risk: a serde default can accidentally accept malformed values or change generated schema truth.
- Risk: claiming broad ClawHub compatibility from one optional field would overstate acceptance.
- Fallback: preserve current parser behavior and keep I198 Blocked/Partial while routing a breaking
  contract through a dedicated ADR/migration owner.

## Actual Activation And Execution

No activation has occurred. This planned iteration remains Unclaimed and follows I197 in the
mainline priority long task; that order is coordination, not implementation authority.

## Verification Evidence

Pending implementation after an effective claim reaches `main`.

## Completion Evidence

Completion Commit: Pending. A status-only commit cannot serve as its own evidence.

## Variance And Residuals

Broader ClawHub compatibility, registry integration and trigger-routing changes require separate
owners and iterations.

## Change Control — 2026-08-14

The maintainer added Issues #69, #79 and #111 to the coordinating long-running task after this
baseline was published. I198's objective, scope and acceptance remain unchanged. Its activation
order now follows I201/#111 disposition; this is coordination only and creates no technical
dependency between the TUI and skill-format slices.

## Change Control - 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode. I198's decision checkpoint,
parser fixtures, documentation and malformed-input acceptance remain unchanged. Exact-head CI,
independent Agent technical review, locked checks, governance validation and CAS remain local merge
gates. The natural-person compatibility review may move to VALIDATION-002/I211/Issue #302 after
implementation merge; I198 remains Review until that row passes. A breaking contract still stops
for a separate ADR/migration owner.

## Retrospective

Pending execution.

## Exact-Main Claim Inventory - 2026-08-20

Baseline: `main@9d5c8a71718b44d424092a45a75d3da0d593547d` after refreshing `origin`; local and
remote heads matched and the primary worktree was clean.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I198 may be proposed, but remains unactivated until its finalized claim reaches `main`. |
| Review / implementation merged | I197, I200, I201, I210, I212 | Preserve every Issue #302/I211 natural-person/manual row. These scopes do not transfer authority or overlap I198 parser compatibility. |
| Planned / Claimed | I189 | Keep unactivated; its protected permission scope is independent of I198. |
| Planned / Unclaimed | I198, I206, I207, I208, I211 | Preserve their owners. I198 is the next ordered implementation child; I211 remains the later evidence-only cleanup. |
| Paused | I164 | Preserve its superseded target; do not resume. |
| Blocked | None with a current iteration document status | Backlog-level blockers, including Issue #59 production children, retain their independent owners and gates. |

Open PRs #120/#121 are archival Drafts. No open implementation or claim PR targets Issue #155,
SKILL-004 or I198. The retained I201 and I210 worktrees are historical evidence and must not be
modified or reused. Stashes `stash@{0}` and `stash@{1}` remain historical and must not be restored
as a unit.

## 2026-08-20 Compatibility Decision And Claim Preparation

Read-only characterization confirms the planned contract is additive: `SkillFrontmatter` remains
the same public struct with `triggers: Vec<String>`; explicit empty and non-empty lists already
parse deterministically; malformed YAML and wrong trigger types fail before validation; omission
fails only at serde deserialization because the vector field has no default. The public comment
describing all fields as required records current behavior, but no ADR, generated schema or migration
contract freezes omission rejection. The implementation may therefore update that comment and add
a missing-field default without a public type break.

The preparation branch is governance-only. PR #324 now contains the finalized proposed claim.
Until it is independently reviewed and merged, target-branch I198 remains Planned/Unclaimed and no
parser, Rust, Cargo, dependency, version, tag or publication action is authorized.
