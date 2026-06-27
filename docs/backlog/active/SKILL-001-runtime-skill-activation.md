# SKILL-001: Runtime Skill Activation

**Status**: Complete (Level 0 runtime activation and explicit Level 1/2 gate verified 2026-06-19)
**Priority**: P1
**Source**: User correction 2026-06-18
**Depends on**: ARCH-009 preferred, prompt cache stability, session startup flow

**Residual owner**: [SKILL-002 explicit runtime activation](SKILL-002-explicit-runtime-activation.md)

## Problem

Talos has a `talos-skill` crate that can parse and manage `SKILL.md` files, and the agent prompt
can render a skill index. However, the runtime startup path does not yet discover skills, inject
the Level 0 index into the agent, or activate Level 1/Level 2 skill content during a session.

This means the mechanism exists as a library, but users cannot rely on Talos to actually call
skills in normal CLI/TUI usage.

## Scope

- Discover skills at session startup from the existing search paths:
  `.talos/skills/`, `~/.talos/skills/`, and inherited parent `.talos/skills/`.
- Inject the Level 0 skill index into `Agent::set_skill_index(...)` before the first model turn.
- Define runtime activation rules for Level 1 skill body loading:
  model-visible skill index first, then explicit activation by matching task/trigger or a user
  command if needed.
- Support Level 2 reference/resource loading through existing `SkillManager::load_reference(...)`.
- Surface available and active skills in TUI/CLI visibility paths without leaking full skill body
  content into history by default.
- Keep prompt cache semantics explicit: skill set is session-stable unless the session is rebuilt.
- Handle bad skills deterministically: skip with diagnostic, or fail startup only under strict mode.
- Expose diagnostics and future explicit activation through BuiltinCommand definitions from
  CMD-001. Skill files do not register arbitrary commands; executable extension commands belong to
  the PluginCommand protocol.

## Acceptance Criteria

- [x] Normal CLI/TUI startup discovers skills and injects Level 0 skill metadata into the system
      prompt.
- [x] A session with no skills still renders a clear `No skills available` prompt section.
- [x] A session with one valid skill exposes that skill name/description to the model before the
      first turn.
- [x] Level 1 skill body activation is implemented or explicitly gated behind a visible command.
- [x] Level 2 reference loading is covered by tests.
- [x] Bad or duplicate skills do not crash normal startup.
- [x] Prompt cache invalidation rules are documented and tested.
- [x] User-facing docs explain where to put skills and how Talos activates them.

## Verification Notes

Add targeted tests around discovery, prompt injection, activation, bad skill handling, duplicate
skill priority, and reference loading. Include at least one CLI or agent integration test proving
runtime startup wires `SkillLoader` to `Agent::set_skill_index(...)`.

2026-06-19 implementation notes:

- Runtime startup now discovers workspace skills and injects Level 0 metadata before the first
  turn in RPC, print, TUI, inline, and legacy interactive modes.
- `/skills` is the visible diagnostic and Level 1/2 gate. It lists available Level 0 metadata and
  explains that full bodies/references require a future explicit activation flow.
- The existing `SkillSidebar` was not used for this work because it is not currently rendered in
  the TUI layout.
- Prompt cache semantics: the skill set is session-start stable. Changing skill files requires
  rebuilding the session/runtime to refresh the stable prompt prefix.
- Level 1/2 execution is not an I033 scope extension. SKILL-002 owns the separate explicit
  activation workflow and remains in Refinement until its context/cache ownership is resolved.
- A binary-facing regression test now creates a workspace Skill, launches the real `talos`
  executable in mock request-preview mode, and verifies that Level 0 metadata reaches the provider
  request. This closes SKILL-001's published runtime wiring scope.

2026-06-27 residual closure note:

- SKILL-002/I058 implemented explicit Level 1 Skill body activation and bounded Level 2 reference
  loading through typed runtime/session context. A real `talos --inline --mock` binary regression
  now proves `/skills activate <name>` reaches provider request preview. SKILL-001 remains closed;
  explicit activation review is owned by SKILL-002/I058.

## Required Reads

- `docs/backlog/active/ARCH-009-skill-module-decomposition.md`
- `docs/iterations/I031-skill-and-cli-module-cleanup.md`
- `docs/backlog/active/CMD-001-interactive-command-runtime-contract.md`
- `docs/backlog/active/SKILL-002-explicit-runtime-activation.md`
- `crates/talos-skill/src/lib.rs`
- `crates/talos-agent/src/prompt.rs`
- `crates/talos-agent/src/lib.rs`
