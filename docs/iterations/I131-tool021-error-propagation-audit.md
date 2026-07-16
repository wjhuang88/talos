# Iteration I131: TOOL-021 Error Propagation Audit

> Document status: Complete
> Published plan date: 2026-07-16
> Planned objective: Establish whether tool failures are preserved, classified, and made available to the next model request on every supported provider path.
> MVP deliverable: Reviewed fixture matrix proving each tool-error route is preserved or explicitly rejected; follow-up owner stories for any finding.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TOOL-021` | none | Refinement | TOOL-019, TOOL-002, RUNTIME-002 (all Complete) | Audit report + 9 fixture tests. No silent loss found. Two findings with follow-up recommendations. |

### Scope

- Trace tool-result and tool-error data from execution through `talos-agent`, message history, provider request serialization (OpenAI + Anthropic), and compaction.
- Build deterministic fixtures for expected non-zero, execution error, paired/orphan result, and compaction preservation.
- Document every observed path with file:line references.

### Acceptance

- ✅ Every tool-error path traced from producer to provider serialization with file:line references.
- ✅ Fixtures prove each path is preserved or explicitly rejected.
- ✅ No silently discarded result reported as success.
- ✅ Two findings have explicit follow-up owner recommendations.

### Non-Goals

- No code fixes implemented (guardrail: "do not implement unapproved repair").
- No new tools, providers, permissions, or prompt policy.

## Verification Evidence

- `cargo fmt --all -- --check`: clean.
- `cargo check --workspace --locked`: clean.
- `cargo clippy --workspace --locked -- -D warnings`: clean.
- `cargo test --workspace --locked`: all pass (0 failures).
- `scripts/validate_project_governance.sh .`: 0 warnings.
- `git diff --check`: clean.
- **Audit report**: `docs/reference/TOOL-021-ERROR-PROPAGATION-AUDIT-2026-07-16.md`
- **Fixture tests**: 9 new tests (3 OpenAI, 3 Anthropic, 3 compaction).
- **Findings**: FINDING-1 (orphan result provider difference — observation), FINDING-2 (provider error may lose unpersisted tool results — caller-dependent).
- **No silent loss found** in any observed path.

## Variance And Residuals

- No code changes to production paths — audit and fixtures only.
- Follow-up stories recommended: (1) session-layer error persistence, (2) Anthropic orphan filtering (conditional).

## Retrospective

- Outcome: met. All acceptance criteria closed.
- Documentation: TOOL-021 owner doc, audit report, Board, iterations README, execution package updated.
- Lessons: The dual-path (UI vs LLM) tool result design and three-layer compaction preserve `is_error` consistently. Provider differences in orphan handling are the main source of behavioral divergence.
