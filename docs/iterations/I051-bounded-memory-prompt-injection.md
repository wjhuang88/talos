# I051: Bounded Memory Prompt Injection

**Status**: Planned
**Created**: 2026-06-26
**Depends On**: I050 consolidation pipeline

## Objective

Use retrieved semantic memory in provider prompts without exposing hidden tool outputs, exceeding
token budgets, or making memory authority stronger than the current session.

## Published Baseline

### Selected Stories

- I019-S3: bounded memory retrieval for prompt assembly.
- MEM-005 runtime policy integration for memory prompt sections.

### MVP Deliverable

A bounded memory prompt section can be enabled in tests and CLI runtime, with provenance and token
limits visible in status/debug surfaces.

### Scope

- Add retrieval-to-prompt assembly with count and token budgets.
- Include provenance, confidence/freshness metadata, and contradiction markers.
- Ensure hidden tool result content is not injected.
- Add config defaults and disable switch.
- Record prompt section ordering relative to AGENTS/context/tools/skills.

### Non-Goals

- No procedural adaptation.
- No permission/security decisions based on memory.
- No LLM-based compaction layers 4-5.

### Acceptance

- Given matching semantic memories, the prompt contains a bounded memory section with provenance.
- Given many matches, output is truncated by deterministic token/count budgets.
- Given hidden tool result content, prompt injection does not expose it.
- Given disabled memory injection, provider request remains unchanged.
- Given contradictory memories, the section marks them as contradictory rather than choosing a
  silent overwrite.

### Validation Plan

- Prompt snapshot/unit tests for enabled/disabled/budgeted cases.
- Hidden-output regression tests using tool result fixtures.
- Runtime mock-provider request-preview test.
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/MEM-001-layered-memory-foundation.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-26 | **Activation** | I051 activated. Dependencies met: I050 in Review (consolidation pipeline + evidence links operational, commit `30bbccf`). Scope: `format_memory_prompt()` formatting function in `talos-memory` with provenance/confidence/freshness/contradiction markers + token/count budgets; `SystemPromptBuilder::with_memory_section()` injection point in `talos-agent`; hidden-output safety filter; config disable switch; mock-provider request-preview test. No procedural adaptation, no permission decisions from memory. |
| 2026-06-26 | **Implementation** | All acceptance criteria delivered: `format_memory_prompt()` with `MemoryPromptConfig` (default disabled, max_items=5, max_chars=2000), hidden-output defense-in-depth filter (`<tool_result>`, `is_error:`, etc.), contradiction markers, budget truncation. `SystemPromptBuilder::with_memory_section(Option<String>)` injects as Dynamic section after Context. 8 tests (6 in talos-memory + 2 in talos-agent). `talos-memory` dep added to `talos-agent` (no circular dep). |

## Verification Evidence

### Workspace Gates (2026-06-26)

- `cargo fmt --all -- --check` — clean
- `cargo check --workspace` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- `cargo test --workspace` — all pass, 0 failures (pre-existing mcp_client_e2e timing flake passes in isolation)
- `scripts/validate_project_governance.sh .` — 0 warnings

### Changed Files

| File | Change |
|---|---|
| `crates/talos-memory/src/lib.rs` | `MemoryPromptConfig`, `format_memory_prompt()`, hidden-output filter, 6 tests |
| `crates/talos-agent/src/prompt.rs` | `memory_section` field, `with_memory_section()` setter, section injection, 2 tests |
| `crates/talos-agent/Cargo.toml` | Added `talos-memory` dependency |

### I057 Acceptance Remediation (2026-06-26)

I057-S2 closed the runtime-wiring gap: `with_memory_section()` is now called from `run_inner()`
via a `memory_provider` callback on the `Agent` struct (keeps `talos-agent` decoupled from
`talos-memory`). `MemoryPromptConfig` has serde derives and is loaded from `[memory_prompt]`
config section (default disabled). `maybe_set_memory_provider()` in mode_runners wires all 5
mode runners. Mock-provider request-preview regression: 2 tests prove memory section appears
when enabled and is absent when disabled.

I057-S4 hardened the hidden-output filter: expanded `HIDDEN_OUTPUT_PATTERNS` with JSON-style
markers (`"type":"tool_result"`, `"role":"tool"`), Anthropic-style (`tool_use`, `function_call`),
whitespace variants, and system markers (`<system-reminder>`). Added normalization pass
(trim + collapse whitespace). 7 new tests cover all bypass vectors.

Changed files: `crates/talos-memory/src/lib.rs`, `crates/talos-agent/src/lib.rs`,
`crates/talos-config/src/lib.rs`, `crates/talos-config/Cargo.toml`,
`crates/talos-cli/src/mode_runners.rs`, `crates/talos-cli/tests/memory_prompt_injection.rs`.
