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
