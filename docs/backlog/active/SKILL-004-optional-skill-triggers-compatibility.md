# SKILL-004: Optional Skill Triggers Compatibility

> Document status: Complete / Closed

| Field | Value |
|---|---|
| Story ID | SKILL-004 |
| Source Issue | #155 |
| Status | Complete / Closed |
| Priority | P1 |
| Type | Skill Format / Compatibility |
| Selected Iteration | I198 — Complete / Closed |
| Depends On | Current `talos-skill` parser and SKILL-001 through SKILL-003 compatibility contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline session |
| Work Slice | I198/SKILL-004 only: confirm the additive omitted-`triggers` contract; default an omitted field to an empty list; preserve explicit empty/non-empty lists and malformed-value rejection; add focused parser/runtime fixtures; update English and Chinese skill-author documentation. Excludes discovery, linked-skill policy, trigger routing, permissions, registry/ClawHub clients, dependencies, persistence, release/publication and unrelated I211 work. |
| Claimed At | 2026-08-20 |
| Source Issue | #155 |
| Governance Claim PR | #324 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #324 exact head `a06e34a51dabd33a3204d2e96e749f2342545438` passed CI `32337065552`, independent Agent claim review `5351981686`, both governance validators and merge-time CAS, then merged to `main` as `ea6686855de971df42de0311333617090c30de47`. |
| Implementation PR | #325 |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Closed after PR #325 delivered compatibility and I232/SKILL-005 corrected explicit malformed-input diagnostics through PR #424 merge `fedd6fac`. |

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
fail. Governance PR #324 now contains the finalized proposed claim. It remains ineffective until
independent exact-head review, CI, merge-time CAS and target-branch merge pass.

## 2026-08-20 Implementation Characterization Correction

The first implementation test pass disproved one over-broad preparation assumption: `yaml_serde`
coerces a numeric sequence item such as `42` into the string `"42"`; `Vec<String>` does not reject
every non-string YAML scalar. Scalar and mapping containers plus mapping entries do fail with
`triggers` diagnostics. The implementation therefore preserves existing scalar coercion, adds a
regression for it, and limits the compatibility change to the missing-field default. Tightening
accepted scalar coercion would be a separate format-policy change and is not authorized by I198.

This correction does not change the omitted-field goal or public struct shape. It narrows an
incorrect evidence statement before implementation completion rather than silently expanding the
parser contract.

## 2026-08-20 Claim Activation And Implementation Checkpoint

PR #324 final head `a06e34a51dabd33a3204d2e96e749f2342545438` passed CI `32337065552`,
independent Agent claim review `5351981686`, both governance validators and merge-time CAS, then
merged as `ea6686855de971df42de0311333617090c30de47`. The implementation worktree starts exactly
from that merge.

The implementation adds only the missing-field default, focused parser compatibility tests, one
real-binary inline activation fixture without `triggers`, and bilingual documentation. No Cargo,
dependency, discovery/routing, permission, persistence, release or publication surface changes.

## 2026-08-20 Implementation Validation Checkpoint

The implementation worktree passed `cargo test -p talos-skill --locked` (81 unit tests and one
doctest), `cargo test -p talos-cli --test skill_runtime_e2e --locked` (2 tests), strict
`talos-skill` Clippy, both governance validators, manifest YAML parsing, `git diff --check`, and the
complete `./scripts/release_preflight.sh`. The real `talos` binary fixture proves that a discovered
Skill without `triggers` can be explicitly activated and projected into the request preview.

SKILL-004 is now `Review / Claimed`. Exact-head implementation CI, independent Agent technical
review, merge-time CAS and the Issue #302 natural-person compatibility row remain open; no
completion is claimed.

## 2026-08-20 Natural-Person Compatibility Disposition

On integrated `main@a2f43248`, the maintainer confirmed through the real binary that omitted,
explicit-empty and non-empty string-list triggers remain discoverable and explicitly activatable,
and their bodies reach the mock request preview. Malformed scalar and mapping trigger containers
remain excluded, but explicit activation exposes only a generic not-found error instead of the
required field-specific diagnostic.

SKILL-005 / Issue #333 is the separate Ready/Unclaimed corrective owner for diagnostic visibility.
SKILL-004 remains Review with Completion Commit Pending; I211 records evidence only and grants no
implementation authority.

The preceding paragraph is a dated pre-correction checkpoint. It is superseded by the 2026-08-28
completion record below and is retained only for historical accuracy.

## 2026-08-28 Corrective Completion

I232/SKILL-005 completed the failed malformed-input activation row through implementation source
`fb47b0c2`, exact-head CI `33141878176`, independent review `5448628671` and merge `fedd6fac`.
Completion Commits: `f719ed913d36ad7ad00f5a99d3d990b414dbbd5d` and
`fedd6fac94708628478836b94b6fd01954de53e0`. This status-only closeout does not self-certify completion.
