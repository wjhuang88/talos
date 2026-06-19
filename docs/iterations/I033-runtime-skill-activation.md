# I033: Runtime Skill Activation

**Status**: Partial (Level 0 runtime activation landed 2026-06-19)
**Target Window**: After I030-I032 architecture cleanup
**Depends On**: I031 complete preferred, prompt cache stability

## Outcome

Make Skill a first-class runtime session capability. Normal CLI/TUI startup should discover
configured skills, inject Level 0 skill metadata into the agent before the first model turn, and
define the activation path for Level 1 skill bodies and Level 2 references.

## Selected Stories

- [x] #SKILL-001-A: Discover skills at session startup in CLI/TUI paths
- [x] #SKILL-001-B: Inject Level 0 skill index into `Agent::set_skill_index(...)` before first turn
- [x] #SKILL-001-C: Define Level 1 skill body activation as an explicit follow-up gate
- [x] #SKILL-001-D: Preserve Level 2 reference/resource loading coverage in `talos-skill`
- [x] #SKILL-001-E: Surface available skills through `/skills` diagnostics
- [x] #SKILL-001-F: Document skill locations, activation behavior, and cache semantics

## Acceptance Criteria

- [x] Normal CLI/TUI startup discovers skills and injects Level 0 metadata before the first turn.
- [x] No-skill sessions still render a clear `No skills available` prompt section.
- [x] Bad or duplicate skills do not crash normal startup.
- [x] Level 1/2 activation behavior is visibly gated with a follow-up story.
- [x] Prompt cache invalidation rules are documented and tested.
- [x] User-facing docs explain where to put skills and how Talos activates them.
- [x] Targeted agent/CLI tests prove `SkillLoader` reaches `Agent::set_skill_index(...)`.

## Risks

- Skill set must be discovered before the first turn to preserve prompt cache stability.
- Full skill bodies can be large; activation should be explicit and token-budget aware.
- Skill content should not be dumped into history by default.

## Verification Log

2026-06-19:

- Added `SkillLoader::for_workspace(...)` so runtime discovery follows the active session workspace
  instead of process current directory.
- Added `talos-cli::skill_runtime` to discover skills, compute the Level 0 index through
  `SkillManager`, and call `Agent::set_skill_index(...)` before first turn submission.
- Wired runtime skill injection into RPC, print, TUI, inline, and legacy interactive agent startup.
- Added `/skills` conversation diagnostics. This is the current visible gate for runtime skills:
  it lists Level 0 metadata and explicitly states that Level 1 bodies and Level 2 references are
  gated until an explicit activation flow lands.
- Removed reliance on the existing TUI `SkillSidebar` for I033 because it is not currently rendered
  in the layout; the status hint now points users to `/skills`.
- Existing `talos-skill` tests continue covering Level 1 loading and Level 2 reference loading.
- Verification:
  - `cargo test -p talos-skill` passed.
  - `cargo test -p talos-conversation` passed.
  - `cargo test -p talos-cli` passed.
  - `cargo clippy -p talos-cli -p talos-conversation -- -D warnings` passed.
