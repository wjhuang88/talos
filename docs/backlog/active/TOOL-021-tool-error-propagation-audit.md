# TOOL-021: End-to-End Tool Error Propagation Audit

| Field | Value |
|---|---|
| Story ID | TOOL-021 |
| Type | Technical audit |
| Priority | P1 |
| Status | Review (2026-07-16, I131) — audit report + 15 fixture tests; FINDING-2 confirmed as data loss; SESSION-006 follow-up created |
| Source | [GitHub Issue #36](https://github.com/wjhuang88/talos/issues/36) |
| Depends On | `TOOL-019`, `TOOL-002`, `RUNTIME-002` |

## Goal

Establish whether a tool failure is preserved, classified, and made available to the next model request on every supported provider path. The audit must identify any path that silently loses an orphan tool result or alters an error's meaning.

## Scope

- Trace tool-result and tool-error data from execution through `talos-agent`, message history, provider request serialization, and model-facing prompt guidance.
- Build deterministic fixtures for expected non-zero exits, execution failures, paired and orphan results, and retry/resume boundaries.

## Acceptance

- An audit record links every observed tool-error path to its producer, stored message form, provider serialization, and model-visible behavior.
- Fixtures prove each path is preserved or explicitly rejected; a silently discarded result is never reported as success.
- Any behavioral change becomes a separately reviewed implementation story.

## Non-Goals

- No new tools, providers, permissions, or prompt policy in this audit.

## Required Reads

- `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-core/src/message.rs`
- `crates/talos-provider/src/openai.rs`
- `docs/backlog/active/TOOL-019-bash-exit-code-classification.md`
- `docs/decisions/021-tool-call-protocol-architecture.md`
