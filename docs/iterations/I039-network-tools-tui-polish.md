# I039: Network Tools & TUI Polish

> Document status: Planned
> Published plan date: 2026-06-21
> Planned objective: Talos gains internet access (HTTP fetch + web search) and a polished terminal
>   experience (status bar redesign + bash streaming output).
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `http_request` tool + `web_search` tool + redesigned status bar/exit + bash line streaming

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WEBFETCH-001 Phase 0 | — | Ready (S9 of I036) | None | `http_request` tool: reqwest + rustls, Network-gated, status/headers/body |
| TOOL-009 | — | Ready (Planned) | WEBFETCH-001 Phase 0 | `web_search` tool: DDG default + Wikipedia fallback + Tavily/SearXNG optional |
| TUI-011 | — | Ready (Planned) | TUI-009 ✅ | Redesigned status bar + branded exit summary |
| TOOL-005 | — | Ready (Planned) | None | Bash streaming: `$ cmd` first, then line-by-line stdout/stderr |

### Execution Order

```
Track A (Network)           Track B (TUI/UX)
─────────────────           ────────────────
WEBFETCH-001 Phase 0        TUI-011 (status bar + exit)
  │   ┌─── 3-4 days         TUI-011 ─── 1-2 days
  │   │
  ▼   │                      TOOL-005 (bash streaming)
TOOL-009 (web_search)        TOOL-005 ─── 1 day
TOOL-009 ── 1-2 days
```

Tracks A and B are independent and can proceed in parallel. TOOL-009 is the only intra-track dependency.

### Scope

**WEBFETCH-001 Phase 0** — `http_request` tool:

- New crate or module: `talos-tools/src/web/` with `reqwest` + `rustls` (no native TLS)
- Tool `nature: Network` — explicit `allow` rule required
- Input: `url`, `method` (default GET), `headers` (optional), `body` (optional)
- Output: `status_code`, `response_headers`, `body` (truncated at configurable max_bytes, default 64KB)
- Safety: domain allowlist/blocklist, response size cap, redirect limit, SSRF guard (private IP rejection)
- Config: optional `[network]` section in `~/.talos/config.toml`

**TOOL-009** — `web_search` tool:

- Uses `websearch` crate (MIT, multi-provider) for DuckDuckGo + optional backends
- Tool `nature: Network` — reuses WEBFETCH-001 permission gate
- Input: `query`, `max_results` (default 10, max 20), `include_snippets` (default true)
- Multi-provider race: DuckDuckGo (always) + Tavily (if `TAVILY_API_KEY` set) + SearXNG (if URL configured)
- Fallback: Wikipedia OpenSearch as last resort
- Config in `~/.talos/config.toml` `[search]` section following `api_key_env` pattern
- Output: compact model-friendly format with title, URL, snippet per result

**TUI-011** — Status bar & exit polish:

- Status bar: left (model), center (progress spinner), right (tokens/queue) with visual hierarchy
- Status bar: collapses gracefully at narrow widths (< 80 cols)
- Exit summary: branded header `⬡ Talos session complete`, grouped sections, human-readable numbers
- Shared formatting helpers: `format_tokens()`, `format_duration()`, color constants
- No changes to data model (`StatusSnapshot`, `Usage`)

**TOOL-005** — Bash streaming output:

- Print `$ <command>` line first, then stream stdout/stderr line-by-line
- Preserve timeout behavior and exit code reporting
- No API/schema changes (`BashInput` unchanged)
- No TTY/PTY mode; no rename to `sh` (that's TOOL-006)

### Non-Goals

- Phase 1+: HTML extraction, link ranking, markdown conversion (WEBFETCH deferred to later phases)
- REMOTE-001 remote session protocol
- Google/Brave/Exa search providers (available via `websearch` crate but not tested/verified)
- Configurable status bar themes or user-customizable layout
- Bash → `sh` rename or cross-OS native CLI support (TOOL-006)
- Any new TUI popups, dialogs, or input modes

### Acceptance

- Given a user with network permission enabled
  When the agent invokes `http_request` with a valid URL
  Then the tool returns status code, headers, and body within limits

- Given a user without network permission
  When the agent invokes `http_request`
  Then the tool returns a permission-denied error

- Given a search query from the agent
  When the `web_search` tool executes with zero config
  Then DuckDuckGo results are returned; Wikipedia fallback works when DDG fails

- Given `TAVILY_API_KEY` environment variable is set
  When the `web_search` tool executes
  Then Tavily results are raced alongside DuckDuckGo

- Given the TUI is running in a terminal >= 80 cols
  When a turn completes
  Then the status bar shows model name (left), spinner (center), tokens/queue (right)

- Given the TUI exits
  When `print_exit_summary()` is called
  Then output shows branded header, grouped sections, human-readable numbers

- Given a long-running bash command like `cargo build`
  When the command produces output
  Then the `$ cargo build` header prints first, followed by streaming stdout/stderr lines

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- TUI-011: manual visual inspection of status bar and exit summary in TUI mode
- TOOL-005: manual test with `cargo build`, `sleep 2 && echo "done"`, and timeout kill
- WEBFETCH-001: manual test with a known URL, verify response format and truncation
- TOOL-009: manual test with a search query, verify DDG results and Wikipedia fallback

### Documentation To Update

- `README.md` — mention new network tools in Built-In Capabilities
- Backlog stories: mark WEBFETCH-001, TOOL-009, TUI-011, TOOL-005 as In Progress
- `docs/BOARD.md` — add I039 to Now
- `docs/iterations/README.md` — add I039 entry

### Risks And Rollback

- Risk: `websearch` crate DuckDuckGo scraping may be unreliable or break.
  Rollback: DuckDuckGo provider disabled by default; tool still works via Wikipedia fallback + optional providers.
- Risk: SSRF guard may have false positives (blocking valid URLs) or false negatives (allowing private IP access).
  Rollback: domain allowlist/blocklist provides manual overrides; guard is conservative by default.
- Risk: Bash streaming may break existing timeout or exit code behavior.
  Rollback: timeout and exit code logic extracted into testable functions before streaming refactor.
- Risk: Status bar redesign may cause rendering regressions at edge-case terminal sizes.
  Rollback: compact-mode fallback at < 80 cols preserves current single-line behavior.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-06-21 | Activation | I036 complete, I038 complete, no active iterations blocking. All 4 stories Ready. |

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
