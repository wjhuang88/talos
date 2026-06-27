# 2026-06-27 Agent Compaction Decomposition Task

**Status**: Complete
**Owner story**: `docs/backlog/active/ARCH-019-agent-compaction-decomposition.md`
**Iteration**: `docs/iterations/I066-agent-compaction-decomposition.md`
**Parent long task**: `docs/tasks/2026-06-27-architecture-debt-burn-down-plan.md`

## Goal

Split `crates/talos-agent/src/compaction.rs` into focused modules without changing context
compaction semantics, hidden-output behavior, prompt text, trigger thresholds, or public import
paths.

## Scope

- Create child modules for constants, policy, public types, compaction engine, and tests.
- Keep `talos_agent::compaction` as the stable public module entrypoint.
- Run targeted and workspace validation.

## Out of Scope

- MEM-003 LLM compaction behavior changes or new proof workloads.
- MEM-007 active context compression.
- Provider protocol changes.
- Prompt text or prompt-cache semantics changes.
- New dependencies, network validation, commit, push, tag, or release.

## Plan

| Step | Action | Status |
|---|---|---|
| 1 | Map current compaction responsibilities and public references. | Complete |
| 2 | Create ARCH-019/I066/task owner records. | Complete |
| 3 | Mechanically split constants, policy, types, engine, and tests. | Complete |
| 4 | Run targeted agent tests and workspace gates. | Complete |
| 5 | Synchronize owner docs, Board, backlog, iterations README, and long-task checkpoint. | Complete |

## Boundary Map

- Public API: `Compactor`, `CompactionPolicy`, `CompactionError`, `CompactionResult`,
  `CompactionStatus`.
- External production reference found: `crates/talos-agent/src/session.rs` imports `Compactor`.
- Internal responsibilities:
  - constants: layer thresholds and circuit breaker defaults;
  - policy: threshold math and source documentation;
  - types: errors, result alias, sanitized status reporting;
  - engine: `Compactor`, layer sequencing, helpers, summarization;
  - tests: layer behavior, policy math, circuit breaker, hidden-output guard.

## Validation Evidence

- 2026-06-27: `crates/talos-agent/src/compaction.rs` reduced from 1447 to 41 lines.
- 2026-06-27: `cargo test -p talos-agent --quiet` passed.
- 2026-06-27: `cargo fmt --all -- --check` passed.
- 2026-06-27: `cargo check --workspace` passed.
- 2026-06-27: `cargo clippy --workspace -- -D warnings` passed.
- 2026-06-27: `cargo test --workspace --quiet` passed.
- 2026-06-27: `scripts/validate_project_governance.sh .` passed with 0 warnings.
- 2026-06-27: `git diff --check` passed.

## Residual Work

- Continue with prompt decomposition in the parent long task after this slice closes.
