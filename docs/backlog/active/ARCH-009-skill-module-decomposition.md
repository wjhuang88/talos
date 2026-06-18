# ARCH-009: Skill Module Decomposition

**Status**: Planned
**Priority**: P2
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

`crates/talos-skill/src/lib.rs` remains at 1484 lines. It mixes skill discovery, SKILL.md
parsing, progressive-disclosure assembly, frontmatter extraction, skill index management, and
test helpers. The original ARCH-002 audit identified three natural split points.

## Scope

Decompose `talos-skill/src/lib.rs` without behavior changes:

- `parser.rs` — SKILL.md frontmatter parsing, section extraction, progressive disclosure
  assembly logic.
- `manager.rs` — `SkillIndex` struct, skill discovery (filesystem scan, embedded lookup),
  caching.
- `loader.rs` — file I/O, embedded asset loading, path resolution.
- Keep `lib.rs` as the thin re-export surface.

## Acceptance Criteria

- [ ] `talos-skill/src/lib.rs` is ≤300 lines after decomposition.
- [ ] No behavior changes. All existing public types (`SkillIndex`, `Skill`, `SkillLevel`)
      remain accessible at the same import paths.
- [ ] `cargo test -p talos-skill` passes.
- [ ] `cargo clippy -p talos-skill -- -D warnings` passes.
- [ ] Architecture reference updated.

## Verification Notes

Baseline: `talos-skill/src/lib.rs` at 1484 lines (2026-06-18 audit).
