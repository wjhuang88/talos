# SKILL-002: User Explicitly Activates Skill Content

| Field | Value |
|---|---|
| Type | State/Product Story |
| Priority | P1 |
| Status | Refinement |
| Depends On | I033 review closure; CMD-001 first-class BuiltinCommand registry |
| Decision Links | ADR-006; prompt cache constraints recorded by ARCH-006 |

## User Goal And Value

An interactive user needs to explicitly activate a discovered Skill and load its referenced
resources on demand, so the model receives task-specific instructions without injecting every
Skill body into every request or leaking hidden content into history.

## Scope

- Add an explicit BuiltinCommand path for selecting one discovered Skill.
- Load Level 1 `SKILL.md` body through `SkillManager` with a bounded token/byte budget.
- Make active Skill state visible through `/skills` without printing the full body.
- Allow bounded Level 2 reference loading for the active Skill with path confinement.
- Inject activated content into provider context through a typed session owner and define prompt
  cache invalidation/rebuild behavior.

## Exclusions

- Arbitrary commands declared by Skill files; executable extension commands belong to PluginCommand.
- Automatic activation based only on fuzzy intent matching.
- Loading all Skill bodies or references at startup.
- Rendering the inactive `SkillSidebar`.

## Acceptance

- Given a valid Level 0 Skill index, when the user explicitly activates one Skill, then the next
  provider request contains that Skill's Level 1 body and `/skills` reports it active.
- Given an unknown or invalid Skill name, when activation is requested, then the active context is
  unchanged and the user receives a deterministic error.
- Given an active Skill references a confined resource, when the resource is explicitly requested,
  then bounded content reaches model context without being dumped into scrollback history.
- Given a reference escapes the Skill root or exceeds budget, when loading is attempted, then Talos
  rejects or truncates according to documented policy without crashing.
- Given activated content changes the stable prompt prefix, when the next turn runs, then cache
  invalidation/rebuild behavior is deterministic and tested.
- [ ] A real `talos` binary scenario proves activation reaches the provider request.
- [ ] README, SKILL-001/SKILL-002, iteration, Product Backlog, and Board owners are synchronized.

## Uncertainty

- The final context owner and whether activation mutates the stable prefix or a per-turn context
  block must be validated against current Agent/session cache behavior before this Story becomes
  Ready. If the choice changes a public protocol or Soft architecture constraint, record an ADR.

## Required Reads

- `docs/backlog/active/SKILL-001-runtime-skill-activation.md`
- `docs/iterations/I033-runtime-skill-activation.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/backlog/active/ARCH-006-prompt-cache-stability.md`
- `crates/talos-skill/src/lib.rs`
- `crates/talos-agent/src/prompt.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-cli/src/skill_runtime.rs`
