# Architecture Decision Records

## Purpose

Record significant technical decisions that affect Soft or Assumption constraints. Not for
routine implementation choices that follow established patterns.

## Naming Convention

```
docs/decisions/
├── README.md           (this file)
├── 001-<slug>.md       (decision record)
├── 002-<slug>.md
└── ...
```

## Template

```markdown
# [Decision Title]

## Context
[Why a decision is needed]

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| [constraint] | Hard / Soft / Assumption | [source] | No / Yes / Maybe |

## Reasoning
[What is the simplest approach satisfying Hard constraints?
Why deviate if we chose to?
Which Assumptions need validation?]

## Decision
[What was chosen and what was rejected]

## Reversal Trigger
[Under what conditions should this be revisited?]
```

## When to Write

| Trigger | Example |
| --- | --- |
| Choosing between approaches satisfying Hard constraints | Async runtime choice |
| Proceeding based on unvalidated Assumption | "WASM is fast enough for plugins" |
| Overriding a Soft constraint | "Using dynamic dispatch despite preferring static" |
| A Hard constraint forces an unpopular choice | "No unsafe without ADR" |

## Current Decisions

1. [001: Self-Evolution as Runtime Primitive](001-runtime-self-evolution.md) — Evolution is a first-class runtime capability (Observe → Learn → Adapt), not just a skill system feature.
2. [002: Local Storage Architecture](002-local-storage-architecture.md) — Progressive storage strategy: pure files first, SQLite introduced only where query patterns (FTS, aggregation) demand it.
3. [003: TUI Progressive Evolution](003-tui-progressive-evolution.md) — Accepted. TUI grows incrementally from I005 onward rather than landing all at once in a final polish iteration.
4. [004: Production-Grade Event Loop Architecture](004-event-loop-architecture.md) — Accepted (amended by ADR-005). Single-mpsc `AppEvent` bus + explicit `AppState` state machine for the TUI-internal event loop.
5. [005: Canonical TUI Event Architecture](005-tui-event-architecture.md) — Accepted. Two-layer model: retain ADR-004's L1 mpsc bus; add an `AppServerSession` L2 seam (bounded SQ / unbounded EQ) so the TUI never spawns the agent loop directly. Phased migration deferred to I010.
6. [006: Event Architecture Boundary](006-event-architecture-boundary.md) — Accepted. Adopt the single-consumer event loop (A, ADR-004) and the `AppServerSession` session seam (B, ADR-005); **reject** a global publish/subscribe event bus (C) on Simplicity-First, security-auditability, and hidden-coupling grounds. Guardrail for implementers.
7. [007: `unsafe` in Process Hardening](007-process-hardening-unsafe.md) — Accepted. Records and justifies the four production `unsafe` sites in `talos-sandbox/hardening.rs` (`env::remove_var` + 3× `libc::setrlimit`), approves `libc` for OS syscalls, and pre-authorizes child-process `pre_exec` hardening. Satisfies Hard Constraint #2.
8. [008: Bundled SQLite for Local Storage](008-sqlite-bundled-storage.md) — Accepted. Approves `rusqlite/bundled` as a scoped exception to the no-C/C++-bindings rule for local storage only; SQLite is statically linked so Talos does not require a system SQLite installation.
9. [009: Tool Provenance Tracking](009-tool-provenance.md) — Accepted. Adds typed provenance for native and MCP-remote tools so TUI/RPC/plugin consumers can distinguish tool sources without changing the agent loop.
10. [010: Git and Search Tool Dependency Boundary](010-git-search-tool-dependency-boundary.md) — Accepted. Rejects `git2`/libgit2 for the first I012 search/Git slices; search starts Rust-native, read-only Git tools target `gix`, and host `git` is fallback/temporary bridge only.
11. [011: Guardian Approval Boundary](011-guardian-approval-boundary.md) — Accepted. Keeps Guardian AI inside the existing permission pipeline, disabled by default, and forbids first-slice write-capable auto-approval.
12. [012: Exec Policy DSL Boundary](012-exec-policy-dsl-boundary.md) — Accepted. Defines the policy DSL as typed permission input, not a shell parser; complex shell features fail back to Ask.
13. [013: Provider Config Schema Boundary](013-provider-config-schema-boundary.md) — Accepted. Limits provider openness to schema/config in #I011-S2 and defers dynamic provider loading to a future ADR.
14. [014: Log Retention and Rotation Boundary](014-log-retention-and-rotation.md) — Accepted. Requires bounded local log files and in-process rotation/cleanup for #ARCH-S8 R2.
15. [015: Embedded Prompt Asset Boundary](015-embedded-prompt-assets.md) — Accepted. Extracts built-in prompts into standalone files embedded at compile time.
16. [016: Layered Agent Memory Architecture](016-layered-memory-architecture.md) — Accepted for architecture. Defines working, episodic, semantic, and procedural memory with explicit consolidation.
17. [017: Exploration and Library Storage Architecture](017-exploration-library-storage.md) — Accepted for direction. Starts research-library storage on SQLite/FTS with vector/graph stores gated by Spike.
18. [018: `unsafe` in TUI Job Control](018-tui-job-control-unsafe.md) — Accepted (drafted for I022). Records and justifies the single `unsafe` site in `talos-tui/src/tui/job_control.rs` (`libc::raise(SIGTSTP)`) for foreground suspend on Ctrl+Z. Follow-on to [ADR-007](007-process-hardening-unsafe.md); reuses the same `libc` FFI discipline in a different module, with no new top-level dependencies.
19. [019: TUI Splash Scrollback-Only Boundary](019-tui-splash-scrollback-boundary.md) — Accepted (2026-06-13). Adopts the scrollback-only splash (Phase 1) and rejects the viewport overlay (Phase 3) on Simplicity-First, no-speculative-features, and rendering-timing-coupling grounds. Guardrail for implementers.
20. [020: Tree-sitter Code Analysis](020-tree-sitter-code-analysis.md) — Accepted (2026-06-15). Approves tree-sitter via `arborium` for code analysis and TUI syntax highlighting.
21. [021: Tool Call Protocol Architecture](021-tool-call-protocol-architecture.md) — Accepted (2026-06-17). Defines the tool call protocol pipeline: schema validation, dedup, fence handling, and ToolNature.
22. [022: Agent Config Compatibility Boundary](022-agent-config-compatibility-boundary.md) — Accepted (2026-06-19). Supports `~/.agents/` as read-only, lowest-priority config source; `~/.talos` remains Talos-owned; one-way import via `talos-config::agents` module.
23. [023: Inline API Key Storage and Display Boundary](023-inline-api-key-boundary.md) — Accepted (2026-06-25). Persists `api_key` as a normal serializable field in `~/.talos/config.toml`; masks it in all non-file output surfaces (CLI display, Debug, logs) via custom `Debug` impls and `mask_secrets`/`is_secret_key`. Rejects `skip_serializing` (caused data loss) and OS keychain (deferred).
24. [024: Embeddable Runtime API Boundary](024-embeddable-runtime-api-boundary.md) — Accepted (2026-06-28). Creates a dedicated `talos-runtime` facade crate for SDK-style embedding while keeping `talos-agent` as the turn-loop implementation crate and `talos-core` as the protocol/trait foundation.
25. [025: Ripgrep Library Search Engine](025-ripgrep-library-search-engine.md) — Accepted (2026-06-28). Uses ripgrep's library crates (`grep-searcher`, `grep-regex`, `grep-matcher`, `ignore`) as the preferred Talos `grep` engine target; rejects top-level `ripgrep` CLI crate and host `rg` as runtime primary paths.
26. [026: Multi-Resource Tool Permissions](026-multi-resource-tool-permissions.md) — Accepted (2026-06-28). Adds invocation-specific permission profiles so hybrid tools such as `save_url`, `git_push`, and `git_pull` can expose every relevant risk facet before execution.
27. [027: Plugin Runtime Boundary](027-plugin-runtime-boundary.md) — Accepted (2026-06-30). Unblocks PLUGIN-001's first runtime slice: WASM-only v1, `wasmtime` preferred pending focused dependency review, local explicit packages only, dylib rejected, Lua deferred.
28. [028: Plugin Tool Provenance Extension](028-plugin-tool-provenance-extension.md) — Accepted (2026-06-30). Extends ADR-009 with future `ToolProvenance::Plugin { name, version, carrier }` so plugin tools do not masquerade as native or MCP tools.
29. [029: Extensibility Atomic Component Model](029-extensibility-atomic-component-model.md) — Accepted (2026-06-30). Defines skill, MCP, and hook as config-introduced atomic components; plugin is a package format that bundles components plus tools.
30. [030: Extensibility Command Taxonomy](030-extensibility-command-taxonomy.md) — Accepted (2026-06-30). Moves MCP status to `/mcp`, reserves `/plugins` for real plugin packages with a transition notice, and adds `/hooks` when hook diagnostics land.
31. [031: WEB-001 Loopback Dashboard Boundary](031-web-loopback-dashboard-boundary.md) — Accepted (2026-07-01; amended 2026-07-02). Clears the WEB-001 design gate for a default-on TUI loopback-only, read-only dashboard MVP with config opt-out. The per-process bearer token is opt-in via `[dashboard] loopback_only = false`; default is loopback-bind-only for the common single-user case. No remote access, write routes, approvals, or browser automation.
32. [032: Wasmtime Dependency and Security Review](032-wasmtime-dependency-security-review.md) — Accepted (2026-07-01). Clears ADR-027's focused `wasmtime` review gate for the first local explicit read-only WASM plugin MVP after manifest parsing; host calls denied by default and resource/failure tests required.
33. [033: Associative Memory Injection Policy](033-associative-memory-injection-policy.md) — Accepted (2026-07-02). Rejects default-on associative memory injection for v1 readiness, keeps graph recall explicit, and requires a separate default-disabled experiment plus benchmark evidence before any automatic associative prompt section.
34. [034: Provider Reasoning / Thinking Boundary](034-reasoning-thinking-boundary.md) — Accepted (2026-07-03; revised same day after architecture review). Clears the MODEL-003/UX100 gate: per-model `reasoning` options in `ModelConfig` (`effort`, `budget_tokens`, `replay`), provider-specific request mapping (Anthropic `thinking` block + `temperature: 1` + `max_tokens` hardcode fix, OpenAI `reasoning_effort` + `max_completion_tokens`, OpenAI-compatible `reasoning_content` stream), `ThinkingDelta` display stream plus new `ReasoningComplete` durable-payload event, structured `ReasoningBlock`/`AssistantReasoning` persistence (signature/redacted-capable) stored via `SessionMetadata.reasoning` with origin-gated replay, transient display/export boundary, and `reasoning_tokens` as an informational `Usage` subset. Rejects the earlier `Option<String>` persistence shape (cannot carry Anthropic signatures).
35. [035: TUI Conversation History Scrollback Boundary](035-tui-history-scrollback-boundary.md) — Accepted (2026-07-03). Generalizes ADR-019 to conversation history: terminal scrollback stays the canonical renderer for finalized history; a viewport-rendered history view is rejected for the current direction. Documents the accepted costs (no re-wrap on resize, no retro theme change, no interactive history) against the benefits (native scroll/search/copy, zero retained render state, O(1) memory), with reversal only on a committed interactive-history requirement.
36. [036: zstd Compression for Session Log Archival](036-zstd-compression-dependency.md) — Accepted (2026-07-09). Approves `zstd` (gyscos/zstd-rs, C binding) as a scoped exception to the no-C-bindings rule for session log segment archival compression only, following the ADR-008 bundled SQLite pattern. zstd is statically linked, behind a `SegmentCompressor` trait, and degrades gracefully on failure. Pure Rust alternatives tracked in COMP-001.
37. [037: Compact Text Session Log Format and Archival Architecture](037-compact-text-session-log-format.md) — Accepted (2026-07-09). Replaces JSONL with a compact text format (TSV header + length-prefixed content, `*.tlog`) for new sessions; JSONL remains as legacy read-only compatibility. Introduces segment-chain archival (LSM-style) for session compaction: frozen segments are zstd-compressed (ADR-036), new active segment holds compacted records. Separates tool output compression (Mechanism A, per-request `raw_flag`) from session compaction (Mechanism B, episodic archival). Fork uses snapshot references with `ref_count`-protected immutable archived segments.
