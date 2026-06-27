# 2026-06-27 Agent Prompt Decomposition Task

**Status**: Complete
**Owner story**: `docs/backlog/active/ARCH-020-agent-prompt-decomposition.md`
**Iteration**: `docs/iterations/I067-agent-prompt-decomposition.md`
**Parent long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`

## Goal

Split `crates/talos-agent/src/prompt.rs` into focused modules without changing prompt output,
cache marker semantics, stable-prefix behavior, hook behavior, memory section placement, or public
import paths.

## Scope

- Create child modules for assets, public types, section metadata, builder behavior, and tests.
- Keep `talos_agent::prompt` as the stable public module entrypoint.
- Run targeted and workspace validation.

## Out of Scope

- Prompt wording changes.
- MODEL-003 reasoning/thinking fields.
- MEM-007 active context compression.
- Provider protocol changes.
- New dependencies, network validation, commit, push, tag, or release.

## Plan

| Step | Action | Status |
|---|---|---|
| 1 | Map current prompt responsibilities and public references. | Complete |
| 2 | Create ARCH-020/I067/task owner records. | Complete |
| 3 | Mechanically split assets, public types, sections, builder, and tests. | Complete |
| 4 | Run targeted agent tests and workspace gates. | Complete |
| 5 | Synchronize owner docs, Board, backlog, iterations README, and long-task checkpoint. | Complete |

## Boundary Map

- Public API: `SystemPromptBuilder`, `ToolDescription`, `ContextFile`,
  `ActivatedSkillContext`, `CacheType`, `CacheMarker`, and embedded prompt constants.
- External production references found:
  - `crates/talos-agent/src/lib.rs` imports and configures `SystemPromptBuilder`.
  - `crates/talos-cli/src/mode_runners.rs` and `mode_runtime.rs` import `ContextFile`.
  - CLI skill-runtime tests call `Agent::build_system_prompt()`, which depends on prompt
    builder behavior.
- Internal responsibilities:
  - assets: embedded prompt text constants;
  - types: public prompt DTOs and cache marker conversion;
  - sections: private cacheable/dynamic section metadata;
  - builder: configuration, template rendering, prompt section assembly, hook/cache marker output;
  - tests: output ordering, cache marker stability, memory section, asset presence.

## Validation Evidence

- 2026-06-27: `crates/talos-agent/src/prompt.rs` reduced from 1232 to 64 lines.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.

## Residual Work

- Continue with session decomposition in the parent long task after this slice closes.
