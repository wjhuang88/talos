# SKILL-005: Invalid Skill Diagnostic Visibility

| Field | Value |
|---|---|
| Story ID | SKILL-005 |
| Type | Bug / Skill CLI Compatibility Story |
| Priority | P1 corrective residual from I211 |
| Status | Review / Claimed |
| Source | [GitHub Issue #333](https://github.com/wjhuang88/talos/issues/333) |
| Selected Iteration | I232 (claim PR #423 merged as `f2c98ff1`) |
| Depends On | SKILL-004/I198 optional-trigger compatibility; existing parser diagnostics |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline SKILL-005 session |
| Work Slice | Implement only SKILL-005 invalid local Skill diagnostic visibility during explicit activation; preserve fail-closed exclusion and absent behavior. Exclude parser policy, routing, permission, persistence, release and `/auto`. |
| Claimed At | 2026-08-28 |
| Source Issue | #333 |
| Governance Claim PR | #423 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer-directed long-task objective; finalized claim requires exact-head CI, validators and review before merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Claim PR #423 merged as `f2c98ff1`; implementation candidate requires exact-head CI/review. |

## Identity / Goal / Value

Keep malformed Skill documents safely excluded while making explicit CLI activation distinguish an
invalid local document from a genuinely absent Skill.

## Observed Failure

I211 real-binary validation confirmed that malformed scalar and mapping `triggers` containers are
excluded from `/skills`, but `/skills activate bad-scalar` and `/skills activate bad-map` report
only `skill '...' was not found`. The parser's field-specific diagnostic is not visible through the
actual activation path.

## Scope

- Preserve fail-closed exclusion of malformed Skill documents.
- Surface a bounded actionable `triggers` diagnostic when explicit activation targets a known
  invalid local Skill document.
- Keep truly absent Skills distinguishable from invalid documents.
- Cover scalar and mapping containers through the real CLI binary path.

## Exclusions

- No automatic activation, discovery policy, routing, permission or persistence change.
- No broad parser redesign, dependency, release or publication work.

## Evidence And Required Reads

- I198 implementation evidence `f719ed91`, merged through PR #325 as `15a3d424`.
- I211 integrated real-binary evidence in Issue #302 and PR #331.
- `docs/backlog/active/SKILL-004-optional-skill-triggers-compatibility.md`
- `docs/iterations/I198-skill004-optional-triggers-compatibility.md`

## Residual Destination

This intake changes no runtime behavior. Select a separate iteration and effective claim before
implementation; I198 remains Review and I211 remains evidence-only.
