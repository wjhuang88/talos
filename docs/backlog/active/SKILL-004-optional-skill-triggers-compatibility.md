# SKILL-004: Optional Skill Triggers Compatibility

| Field | Value |
|---|---|
| Story ID | SKILL-004 |
| Source Issue | #155 |
| Status | Ready — I198 Planned / Unclaimed |
| Priority | P1 |
| Type | Skill Format / Compatibility |
| Selected Iteration | I198 — Planned / Unclaimed |
| Depends On | Current `talos-skill` parser and SKILL-001 through SKILL-003 compatibility contracts |

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
| Handoff / Release Condition | Establish an effective I198 claim on `main`; confirm the compatibility contract before editing the parser; implement from the claim merge or later current `main`. Per-child CI, Agent technical review and CAS remain merge gates; eligible natural-person compatibility review moves to VALIDATION-002/I211/Issue #302 while this Story stays Review. |

## Claim Preparation - 2026-08-20

| Field | Value |
|---|---|
| Claim State | Unclaimed; governance proposal not yet effective |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I198/SKILL-004 only: confirm the additive omitted-`triggers` contract; default an omitted field to an empty list; preserve explicit empty/non-empty lists and malformed-value rejection; add focused parser/runtime fixtures; update English and Chinese skill-author documentation. Excludes discovery, linked-skill policy, trigger routing, permissions, registry/ClawHub clients, dependencies, persistence, release/publication and unrelated I211 work. |
| Claimed At | Not effective until target-branch merge |
| Source Issue | #155 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review proposed |
| Authorization Evidence | Pending exact-head review, CI, both governance validators and merge-time CAS. |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | This preparation record grants no implementation authority. Finalize the actual claim PR number, merge the claim to `main`, then create implementation work only from that merge or later exact `main`. |

## Goal And Compatibility Contract

Register the ClawHub-compatible `SKILL.md` parsing gap for contract and compatibility refinement.
I198's planned compatibility target is that omitted `triggers` deserialize as an empty list while
explicit trigger lists retain current behavior and malformed trigger values still fail validation.
This is a planning assumption, not an implemented or accepted public contract: I198 must confirm it
against current fixtures and public documentation before changing parser behavior. If that review
finds a breaking semantic or format conflict, I198 stops at the decision checkpoint and routes the
change through an ADR/migration owner instead of silently widening scope.

## Scope

- Confirm the supported public `SKILL.md` schema and compatibility target before parser edits.
- Make omitted `triggers` equivalent to explicit `triggers: []` only if the contract checkpoint
  confirms this is additive and compatible.
- Cover minimal frontmatter, explicit empty triggers, malformed triggers, and current Talos skill
  fixtures with locked tests.
- Update affected skill-author documentation in the same runnable iteration.

## Exclusions

- No skill discovery, linked-skill policy, invocation routing, permission, remote registry, ClawHub
  client or unrelated frontmatter change.
- No weakening of malformed-value validation and no acceptance claim for arbitrary external skill
  formats.
- No implementation before I198 has an effective Collaboration Claim.

## Acceptance

- A minimal otherwise-valid `SKILL.md` with omitted `triggers` has a deterministic, documented
  result matching the confirmed I198 contract.
- Explicit empty and explicit non-empty trigger lists retain deterministic behavior.
- Malformed trigger types or entries fail with actionable diagnostics.
- Current repository skill fixtures and focused compatibility tests pass with `--locked` workspace
  validation.
- Skill-author documentation states whether `triggers` is optional and the behavior when omitted.

## Dependencies

Coordinate with SKILL-001 through SKILL-003 and the current `talos-skill` parser contract. Keep
this compatibility decision separate from I175 conversation-engine source decomposition.

## State Owners And Residuals

- Story scope and acceptance: this file.
- Execution baseline: `docs/iterations/I198-skill004-optional-triggers-compatibility.md`.
- Remote discussion: Issue #155.
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`.
- Broader ClawHub-format compatibility requires a separate owner and iteration.

## 2026-08-18 Deferred Human Validation Timing

The maintainer selected the long-task Deferred Human Validation Mode. The public-contract decision,
malformed-value safety and documentation acceptance remain unchanged. After exact-head CI,
independent Agent technical review and CAS, the natural-person compatibility review may be added to
Issue #302 for I211. SKILL-004 remains Review until that row passes; a breaking contract still
stops for an ADR/migration owner.

## 2026-08-20 Compatibility Decision And Claim Preparation

Current code and public documentation were checked before parser edits. `SkillFrontmatter` keeps
the same public `Vec<String>` field, explicit `triggers: []` already succeeds, explicit non-empty
lists preserve their values, and malformed YAML/types already fail in `serde_yaml`. Omission fails
only because the field has no serde default. The public type comment describes all fields as
required, but no accepted ADR, generated schema or migration guarantee freezes that rejection as an
invariant. Updating that current-behavior statement together with the parser therefore forms an
additive input-compatibility extension rather than a public type or semantic break.

The proposed implementation may add only a missing-field default and the acceptance fixtures above.
It must stop and create a separate ADR/migration owner if implementation evidence shows malformed
values become accepted, explicit lists change, or another published contract requires omission to
fail. This governance proposal remains ineffective until its actual PR number, exact-head review,
CI and merge-time CAS are complete on `main`.
