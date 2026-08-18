# Iteration I198: Optional Skill Triggers Compatibility

> Document status: Planned
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
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #155 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | After predecessors have implementation/deferred-validation dispositions, establish an effective claim on `main`; confirm the public compatibility contract before parser edits and branch only from that claim merge or later current `main`. Per-child CI, Agent technical review and CAS remain merge gates; eligible natural-person review moves to VALIDATION-002/I211/Issue #302 while I198 stays Review. |

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
