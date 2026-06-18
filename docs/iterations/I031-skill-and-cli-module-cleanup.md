# I031: Skill And CLI Module Cleanup

**Status**: Planned
**Target Window**: Week 2 of next month plan
**Depends On**: I030 complete preferred

## Outcome

Complete the next architecture residual cleanup slice after session decomposition: ARCH-009
skill module decomposition plus the lower-risk CLI portion of ARCH-010. This keeps module
boundaries understandable before protocol/config work expands those surfaces.

## Selected Stories

- [ ] #ARCH-009-A: Inventory `talos-skill/src/lib.rs` before decomposition
- [ ] #ARCH-009-B: Extract SKILL.md frontmatter parsing and section helpers into `parser.rs`
- [ ] #ARCH-009-C: Extract skill discovery and index/cache management into `manager.rs`
- [ ] #ARCH-009-D: Extract file loading, embedded asset loading, and path resolution into `loader.rs`
- [ ] #ARCH-010-A: Extract CLI mode runner functions into `mode_runners.rs`
- [ ] #ARCH-010-B: Keep public imports and CLI behavior stable; update architecture docs

## Acceptance Criteria

- [ ] `crates/talos-skill/src/lib.rs` is <=300 lines.
- [ ] `crates/talos-cli/src/main.rs` is <=400 lines, or residual CLI scope is explicitly
      re-registered if the target proves too large for this iteration.
- [ ] `SkillIndex`, `Skill`, `SkillDisclosure`, `SkillLoader`, and `SkillManager` remain
      importable from `talos_skill`.
- [ ] No CLI mode behavior changes are intentionally introduced.
- [ ] `cargo test -p talos-skill -p talos-cli` passes.
- [ ] `cargo clippy -p talos-skill -p talos-cli -- -D warnings` passes.
- [ ] `cargo check --workspace` passes.

## Risks

- Skill tests are currently colocated with implementation; moving logic should not weaken test
  coverage.
- CLI mode runners depend on provider, session, registry, TUI, RPC, and MCP composition. Extract
  by moving functions, not by redesigning startup flow.

## Deferred Scope

`crates/talos-tools/src/file_tools.rs` remains the high-risk ARCH-010 slice and is scheduled for
I032 unless I031 finishes with enough time and clean verification.

## Verification Log

(to be filled as stories land)
