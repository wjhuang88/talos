# SKILL-002: User Explicitly Activates Skill Content

| Field | Value |
|---|---|
| Type | State/Product Story |
| Priority | P1 |
| Status | Complete (I058 runtime evidence accepted 2026-06-29; reconciled 2026-07-22) |
| Depends On | I033 review closure; CMD-001 first-class BuiltinCommand registry |
| Decision Links | ADR-006; prompt cache constraints recorded by ARCH-006 |

> Completion Commit: `2495855` — `feat(cli): implement explicit runtime skill activation (#I058)`

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
- [x] A real `talos` binary scenario proves activation reaches the provider request.
- [x] README, SKILL-002, iteration, Product Backlog, and Board owners are synchronized.

## Implementation Evidence

- `talos-agent` accepts an optional activated Skill context, renders it as a cacheable prompt
  section, and invalidates the stable prefix when activation changes.
- `talos-core` exposes typed `SessionOp::SetSkillContext` for session-owned Skill context mutation.
- `talos-cli::skill_runtime` activates one discovered Skill, loads bounded path-confined
  references, tracks active diagnostics, and avoids printing full content in diagnostics.
- `talos-conversation` routes `/skills activate <name>` and `/skills reference <path>` as typed UI
  outputs instead of appending Skill content to chat history.
- `talos-cli` TUI bridge applies Skill command requests to runtime state and session context while
  only emitting bounded status messages to the visible transcript.
- `talos-cli` inline mode accepts the same Skill activation/reference commands, enabling a
  deterministic real-binary request-preview regression without TUI screenshot coupling.

Targeted checks already passed on 2026-06-27:

- `cargo check -p talos-agent -p talos-conversation -p talos-cli -p talos-tui`
- `cargo test -p talos-agent -p talos-conversation -p talos-cli skill -- --nocapture`
- `cargo test -p talos-agent set_skill_context_reaches_request_preview -- --nocapture`
- `cargo test -p talos-cli conversation_loop_routes_skill_activation_to_session_op -- --nocapture`
- `cargo clippy -p talos-core -p talos-agent -p talos-conversation -p talos-cli -p talos-tui -- -D warnings`

Workspace checks passed on 2026-06-27:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`
- `git diff --check`

Real-binary proof passed on 2026-06-27:

- `cargo test -p talos-cli --test skill_runtime_e2e -- --nocapture`
  - Real `talos --inline --mock` binary flow activates a workspace Skill, runs `/mock-request`, and
    verifies the activated Skill body appears in the provider request preview.

## Uncertainty

- Resolved 2026-06-27 by R27/T2 code inspection and I058 planning:
  - `talos-cli::skill_runtime` owns runtime SkillManager state, activation budgets, path-confined
    reference loading, and diagnostics.
  - `talos-agent` owns model-visible activated Skill context through a typed prompt-builder field.
  - Activated Skill body/reference content belongs to the cacheable stable prefix after activation;
    changing activation invalidates and rebuilds `cached_stable_prefix`.
  - Conversation/TUI command handling must route activation through a typed runtime/session
    operation and must not append full Skill content to chat history or scrollback.
  - No new ADR is required unless implementation changes a public protocol, adds plugin command
    behavior, or changes the prompt-cache boundary beyond ARCH-006.

## Iteration Selection

Selected into [I058 Explicit Runtime Skill Activation](../../iterations/I058-explicit-runtime-skill-activation.md)
on 2026-06-27 as the R27/T2 implementation carrier.

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
