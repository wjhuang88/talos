# SESSION-007: Session Title, Summary, And User Rename

| Field | Value |
| --- | --- |
| Story ID | SESSION-007 |
| Type | Product / session-discovery story |
| Priority | P1 |
| Status | Refinement — pending design and iteration selection |
| Source | Maintainer request 2026-07-27 |
| Parent Epic | None |
| Depends On | SESSION-001, SESSION-004, session persistence/index compatibility, title-generation policy decision |
| Blocks | None |

## Identity / Goal / Value

Make sessions recognizable and manageable without opening each transcript. Talos
should provide a concise session title and bounded summary for session lists and
resume flows, while users can explicitly rename a session and retain that title
until they change or clear it.

## Scope

- Define session-level metadata for a display title, a bounded summary, and the
  provenance/override state needed to distinguish an automatic value from a
  user-authored title.
- Generate a concise automatic title and summary from an explicitly approved,
  privacy-safe session projection. Define the trigger, replacement policy,
  failure fallback, output length limits, and whether generation is heuristic,
  model-backed, or separately configured.
- Provide a user-facing rename flow for the active session and a way to inspect
  the effective title/summary in session discovery or resume surfaces.
- Once a user title exists, never overwrite it with automatic generation. A
  documented clear/reset action may restore automatic presentation.
- Persist and index the chosen session metadata durably, with compatibility for
  existing JSONL/TLOG sessions and SQLite indexes that have no title or summary.
- Ensure listing, resume, fork, export, compaction, and session reconstruction
  have defined metadata behavior without changing the canonical message
  transcript.
- Bound title/summary storage and rendering so repeated turns do not create
  per-turn session metadata records or unbounded context/session growth.

## Exclusions

- No change to the canonical conversation messages, provider request history,
  tool permissions, model protocol, or transcript/export format unless a
  separate compatibility decision is accepted.
- No hidden full-transcript copy, raw tool output, credential, absolute path,
  or private reasoning in titles or summaries.
- No automatic session merging, semantic search ranking, background fleet-wide
  reprocessing, or dashboard redesign.
- No implementation in I157 or another iteration until this story is selected.

## Decision Links And Constraints

- SESSION-001 owns session lifecycle transitions; rename must be ordered with
  new/resume/fork operations and cannot race a session switch.
- SESSION-004 and its ADR-036/ADR-037 storage contracts require an explicit
  compatibility and migration assessment before durable metadata changes.
- MEM-005 owns context-pressure/compaction policy. A display summary is not a
  substitute for compaction and must not be appended into provider context.
- If automatic generation calls a model, it needs a separately approved
  bounded execution, cost, privacy, and failure policy; this story must not
  silently add model calls to normal turns.
- Public session APIs or persisted format changes require the applicable ADR
  and migration plan before implementation.

## Uncertainty And Validation Path

Before the story becomes Ready, decide the automatic-generation strategy and
its data boundary. The preferred initial path should be deterministic and
local when it meets the product need; a model-backed generator needs explicit
opt-in/availability behavior, bounded input/output, and a no-cost failure
fallback. Define whether the summary refreshes after a turn, at explicit user
request, or on exit; how forks inherit or reset metadata; title/summary length
budgets; trimming rules; and the user-visible command/UI grammar for rename and
clear. Inventory JSONL, TLOG, SQLite index, listing, resume, and export paths
before selecting the persistence shape.

## State / Status Owners

- Durable session metadata, migration, index/listing behavior: `talos-session`.
- Session transition and CLI command dispatch: `talos-cli`.
- Interactive rename and presentation: `talos-tui`.
- Automatic-generation policy and privacy boundary: future selected iteration
  with the required decision record.
- Story status: this document.

## User-Facing Documentation

- Document how automatic titles/summaries are created, bounded, and handled
  when unavailable.
- Document rename, clear/reset, and fork behavior in CLI/TUI session guidance.
- State that a user title overrides automation and that session summaries do
  not expose credentials or hidden tool/reasoning content.

## Required Reads

- `docs/backlog/active/SESSION-001-interactive-session-lifecycle.md`
- `docs/backlog/active/SESSION-004-binary-session-log-format.md`
- `docs/backlog/active/MEM-005-context-compaction-policy.md`
- `docs/decisions/036-compact-text-session-log-format.md`
- `docs/decisions/037-session-archive-segment-chain.md`
- `crates/talos-session/src/types.rs`
- `crates/talos-session/src/store.rs`
- `crates/talos-session/src/sqlite.rs`
- `crates/talos-session/src/manager.rs`
- `crates/talos-cli/src/session_transition.rs`
- `crates/talos-tui/src/app.rs`

## Acceptance

- Given a new or legacy session without session metadata, when it is listed or
  resumed, then Talos presents a deterministic safe fallback and does not fail
  or rewrite canonical transcript messages.
- Given approved automatic title/summary inputs and a successful generation
  policy, when metadata is updated, then exactly one bounded session-level
  title and summary are stored and visible in the documented discovery surface.
- Given automatic generation is unavailable, fails, or is disabled, when a
  session is used, then the session remains usable with its fallback title and
  no blocking provider call or repeated retry loop.
- Given a user renames a session, when it is listed, resumed, forked, and
  reopened, then the user title persists and automatic generation does not
  overwrite it.
- Given a user clears the explicit title, when automatic metadata is available,
  then the documented automatic/fallback title becomes effective without
  corrupting the summary or canonical transcript.
- Given a session has long messages, tool output, credentials, paths, or hidden
  reasoning, when a title/summary is produced, then configured bounds and
  redaction rules hold and prohibited content is absent.
- Given at least 100 turns, when session files, indexes, and provider requests
  are inspected, then title/summary state remains bounded and is not appended
  once per turn into conversation history or provider context.
- JSONL/TLOG/SQLite compatibility, rename lifecycle, fallback, override,
  privacy, boundedness, and no-context-growth tests pass; `cargo test
  --workspace --locked` passes.

## Residuals

- Automatic-generation mechanism, trigger, and refresh policy require an
  explicit product/architecture decision before implementation.
- This story is unselected. It requires an explicitly sequenced future
  iteration and must not displace the program's currently planned work.
