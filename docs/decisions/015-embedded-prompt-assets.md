# ADR-015: Embedded Prompt Asset Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Iteration**: I018

## Context

Talos currently assembles system prompts from Rust string literals and runtime inputs. As the
agent gains provider schemas, memory, exploration, research, and tool-library behavior, prompt
text will become a first-class product surface. Keeping large built-in prompts inline in Rust
makes review, diffing, and governance harder.

The user wants built-in prompts extracted into independent configuration-like files while still
being embedded at compile time.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| Built-in prompts must ship with the binary | Hard | User decision point; self-contained-first | No |
| Prompt assets must be reviewable as standalone text | Hard | Governance/readability | No |
| Runtime user config must not silently override safety/system prompts | Hard | Safety boundary | No |
| Existing CLI prompt overrides must keep working | Soft | Current CLI behavior | Yes, with migration |
| Prompt caching benefits from stable prompt sections | Soft | Existing `talos-agent::caching` design | Yes |

## Reasoning

Runtime prompt files would be convenient but weaken reproducibility and increase support burden:
the binary's behavior could depend on local files that are hard to inspect. Compile-time embedding
with `include_str!` keeps Talos self-contained while moving prompt text into files that code
reviewers and future agents can read directly.

Prompt files should be assets, not user configuration. User customization remains explicit through
existing CLI/config override surfaces.

## Decision

Talos will extract built-in prompts into repository text assets and embed them at compile time.

Initial target shape:

```text
crates/talos-agent/prompts/
├── system.md
├── tool_policy.md
├── memory.md
└── exploration.md
```

Rules:

- Use `include_str!` or equivalent compile-time embedding.
- Prompt assets are versioned with the binary and reviewed like code.
- Runtime prompt overrides may append or replace through existing explicit CLI/config surfaces,
  but built-in safety/tool/memory sections must not be implicitly replaced by arbitrary local files.
- Prompt section names should remain stable to preserve prompt-cache behavior.
- Tests must assert that required embedded prompt assets are non-empty and reachable.

Rejected:

- **Load built-in prompt files from disk at runtime**: not self-contained and hard to support.
- **Keep all prompt text inline in Rust**: obscures prompt review and change history.
- **User-editable built-in prompt directory**: conflates product defaults with user policy.

## Reversal Trigger

Revisit if Talos introduces signed prompt packs or an explicit plugin-prompt contract with
permission and provenance controls.

