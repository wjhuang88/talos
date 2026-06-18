# ARCH-009: Skill Module Decomposition

**Status**: Complete (2026-06-19)
**Priority**: P2
**Source**: Architecture decay audit 2026-06-18 (post-ARCH-005)
**Depends on**: ARCH-005 partial complete

## Problem

`crates/talos-skill/src/lib.rs` remained at 1484 lines. It mixed skill discovery, SKILL.md
parsing, progressive-disclosure assembly, frontmatter extraction, skill index management, and
test helpers. The original ARCH-002 audit identified three natural split points.

## Scope

Decompose `talos-skill/src/lib.rs` without behavior changes:

- `error.rs` — `SkillError` and `Result` public error surface.
- `types.rs` — public skill data types (`Skill`, `SkillFrontmatter`, `SkillIndex`,
  `SkillDisclosure`).
- `token.rs` — token-estimation helper.
- `parser.rs` — SKILL.md frontmatter splitting and validation.
- `loader.rs` — file I/O, filesystem discovery, SKILL.md parsing, path resolution.
- `manager.rs` — `SkillManager`, Level 0 index caching, Level 1 skill activation, Level 2
  reference loading.
- `tests.rs` — existing skill unit tests moved out of the public re-export surface.
- Keep `lib.rs` as the thin re-export surface.

## Acceptance Criteria

- [x] `talos-skill/src/lib.rs` is ≤300 lines after decomposition.
- [x] No behavior changes. Existing public types (`SkillIndex`, `Skill`, `SkillDisclosure`,
      `SkillFrontmatter`, `SkillLoader`, `SkillManager`, `SkillError`, `Result`) remain
      accessible at the same import paths.
- [x] `cargo test -p talos-skill` passes.
- [x] `cargo clippy -p talos-skill -- -D warnings` passes.
- [x] Architecture reference updated.

## Verification Notes

Baseline: `talos-skill/src/lib.rs` at 1484 lines (2026-06-18 audit).

Completion evidence (2026-06-19):

- `talos-skill/src/lib.rs`: 45 lines after decomposition.
- New focused modules: `error.rs`, `types.rs`, `token.rs`, `parser.rs`, `loader.rs`,
  `manager.rs`, `tests.rs`.
- Function/type inventory preserved:
  - Public types: `SkillError`, `Result`, `SkillFrontmatter`, `Skill`, `SkillIndex`,
    `SkillDisclosure`, `SkillLoader`, `SkillManager`.
  - Public functions/methods: `estimate_tokens`, `SkillLoader::new`, `discover`, `parse`,
    `get_index`, `Default`, `SkillManager::new`, `get_index`, `get_index_tokens`,
    `load_skill`, `load_reference`, `match_skill`, `unload_skill`, `get_active_skills`.
  - Internal helpers moved: `split_frontmatter`, `validate_frontmatter`, `home_dir`.
- Verification:
  - `cargo test -p talos-skill` passed: 46 tests + 2 doctests.
  - `cargo clippy -p talos-skill -- -D warnings` passed.
