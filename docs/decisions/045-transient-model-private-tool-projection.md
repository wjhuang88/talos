# ADR-045: Transient Model-Private Tool Projection

## Status

Accepted (2026-07-16)

## Context

TOOL-022 needs `read` to give the active model a short snapshot handle and two-hex-digit line check
codes without exposing those coordination values in TUI history, approval presentation, exports,
transcripts, RPC/dashboard projections, or TLOG. Existing `ToolResult` has one content value;
renderer-only suppression cannot prevent Session/TLOG capture. ADR-042 also says durable tool results
use their model-visible representation, which is too broad for short-lived coordination data.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| Two hexadecimal digits are the model-visible line check code. | Hard | Maintainer direction 2026-07-16 | No |
| Snapshot handles/hashes never enter user history or durable transcript. | Hard | Maintainer direction 2026-07-16 | No |
| Session actor remains the authoritative durable writer. | Hard | ADR-039 | No |
| Failed/denied writes cannot report success or bypass permission. | Hard | AGENTS.md / ADR-042 | No |
| Durable replay normally matches model-visible messages. | Soft | ADR-042 | Yes, for explicitly transient coordination data |
| Existing tools and embedders remain source-compatible. | Hard | AGENTS.md public API rule | No |

## Decision

`AgentTool` gains additive default projection methods:

- `project_input()` returns the observer/persistence-safe tool input;
- `project_result()` returns model, display, and persistence content.

Defaults preserve the complete existing input and one shared result. Snapshot-aware file tools use
the model projection only during the active provider loop. UI events and approval handlers receive
the display projection. All provider/tool observation hooks receive persistence-safe messages,
calls, and results. If a hook returns that projection unchanged, the Agent restores the private
payload only for the provider or actual tool execution; a hook modification replaces the private
payload and may force a safe re-read. Before successful turn messages leave the Agent, tool calls
and results are rewritten to the persistence projection; the session actor therefore remains the
sole durable writer and TLOG never sees the model-private payload. Runtime permission evaluation
and execution always use the original input; projection grants no authority.

This narrowly amends ADR-042: durable transcript uses the model-visible representation except when
a tool explicitly declares transient model-only coordination data. Such data must have a safe
persistence projection, must be unnecessary after Runtime rebuild, and must cause a recoverable
re-read/reacquire flow after resume.

Snapshot registries are Runtime-local, bounded, and memory-only. The two-digit code is diagnostic;
the correctness boundary is the stored full file revision plus normal path and permission checks.
No automatic relocation may use the two-digit code.

## Rejected Alternatives

- Renderer-only hiding: rejected because raw event/session paths can still persist data.
- Long per-line hashes: rejected because every read line consumes provider context.
- Persisting snapshot handles for resume: rejected because handles are process-local and would
  create stale capabilities.
- Treating two hex digits as unique identity: rejected because collisions are expected.
- A global projection/event bus: rejected by ADR-006 and ADR-039.

## Consequences

- Live model context can contain information that durable replay intentionally omits.
- Resumed sessions must re-read before an anchored edit.
- New tools can use the projection boundary, but every non-default projection requires negative
  leakage tests.
- Existing AgentTool implementers require no changes because both methods have defaults.
- Provider/tool hooks cannot observe original model-private inputs or results. Hook modifications
  operate on the sanitized projection; unchanged hooks do not interfere with the active model flow.
- In-tree diagnostics and logs cannot serialize original model-private inputs/results; external
  hook carriers remain governed by existing plugin/hook restrictions.

## Reversal Trigger

Revisit if a durable workflow proves it must replay transient coordination state across Runtime
rebuild. Any replacement must define expiry, authority, redaction, and migration without weakening
the permission pipeline or changing TLOG format implicitly.

## Related

- ADR-039 Runtime Event Semantic Single-Flow Boundary
- ADR-042 Embedded Durable Runtime Session Boundary
- TOOL-022 Model-Private Snapshot-Anchored File Edits
