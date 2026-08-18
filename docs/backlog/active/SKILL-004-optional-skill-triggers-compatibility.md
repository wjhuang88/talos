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
