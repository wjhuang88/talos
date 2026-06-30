# TOOL-014: Conditional Tool Backends And Schema Disclosure

| Field | Value |
|---|---|
| Type | Story |
| Priority | P2 |
| Status | Partial (backend disclosure core complete) |
| Depends On | TOOL-012; TOOL-013 |
| Relates To | WEBFETCH-001; WEB-005; ARCH-006 |
| Owner Boundary | H2 architect-owned tool presentation and permission boundary work |

## Outcome

Talos can keep the model-facing tool surface small while still enabling specialized capabilities
when a previous tool result or strong user intent proves they are needed.

The first target use case is web/page ingestion: `fetch_url` remains the unified read-context tool,
while browser-page access, advanced HTTP request shaping, and document-specific extraction are
exposed as conditional backends or schema branches instead of always-present sibling tools.

## Problem

TOOL-012 added tool-family presentation, but it still treats visible tools mostly as whole tools.
WEBFETCH and browser-session work can easily grow into a redundant surface:

- `http_request`
- `fetch_url`
- `web_fetch`
- `document_extract`
- `save_url`
- browser-page read/list/revisit tools

That would increase prompt size, make the model over-select high-risk tools, and fragment one user
intent ("read this web/page/document context") across too many names.

## Design Direction

Prefer **few model-visible tools with conditional backends**:

- `fetch_url`: unified read-context entry for URL, API, HTML page, document-like resource, or an
  already-authorized browser page.
- `save_url`: explicit remote-resource download/write tool; stays separate because it writes files.
- `http_request`: advanced API/debug tool; conditionally disclosed when custom method, headers,
  body, or low-level HTTP inspection is clearly needed.

`fetch_url` may support conditional target/access branches such as:

```text
target:
  url
  current_browser_page
  page_record_id

access:
  auto
  http
  browser
```

These branches do not need to be fully presented on every turn. A prior result can return a
structured continuation such as `browser_page_read_required` or `advanced_http_required`, allowing
the agent runtime to disclose only the relevant schema branch or backend description on the next
turn.

## Continuation Model

Tools may return a non-executing continuation hint:

```text
kind: browser_page_read_required
reason: login_redirect | js_rendered_empty | user_requested_current_browser_page
suggested_tool: fetch_url
suggested_backend: browser_page
permission_preview: read visible text and links from a user-approved browser page
```

The runtime uses the continuation to update tool presentation. The original tool must not directly
execute the higher-risk backend behind the scenes.

Rules:

- A continuation is not a permission grant.
- A conditional backend still goes through TOOL-013 permission facets.
- If the model calls a hidden backend without disclosure, execution fails safely with a recoverable
  tool error.
- Prompt cache stability remains a goal: disclose narrow schema/backend blocks rather than rewriting
  unrelated tool-family sections.

## Acceptance Criteria

- [x] Define core types for conditional backend/schema disclosure.
- [x] Extend tool presentation policy to disclose backend/schema branches independently of whole
      tool families where supported.
- [x] Preserve provider compatibility for APIs that require full tool definitions per request.
- [x] Add tests proving undisclosed backends cannot execute.
- [x] Add tests proving backend disclosure can expose a narrow schema branch without loading
      unrelated tools.
- [ ] Define core types for conditional tool continuations.
- [ ] Add tests proving continuations can disclose a narrow backend without loading unrelated
      tools.
- [ ] Document how WEBFETCH-001 and WEB-005 use `fetch_url` as the unified model-visible read
      entry.
- [ ] Keep `save_url` outside this convergence because it is write-capable.

## Execution Notes

- 2026-06-30: Added `ToolBackend` and `ToolBackendDisclosure` to `talos-core`.
- 2026-06-30: Extended `ToolPresentationPolicy` with backend disclosure entries and backend
  execution checks.
- 2026-06-30: Added `AgentTool` hooks for conditional backend metadata, backend selection from
  concrete input, and backend-specific description/schema presentation.
- 2026-06-30: Updated `talos-agent` prompt/provider tool definition generation to use disclosed
  backend schema branches.
- 2026-06-30: Updated tool execution to reject calls that select an undisclosed backend before
  permission evaluation or execution.
- 2026-06-30: Updated CLI/TUI permission-aware wrappers to preserve backend metadata from wrapped
  tools.

## Validation Notes

- `cargo test -p talos-core tool_presentation_policy`
- `cargo test -p talos-agent backend`
- `cargo test -p talos-cli registry`
- `cargo check --workspace`
- `cargo fmt --all -- --check`
- `scripts/validate_project_governance.sh .`

## Non-Goals

- No browser connector implementation.
- No new browser automation capabilities.
- No permission approval bypass.
- No removal or rename of existing tools until a migration story is created.

## Required Reads

- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/WEB-005-browser-session-continuity-research.md`
- `docs/backlog/active/ARCH-006-prompt-cache-stability.md`
- `crates/talos-core/src/tool.rs`
- `crates/talos-agent/src/prompt/builder.rs`
