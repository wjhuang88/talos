# Iterations

## Purpose

Track iteration plans, execution progress, and retrospectives.

## Naming Convention

```
docs/iterations/
├── README.md           (this file)
├── R0-<slug>.md        (remediation gate / execution round)
├── I001-<slug>.md      (iteration plan + execution record)
├── I002-<slug>.md
└── ...
```

## Lifecycle

1. **Planned** — Iteration created with scope, selected stories, and acceptance criteria.
2. **Active** — Work in progress. Update story status as work proceeds.
3. **Review** — All stories implemented. Run verification checklist.
4. **Complete** — Verification passed, retrospective written.

## Rules

- Each iteration has a unique ID (`I001`, `I002`, ...).
- Published iteration baselines must not be silently overwritten by later execution.
- Start a new iteration only after inventorying all existing active, review, planned, and blocked iterations.
- Record execution results by appending to the plan, not replacing it.

## Current Iterations

| ID | Codename | State | Deliverable verified end-to-end? |
|----|----------|-------|----------------------------------|
| I001 | Project Scaffold | Complete | ✅ |
| I002 | Hello Agent | Complete | ✅ |
| I003 | Tool User | Complete | ✅ |
| I004 | Safe Agent | Complete | ✅ Original #I004-S5 runtime-hardening gap was closed by R0/#ARCH-S3; see `R0-remediation-gate.md` and ADR-007 |
| I005 | Smart Agent | Complete | ✅ |
| I006 | Data Agent | Complete | ✅ Session index, fork identity, and search highlight residuals were closed by R0/#ARCH-S5..S7; dead event-loop variant removal remains scoped to I010-S7 |
| I007 | Skilled Agent | Complete | ✅ |
| I008 | Learning Agent | **Complete** (2026-06-03) | Re-scoped 2026-06-01: evolution ships as a builtin `HookHandler` (per-Agent registration covers all paths uniformly). Implementation landed 2026-06-01 (509 tests, E2E print + TUI mode verified). TUI is now the default TTY mode (legacy readline REPL retained as `--repl`). R1 Review Closure recorded runtime evidence 2026-06-03 (519 tests). See `I008-learning-agent.md`. |
| R0 | Remediation Gate | **Complete** (2026-06-01) | All 7 ARCH findings closed; 480 tests pass; I009 unblocked |
| R1 | Review Closure | **Complete** (2026-06-03) | I008/I009 moved to Complete; I009 TUI consumer work deferred to #I009-S6. I010 R2 is the next mainline slice. See `R1-review-closure.md`. |
| I009 | Extensible Agent | **Complete** (2026-06-03) | Backend/runtime surface shipped (S2 hooks, S3 MCP client, S4 MCP server, S5 JSON-RPC, S1 ToolProvenance producers). TUI consumer markers + `/plugins` deferred to #I009-S6. See `I009-extensible-agent.md`. |
| I010 | Polished Agent | **Complete** (2026-06-04; status repaired 2026-06-19) | R2 and R3 evidence is preserved; later command/session/TUI residuals have separate owners. See `I010-polished-agent.md`. |
| I011 | Open Providers | **Complete** (S1 delivered; S2 superseded by I015) | OpenAI-compatible `base_url` override + `OPENAI_COMPAT_API_KEY` env var shipped (S1). S2 (provider plugin architecture foundation) superseded by I015, which delivered the typed schema + opencode import under ADR-013 (PROV-001 Complete). Closed 2026-06-30. See `I011-open-providers.md`. |
| I012 | Portable Tools | **Superseded** (2026-06-19 mapping) | Umbrella split into I016/I017; actual deliveries mapped to I025/I026 with residuals under TOOL-001/GIT-001. See `I012-portable-tools.md`. |
| I013 | Boundary Control | **Complete** (2026-06-05) | Front-loaded high-risk boundary work: ADR-011 Guardian, ADR-012 exec DSL, ADR-013 provider schema, and #ARCH-S8 R1 centralized logging. See `I013-boundary-control.md`. |
| I014 | TUI Completion | **Complete** (2026-06-06) | Finish TUI provenance/plugin visibility and copy/export workflows. Two stories: #I009-S6 (provenance markers + `/plugins`) and #I010-S9 (clipboard copy/export) landed via 2 atomic commits. 652 tests pass workspace-wide (was 615; +37 from talos-tui). See `I014-tui-completion.md`; EXT-001 backlog; ADR-009. |
| I015 | Provider Schema | **Complete** (2026-06-08) | Schema types and built-in defaults landed 2026-06-06; one-way opencode import (`talos-config::opencode`) with 9 unit tests landed 2026-06-08. `cargo test -p talos-config -p talos-provider -p talos-cli` passes. See `I015-provider-schema.md`; `PROV-001-provider-schema.md`; ADR-013. |
| I016 | Portable File And Search Tools | **Superseded** by I025 | Native tool outcome delivered through I025/TOOL-003; residual portability/index work remains TOOL-001. |
| I017 | Embedded Git Tools | **Superseded** by I026 | P0-P2 Git outcome delivered through I026/GIT-001; advanced/fallback residuals remain GIT-001. |
| I018 | Observability and Prompt Assets | Planned | Bounded file-log retention and compile-time embedded prompt assets. See `I018-observability-prompt-assets.md`. |
| I019 | Layered Memory Foundation | Planned | Four-layer memory foundation under ADR-016. See `I019-layered-memory-foundation.md`. |
| I020 | Exploration Library | Planned | Local research library, source/claim/synthesis storage, and vector/graph storage Spike under ADR-017. See `I020-exploration-library.md`. |
| I021 | Evolution MenteDB Realignment | **Complete** (2026-06-06) | Root-cause fix for the 5MB knowledge.db bloat and `400 Bad Request` loop. 5 atomic commits (#I021-S1..S5): Signal/TurnObservation restructure, `find_marker + capture_window`, Pattern MenteDB fields, hard-reset migration, lesson #19 annotation. 615 tests pass; runtime evidence recorded (model responds normally to `cargo run -p talos-cli -- -p "你好"`). Defense layer (commit `7470ac5`) preserved as belt-and-suspenders. See `I021-evolution-mentedb-realignment.md`; EVOLUTION.md lessons #19 and #20. |
| I022 | TUI Inline-by-Default | **Complete** (2026-06-08) | Codex-style inline-by-default TUI: viewport at cursor y, finalized turns push to scrollback, fixed 4-line viewport (input+status only), real-time scrollback flush, status bar tips with TTL, diff+force_clear rendering. 127 TUI tests pass; workspace clean. State model refactor deferred to I023. See `I022-tui-inline-default.md`; `docs/proposals/tui-codex-overhaul.md`; ADR-018. |
| I023 | TUI State Model | **Complete** (2026-06-12) | Event-driven architecture: `talos-conversation` + `talos-tui` separation. Codex-style single-row history insertion with styled scrollback, 3-column prefix padding, multiline user messages with Nord bg color + top/bottom padding, one-row preview with Markdown block classifier, conservative styled Markdown rendering, animated braille spinner, native cursor sync. Review remediation closed: non-lossy mpsc delivery, agent abort-on-cancel, SIGINT fallback, engine-owned mutation verified. 114 focused tests pass (61 TUI + 53 conversation). See `I023-tui-state-model.md`. |
| I024 | Conversation Context Continuity | **Complete** (2026-06-13) | P0 context gap closed: agent receives current-session history, JSONL resume is wired, TUI visible resume history hydrates scrollback, implicit resume is workspace-scoped, and layers 1-3 compaction are active. Accepted residuals: LLM compaction layers 4-5/50-turn proof → MEM-003; first-class workspace/session topology → MEM-004. See `I024-conversation-context.md`. |
| I025 | Tool Pipeline Completion | **Complete** (2026-06-17) | Tool protocol and display pipeline closed: schema validation/dedup, diff/stat, CommonMark fence handling, Mermaid rendering via `mermaid-text`, and ToolNature permission/display metadata. See `I025-tool-pipeline-completion.md`. |
| I026 | Approval UX + Git Tools + Prompt Optimization | **Complete** (2026-06-18) | All 7 stories implemented: approval ordering and inline result display, read/write Git tools, dynamic prompt templates, Anthropic cache-control emission, tree tool, and active documentation validation. Closure re-verification on 2026-06-18: `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` both pass. Residual: `--all-targets` clippy scope gap registered as ARCH-007. See `I026-approval-ux-doc-validation.md`. |
| I027 | ARCH-003 Crate Boundary Cleanup | **Complete** (2026-06-18) | Removed dead `talos-mcp -> talos-agent` dependency, renamed `message::ToolResult` → `MessageToolResult` (14 files), and introduced an RPC `Runtime` trait so `talos-rpc` no longer names concrete `Agent` (`AgentRuntime` adapter in `talos-cli`). No behavior change; workspace check/tests/clippy/fmt/governance all clean. See `I027-crate-boundary-cleanup.md`; `docs/backlog/active/ARCH-003-crate-boundary-cleanup.md`. |
| I028 | Delayed and Scheduled Task Execution | **Planned** (2026-06-18) | 4 built-in tools (`delay`, `schedule`, `cancel_scheduled_task`, `list_scheduled_tasks`) for session-scoped delayed/recurring message injection via `SessionOp::Submit`. LLM mediates tool calls through normal permission pipeline. No external scheduling crate. See `I028-delayed-scheduled-tasks.md`; `docs/backlog/active/SCHED-001-delayed-scheduled-tasks.md`. |
| I029 | Architecture Cleanup Completion | **Complete** (2026-06-18) | Closed ARCH-004/005/006/007: anti-corruption layers, clippy `--all-targets` cleanup, prompt cache stability, and I029 god-module decomposition slice. Post-ARCH-005 residuals are tracked separately as ARCH-008/009/010. See `I029-architecture-cleanup-completion.md`. |
| I030 | Session Module Decomposition | **Complete** (2026-06-19) | ARCH-008 closed: `talos-session/src/lib.rs` split into error/types/jsonl/topology/manager/tests modules with no behavior change. See `I030-session-module-decomposition.md`. |
| I031 | Skill And CLI Module Cleanup | **Complete** (2026-06-19) | ARCH-009 skill split and ARCH-010 CLI mode-runner extraction landed; `talos-cli/src/main.rs` is now 241 lines. See `I031-skill-and-cli-module-cleanup.md`. |
| I032 | Tools Module Cleanup | **Complete** (2026-06-19) | Finished ARCH-010 tools cleanup: `talos-tools/src/file_tools.rs` is now a 108-line shared/re-export module with focused child modules for read, write/edit, delete, and ls. See `I032-tools-module-cleanup.md`. |
| I033 | Runtime Skill Activation | **Complete** (2026-06-19) | Real `talos` binary regression evidence proves workspace Skill Level 0 metadata reaches the provider request. Level 1/2 execution remains a separate SKILL-002 follow-up. See `I033-runtime-skill-activation.md`. |
| I034 | MCP Session Integration | **Complete** (2026-06-19) | Configured stdio MCP tools are discovered before the first turn across normal modes, permission/provenance/status are preserved, and failures degrade with bounded cleanup. See `I034-mcp-session-integration.md`. |
| I035 | Agent Protocol Compatibility Foundation | **Complete** (2026-06-19) | Turn AGENT-001 into a dated protocol/config compatibility plan and read/import-first foundation for shared Agent config such as `~/.agents`. See `I035-agent-protocol-compatibility-foundation.md`. |
| I036 | Research Consolidation | **Complete** (2026-06-20) | End-of-plan research-only iteration that consolidates REMOTE-001, WEB-001, PLUGIN-001, OKF-001, MEM-005, MODEL-001/002, DIST-001, WEBFETCH-001, and STORE-001 into decisions, ADR candidates, and executable follow-up stories. See `I036-research-consolidation.md`. |
| I037 | Slash Command Menu | **Complete** (2026-06-20) | TUI-010: `/` opens Codex-style command menu below composer using CMD-001 registry. See `I037-slash-command-menu.md`. |
| I038 | Model Catalog Foundation | **Complete** (2026-06-20) | MODEL-001: built-in model dataset + models.dev import. Catalog-only (reasoning split to MODEL-003). See `I038-model-catalog-foundation.md`. |
| I039 | Network Tools & TUI Polish | **Complete** (2026-06-21) | WEBFETCH-001 Phase 0 + TOOL-009 + TUI-011 + TOOL-005. See `I039-network-tools-tui-polish.md`. |
| I040 | Session Foundation & Tool Refinement | **Complete** (2026-06-22) | SESSION-001-A + http_request content detection + save_url + fetch_url merge. See `I040-session-foundation-tool-refinement.md`. |
| I041 | Interactive Session Lifecycle & Operation-Scoped Permissions | **Complete** (2026-06-22) | SESSION-001-B + SESSION-001-C + PERM-002. Closed 4 weeks early. See `I041-interactive-session-lifecycle-permission-ux.md`. |
| I042 | I041 Hotfix — Persistence Continuity, /resume UX, Execute Semantics | **Complete** (2026-06-23) | P1 persistence fix (watch channels), /resume ordinal selection, Execute first-token extraction. See `I042-i041-hotfix-persistence-resume-ux.md`. |
| I043 | Bottom Panel Generalization, Session Picker, Approval UX | **Complete** (2026-06-23) | BottomPanelState + PanelKind {SlashCommand, SessionPicker, Approval}; /resume picker; R1/R2 tech debt; TUI-008 approval reuses bottom panel with nested-approval queueing; IME guard and menu-close contract refined. Pre-closeout audit run. See `I043-bottom-panel-session-picker-approval-ux.md`. |
| I044 | Session Integrity And Lifecycle Hardening | **Complete** (2026-06-23) | Six SESSION-002 consistency fixes (O(1) append, concurrent write safety, crash reconciliation, switch ordering, failure cleanup) + `/delete` via picker UX. Pre-closeout audit fixed sort tiebreaker, bridge send errors, fork snapshot race. See `I044-session-integrity-lifecycle-hardening.md`. |
| I045 | Product Readiness — Model Lifecycle, Config, Observability | **Complete (2026-06-24)** | All 4 stories complete: MODEL-004-R (catalog integration), MODEL-005-R (/model picker + credential input + first-run wizard + --init/--available-models/--use-model), CONF-001-S (config CLI), OBS-001 (log rotation). api_key data-loss bug fixed (skip_serializing reverted). Non-navigable group headers in model picker. Closed in ~4 hours, not 1 month. Post-closeout correction (I046): two workspace tests were actually failing at closeout (stale `gpt-4.1` + lost `/resume` fallback). See `I045-product-readiness-model-lifecycle-observability.md`. |
| I046 | Architecture, Structure, And Governance Repair | **Complete (2026-06-25)** | All 5 stories: S1 (test baseline restored), S2 (provider-aware model identity — `find_model_by_provider`, `(provider, model_id)` semantics), S3 (inline api_key boundary — custom Debug masking, config_get_dotted api_key case, ADR-023), S4 (model_lifecycle.rs extracted from mode_runners.rs), S5 (docs sync — config.reference.toml, AGENTS.md, README, I045 correction). `cargo test/clippy/governance` all pass. See `I046-architecture-structure-governance-repair.md`. |
| I047 | v0.1.2 Release Readiness And Runtime Polish | **Complete (2026-06-29)** | All slices delivered. v0.1.2 tag pushed, GitHub release published, `v0.2.0` workspace version deployed. See `I047-v012-release-readiness-and-runtime-polish.md`. |
| I048 | Local Data Lifecycle And Storage Hygiene | **Complete (delivered via I049–I055, 2026-06-29)** | DATA-001 scope fulfilled by I049 (storage CLI), I050–I053 (memory pipeline/quality), and I054–I055 (exploration storage). See `I048-local-data-lifecycle-storage-hygiene.md`. |
| I049 | Storage Status And Cleanup CLI | **Complete (2026-06-29)** | DATA-001 user-facing CLI slice delivered: storage status, cleanup dry-run/apply, active-session protection, fork visibility, and maintenance commands. All workspace gates + runtime smoke test verified. See `I049-storage-status-and-cleanup-cli.md`. |
| I050 | Memory Consolidation Pipeline | **Complete (2026-06-29)** | Episodic-to-semantic consolidation pipeline delivered: `EpisodeExtractor` trait + deterministic `RuleBasedExtractor`, `consolidate_episodes()` ADD-only with evidence links, CLI `talos memory consolidate`. All gates + runtime smoke verified. See `I050-memory-consolidation-pipeline.md`. |
| I051 | Bounded Memory Prompt Injection | **Complete (2026-06-29)** | `format_memory_prompt()` + `SystemPromptBuilder::with_memory_section()` delivered with hidden-output filter, budget truncation, contradiction markers, and disable switch. 8 tests. See `I051-bounded-memory-prompt-injection.md`. |
| I052 | Procedural Memory And Entity Linking | **Complete (2026-06-29)** | Entity extraction (files/URLs/code), entity-linked retrieval boost, procedural memory, and permission-boundary regression delivered. Schema v2. 8 tests. See `I052-procedural-memory-and-entity-linking.md`. |
| I053 | Memory Quality And Release Hardening | **Complete (2026-06-29)** | I019 quality gate closed: memory status, retention dry-run, corruption tolerance, I019 acceptance criteria all verified. 7 tests. See `I053-memory-quality-and-release-hardening.md`. |
| I019 | Layered Memory Foundation | **Complete (2026-06-29)** | All 6 acceptance criteria closed via I050-I053. Four-layer memory, ADD-only consolidation, bounded retrieval, entity linking, procedural memory, contradiction handling. No vector/graph dep. See `I019-layered-memory-foundation.md`. |
| I054 | Exploration Library Storage Foundation | **Complete (2026-06-29)** | New `talos-exploration` crate with SQLite/FTS5 schema: research_runs, sources, chunks, claims, edges, syntheses. Citation integrity enforced. FTS5 search. 8 tests. See `I054-exploration-library-storage-foundation.md`. |
| I055 | Exploration Ingestion And Citation Workflow | **Complete (2026-06-29)** | Ingestion pipeline + claim extraction + citation-preserving synthesis + CLI explore ingest/search delivered. 8 tests + runtime verified. See `I055-exploration-ingestion-and-citation-workflow.md`. |
| I020 | Exploration Library | **Complete (2026-06-29)** | S1-S3 delivered via I054-I055: ExplorationStore schema + FTS5 + citation integrity + ingestion + claim extraction + synthesis. S4 (vector/graph Spike) deferred per ADR-017. See `I020-exploration-library.md`. |
| I056 | Two-Month Closeout And v0.2.0 Readiness | **Complete (2026-06-29)** | All gates green. I019/I020/DATA-001 acceptance synchronized. README updated. v0.2.0 tag pushed and published after user approval. See `I056-two-month-closeout-and-v020-readiness.md`. |
| I057 | Acceptance Remediation And Release Gate | **Complete (2026-06-29)** | All 5 stories delivered: S1 storage cleanup permission gate, S2 memory prompt runtime wiring + mock-provider evidence, S3 UTF-8/resource-budget hardening, S4 hidden-output filter expansion, S5 governance sync. All workspace gates pass. v0.2.0 tag pushed after user approval. |
| I058 | Explicit Runtime Skill Activation | **Complete (2026-06-29)** | `/skills activate <name>` + bounded Level 2 reference loading implemented through typed session context. Workspace gates and real `talos --inline --mock` request-preview proof pass. See `I058-explicit-runtime-skill-activation.md`. |
| I059 | Architecture Corrosion And Memory Module Decomposition | **Complete (2026-06-27)** | ARCH-012 promoted from a fresh oversized-module audit; `talos-memory/src/lib.rs` split into focused modules while preserving public API. Workspace gates and governance validation pass. See `I059-architecture-corrosion-memory-decomposition.md`. |
| I060 | Config Module Decomposition | **Complete (2026-06-27)** | ARCH-013 promoted after the follow-up oversized-module audit; `talos-config/src/lib.rs` split into focused modules while preserving public API. Workspace gates and governance validation pass. See `I060-config-module-decomposition.md`. |
| I061 | CLI Mode Runtime Helper Extraction | **Complete (2026-06-27)** | ARCH-014 extracted reusable runtime helpers from `talos-cli/src/mode_runners.rs`; workspace gates and governance validation pass. See `I061-cli-mode-runtime-helper-extraction.md`. |
| I062 | TUI Scrollback Helper Decomposition | **Complete (2026-06-27)** | ARCH-015 extracted input and status helpers from `talos-tui/src/scrollback.rs`; workspace gates and governance validation pass. See `I062-tui-scrollback-helper-decomposition.md`. |
| I063 | TUI Scrollback Markdown Decomposition | **Complete (2026-06-27)** | ARCH-016 extracted Markdown/code/table rendering helpers from `talos-tui/src/scrollback.rs`; workspace gates and governance validation pass. See `I063-tui-scrollback-markdown-decomposition.md`. |
| I064 | CLI Print Mode Decomposition | **Complete (2026-06-27)** | ARCH-017 extracted print-mode execution from `talos-cli/src/mode_runners.rs` as the first architecture debt burn-down CLI flow slice; workspace gates and governance validation pass. See `I064-cli-print-mode-decomposition.md`. |
| I065 | TUI App Stream Render Decomposition | **Complete (2026-06-27)** | ARCH-018 extracted stream rendering state from `talos-tui/src/app.rs` into `app_stream.rs`; workspace gates and governance validation pass. See `I065-tui-app-stream-render-decomposition.md`. |
| I066 | Agent Compaction Decomposition | **Complete (2026-06-27)** | ARCH-019 split `talos-agent/src/compaction.rs` into constants/policy/types/engine/tests while preserving public imports and behavior; workspace gates and governance validation pass. See `I066-agent-compaction-decomposition.md`. |
| I067 | Agent Prompt Decomposition | **Complete (2026-06-27)** | ARCH-020 split `talos-agent/src/prompt.rs` into assets/types/sections/builder/tests while preserving prompt output, cache markers, and public imports; workspace gates pass. See `I067-agent-prompt-decomposition.md`. |
| I068 | Agent Session Turn Decomposition | **Complete (2026-06-27)** | ARCH-021 split `talos-agent/src/session.rs` turn forwarding and tests while preserving actor-loop behavior; workspace gates pass. See `I068-agent-session-turn-decomposition.md`. |
| I069 | CLI Inline Mode Decomposition | **Complete (2026-06-27)** | ARCH-024 split CLI inline mode and inline `/skills` handling into `mode_inline.rs`; `mode_runners.rs` 1778→1500 lines; workspace gates pass. See `I069-cli-inline-mode-decomposition.md`. |
| I070 | TUI Exit Summary Decomposition | **Complete (2026-06-27)** | ARCH-025 split TUI exit-summary formatting into `app_summary.rs`; `app.rs` 1118→1005 lines; workspace gates pass. See `I070-tui-exit-summary-decomposition.md`. |
| I071 | Agent Configuration Decomposition | **Complete (2026-06-27)** | ARCH-026 split Agent constructors/configuration setters into `configuration.rs`; duplicate prompt-builder mutation centralized; `lib.rs` 914→655 lines. See `I071-agent-configuration-decomposition.md`. |
| I072 | Conversation Command Registry Decomposition | **Complete (2026-06-27)** | ARCH-027 split command registry metadata/completion into `command_registry.rs`; `engine.rs` 960→739 lines; workspace gates pass. See `I072-conversation-command-registry-decomposition.md`. |
| I073 | OpenAI Request Assembly Decomposition | **Complete (2026-06-28)** | ARCH-028 split OpenAI request DTOs/body assembly/redaction into `openai_request.rs`; `openai.rs` 1001→848 lines; workspace gates pass. See `I073-openai-request-assembly-decomposition.md`. |
| I074 | Exploration Types Decomposition | **Complete (2026-06-28)** | ARCH-029 split exploration domain entities into `types.rs`; `lib.rs` 1070→958 lines; workspace gates pass. See `I074-exploration-types-decomposition.md`. |
| I075 | Month 1 — Starting Gate and Tooling Hardening | **Complete (2026-07-01)** | Original self-bootstrap plan executed through Month-3 closeout; its Month-4 future work is superseded by the 2026-07-01 replan. See `I075-month1-starting-gate-and-tooling-hardening.md` and `../tasks/2026-07-01-four-month-self-bootstrap-replan.md`. |
| I076 | Month 1 — Provider, Tooling, And Validation Loop | **Complete (2026-07-01)** | Weeks 1-4 of the 2026-07-01 replan delivered: provider usage, status display, write/edit output, model-switch markers, validation-loop design, and read-only validation plan surface. Full workspace and governance closeout passed. See `I076-month1-provider-tooling-validation.md`. |
| I077 | Month 2 — Plugin, Exec, And Web Security | **Complete (2026-07-02)** | Weeks 5-8 of the 2026-07-01 replan delivered plugin security review/tool integration, web/dashboard security review/fixes, and direct exec policy/implementation. See `I077-month2-plugin-exec-web-security.md`. |
| I078 | Month 3 — Session Orchestration, Todo, Memory, And Thinking | **Complete (2026-07-02)** | Weeks 9-12 of the 2026-07-01 replan delivered slash auto-execute, session todo foundations, thinking separation, and self-bootstrap gap evidence. See `I078-month3-session-todo-memory-thinking.md`. |
| I079 | Month 4 — Release Readiness And Handoff | **Complete (2026-07-02)** | Weeks 13-16 of the 2026-07-01 replan delivered reliability sweep, memory injection decision, publish gate packet, REL-002 readiness report, closeout, handoff, and maintainer feedback fixes. See `I079-month4-release-readiness-handoff.md`. |
| I080 | Frontline Month 1 — Config And Governance Visibility | **Complete** (2026-07-02) | Config subcommands verified, Ctrl+A/Ctrl+E composer navigation, `/agile status` slash command, dashboard `loopback_only` default-on, ADR-031 amended. F103 deferred. See `I080-frontline-config-governance-visibility.md`. |
| I081 | Frontline Month 2 — Extension And Distribution Discipline | **Superseded before activation** | Historical baseline; replaced by later product-hardening plans. See `I081-frontline-extension-distribution.md`. |
| I082 | Frontline Month 3 — Document Ingestion And Parser Footprint | **Superseded before activation** | Historical baseline; replaced by later product-hardening plans. See `I082-frontline-ingestion-footprint.md`. |
| I083 | Frontline Month 4 — Ecosystem Compatibility And Release Posture | **Superseded before activation** | Historical baseline; replaced by later product-hardening plans. See `I083-frontline-ecosystem-release-posture.md`. |
| I084 | Experience Reliability — Thinking, Timeout, Retry, And Status | **Complete (2026-07-03)** | UX100-UX106 complete: ADR-034 v3, structured reasoning persistence, Anthropic/OpenAI reasoning paths, timeout detection, retry/backoff, TUI status states. Evidence recorded: 1497 workspace tests, clippy, fmt, governance validation. See `I084-experience-reliability.md`. |
| I085 | Model Catalog Modernization — talos-models, /model, /connect | **Complete (2026-07-12)** | Stage 1/2 and DB-free packaged-catalog behavior are closed. MC107 real-terminal walkthrough verified `/connect` grouping/filtering, safe credential cancel, and `/model` rendering in a disposable HOME. See `I085-model-catalog-modernization.md`. |
| I086 | Experience Polish And Retry Visibility | **Superseded before activation (2026-07-12)** | Later I107/I114/I115 work delivered or changed the target; remaining work is replanned under I116-I119. See `I086-experience-polish-and-retry-visibility.md`. |
| I087 | Site Install Distribution Entrypoints | **Superseded before activation (2026-07-12)** | Revised distribution acceptance is replanned under I118. See `I087-site-install-distribution-entrypoints.md`. |
| I088 | Extension And Ingestion Risk Bounding | **Superseded before activation (2026-07-12)** | I090/I091 changed the starting state; remaining bounded productization is replanned under I118. See `I088-extension-ingestion-risk-bounding.md`. |
| I089 | Ecosystem, Self-Bootstrap, And Closeout | **Superseded before activation (2026-07-12)** | I106-I109 produced non-qualifying evidence; the revised sole-primary target is replanned under I119. See `I089-ecosystem-self-bootstrap-closeout.md`. |
| I090 | High-Risk Ingestion And Search Boundary | **Complete (2026-07-04)** | Month 1 of the direct senior-agent high-risk execution track. Bounded document extraction and ripgrep-backed search stabilization are complete; full workspace and governance closeout passed. No browser/PDF/Office/OCR scope. See `I090-high-risk-ingestion-search-boundary.md` and `../tasks/2026-07-04-architect-owned-four-month-high-risk-execution.md`. |
| I091 | Plugin, Hook, And Distribution Boundary | **Complete (2026-07-04)** | Month 2 of the direct senior-agent high-risk execution track. `/hooks` diagnostics, hook manifest declaration validation, and optional runtime asset distribution policy are complete; no remote install, marketplace, write-capable plugin tools, or auto-discovery. See `I091-plugin-hook-distribution-boundary.md`. |
| I092 | Context Compression And Autonomy Gates | **Complete (2026-07-04)** | Month 3 of the direct senior-agent high-risk execution track. Bash-only cache-safe compression proof and deny/ask/allow autonomy matrix closed without runtime autonomy expansion. See `I092-context-compression-autonomy-gates.md`. |
| I093 | Self-Bootstrap, Runtime, And Release Gate | **Complete (2026-07-04)** | Month 4 of the direct senior-agent high-risk execution track. REL-002 readiness was audited and non-qualification evidence recorded; `v1.0.0` remains No-go. See `I093-self-bootstrap-runtime-release-gate.md` and `../reference/I090-I093-HIGH-RISK-CLOSEOUT-2026-07-04.md`. |
| I094 | gix Upgrade And Git Boundary | **Complete (2026-07-04)** | `gix 0.85.0` landed with no feature expansion; host-`git` fallback decisions were audited and recorded; workspace/clippy/governance gates passed. See `I094-gix-upgrade-git-boundary.md` and `../tasks/2026-07-04-high-risk-execution-gix-runtime-governance-plan.md`. |
| I095 | Runtime Validation Evidence | **Complete (2026-07-04)** | `talos validate run` emits allowlisted validation evidence with command, exit status, output summaries, and permission decision. See `I095-runtime-validation-evidence.md`. |
| I096 | Governance Mutation Gates | **Complete (2026-07-04)** | `talos governance iteration-record preview/write` provides a narrow owner-doc mutation gate with confirm, post-write validation, and rollback. See `I096-governance-mutation-gates.md`. |
| I097 | Controlled Self-Bootstrap Rehearsal | **Complete (2026-07-04)** | Closed with non-qualifying REL-002 evidence: Talos executed bounded validation and owner-doc mutation commands, but Codex remained primary. See `I097-controlled-self-bootstrap-rehearsal.md`. |
| I098 | Permission Preflight And Low-Noise Execution Policy | **Complete (2026-07-06)** | Month 1 of the 2026-07-06 autonomy/permission/runtime hardening plan. `talos permissions preflight` landed as read-only planning output; no broad bash allow or permission-default relaxation. See `I098-permission-preflight-noise-reduction.md`. |
| I099 | Structured Exec And Bash Fallback Reduction | **Complete (2026-07-06)** | Month 2 of the 2026-07-06 plan. Closed bounded direct-argv `exec` parallel steps, pipe chains, parallel pipe chains, per-step permission facets, and bash fallback matrix without shell parsing. See `I099-structured-exec-and-bash-reduction.md`. |
| I100 | Project Intelligence And Validation Routing | **Complete (2026-07-06)** | Month 3 of the 2026-07-06 plan. Closed detector descriptor metadata, independent project-type fixture tests, demand-driven adapter guidance evidence, and internal governance mutation validation. See `I100-project-intelligence-validation-routing.md`. |
| I101 | Model, Git, And Self-Bootstrap Evidence Closeout | **Complete (2026-07-06)** | Month 4 of the 2026-07-06 plan. Closed model-browser walkthrough residuals, standard-provider no-URL connect, custom-provider URL requirement, viewport-windowed model-list rendering, continued `gix` fallback tracking, and honest non-qualifying REL-002 evidence without release overclaim. See `I101-model-git-self-bootstrap-evidence.md` and `../reference/I098-I101-AUTONOMY-PERMISSION-RUNTIME-CLOSEOUT-2026-07-06.md`. |
| I102 | Provider Runtime Reliability Gate | **Complete (2026-07-08)** | Month 1 of the 2026-07-07 developer operating plan. Closed SSE fixture matrix extension (D101), agent degenerate-tool-call invariant + MaxTokens boundary guard (D102), conversation-loop cancel integration (D103), month-1 closeout (D104), and architecture-review provider error chunk fix. 1791 workspace tests pass. See `I102-provider-runtime-reliability-gate.md`. |
| I103 | First-Run Model And Diagnostics | **Complete (2026-07-08)** | Month 2 of the 2026-07-07 developer operating plan. D110-D113 verified complete: standard/custom provider connect UX, bounded model browsing, redacted diagnostics. 1791 workspace tests pass. See `I103-first-run-model-diagnostics.md`. |
| I104 | Long-Session Stability And Permission Evidence | **Complete (2026-07-08)** | Month 3 of the 2026-07-07 developer operating plan. D120-D123 verified complete: permission-noise evidence with deny precedence preserved, validation routing, tool display readability. 1791 workspace tests pass. See `I104-long-session-stability-and-permission-evidence.md`. |
| I105 | Trial Readiness Closeout | **Complete (2026-07-08)** | Month 4 of the 2026-07-07 developer operating plan. D130-D133 closed: trial docs (README troubleshooting section), smoke checklist (6 direct binary commands plus integration coverage for interactive/failure paths), REL-002 non-qualifying classification, final go/no-go report (GO for controlled local trial, NO-GO for v1.0). 1791 workspace tests pass. See `I105-trial-readiness-closeout.md`. |
| I106 | Self-Bootstrap Control Plane | **Complete (2026-07-12)** | SBT100-SBT104 complete; external-runtime evidence remains non-qualifying for REL-002. See `I106-self-bootstrap-control-plane.md`. |
| I107 | Talos-Primary Feature Polish | **Complete (2026-07-12)** | #18 request-dispatch timeout fixed and validated; external-runtime evidence remains non-qualifying. See `I107-talos-primary-feature-polish.md`. |
| I108 | Architecture-Sensitive Self-Bootstrap | **Complete (2026-07-12)** | ARCH-032 audit complete; external-runtime evidence remains non-qualifying. See `I108-architecture-sensitive-self-bootstrap.md`. |
| I109 | REL-002 Self-Bootstrap Closeout | **Complete (2026-07-12)** | Evidence audit and NO-GO report complete; REL-002 remains unmet/partial as recorded. See `I109-rel002-self-bootstrap-closeout.md`. |
| I114 | TUI Runtime Visual Stability | Complete (2026-07-10) | TUI-028 #24 cadence, #25 thinking ripple, #31 compact status rendering, and #39 transient dashboard notification accepted in native Alacritty PTY evidence. See `I114-tui-runtime-visual-stability.md`. |
| I115 | Runtime Event Semantic Convergence | **Complete (2026-07-11)** | ARCH-033 corrected the semantic gap after ARCH-032: one ordered live queue, authoritative session lifecycle, actor-owned persistence, replay equivalence, and shared runtime-surface semantics. Full workspace and binary E2E validation passed. See `I115-runtime-event-semantic-convergence.md`. |
| I116 | State Truth And Operator Baseline | **Complete (2026-07-12)** | State trace matrix reconciled 3 drifts (SESSION-004/PERF-001/TOOL-020); operator smoke 13/13; `talos diagnostics status` shipped. See `I116-state-truth-operator-baseline.md`. |
| I117 | Command Sandbox Evidence | **Complete (2026-07-12)** | Independent security review accepted diagnostic-only evidence for bash and exec single/steps/pipes; evidence never changes permission decisions. See `I117-command-sandbox-evidence.md`. |
| I118 | Bounded Local Productization | **Complete (2026-07-12)** | LT030-LT034 verified; installer fixture tests (8/8 pass). See `I118-bounded-local-productization.md`. |
| I119 | Talos-Primary Release Decision | **Complete (2026-07-12)** | REL-002 re-audited: 1 MET, 3 PARTIAL, 4 UNMET. NO-GO for v1.0.0. Two bounded task packets classified non-qualifying (external runtime primary). See `I119-talos-primary-release-decision.md`. |
| I120 | Dynamic Diagnostics Truth | **Complete** (2026-07-13) | Month 1. serde JSON, dynamic iteration state, malformed→unavailable, and typed ResidualGate pass the maintainer-confirmed repository validation gate. See `I120-dynamic-diagnostics-truth.md`. |
| I121 | TUI Attention And Thinking Clarity | **Complete** (2026-07-13) | Width-aware approval, thinking-title extraction, real export regressions, and a maintainer-supplied native Alacritty walkthrough pass. See `I121-tui-attention-and-thinking-clarity.md`. |
| I122 | Local Extension And Control Diagnostics | **Planned — ready after I121** | Month 3. Shared read-only MCP/plugin/hook diagnostics across command surfaces and loopback dashboard. See `I122-local-extension-control-diagnostics.md`. |
| I123 | Installation And Trial Confidence | **Planned — blocked on I122** | Month 4. Installer fixtures, clean-HOME smoke, second-operator replay, and honest trial report. See `I123-installation-and-trial-confidence.md`. |

> Update this table whenever an iteration changes state. "Complete" requires runtime
> evidence, not only passing unit tests — see `docs/sop/ITERATION-WORKFLOW.md`.

## Non-Terminal Inventory (2026-06-25 Refresh)

This inventory is the required disposition before selecting or activating more work. It does not
rewrite published iteration baselines.

| Iteration | Current State | Disposition Before Next Activation |
|---|---|---|
| I011 | Complete (2026-06-30 closure) | S1 delivered; S2 (provider plugin arch) superseded by I015 (PROV-001 Complete). Removed from non-terminal inventory. |
| I018 | Planned | Deferred; remains a valid future observability/prompt-assets baseline. |
| I019 | Planned | Blocked from activation until I018 or an explicit dependency replan. |
| I020 | Planned | Blocked from activation until I019 or an explicit research-priority replan. |
| I028 | Planned | Deferred; scheduling is not the current priority. |
| I047 | Complete (2026-06-29) | Removed from non-terminal inventory. v0.1.2 and v0.2.0 tags pushed and published. |
| I048 | Planned | Scope fulfilled by I049–I055; superseded by actual execution flow. |
| I049 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I049 doc, delivered via I049–I055 sequence). |
| I050 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I050 doc). |
| I051 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I051 doc). |
| I052 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I052 doc). |
| I053 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I053 doc). |
| I054 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I054 doc). |
| I055 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I055 doc). |
| I056 | Complete (2026-06-29) | Removed from non-terminal inventory (Complete record in I056 doc). |
| I057 | Complete (2026-06-29) | Removed from non-terminal inventory. All 5 stories delivered; v0.2.0 tag pushed after user approval. |
| I058 | Complete (2026-06-29) | Removed from non-terminal inventory. Implementation and validation recorded. |
| I075 | Complete (2026-07-01) | Original self-bootstrap plan executed through Month 3; remaining Month 4 tasks are superseded into I076-I079. |
| I076 | Complete | Closed 2026-07-01. Removed from non-terminal inventory. |
| I077 | Complete | Closed 2026-07-02. Removed from non-terminal inventory. |
| I078 | Complete | Closed 2026-07-02. Removed from non-terminal inventory. |
| I079 | Complete | Closed 2026-07-02. Removed from non-terminal inventory. |
| I080 | Complete | Closed 2026-07-02. Removed from non-terminal inventory. |
| I081 | Superseded before activation | Historical shell from the 2026-07-02 frontline plan. The unexecuted path is replaced by the 2026-07-03 product hardening plan. |
| I082 | Superseded before activation | Historical shell from the 2026-07-02 frontline plan. The unexecuted path is replaced by the 2026-07-03 product hardening plan. |
| I083 | Superseded before activation | Historical shell from the 2026-07-02 frontline plan. The unexecuted path is replaced by the 2026-07-03 product hardening plan. |
| I084 | Complete | UX100-UX106 complete and release-facing validation evidence recorded. Removed from non-terminal inventory after this release closeout. |
| I085 | Complete (2026-07-12) | MC107 real-terminal walkthrough passed; removed from non-terminal inventory. |
| I086 | Superseded before activation | Replaced by the changed post-v0.3.4 objectives in I116-I119. |
| I102 | Complete | Month-1 developer operating plan closed 2026-07-08. D100-D104 delivered; architecture-review provider error chunk fix landed; 1791 workspace tests pass. I103 is the next to activate. |
| I103 | Complete | Month-2 developer operating plan closed 2026-07-08. D110-D113 verified (all behavior shipped in I101). I104 is the next to activate. |
| I104 | Complete | Month-3 developer operating plan closed 2026-07-08. D120-D123 verified (all behavior shipped in PERM-002/003, VALIDATION-001, TUI-015/019/025). I105 is the final iteration. |
| I105 | Complete | Month-4 developer operating plan closed 2026-07-08. D130-D133 closed. GO for controlled local trial, NO-GO for v1.0 (REL-002 not met). Four-month plan complete. |
| I106 | Complete (2026-07-12) | Delivery acceptance met; REL-002 evidence remains non-qualifying (external runtime). Removed from non-terminal inventory. |
| I107 | Complete (2026-07-12) | Corrective delivery acceptance met; REL-002 evidence remains non-qualifying. Removed from non-terminal inventory. |
| I108 | Complete (2026-07-12) | Architecture audit acceptance met; REL-002 evidence remains non-qualifying. Removed from non-terminal inventory. |
| I109 | Complete (2026-07-12) | NO-GO closeout acceptance met; REL-002 remains unmet/partial. Removed from non-terminal inventory. |
| I114 | Complete (2026-07-10) | TUI-028 #24/#25/#31/#39 accepted in native Alacritty; removed from the active selection set. I018-I020, I028, I086-I089, and other planned work remain deferred or blocked as individually recorded. |
| I087 | Superseded before activation | Revised site-install acceptance moves to I118. |
| I088 | Superseded before activation | Revised extension/ingestion acceptance moves to I118. |
| I089 | Superseded before activation | Revised Talos-primary evidence/release decision moves to I119. |
| I116 | Complete (2026-07-12) | State truth, operator smoke, and diagnostics status delivered. Removed from non-terminal inventory. I117 is the next to activate. |
| I117 | Complete (2026-07-12) | Evidence wired; sign-off recorded. Removed from non-terminal inventory. |
| I118 | Complete (2026-07-12) | LT032 fixture tests delivered (8/8 pass). Removed from non-terminal inventory. |
| I119 | Complete (2026-07-12) | REL-002 re-audit NO-GO; two bounded packets non-qualifying; four-month plan complete. Removed from non-terminal inventory. |
| I091 | Complete | Closed 2026-07-04 with audit-first plugin/hook diagnostics and optional runtime asset distribution policy. |
| I092 | Complete | Closed 2026-07-04 with bash-only cache-stability/export evidence and autonomy permission matrix. |
| I093 | Complete | Closed 2026-07-04 with readiness/reporting only, non-qualifying REL-002 evidence, and no v1.0 claim, tag, publish, or release action. |
| I094 | Complete | Closed 2026-07-04 with `gix 0.85.0`, unchanged feature scope, fallback matrix, and full validation. |
| I095 | Complete | Closed 2026-07-04 with allowlisted validation evidence and full validation. |
| I096 | Complete | Closed 2026-07-04 with narrow iteration-record preview/write gate and full validation. |
| I097 | Complete | Closed 2026-07-04 with non-qualifying REL-002 evidence and explicit primary-executor boundary. |
| I098 | Complete | Closed 2026-07-06 with read-only permission preflight and full validation. |
| I100 | Complete | Closed 2026-07-06 with detector metadata, adapter-instruction tests, and internal governance mutation validation. |
| I101 | Complete | Closed 2026-07-06 with MODEL-006 residuals complete, `gix 0.85.0` tracking updated, and non-qualifying REL-002 evidence recorded. |
| I035 | Complete | Removed from non-terminal inventory. |
| I036 | Complete | Removed from non-terminal inventory (activation record in I039). |
| I040 | Complete (2026-06-22) | Removed from non-terminal inventory (Complete record in I040 doc). |
| I041 | Active → Complete (2026-06-22) | Activated 2026-06-22; SESSION-001-B + SESSION-001-C + PERM-002 all landed; closed 4 weeks early; T9 TUI smoke boundary documented as residual. |
| I043 | Active → Complete (2026-06-23) | Closed 2026-06-23. Bottom panel generalization + session picker + R1/R2 + TUI-008 all landed. Pre-closeout audit run. |
| I044 | Active → Complete (2026-06-23) | Closed 2026-06-23. Six SESSION-002 consistency fixes + deletion all landed. Pre-closeout audit fixed sort tiebreaker, bridge send errors, fork snapshot race. |
| I045 | Complete (2026-06-24) | Removed from non-terminal inventory (Complete record in I045 doc). |
| I046 | Complete (2026-06-25) | Removed from non-terminal inventory (Complete record in I046 doc). |

I010/I012/I016/I017 were removed from this non-terminal inventory after GOV-002 appended explicit
Complete/Superseded dispositions without erasing their published objectives.

## Next Execution Rounds

These rounds are the current operating plan for entering the next iterations. They reference
existing backlog stories only; new ideas still go through `docs/proposals/` or requirement intake.

| Round | When | Work Items | Promotion Rule |
|-------|------|------------|----------------|
| R0: Remediation Gate | ✅ Done (2026-06-01) | `R0-remediation-gate.md` | All 7 ARCH stories closed; runtime evidence recorded |
| R1: Review Closure | ✅ Done (2026-06-03) | `R1-review-closure.md` | I008/I009 Complete; I009 TUI consumer work in #I009-S6 |
| R2: I010 Architecture Slice | ✅ Done (2026-06-03) | `I010-polished-agent.md` / Slice R2 | AppServerSession seam, TUI approval, inline mode. 532 tests |
| R3: I010 Product Polish | ✅ Done (2026-06-04) | `I010-polished-agent.md` / Slice R3 | All 5 stories done (S1-S5); 567 tests; move I010 to Review/Complete |
| R4: I013 Boundary Control | ✅ Done (2026-06-05) | `I013-boundary-control.md` | High-risk permission/provider boundaries have ADRs; logging R1 implemented |
| R5: I014 TUI Completion | Next product-facing slice | `I014-tui-completion.md` | TUI provenance, `/plugins`, copy, and export workflows verified |
| R6: I015 Provider Schema | After I014 or explicit provider priority | `I015-provider-schema.md` | Schema-only provider config foundation lands under ADR-013 |
| R7: I016 Portable File And Search Tools | When environment-dependency reduction becomes release-critical | `I016-portable-file-search.md` | Native POSIX subset and search tools work on a minimal `PATH` |
| R8: I017 Embedded Git Tools | After I016 or explicit Git priority | `I017-embedded-git-tools.md` | Git read-only tools target `gix` per ADR-010 |
| R9: I018 Observability and Prompt Assets | Before memory/exploration prompt expansion | `I018-observability-prompt-assets.md` | Log files are bounded; built-in prompts are standalone embedded assets |
| R10: I019 Layered Memory Foundation | Before durable research conclusions affect agent behavior | `I019-layered-memory-foundation.md` | Memory writes/retrieval are layered, bounded, and provenance-backed |
| R11: I020 Exploration Library | After I019 or explicit research priority | `I020-exploration-library.md` | Research artifacts persist locally with sources, claims, synthesis, and storage Spike results |
| R12: I021 Evolution MenteDB Realignment | ✅ Done (2026-06-06) | `I021-evolution-mentedb-realignment.md` | `talos-evolution` data structure aligned with MenteDB blueprint; 5 atomic commits landed; 615 tests pass; runtime regression confirmed |
| R13: I022 TUI Inline-by-Default | ✅ Done (2026-06-08) | `I022-tui-inline-default.md` | Codex-style inline-by-default TUI landed; fixed viewport + scrollback flush + status bar tips. State model refactor deferred to I023. |
| R14: I023 TUI State Model | ✅ Done (2026-06-12) | `I023-tui-state-model.md` | Review remediation closed: broadcast→mpsc non-lossy delivery, agent abort-on-cancel, SIGINT fallback, engine-owned mutation verified; workspace verification clean |
| R15: I024 Conversation Context | ✅ Done (2026-06-13) | `I024-conversation-context.md`; `../roadmap/TWO-WEEK-HANDOFF-PLAN.md` | Agent receives session history in every turn; JSONL persists episodes; resume context and visible history are restored; residual long-session compaction/topology work is registered |
| R16: Two-Week Handoff | ✅ Done (2026-06-18) | `../roadmap/TWO-WEEK-HANDOFF-PLAN.md`; `I024-conversation-context.md`; `../backlog/active/TUI-005-logo-splash.md` | TUI-005 in-scope splash delivered and corrected; README repositioned as user guide; I025-I029 closed follow-up tool, prompt, Git, and architecture cleanup slices. |
| R17: Session Boundary Cleanup | ✅ Done (2026-06-19) | `I030-session-module-decomposition.md`; `../backlog/active/ARCH-008-session-module-decomposition.md` | `talos-session/src/lib.rs` decomposed without behavior change; session tests/clippy pass. |
| R18: Skill And CLI Cleanup | ✅ Done (2026-06-19) | `I031-skill-and-cli-module-cleanup.md`; `../backlog/active/ARCH-009-skill-module-decomposition.md`; `../backlog/active/ARCH-010-cli-tools-module-cleanup.md` | ARCH-009 and ARCH-010 CLI slice complete; targeted tests/clippy pass. |
| R19: Tools Cleanup | ✅ Done (2026-06-19) | `I032-tools-module-cleanup.md`; `../backlog/active/ARCH-010-cli-tools-module-cleanup.md` | `file_tools.rs` split before new tool growth; tool tests/clippy and workspace tests pass. |
| R20: Runtime Skill Activation | Complete | `I033-runtime-skill-activation.md`; `../backlog/active/SKILL-001-runtime-skill-activation.md` | Real binary request-preview regression proves Level 0 discovery/injection; Level 1/2 execution uses SKILL-002 and a new iteration. |
| R21: MCP Session Integration | Complete | `I034-mcp-session-integration.md`; `../backlog/active/MCP-001-session-mcp-integration.md` | Startup-stable MCP tools are model-visible, permission/provenance/status routed, and covered by real fixture evidence. |
| R22: Agent Protocol Compatibility | ✅ Done (2026-06-19) | `I035-agent-protocol-compatibility-foundation.md`; `../backlog/active/AGENT-001-standard-agent-protocol-support.md`; `../proposals/standard-agent-protocol-support.md`; `../decisions/022-agent-config-compatibility-boundary.md` | Survey + ADR-022 + DTOs + prototype `~/.agents/models.json` import landed. |
| R23: Research Consolidation | ✅ Done (2026-06-20) | `I036-research-consolidation.md`; `../backlog/active/REMOTE-001-remote-session-protocol.md`; `../backlog/active/WEB-001-embedded-web-control-surface.md`; `../backlog/active/PLUGIN-001-wasm-runtime-plugins.md`; `../backlog/active/OKF-001-native-okf-support.md`; `../backlog/active/MEM-005-context-compaction-policy.md`; `../backlog/active/MODEL-001-model-catalog-and-reasoning.md`; `../backlog/active/MODEL-002-local-micro-model-decision-layer.md`; `../backlog/active/DIST-001-optional-runtime-asset-distribution.md`; `../backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`; `../backlog/active/STORE-001-zvec-storage-evaluation.md` | Research-heavy items are deduplicated into ADR candidates, deferred decisions, or executable follow-up stories; no implementation code changes. |
| R24: Model Catalog Foundation | ✅ Done (2026-06-20) | `I038-model-catalog-foundation.md`; `../backlog/active/MODEL-001-model-catalog-and-reasoning.md` | Built-in model dataset + models.dev import; catalog-only, reasoning split to MODEL-003. |
| R25: Network Tools & TUI Polish | ⏳ Active (2026-06-21) | `I039-network-tools-tui-polish.md` | WEBFETCH-001 Phase 0 (http_request) → TOOL-009 (web_search) ∥ TUI-011 (status bar) + TOOL-005 (bash streaming). |
| R26: DATA/I019/I020 Two-Month Sequence | ✅ Done (2026-06-29) | `../tasks/2026-06-26-data-memory-exploration-two-month-plan.md`; all child iterations I049–I055 Complete; I056 Complete (v0.2.0 tag pushed); I019 Complete (all 6 acceptance closed); I020 Complete (S4 deferred per ADR-017) | All iterations now Complete. Release evidence: v0.1.2 and v0.2.0 tags both pushed and published. |
| R27: High-Risk Governance Gate | In Progress (2026-06-27) | `../tasks/2026-06-27-personal-oversight-high-risk-roadmap.md`; `I047-v012-release-readiness-and-runtime-polish.md`; `I056-two-month-closeout-and-v020-readiness.md`; `I057-acceptance-remediation-and-release-gate.md`; `../backlog/active/SKILL-002-explicit-runtime-activation.md`; `../backlog/active/PERM-001-guardian-exec-policy.md`; `../backlog/active/MEM-007-active-context-compression.md`; `../backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`; `../backlog/active/PLUGIN-001-wasm-runtime-plugins.md`; `../backlog/active/MODEL-003-reasoning-thinking-support.md` | Release gates, Skill activation, permission-sensitive work, context compression, web/document ingestion, and protocol/extension ADRs proceed only through governance gates; this does not grant any current executor personal approval authority. |
| R28: Architect-Owned Four-Month High-Risk Execution | Complete (2026-07-04) | `../tasks/2026-07-04-architect-owned-four-month-high-risk-execution.md`; `I090-high-risk-ingestion-search-boundary.md`; `I091-plugin-hook-distribution-boundary.md`; `I092-context-compression-autonomy-gates.md`; `I093-self-bootstrap-runtime-release-gate.md`; `../reference/I090-I093-HIGH-RISK-CLOSEOUT-2026-07-04.md` | Closed with REL-002 No-go and no publish/tag/release action. |
| R29: High-Risk gix/Runtime/Governance Execution Set | Complete (2026-07-04) | `../tasks/2026-07-04-high-risk-execution-gix-runtime-governance-plan.md`; `../reference/I094-I097-HIGH-RISK-GIX-RUNTIME-GOVERNANCE-CLOSEOUT-2026-07-04.md`; `I094-gix-upgrade-git-boundary.md`; `I095-runtime-validation-evidence.md`; `I096-governance-mutation-gates.md`; `I097-controlled-self-bootstrap-rehearsal.md`; `../backlog/active/GIT-001-embedded-git-tools.md`; `../backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`; `../backlog/active/GOV-003-builtin-project-governance.md`; `../backlog/active/REL-002-v1-self-bootstrap-release-gate.md` | I094-I097 complete. `gix 0.85.0`, validation evidence, governance mutation gate, and non-qualifying REL-002 rehearsal are recorded. No publish, tag, release, permission-default change, or v1.0 claim occurred. |
