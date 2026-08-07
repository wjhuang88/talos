# SKILL-004: Optional Skill Triggers Compatibility

| Field | Value |
|---|---|
| Story ID | SKILL-004 |
| Source Issue | #155 |
| Status | Intake |
| Priority | P1 |
| Type | Skill Format / Compatibility |

## Disposition

Register the ClawHub-compatible `SKILL.md` parsing gap for contract and compatibility refinement.
This intake record does not choose whether omitted `triggers` default to an empty list or remain a
documented Talos requirement, and it does not authorize parser implementation.

## Required follow-up

- Confirm the supported public `SKILL.md` schema and compatibility target.
- Decide omitted-field behavior without weakening validation of malformed trigger values.
- Cover minimal frontmatter, explicit empty triggers, malformed triggers, and current Talos skill
  fixtures with locked tests.
- Update affected skill-author documentation in the same runnable iteration.

## Dependencies

Coordinate with SKILL-001 through SKILL-003 and the current `talos-skill` parser contract. Keep
this compatibility decision separate from I175 conversation-engine source decomposition.
