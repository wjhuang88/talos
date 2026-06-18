# I031: Skill Runtime Activation And CLI Cleanup

**Status**: Planned
**Target Window**: Week 2 of next month plan
**Depends On**: I030 complete preferred

## Outcome

Complete the next architecture residual cleanup slice after session decomposition, then make Skill
a real runtime session capability. ARCH-009 creates clean parser/manager/loader boundaries;
SKILL-001 wires discovery and prompt injection into normal CLI/TUI startup. The lower-risk CLI
portion of ARCH-010 is included only where it supports the startup composition changes.

## Selected Stories

- [ ] #ARCH-009-A: Inventory `talos-skill/src/lib.rs` before decomposition
- [ ] #ARCH-009-B: Extract SKILL.md frontmatter parsing and section helpers into `parser.rs`
- [ ] #ARCH-009-C: Extract skill discovery and index/cache management into `manager.rs`
- [ ] #ARCH-009-D: Extract file loading, embedded asset loading, and path resolution into `loader.rs`
- [ ] #SKILL-001-A: Discover skills at session startup in CLI/TUI paths
- [ ] #SKILL-001-B: Inject Level 0 skill index into `Agent::set_skill_index(...)` before first turn
- [ ] #SKILL-001-C: Define and implement Level 1/Level 2 activation path or explicit command gate
- [ ] #SKILL-001-D: Surface available/active skills in user-visible diagnostics
- [ ] #ARCH-010-A: Extract CLI mode runner functions into `mode_runners.rs` if needed for clean startup composition

## Acceptance Criteria

- [ ] `crates/talos-skill/src/lib.rs` is <=300 lines.
- [ ] `SkillIndex`, `Skill`, `SkillDisclosure`, `SkillLoader`, and `SkillManager` remain
      importable from `talos_skill`.
- [ ] Normal CLI/TUI startup discovers skills and injects Level 0 metadata before the first turn.
- [ ] Bad or duplicate skills do not crash normal startup.
- [ ] Level 1/2 activation behavior is implemented or visibly gated with a follow-up story.
- [ ] `crates/talos-cli/src/main.rs` is <=400 lines, or residual CLI scope is explicitly
      re-registered if the target proves too large for this iteration.
- [ ] No CLI mode behavior changes are intentionally introduced.
- [ ] `cargo test -p talos-skill -p talos-cli` passes.
- [ ] `cargo clippy -p talos-skill -p talos-cli -- -D warnings` passes.
- [ ] `cargo check --workspace` passes.

## Risks

- Skill tests are currently colocated with implementation; moving logic should not weaken test
  coverage.
- CLI mode runners depend on provider, session, registry, TUI, RPC, and MCP composition. Extract
  by moving functions, not by redesigning startup flow.
- Skill set must be discovered before first turn to preserve prompt cache stability.

## Deferred Scope

MCP session integration is a separate follow-up (MCP-001 / I034). `crates/talos-tools/src/file_tools.rs`
remains the high-risk ARCH-010 slice and is scheduled for I032.

## Verification Log

(to be filled as stories land)
