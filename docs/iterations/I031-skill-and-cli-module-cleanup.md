# I031: Skill And CLI Module Cleanup

**Status**: Planned (remaining CLI slice; ARCH-009 pulled forward 2026-06-19)
**Target Window**: Week 2 of next month plan
**Depends On**: I030 complete preferred

## Outcome

Complete the next architecture residual cleanup slice after session decomposition. The ARCH-009
skill module decomposition was pulled forward and completed alongside I030 on 2026-06-19 because it
was a pure library boundary split and directly reduces risk before runtime Skill/MCP activation.

The remaining I031 scope is the lower-risk CLI portion of ARCH-010. This keeps startup mode
boundaries understandable before runtime Skill/MCP features expand those surfaces.

## Selected Stories

- [x] #ARCH-009-A: Inventory `talos-skill/src/lib.rs` before decomposition
- [x] #ARCH-009-B: Extract SKILL.md frontmatter parsing and section helpers into `parser.rs`
- [x] #ARCH-009-C: Extract skill discovery and index/cache management into `manager.rs`
- [x] #ARCH-009-D: Extract file loading, embedded asset loading, and path resolution into `loader.rs`
- [ ] #ARCH-010-A: Extract CLI mode runner functions into `mode_runners.rs`
- [ ] #ARCH-010-B: Keep public imports and CLI behavior stable; update architecture docs

## Acceptance Criteria

- [x] `crates/talos-skill/src/lib.rs` is <=300 lines.
- [x] `SkillIndex`, `Skill`, `SkillDisclosure`, `SkillLoader`, and `SkillManager` remain
      importable from `talos_skill`.
- [ ] `crates/talos-cli/src/main.rs` is <=400 lines, or residual CLI scope is explicitly
      re-registered if the target proves too large for this iteration.
- [ ] No CLI mode behavior changes are intentionally introduced.
- [ ] `cargo test -p talos-cli` passes.
- [ ] `cargo clippy -p talos-cli -- -D warnings` passes.
- [ ] `cargo check --workspace` passes.

## Risks

- Skill tests are currently colocated with implementation; moving logic should not weaken test
  coverage.
- CLI mode runners depend on provider, session, registry, TUI, RPC, and MCP composition. Extract
  by moving functions, not by redesigning startup flow.

## Deferred Scope

Runtime Skill activation is a separate follow-up (SKILL-001 / I033). MCP session integration is a
separate follow-up (MCP-001 / I034). `crates/talos-tools/src/file_tools.rs` remains the high-risk
ARCH-010 slice and is scheduled for I032.

## Verification Log

2026-06-19:

- ARCH-009 skill module decomposition pulled forward and completed:
  - `talos-skill/src/lib.rs`: 1484 lines → 45 lines.
  - Added `error.rs`, `types.rs`, `token.rs`, `parser.rs`, `loader.rs`, `manager.rs`,
    `tests.rs`.
  - Public `talos_skill::*` imports preserved.
- Verification:
  - `cargo test -p talos-skill` passed: 46 tests + 2 doctests.
  - `cargo clippy -p talos-skill -- -D warnings` passed.
- Deferred deliberately:
  - CLI mode runner extraction remains in I031 because it touches CLI startup composition and
    should be verified as its own behavior-sensitive slice.
