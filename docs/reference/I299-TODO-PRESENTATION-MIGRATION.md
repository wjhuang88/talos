# I299 Todo Presentation API Migration

Status: local implementation candidate; not released.
Contract: [ADR-088](../decisions/088-trusted-todo-result-presentation.md).

This source-breaking change is eligible only for the next pre-1.0 minor release,
not a patch. I299 does not authorize a release.

## Rust callers

| Surface | Migration |
|---|---|
| `ToolResult` struct literals | Add `todo_presentation: None`. Prefer `ToolResult::success` / `ToolResult::error`, which default to absent metadata. |
| `AgentEvent::ToolResult` construction | Add `todo_presentation: None` for ordinary or restored results. |
| `AgentEvent::ToolResult` matching | Use `{ result, .. }` unless consuming the transient metadata. |
| `ToolResultDisplay` struct literals | Add `todo_presentation: None` for ordinary or restored displays. |
| `AgentTool` implementation | New `produces_todo_presentation` method defaults to false; existing implementations need no override. Known wrappers forward the inner method. |

`ToolResultProjection` and durable `MessageToolResult` remain unchanged. The external
Runtime SDK fixture includes the updated literal and pattern forms.

## Trust and lifetime

Native Todo formatting captures positions while producing text and binds them with SHA-256.
Agent checks the registered producer capability and native provenance; a Todo-like tool name
does not grant eligibility. Errors, changed hook output, changed invocation or redacted output
lose metadata. Agent binds validated positions to an immutable copy of final visible text only;
the UI never receives an additional unredacted source copy. Conversation and TUI check that
visible text still matches before using positions. The metadata is not an execution permission.

`TodoPresentation` and `VisibleTodoPresentation` have no serialization implementation.
The optional Agent event field is skipped by serde in both directions. Recorded events and
legacy transcripts therefore remain readable and display their literal status tokens.
No storage migration is needed. Model text, durable raw results and export text are not
rewritten by glyph rendering.

## Validation still required

The implementation candidate must pass full locked preflight, independent API/privacy review
and native TUI acceptance. Focused tests are not evidence of those gates. The authoritative
delivery status and remaining work are in [I299](../iterations/I299-preserved-presentation-recovery.md).
