# Provider Runtime Market Trial Readiness Package

## Status

Planned.

## Objective

Prepare Talos for a controlled market trial over four months. The target is not a broad public
release claim; it is a usable trial build that external early users can install, connect common
providers, run real project-analysis/development sessions, recover from provider/tool failures, and
produce actionable bug reports without maintainer hand-holding.

This package is written for frontline implementation. Tasks must be small enough to hand to an
implementer with limited project context, with explicit files, validation commands, forbidden
changes, and escalation rules. Architecture changes, permission-boundary changes, and release gates
remain senior-review items and are not implicitly delegated.

## Trigger

The 2026-07-07 Alibaba-compatible provider incident showed that Talos can still enter a stuck
processing state when an OpenAI-compatible streaming provider emits partial tool-use metadata that
does not exactly match OpenAI's canonical shape. The immediate fix synthesizes missing streaming
tool-call ids and rejects `ToolUse` turns with no collected tool calls. That fix is the baseline for
a broader trial-readiness package.

## Success Criteria

- A new user can install a documented trial build and complete a first TUI session without editing
  source files.
- Standard providers do not require custom URLs during `/connect`; custom providers still support
  explicit URLs.
- Provider/tool/runtime failures end in visible terminal states with persisted diagnostic evidence.
- Repeated permission prompts are bounded enough for multi-hour development sessions.
- Large model lists, long tool outputs, and session histories remain responsive on normal laptops.
- Trial docs explain known limits, supported provider protocols, data storage, permissions, and
  bug-report evidence.

## Non-Goals

- No `v1.0` claim.
- No crates.io publish or release tag unless a separate release gate authorizes it.
- No remote plugin marketplace, remote write-capable plugins, or auto-installed extensions.
- No weakening of write-tool permission gates.
- No new runtime database for model catalog behavior.

## Frontline Delegation Rules

Every assigned task must be handed off as a small packet with these fields:

- **Goal:** one runnable, testable behavior change.
- **Allowed files:** exact files or directories the implementer may edit.
- **Forbidden changes:** permission defaults, release/tag/publish actions, broad refactors, new
  runtime dependencies, and unrelated UI redesigns unless the task explicitly allows them.
- **Required reads:** owner docs plus the smallest relevant code files.
- **Implementation notes:** concrete local patterns to follow.
- **Acceptance:** user-visible behavior or deterministic failure mode.
- **Verification:** exact commands and, for TUI/manual items, exact scenario evidence.
- **Residuals:** any deferred item must be recorded in an owner doc before the task is accepted.

Frontline implementers may proceed without additional approval on:

- Adding tests and fixtures for existing provider/runtime behavior.
- Fixing parser/adapter edge cases without changing public config schemas.
- TUI status text and bounded rendering changes that do not alter model-facing payloads.
- Documentation, smoke checklists, and redacted diagnostics.

Frontline implementers must stop and request senior review before:

- Changing permission deny/allow precedence or write-tool gating.
- Adding or upgrading dependencies.
- Changing provider config schema or credential persistence semantics.
- Introducing background watchdogs, global event buses, or new runtime databases.
- Tagging, publishing, pushing release artifacts, or inviting external trial users.
- Making host tools the primary implementation path for a capability that should be internal.

## Assignment Packets

Use the monthly task list below as the source of truth, but split work into these handoff packets.
Each packet should be a separate branch/commit unless the maintainer explicitly groups them.

| Packet | Source Tasks | Frontline Scope | Senior Review Required |
|---|---|---|---|
| FP1 Provider SSE Fixtures | PRH01 | Add deterministic fixture tests for OpenAI-compatible streaming edge cases. Prefer test-only changes first; implementation changes are allowed only when a fixture proves a bug. | Only if the fix changes request schema or provider selection logic. |
| FP2 Agent ToolUse Invariants | PRH02 | Add agent turn-loop invariant tests and bounded error handling for malformed provider event sequences. | Required if behavior changes valid multi-tool turn semantics. |
| FP3 Processing Status Visibility | PRH03 | Improve existing status/phase labels so users can tell model-waiting from tool-waiting and terminal failure. | Required before adding watchdog timers or new async loops. |
| FP4 Session Incident Evidence | PRH04 | Persist or expose redacted diagnostic evidence for malformed tool-use incidents. | Required before changing session file format defaults. |
| FP5 Connect And Provider Diagnostics | MTR10-MTR11, MTR14 | Verify standard provider connect flow, protocol diagnostics, and redacted doctor output. | Required for config schema or credential behavior changes. |
| FP6 Large Model UX | MTR12-MTR13 | Make large model inventory browsing responsive in TUI and CLI surfaces. | Required only if changing catalog data format. |
| FP7 Tool Output Ergonomics | MTR23 | Keep long output truncation and argument display readable while preserving model-facing payload semantics. | Required if model-facing tool result compression changes. |
| FP8 Trial Docs And Smoke | MTR30-MTR32 | Produce install, first-run, provider, permission, local-data, and bug-report docs plus repeatable smoke checklist. | Required before any external trial invitation. |

Packets not suitable for unsupervised frontline implementation:

- MTR20 permission policy changes: frontline may collect traces/tests, but policy changes need
  senior security review.
- MTR21-MTR22 project intelligence/internal validation architecture: frontline may implement
  narrow strategy tests after design is approved.
- MTR24 session storage default changes: frontline may update evaluation docs or compatibility
  tests, but binary-as-default needs a separate implementation gate.
- MTR33-MTR34 self-bootstrap evidence and market-trial go/no-go: senior/maintainer owned.

## Month 1: Provider And Runtime Reliability

| ID | Theme | Task | Acceptance | Verification |
|---|---|---|---|---|
| PRH00 | Baseline | Record the Alibaba missing-id tool-use fix as the baseline. | RUNTIME-002 references the incident, fix, and tests. | Existing commit + `cargo test -p talos-provider`; `cargo test -p talos-agent`. |
| PRH01 | Provider fixtures | Add OpenAI-compatible SSE fixture tests for missing id, split id/name/args chunks, empty final delta, `[DONE]` after `tool_calls`, provider-specific usage-only chunks, and malformed tool arguments. | Each fixture either emits a complete `ToolCall` + `TurnEnd(ToolUse)` or emits a terminal `Error`; no fixture can produce `ToolCallStarted -> TurnEnd(ToolUse)` without `ToolCall`. | `cargo test -p talos-provider openai::tests::parse_sse_stream`. |
| PRH02 | Agent invariants | Add invariant tests for malformed provider event sequences: `ToolUse` with zero calls, tool calls without `ToolUse`, duplicate ids, and provider rejection after tool results. | Malformed sequences become explicit `AgentError::UnexpectedEvent` or bounded recoverable errors; valid multi-tool turns remain unchanged. | `cargo test -p talos-agent tool_use`; targeted invariant tests. |
| PRH03 | Runtime/TUI status | Split processing visibility into model-waiting, tool-waiting, and terminal failed/timed-out phases using existing status plumbing. | The TUI can distinguish waiting for provider stream, waiting for local tool execution, and terminal failure. No background watchdog is added unless deterministic transitions cannot cover the case. | `cargo test -p talos-cli conversation_loop`; `cargo test -p talos-tui processing`. |
| PRH04 | Session evidence | Persist enough event/session evidence to diagnose missing `ToolCall`/`ToolResult` incidents. | A reproduced malformed provider stream leaves a clear persisted terminal error or diagnostic entry; no silent processing-only tail. | `cargo test -p talos-session tool`; targeted CLI/session test if needed. |

## Month 2: Model, Connect, And First-Run Experience

| ID | Theme | Task | Acceptance | Verification |
|---|---|---|---|---|
| MTR10 | Provider metadata | Audit models.dev protocol/package metadata for Alibaba, OpenAI-compatible, Anthropic, Gemini, OpenRouter, and custom providers. | Diagnostics can show provider protocol/adapter for configured providers; incorrect metadata becomes a follow-up owner story. | `cargo test -p talos-config provider`; docs evidence. |
| MTR11 | Connect flow | Ensure standard-provider `/connect` does not ask for URL; only custom providers ask for URL. | Standard providers need credential/env setup only; custom provider setup still captures base URL and protocol. | `cargo test -p talos-cli connect`; TUI command test if available. |
| MTR12 | Model list performance | Finish viewport/windowed or paginated loading for large model lists. | Large provider/model inventories do not freeze TUI rendering; search remains responsive. | `cargo test -p talos-tui model`; manual large-list evidence. |
| MTR13 | Available models UX | Make `--available-models` readable for large catalogs without coupling it to the main TUI. | CLI output supports bounded, searchable, or pager-like browsing and shows `provider/model` where relevant. | CLI snapshot/manual command evidence. |
| MTR14 | First-run diagnostics | Add a lightweight doctor/diagnostic path for config, provider protocol, model id, credential source, and writable data directories. | Trial users can collect a redacted diagnostic bundle without exposing secrets. | CLI tests + manual redaction check. |

## Month 3: Tooling, Permissions, And Long-Session Stability

| ID | Theme | Task | Acceptance | Verification |
|---|---|---|---|---|
| MTR20 | Permission noise | Revisit bash/exec repeated approval behavior for real development sessions. | Low-risk repeated reads do not prompt every few seconds; write permissions remain directory-scoped and deny precedence remains intact. | Permission tests; long-task trace evidence. |
| MTR21 | Built-in project intelligence | Advance extensible project-type detection and host-tool adapter guidance injection. | Rust and common non-Rust project types are detected through pluggable strategies; adapter instructions are injected only after detection. | Validation/project-intelligence tests. |
| MTR22 | Internal validation | Move validation toward an internal callable service with language-specific adapters. | Governance validation and common project checks are callable without assuming Cargo for every project. | `cargo test -p talos-* validation` or targeted service tests. |
| MTR23 | Tool output ergonomics | Ensure long tool outputs, omitted middle content, and tool argument display remain readable and bounded. | Omitted content keeps the agreed small head/tail context, arguments show fully when line space allows, and model-facing payload semantics are unchanged. | TUI/tool-display tests. |
| MTR24 | Session storage readiness | Decide the first market-trial posture for binary session logs vs JSONL compatibility. | New default and migration/compat rules are documented before implementation; export path remains human-readable. | SESSION-004 owner doc update and targeted tests if implemented. |

## Month 4: Trial Packaging, Documentation, And Gate

| ID | Theme | Task | Acceptance | Verification |
|---|---|---|---|---|
| MTR30 | Install path | Define the trial install path and rollback story. | README/site docs give one supported trial install path, prerequisites, upgrade, and rollback instructions. | Fresh-machine or clean-user smoke evidence. |
| MTR31 | Trial docs | Write market-trial docs for supported providers, permissions, local data, known limits, and bug-report evidence. | A trial user can report provider/runtime/tool issues with session id, provider/model, redacted config, and diagnostic logs. | Docs review checklist. |
| MTR32 | Smoke suite | Build a repeatable smoke checklist for first run, `/connect`, `/model`, project analysis, tool use, provider failure, session resume, and exit summary. | Every trial candidate runs the same smoke script/checklist before handoff. | Recorded smoke evidence. |
| MTR33 | Self-bootstrap evidence | Run one qualifying Talos-primary development session if reliability gates are green. | REL-002 evidence is updated honestly as qualifying or non-qualifying; no v1.0 claim unless gates are met. | REL-002 owner doc update. |
| MTR34 | Trial gate | Produce a market-trial readiness report with go/no-go, residual risks, and rollback instructions. | Maintainer can decide whether to invite external trial users based on concrete evidence. | Readiness report + `cargo check --workspace`; governance validation. |

## Activation Guidance

- Start with Month 1. Do not move into first-run or market-facing polish while provider/tool-use
  invariants can still produce silent processing states.
- Month 2 is the first user-facing slice: provider setup, model browsing, and diagnostics.
- Month 3 addresses long-session usability and extensible project intelligence. Permission changes
  require explicit security review and must preserve deny precedence.
- Month 4 prepares the trial artifact and docs. Release/tag/publish remains a separate gate.

## Required Reads

- `crates/talos-provider/src/openai.rs`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-conversation/src/engine.rs`
- `crates/talos-config/src/model.rs`
- `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md`
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`
- `docs/backlog/active/PERM-003-permission-experience-reference-study.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/SESSION-004-binary-session-log-format.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
