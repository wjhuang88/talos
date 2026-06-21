# I040: Session Foundation & Tool Refinement

> Document status: Planned
> Published plan date: 2026-06-21
> Planned objective: Talos gains atomic session lifecycle transitions (prepare/commit/rollback)
>   and tools receive content-aware response handling (HTML extraction, JSON formatting).
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `SessionTransition` service + `http_request` mode=auto content extraction

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SESSION-001-A | SESSION-001 | Ready | MEM-002/004 ✅, ADR-005/006 ✅ | Atomic prepare/commit/rollback session transition service |
| WEBFETCH-001 Phase 0+ | — | Content gap | http_request tool ✅ | Content-type detection, HTML text extraction, JSON formatting |

### Execution Order

```
SESSION-001-A ─── 3-4 days (backend, blocks nothing else)
         ∥
http_request content detection ─── 1-2 days (tools, independent)
```

Both are independent and can proceed in parallel.

### Scope

**SESSION-001-A — Session Runtime Transition Service**:
- Define `SessionTransition` service with prepare/commit/rollback lifecycle
- Prepare: build new Agent/session bundle while old runtime stays active
- Commit: atomically swap to new runtime, shut down old session resources
- Rollback: discard prepared state, old runtime untouched
- **Empty-session guard**: durable storage (JSONL/SQLite) NOT created until first user message
- Reuse existing `SessionManager::create_session`/`resume_session` where practical
- No UI commands — pure backend service. SESSION-001-B/C consume this later.

**http_request — Content Type Detection**:
- Add `mode: "auto"` (default) and `mode: "raw"` to `HttpRequestInput`
- After response: check `Content-Type` header
- HTML (`text/html`) → extract visible text via `scraper` crate (strip tags, decode entities, normalize whitespace)
- JSON (`application/json`) → pretty-format with `serde_json::to_string_pretty`
- Plain text → return as-is
- Other/binary → return content type info + byte count, don't dump raw bytes
- `mode: "raw"` preserves current behavior (return raw body as-is)

**TUI-006-A — Rounded Code Block Borders**:
- Replace current `[lang] ───` flat header with Unicode box-drawing frame
- Use `╭───╮` for top border, `│` for sides, `╰───╯` for bottom
- Syntax highlighting (Sub-slice B) already done — borders are the only remaining piece
- Independent: affects only `build_code_block()` in `scrollback.rs`

### Non-Goals

- SESSION-001-B/C (`/new`, `/resume`, `/fork` commands) — separate iteration after A
- WEBFETCH Phase 1+ (link ranking, markdown conversion, document extraction)
- No new session operations beyond transition service
- No TUI changes for session transitions (UI flows in B/C)
- No configurable extraction modes beyond auto/raw

### Acceptance

- Given an active session with pending messages
  When `SessionTransition::prepare(New)` completes
  Then old session remains active and writable

- Given a prepared transition
  When `SessionTransition::commit()` succeeds
  Then new session ID, Agent context, conversation state, and persistence target are atomically updated

- Given a prepared transition that fails during commit
  When rollback is invoked
  Then old session is undamaged and still writable

- Given an empty session (no user messages submitted)
  When the user exits
  Then no JSONL file or SQLite entry exists for that session

- Given `http_request` with `mode: "auto"` (default) to an HTML page
  When the tool executes
  Then response contains extracted text (no HTML tags), status, and truncated size

- Given `http_request` with `mode: "auto"` to a JSON API
  When the tool executes
  Then response contains pretty-printed JSON

- Given `http_request` with `mode: "raw"`
  When the tool executes
  Then response body is returned as-is (preserving current behavior)

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- SESSION-001-A: unit tests for prepare/commit/rollback, empty-session guard test
- http_request: test content-type detection with mock responses (HTML, JSON, plain text, binary)

### Documentation To Update

- `README.md` — mention content-aware http_request in Built-In Capabilities
- Backlog stories: mark SESSION-001-A as In Progress
- `docs/BOARD.md` — add I040 to Now
- `docs/iterations/README.md` — add I040 entry

### Risks And Rollback

- Risk: SessionTransition may expose ownership conflicts in Agent/session composition.
  Rollback: Use the existing AppServerSession pattern; if public API break is needed, stop for ADR.
- Risk: HTML text extraction may produce poor-quality output on JS-heavy pages.
  Rollback: `mode: "raw"` is always available as fallback; extraction is best-effort.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-21 | Activation | I039 complete, no active iterations. SESSION-001-A Ready, WEBFETCH gap identified, TUI-006-A Planned. |

## Verification Evidence

- `cargo check --workspace`: 
- `cargo clippy --workspace -- -D warnings`: 
- `cargo test --workspace`: 
- Runtime evidence: 

## Variance And Residuals

- 

## Retrospective

- Outcome: 
- Documentation: 
- Lessons: 
