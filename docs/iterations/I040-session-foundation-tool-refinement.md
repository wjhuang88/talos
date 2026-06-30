# I040: Session Foundation & Web Fetch Pipeline

> Document status: Complete
> Published plan date: 2026-06-21
> Closed date: 2026-06-22
> Planned objective: Talos gains atomic session lifecycle transitions (prepare/commit/rollback)
>   and a complete web fetch pipeline (http_request content detection + fetch_url content
>   extraction + save_url file persistence).
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `SessionTransition` service + `fetch_url` with content extraction + `save_url` tool

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| SESSION-001-A | SESSION-001 | Ready | MEM-002/004 ✅, ADR-005/006 ✅ | Atomic prepare/commit/rollback session transition service |
| WEBFETCH-001 Phase 1 | WEBFETCH-001 | Specified | http_request tool ✅ | fetch_url tool: content extraction, link collection, mode=auto |
| WEBFETCH-001 Phase 1b | WEBFETCH-001 | Specified | fetch_url tool | save_url tool: write-capable URL-to-file download |

### Execution Order

```
SESSION-001-A ─── 3-4 days (backend, blocks nothing else)
         ∥
WEBFETCH Phase 1 (fetch_url) ─── 2-3 days (tools, builds on http_request)
         │
         └── Phase 1b (save_url) ─── 1 day (tools, depends on Phase 1)
```

Phase 1 deepens http_request with content detection. Phase 1b adds the file-save tool.

### Scope

**SESSION-001-A — Session Runtime Transition Service**:
- Define `SessionTransition` service with prepare/commit/rollback lifecycle
- Prepare: build new Agent/session bundle while old runtime stays active
- Commit: atomically swap to new runtime, shut down old session resources
- Rollback: discard prepared state, old runtime untouched
- **Empty-session guard**: durable storage (JSONL/SQLite) NOT created until first user message
- Reuse existing `SessionManager::create_session`/`resume_session` where practical
- No UI commands — pure backend service. SESSION-001-B/C consume this later.

**WEBFETCH-001 Phase 1 — `fetch_url` tool**:
- New tool `fetch_url` with `mode: "auto"` (default)
- Static HTTP fetch (reuses http_request's reqwest client + SSRF + size cap)
- Content detection + HTML text extraction (reuses http_request's `extract_html_text`)
- Link extraction: collect all `<a href>` from HTML, normalize URLs, deduplicate
- Link classification: internal vs external, absolute vs relative
- Return: extracted text content + top-N highest-value links
- `mode: "raw"` returns the unprocessed body

**WEBFETCH-001 Phase 1b — `save_url` tool**:
- Write-capable tool `save_url` with `ToolNature::Write` + Network
- Input: `url`, `destination` (file path within workspace)
- Downloads URL bytes to the specified file
- Reuses http_request fetch pipeline (SSRF, size cap, timeout)
- Separate permission surface: requires file-write approval
- Does NOT dump file content into agent context

**TUI-006-A — Rounded Code Block Borders**:
- Replace current `[lang] ───` flat header with Unicode box-drawing frame
- Use `╭───╮` for top border, `│` for sides, `╰───╯` for bottom
- Syntax highlighting (Sub-slice B) already done — borders are the only remaining piece
- Independent: affects only `build_code_block()` in `scrollback.rs`

### Non-Goals

- SESSION-001-B/C (`/new`, `/resume`, `/fork` commands) — separate iteration after A
- WEBFETCH Phase 2+ (PDF, Office documents) — blocked on PLUGIN-001 WASM
- No browser rendering, anti-bot bypass, or JS execution
- No TUI changes for session transitions (UI flows in B/C)
- No crawling or automatic link following

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
| 2026-06-21 | Implementation | SESSION-001-A landed: `SessionTransition` service with prepare/commit/rollback in `crates/talos-cli/src/session_transition.rs`; wired into TUI mode runner; empty-session guard via `defer_create_session()` and `Session::new_deferred()`/`ensure_persisted()`. |
| 2026-06-21 | Implementation | http_request content type detection: `mode = auto / raw`; HTML text extraction via `scraper`; JSON pretty-print; binary detection. 22 http_request tests pass. |
| 2026-06-21 | Implementation | `fetch_url` merged into `http_request` (added `extract_links: bool`); `crates/talos-tools/src/fetch_url.rs` deleted; 18 web_search tests pass. |
| 2026-06-21 | Implementation | `save_url` tool: write-capable URL-to-file download, 10MB limit, SSRF guard reused, `ToolNature::Write`. |
| 2026-06-22 | Scope change | TUI-006-A removed from I040 (already implemented; see commit `402d30d`); TUI-006 closed entirely per commit `aea9ddc`. |
| 2026-06-22 | Verification | Final `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` all clean. |
| 2026-06-30 | Architecture correction | Later TOOL-014/WEB-005 review found that merging `fetch_url` into `http_request` blurred the unified read-context facade with the advanced HTTP tool. The I040 merge decision remains recorded as the execution fact, but the product architecture was corrected in a follow-up: `fetch_url` is restored as the model-visible URL context tool and `http_request` is narrowed to on-demand advanced HTTP/API inspection. |
| 2026-06-30 | GitHub issue #5 | Included embeddable runtime prompt customization in this iteration follow-up: `RuntimeBuilder::custom_prompt(...)` replaces the default Talos identity and `RuntimeBuilder::append_prompt(...)` appends product-specific instructions before session execution. |

## Verification Evidence

- `cargo check --workspace`: ✅ clean (34.62s after dependency upgrade; 18 transitive crates added)
- `cargo clippy --workspace -- -D warnings`: ✅ clean (20.06s)
- `cargo test --workspace`: ✅ all tests pass (zero failures; only 1 pre-existing `#[ignore]`)
- Runtime evidence: `http_request` and `save_url` exercised via mock tests; `SessionTransition` prepare/commit/rollback covered by unit tests; empty-session guard verified by `Session::new_deferred()` + `defer_create_session()` path

## Variance And Residuals

- **TUI-006-A removed**: Rounded code-block borders were already implemented via I023 arborium syntax highlighting; commit `402d30d` removed TUI-006-A from I040 scope.
- **fetch_url merged into http_request**: Original plan had fetch_url as a separate tool; orthogonality review merged it into http_request with `extract_links` parameter. No user-facing capability lost.
- **2026-06-30 correction**: TOOL-014/WEB-005 architecture review reversed this merge for naming,
  prompt-surface, and future browser-page backend reasons. The correction is implemented as a new
  follow-up, not by rewriting the I040 execution history.
- **GitHub issue #5 included**: Runtime prompt customization was handled in the same follow-up
  because it only exposes existing `Agent` prompt controls through the embeddable runtime facade.
- **SESSION-001-B / SESSION-001-C explicitly out of scope**: Per the I040 plan, these are deferred to a separate iteration (I041 or later).
- **clippy --all-targets residual**: pre-existing ARCH-007 (`unwrap_used` lint in test code); not introduced by I040.

## Retrospective

- Outcome: All 3 stories landed (SESSION-001-A + http_request content detection + save_url). fetch_url merged for orthogonality. No capability lost. 4 atomic commits in the I040 window. Workspace verification clean.
- Documentation: README updated with new tool names and capabilities. Backlog story SESSION-001-A marked complete (implicit via iteration closure); WEBFETCH-001 noted "I039/I040 Complete". PERM-002 documentation drafted in same window but implementation deferred to a follow-up.
- Lessons: (1) Tool orthogonality check should happen at the planning stage, not after the first cut. (2) Pre-existing iteration docs are valuable — TUI-006-A's "not implemented" claim was already stale when I040 was activated, suggesting a governance doc-drift signal worth tracking. (3) Empty-session guard (`defer_create_session`) is a clean pattern worth reusing for any "session may not be created" path.
