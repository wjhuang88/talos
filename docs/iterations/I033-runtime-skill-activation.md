# I033: Runtime Skill Activation

**Status**: Planned
**Target Window**: After I030-I032 architecture cleanup
**Depends On**: I031 complete preferred, prompt cache stability

## Outcome

Make Skill a first-class runtime session capability. Normal CLI/TUI startup should discover
configured skills, inject Level 0 skill metadata into the agent before the first model turn, and
define the activation path for Level 1 skill bodies and Level 2 references.

## Selected Stories

- [ ] #SKILL-001-A: Discover skills at session startup in CLI/TUI paths
- [ ] #SKILL-001-B: Inject Level 0 skill index into `Agent::set_skill_index(...)` before first turn
- [ ] #SKILL-001-C: Define and implement Level 1 skill body activation or explicit command gate
- [ ] #SKILL-001-D: Cover Level 2 reference/resource loading in runtime tests
- [ ] #SKILL-001-E: Surface available/active skills in CLI/TUI diagnostics
- [ ] #SKILL-001-F: Document skill locations, activation behavior, and cache semantics

## Acceptance Criteria

- [ ] Normal CLI/TUI startup discovers skills and injects Level 0 metadata before the first turn.
- [ ] No-skill sessions still render a clear `No skills available` prompt section.
- [ ] Bad or duplicate skills do not crash normal startup.
- [ ] Level 1/2 activation behavior is implemented or visibly gated with a follow-up story.
- [ ] Prompt cache invalidation rules are documented and tested.
- [ ] User-facing docs explain where to put skills and how Talos activates them.
- [ ] Targeted agent/CLI tests prove `SkillLoader` reaches `Agent::set_skill_index(...)`.

## Risks

- Skill set must be discovered before the first turn to preserve prompt cache stability.
- Full skill bodies can be large; activation should be explicit and token-budget aware.
- Skill content should not be dumped into history by default.

## Verification Log

(to be filled as stories land)
