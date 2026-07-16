# Iteration I131: TOOL-021 Error Propagation Audit

> Document status: Complete — audit deliverable closed; FINDING-2 data loss tracked by SESSION-006 (Open)
> Published plan date: 2026-07-16
> Planned objective: Establish whether tool failures are preserved, classified, and made available to the next model request on every supported provider path.
> MVP deliverable: Reviewed fixture matrix proving each tool-error route is preserved or explicitly rejected; follow-up owner stories for any finding.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `TOOL-021` | none | Refinement | TOOL-019, TOOL-002, RUNTIME-002 (all Complete) | Audit report + 15 fixture tests. FINDING-2 confirmed as data loss; SESSION-006 created. |

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
- **Fixture tests**: 15 total (3 OpenAI, 4 Anthropic incl. orphan-error, 3 compaction, 1 agent→session integration proving FINDING-2 data loss, 4 existing scheduler fixtures).
- **FINDING-2**: Confirmed tool-result data loss in canonical session path; integration test proves it. SESSION-006 tracks the fix.
- **Findings**: FINDING-1 (orphan result provider difference — observation). FINDING-2 (confirmed data loss in canonical session path — tracked by SESSION-006).
- **FINDING-2 confirmed**: Integration test `fixture_provider_error_drops_tool_results` proves tool results are lost on provider error in the canonical session path.

## Variance And Residuals

- No code changes to production paths — audit and fixtures only.
- Follow-up owner story created: SESSION-006 (session-layer error-path persistence). Anthropic orphan filtering remains conditional.

## Retrospective

- Outcome: met. All acceptance criteria closed.
- Documentation: TOOL-021 owner doc, audit report, Board, iterations README, execution package updated.
- Lessons: The dual-path (UI vs LLM) tool result design and three-layer compaction preserve `is_error` consistently. Provider differences in orphan handling are the main source of behavioral divergence.
