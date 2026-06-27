# Talos Board

This board is a derived operating view. It is not the source of truth for story scope,
acceptance criteria, verification evidence, or iteration state.

## Rules

- Owner docs define truth; this board only summarizes the current operating view.
- Every row must link to an owner doc.
- Every row must have a gate: exit, resume, activation, or deferral condition.
- Status changes must be made in owner docs first, then reflected here.
- Do not add story details, acceptance checklists, execution logs, or new requirements here.
- Keep each table to these four columns only: `Item`, `State`, `Owner Doc`, `Gate`.

## Now

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| R27 High-Risk Governance Gate | In Progress | [R27 task](tasks/2026-06-27-personal-oversight-high-risk-roadmap.md) | T2/I058 moved to Review with full validation and real binary proof. Continue next high-risk packet only through the task gates. This gate does not grant any current executor personal approval authority. No tag, push, destructive cleanup, network spend, new runtime dependency, or permission-boundary change without the task's explicit gate. |
| I047 v0.1.2 Release Readiness And Runtime Polish | Review | [I047](iterations/I047-v012-release-readiness-and-runtime-polish.md) | `v0.1.2` tag pushed; record release workflow evidence before moving I047 to Complete. |
| I057 Acceptance Remediation And Release Gate | Review | [I057](iterations/I057-acceptance-remediation-and-release-gate.md) | All 5 stories delivered: storage cleanup permission gate, memory prompt runtime wiring + mock-provider evidence, UTF-8/resource hardening, hidden-output filter, governance sync. Workspace gates + targeted regressions pass. v0.2.0 still needs architect approval. |
| I058 Explicit Runtime Skill Activation | Review | [I058](iterations/I058-explicit-runtime-skill-activation.md) | `/skills activate` + bounded references implemented; workspace gates and real `talos --inline --mock` request-preview proof pass. |
| Two-Month Architecture Optimization | In Progress | [task](tasks/2026-06-27-two-month-architecture-optimization-plan.md) | M0-M4 complete through ARCH-024/I069 CLI inline mode and ARCH-025/I070 TUI exit-summary extraction. Next gate is M5: map `talos-agent/src/lib.rs`. No commit, push, release, dependency, network, or destructive action is authorized under this task. |
| I069 CLI Inline Mode Decomposition | Complete | [I069](iterations/I069-cli-inline-mode-decomposition.md) | Inline mode now lives in `mode_inline.rs`; `mode_runners.rs` 1778→1500 lines; workspace, governance, and diff gates pass. |
| I070 TUI Exit Summary Decomposition | Complete | [I070](iterations/I070-tui-exit-summary-decomposition.md) | Exit-summary formatting now lives in `app_summary.rs`; `app.rs` 1118→1005 lines; workspace, governance, and diff gates pass. |

## Review

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I041 Interactive Session Lifecycle & Operation-Scoped Permissions | Complete | [I041](iterations/I041-interactive-session-lifecycle-permission-ux.md) | SESSION-001-B (/new + /resume) + SESSION-001-C (/fork) + PERM-002 (nature+resource rules) all landed 2026-06-22; closed 4 weeks early; 700+ tests pass; TUI smoke documented as residual |
| I040 Session Foundation & Tool Refinement | Complete | [I040](iterations/I040-session-foundation-tool-refinement.md) | SESSION-001-A + http_request content detection + save_url + fetch_url merge landed 2026-06-22; `cargo check/clippy/test --workspace` all clean; TUI-006-A removed (already done in I023) |
| I022 TUI Inline-by-Default | Complete | [I022 TUI Inline-by-Default](iterations/I022-tui-inline-default.md) | Core flip + viewport refactor + scrollback flush + status bar tips landed; 127 TUI tests pass; state model refactor deferred to I023 |
| I014 TUI Completion | Complete | [I014 TUI Completion](iterations/I014-tui-completion.md) | Both stories landed (2 atomic commits: 7f783fa #I009-S6, 3b526c8 #I010-S9); 652 tests pass workspace-wide (was 615; +37 from talos-tui); runtime evidence recorded |
| I021 Evolution MenteDB Realignment | Complete | [I021 Evolution MenteDB Realignment](iterations/I021-evolution-mentedb-realignment.md) | All 5 stories landed; 615 tests pass; runtime regression confirmed (model responds to `cargo run -p talos-cli -- -p "你好"`); 5 atomic commits #I021-S1..S5; defense layer (commit `7470ac5`) preserved |
| I013 Boundary Control | Complete | [I013 Boundary Control](iterations/I013-boundary-control.md) | Boundary ADRs recorded; #ARCH-S8 R1 implemented |
| I010 R3 Product Polish | Complete | [I010 Polished Agent](iterations/I010-polished-agent.md) | All 5 stories done; 567 tests, clippy clean |
| I010 R2 Architecture Convergence | Complete | [I010 Polished Agent](iterations/I010-polished-agent.md) | All acceptance criteria met; 532 tests, clippy clean |
| I015 Provider Schema | Complete | [I015 Provider Schema](iterations/I015-provider-schema.md) | Schema types and built-in defaults landed 2026-06-06; one-way opencode import with 9 tests landed 2026-06-08; `cargo test -p talos-config -p talos-provider -p talos-cli` passes |

## Blocked / Paused

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I011 S2 Provider Plugin Architecture | Paused | [PROV-001 Provider Schema](backlog/active/PROV-001-provider-schema.md) | Resume as I015 schema-only work under ADR-013 |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I035 Agent Protocol Compatibility Foundation | Complete | [I035 Agent Protocol Compatibility Foundation](iterations/I035-agent-protocol-compatibility-foundation.md) | Survey + ADR-022 + DTOs + prototype import + config precedence specified |
| I036 Research Consolidation | Planned | [I036 Research Consolidation](iterations/I036-research-consolidation.md) | End-of-plan research-only pass turns REMOTE/WEB/WEBFETCH/PLUGIN/OKF/MEM/MODEL/STORE items into decisions or follow-up stories |
| I028 SCHED-001 Delayed and Scheduled Tasks | Planned | [SCHED-001 Delayed/Scheduled Tasks](backlog/active/SCHED-001-delayed-scheduled-tasks.md) | 4 tools (`delay`, `schedule`, `cancel_scheduled_task`, `list_scheduled_tasks`); session-scoped message injection; start after I029 architecture cleanup |
| I048 Local Data Lifecycle And Storage Hygiene | Planned | [I048](iterations/I048-local-data-lifecycle-storage-hygiene.md) | Storage status, session cleanup, SQLite maintenance, and memory lifecycle gate must land before I019 automatic memory writes |
| I049 Storage Status And Cleanup CLI | Review | [I049](iterations/I049-storage-status-and-cleanup-cli.md) | `talos storage status/cleanup/maintenance` landed with all gates + runtime smoke verified; memory retention dry-run deferred to I053 |
| I050 Memory Consolidation Pipeline | Review | [I050](iterations/I050-memory-consolidation-pipeline.md) | `talos memory consolidate` landed with deterministic RuleBasedExtractor, ADD-only pipeline, evidence links, and dedup verified; activate I051 after commit |
| I051 Bounded Memory Prompt Injection | Review | [I051](iterations/I051-bounded-memory-prompt-injection.md) | `format_memory_prompt()` + `with_memory_section()` landed with hidden-output filter, budgets, and contradiction markers; activate I052 after commit |
| I052 Procedural Memory And Entity Linking | Review | [I052](iterations/I052-procedural-memory-and-entity-linking.md) | Entity extraction + linking + retrieval boost + procedural memory + permission regression landed; activate I053 after commit |
| I053 Memory Quality And Release Hardening | Review | [I053](iterations/I053-memory-quality-and-release-hardening.md) | Memory status + retention dry-run + corruption tolerance landed; I019 acceptance criteria all closed; activate I054 after commit |
| I054 Exploration Library Storage Foundation | Review | [I054](iterations/I054-exploration-library-storage-foundation.md) | New `talos-exploration` crate with schema + FTS5 + citation integrity landed; activate I055 after commit |
| I055 Exploration Ingestion And Citation Workflow | Review | [I055](iterations/I055-exploration-ingestion-and-citation-workflow.md) | Ingestion + claims + synthesis + CLI landed and runtime-verified; activate I056 closeout after commit |
| I056 Two-Month Closeout And v0.2.0 Readiness | Review | [I056](iterations/I056-two-month-closeout-and-v020-readiness.md) | All gates green, I019/I020/DATA-001 synced, README updated, release decision package ready — v0.2.0 tag requires architect approval |
| ARCH-011 Architecture Watchlist | Tracking | [ARCH-011](backlog/active/ARCH-011-architecture-watchlist.md) | Promote only if future work creates concrete file growth, change-frequency, or boundary evidence |
| WEB-002 GitHub Pages Product Site And Custom Domain | Planned | [WEB-002](backlog/active/WEB-002-github-pages-product-site.md) | Activate when a release-site slice is selected; first implementation should keep `site/` separate from internal `docs/` and support `CNAME`. |
| ARCH-022 CLI Mode Runner Residual Decomposition | Planned | [ARCH-022](backlog/active/ARCH-022-cli-mode-runner-residual-decomposition.md) | Activate only as a future behavior-preserving CLI flow slice after current release/governance priorities. |
| ARCH-023 TUI App Residual Decomposition | Planned | [ARCH-023](backlog/active/ARCH-023-tui-app-residual-decomposition.md) | Activate only as a future behavior-preserving TUI app slice with visual-risk notes when frame/cursor behavior is touched. |
| TUI-008 Approval Dialog UX | Planned | [TUI-008 Approval Dialog UX](backlog/active/TUI-008-approval-dialog-ux.md) | Move approval from bottom-right to prominent position; easy to miss currently |
| TUI-009 Input Clear And Session Exit Summary | Complete | [TUI-009 Input Clear And Session Exit Summary](backlog/active/TUI-009-input-and-session-exit-polish.md) | Ctrl+C clears idle input; Esc no longer clears composer; exit prints session summary with model, duration, turns, tokens, cost |
| CMD-001 Interactive Command Runtime Contract | Complete | [CMD-001](backlog/active/CMD-001-interactive-command-runtime-contract.md) | Shared registry, tool-backed infrastructure, availability predicates, copy/export restored, README synced |
| SESSION-001 Interactive Session Lifecycle | Refinement | [SESSION-001](backlog/active/SESSION-001-interactive-session-lifecycle.md) | Select ready child SESSION-001-A first; expose `/new`, `/resume`, and `/fork` only through later verified children |
| I016 Portable File And Search Tools | Planned | [TOOL-001 Portable File/Search](backlog/active/TOOL-001-portable-file-search.md) | Residual scope beyond TOOL-003 (persistent indexes, extra native deps) |
| I018 Observability and Prompt Assets | Planned (selected into I047) | [OBS-001 Observability and Prompt Assets](backlog/active/OBS-001-observability-prompt-assets.md) | I047 must close bounded logs + embedded prompt assets before MEM-001-A starts; follow ADR-014/ADR-015 |

## Done This Cycle

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I046 Architecture, Structure, And Governance Repair | Complete | [I046](iterations/I046-architecture-structure-governance-repair.md) | All 5 stories: S1 workspace tests restored, S2 provider-aware model identity, S3 inline api_key boundary (ADR-023), S4 model_lifecycle.rs extracted, S5 docs synced. `cargo test/clippy/governance` all pass. |
| I059 Architecture Corrosion And Memory Module Decomposition | Complete | [I059](iterations/I059-architecture-corrosion-memory-decomposition.md) | ARCH-012 complete: `talos-memory/src/lib.rs` 2141→39 lines; types/store/entities/prompt/tests split; workspace gates and governance validation pass. |
| I060 Config Module Decomposition | Complete | [I060](iterations/I060-config-module-decomposition.md) | ARCH-013 complete: `talos-config/src/lib.rs` 2083→28 lines; error/types/credentials/config/builtin/env/tests split; workspace gates and governance validation pass. |
| I061 CLI Mode Runtime Helper Extraction | Complete | [I061](iterations/I061-cli-mode-runtime-helper-extraction.md) | ARCH-014 complete: `talos-cli/src/mode_runners.rs` 2062→1912 lines via `mode_runtime.rs`; workspace gates and governance validation pass. |
| I062 TUI Scrollback Helper Decomposition | Complete | [I062](iterations/I062-tui-scrollback-helper-decomposition.md) | ARCH-015 complete: `talos-tui/src/scrollback.rs` 1614→1386 lines via `scrollback_input.rs` and `scrollback_status.rs`; workspace gates and governance validation pass. |
| I063 TUI Scrollback Markdown Decomposition | Complete | [I063](iterations/I063-tui-scrollback-markdown-decomposition.md) | ARCH-016 complete: `talos-tui/src/scrollback.rs` 1386→756 lines via `scrollback_markdown.rs`; workspace gates and governance validation pass. |
| I064 CLI Print Mode Decomposition | Complete | [I064](iterations/I064-cli-print-mode-decomposition.md) | ARCH-017 complete: `talos-cli/src/mode_runners.rs` 1912→1778 lines via `mode_print.rs`; workspace gates and governance validation pass. |
| I065 TUI App Stream Render Decomposition | Complete | [I065](iterations/I065-tui-app-stream-render-decomposition.md) | ARCH-018 complete: `talos-tui/src/app.rs` 1503→1118 lines via `app_stream.rs`; workspace gates and governance validation pass. |
| I066 Agent Compaction Decomposition | Complete | [I066](iterations/I066-agent-compaction-decomposition.md) | ARCH-019 complete: `talos-agent/src/compaction.rs` 1447→41 lines via constants/policy/types/engine/tests split; workspace gates and governance validation pass. |
| I067 Agent Prompt Decomposition | Complete | [I067](iterations/I067-agent-prompt-decomposition.md) | ARCH-020 complete: `talos-agent/src/prompt.rs` 1232→64 lines via assets/types/sections/builder/tests split; workspace gates pass. |
| I068 Agent Session Turn Decomposition | Complete | [I068](iterations/I068-agent-session-turn-decomposition.md) | ARCH-021 complete: `talos-agent/src/session.rs` 1150→193 lines via turn/tests split; workspace gates pass. |
| Architecture Debt Burn-down Long Task | Complete | [task](tasks/2026-06-27-architecture-debt-burn-down-plan.md) | T0-T11 complete. Agent target roots are decomposed; remaining CLI/TUI root residuals are owned by ARCH-022 and ARCH-023. No release or commit action was performed. |
| I044 Session Integrity And Lifecycle Hardening | Complete | [I044](iterations/I044-session-integrity-lifecycle-hardening.md) | Six SESSION-002 consistency fixes (O(1) append, concurrent write safety, crash reconciliation, switch ordering, failure cleanup) + `/delete` via picker UX; pre-closeout audit fixed sort tiebreaker, bridge send errors, fork snapshot race; 48 test groups pass; clippy clean; governance 0 warnings |
| I045 Product Readiness — Model Lifecycle, Config, Observability | Complete | [I045](iterations/I045-product-readiness-model-lifecycle-observability.md) | MODEL-004-R (catalog integration), MODEL-005-R (/model picker + credential input + first-run wizard + --init/--available-models/--use-model), CONF-001-S (config CLI), OBS-001 (log rotation). api_key data-loss bug fixed. Group headers in model picker. Closed 2026-06-24 in ~4 hours. |
| I043 Bottom Panel Generalization, Session Picker, Approval UX | Complete | [I043](iterations/I043-bottom-panel-session-picker-approval-ux.md) | BottomPanelState with PanelKind {SlashCommand, SessionPicker, Approval}; /resume picker; R1 interrupt_tx follows session switches; R2 model_context_limit from config; TUI-008 approval reuses bottom panel with nested-approval output queueing; IME guard and menu-close contract refined |
| I042 I041 Hotfix — Persistence, /resume UX, Execute Semantics | Complete | [I042](iterations/I042-i041-hotfix-persistence-resume-ux.md) | P1-1 persistence continuity (watch channels); /resume ordinal selection; Execute first-token extraction; all tests pass |
| I041 Interactive Session Lifecycle & Operation-Scoped Permissions | Complete | [I041](iterations/I041-interactive-session-lifecycle-permission-ux.md) | SESSION-001-B + SESSION-001-C + PERM-002 over 4 weeks (closed 4 weeks early); 8 atomic commits + 4 task checkpoints; T9 TUI smoke boundary documented; 700+ tests pass |
| I040 Session Foundation & Tool Refinement | Complete | [I040](iterations/I040-session-foundation-tool-refinement.md) | SESSION-001-A SessionTransition + http_request content detection + save_url + fetch_url merge; TUI-006-A removed (superseded by I023); `cargo check/clippy/test --workspace` clean |
| I039 Network Tools & TUI Polish | Complete | [I039 Network Tools & TUI Polish](iterations/I039-network-tools-tui-polish.md) | All 4 stories landed: http_request + web_search + status bar/exit redesign + bash streaming. 5-agent review passed. |
| I037 Slash Command Menu | Complete | [I037 Slash Command Menu](iterations/I037-slash-command-menu.md) | Post-completion review repaired composer-backed filtering, Approval priority, deterministic height fallback, tests, and documentation sync |
| TUI-010 Slash Command Menu Below Input | Complete | [TUI-010 Slash Command Menu Below Input](backlog/active/TUI-010-slash-command-menu.md) | All acceptance paths covered by 120 TUI tests; workspace quality gates pass |
| I034 MCP Session Integration | Complete | [I034 MCP Session Integration](iterations/I034-mcp-session-integration.md) | Startup-stable stdio MCP tools, permissions, provenance/status, failure handling, and real fixture evidence all passed |
| MCP-001 MCP Session Integration | Complete | [MCP-001](backlog/active/MCP-001-session-mcp-integration.md) | Published session integration scope closed in I034; HTTP/dynamic follow-ups require new requirements |
| I033 Runtime Skill Activation | Complete | [I033 Runtime Skill Activation](iterations/I033-runtime-skill-activation.md) | Real-binary request-preview regression proves Level 0 discovery/injection; Level 1/2 activation remains SKILL-002 |
| SKILL-001 Runtime Skill Activation | Complete | [SKILL-001](backlog/active/SKILL-001-runtime-skill-activation.md) | Published Level 0 runtime scope verified; explicit body/reference activation remains SKILL-002 |
| I029 Architecture Cleanup Completion | Complete | [I029 Architecture Cleanup](iterations/I029-architecture-cleanup-completion.md) | ARCH-004/007/006/005 all landed. 10345→2082 lines across 4 god modules (-80%). |
| GOV-002 Legacy Iteration Status Repair | Complete | [GOV-002](backlog/active/GOV-002-legacy-iteration-status-repair.md) | I010 closed and I012/I016/I017 supersession mapped; Manifest governance state restored |
| I030 Session Module Decomposition | Complete | [I030 Session Module Decomposition](iterations/I030-session-module-decomposition.md) | ARCH-008 landed: `talos-session/src/lib.rs` 1737→45 lines; session tests and clippy pass. |
| ARCH-008 Session Module Decomposition | Complete | [ARCH-008](backlog/active/ARCH-008-session-module-decomposition.md) | error/types/jsonl/topology/manager/tests split landed; public `talos_session::*` imports preserved. |
| ARCH-009 Skill Module Decomposition | Complete | [ARCH-009](backlog/active/ARCH-009-skill-module-decomposition.md) | error/types/token/parser/loader/manager/tests split landed; public `talos_skill::*` imports preserved. |
| I031 Skill And CLI Module Cleanup | Complete | [I031 Skill And CLI Module Cleanup](iterations/I031-skill-and-cli-module-cleanup.md) | ARCH-009 + ARCH-010 CLI slice landed; `talos-cli/src/main.rs` 1250→241 lines; CLI tests/clippy pass. |
| I032 Tools Module Cleanup | Complete | [I032 Tools Module Cleanup](iterations/I032-tools-module-cleanup.md) | ARCH-010 tools slice landed; `file_tools.rs` 1308→108 lines; tools/workspace tests and clippy pass. |
| ARCH-010 CLI and Tools Module Cleanup | Complete | [ARCH-010](backlog/active/ARCH-010-cli-tools-module-cleanup.md) | CLI and tools slices both complete; no touched source file remains above the I032 size gate. |
| I027 ARCH-003 Crate Boundary Cleanup | Complete | [I027 Crate Boundary Cleanup](iterations/I027-crate-boundary-cleanup.md) | All 3 stories landed. |
| I026 Approval UX + Git + Prompt Optimization | Complete | [I026 Approval UX + Doc Validation](iterations/I026-approval-ux-doc-validation.md) | All 7 stories. |
| TOOL-002 P0 Tool Calling Remediation | P0 Complete | [TOOL-002 Tool Calling Remediation](backlog/active/TOOL-002-tool-calling-remediation.md) | Schema in prompt, permission pipeline, agent message cleanup, streaming filter, format cleanup; P1-P2 residual in I025 |
| I025 Tool Pipeline Completion | Complete | [I025 Tool Pipeline Completion](iterations/I025-tool-pipeline-completion.md) | All 5 stories: schema validation+dedup, diff+stat, fence fix, Mermaid rendering, ToolNature. 142+ tests pass |
| CODE-002 Symbol Tools | Complete | [CODE-002 Symbol Tools](backlog/active/CODE-002-symbol-tools.md) | All 4 tools (find_symbol, find_references, list_symbols, list_imports) implemented in symbol.rs; registered in all 4 builders |
| CODE-001 Tree-sitter Research | Complete | [CODE-001 Tree-sitter Code Analysis Research](backlog/active/CODE-001-tree-sitter-code-analysis-research.md) | All 7 research questions answered; ADR-020 created; arborium integrated; TUI highlighting delivered; next step CODE-002 Symbol Tools planned |
| TUI-007 Theme System | Complete | [TUI-007 Theme System](backlog/active/TUI-007-theme-system.md) | Theme struct with 42 semantic roles; Nord + Solarized Dark built-in themes; all rendering files migrated to semantic::; workspace tests pass |
| MEM-004 Workspace Session Topology | Complete | [MEM-004 Workspace-Scoped Session Topology](backlog/active/MEM-004-workspace-session-topology.md) | SHA256-hashed workspace identity; workspace_root column in SQLite; --continue/--resume filtered by workspace; same-basename collision prevented; old sessions backward-compatible |
| TUI-005 Logo & Splash Screen | Complete (in-scope); overlay deferred | [TUI-005 Logo & Splash Screen](backlog/active/TUI-005-logo-splash.md) | Branded inline `TALOS` block wordmark (Nord Frost gradient), `⬡ The watchman never sleeps` tagline, narrow-width fallback; scrollback-only, no alt-screen. 2026-06-14 correction fixed first-row column drift and preserved the intended blank line before the logo; README synced, 82 TUI tests and workspace verification pass. Phase 3-4 viewport overlay (subsystem badges + auto-dismiss) deferred per Scope Boundary. |
| I024 Conversation Context | Complete | [I024 Conversation Context](iterations/I024-conversation-context.md) | Closed 2026-06-13: session history reaches provider calls, JSONL resume and visible TUI history hydration work, implicit resume is workspace-scoped, and residual long-session compaction/topology work is registered in MEM-003/MEM-004. |
| TUI-012 Session Resume History Rendering | Complete | [TUI-012](backlog/active/TUI-012-session-resume-rendering.md) | Fixed 2026-06-21: persistence layer restored agent new_messages; replay matches live |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| PERM-002 Operation-Scoped Permissions | Refinement | [PERM-002](backlog/active/PERM-002-operation-scoped-permissions.md) | P1: no-repeat-approval — same resource auto-allowed after first approval |
| REMOTE-001 Remote Session Protocol | Research | [REMOTE-001 Remote Session Protocol](backlog/active/REMOTE-001-remote-session-protocol.md) | Research transport, auth, and minimum viable slice before any implementation |
| PLUGIN-001 WASM Runtime Plugin Protocol | Research (→ P2) | [PLUGIN-001 WASM Runtime Plugin Protocol](backlog/active/PLUGIN-001-wasm-runtime-plugins.md) | Elevated P2: unblocks TOOL-008 Phase 3 + WEBFETCH Phase 2+ WASM loading |
| I017 Embedded Git Tools | P0-P2 Complete | [GIT-001 Embedded Git Tools](backlog/active/GIT-001-embedded-git-tools.md) | Read/write Git tools delivered in I026; future scope starts from P3 advanced tools |
| I019 Layered Memory Foundation | Planned (prerequisites targeted by I047) | [MEM-001 Layered Memory Foundation](backlog/active/MEM-001-layered-memory-foundation.md) | I047 must satisfy all known prerequisites and deliver MEM-001-A starter; full I019 activation decision follows the long-running task |
| MODEL-001 Model Catalog Foundation | Planned | [MODEL-001 Model Catalog](backlog/active/MODEL-001-model-catalog-and-reasoning.md) | Built-in model dataset + models.dev import; catalog-only (reasoning split to MODEL-003) |
| MODEL-002 Local Micro-Model Decision Layer | Research | [MODEL-002 Local Micro-Model](backlog/active/MODEL-002-local-micro-model-decision-layer.md) | Evaluate local small-model hints; no permission authority before ADR |
| MODEL-003 Reasoning / Thinking Support | ADR-needed | [MODEL-003 Reasoning/Thinking](backlog/active/MODEL-003-reasoning-thinking-support.md) | Per-provider reasoning fields + stream + persistence + TUI; ADR gate |
| DIST-001 Optional Runtime Asset Distribution | Research | [DIST-001 Optional Runtime Asset Distribution](backlog/active/DIST-001-optional-runtime-asset-distribution.md) | Keep default release light; optional large assets install post-install with consent, verification, and offline/mirror support |
| WEBFETCH-001 Web And Document Fetch Tools | Phase 0/1 Complete; Phase 2+ Research | [WEBFETCH-001](backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md) | Plan Phase 2+ with TOOL-007 so document/web ingestion shares the tool family, permission model, and progressive-loading strategy |
| STORE-001 Zvec Storage Evaluation | Research | [STORE-001 Zvec Storage Evaluation](backlog/active/STORE-001-zvec-storage-evaluation.md) | Decide whether Zvec is rejected, watch-only, optional derived index, or ADR-ready; no dependency before Spike |
| TOOL-004 Ripgrep Engine Evaluation | Research | [TOOL-004](backlog/active/TOOL-004-ripgrep-engine-evaluation.md) | Next research priority before TOOL-007: compare embedded ripgrep crates, external `rg`, and current engine before implementation |
| TOOL-005 Bash Streaming Output | I039 Complete | [TOOL-005](backlog/active/TOOL-005-bash-streaming-output.md) | Bash streaming: $cmd header + line-by-line output landed in I039 |
| TOOL-006 Bash → Sh Rename + Cross-OS CLI | Planned | [TOOL-005 §Future](backlog/active/TOOL-005-bash-streaming-output.md) | Rename `bash` → `sh`; Windows `cmd`/`powershell` support; backward-compat alias |
| TOOL-007 Tool Set Design Audit | Research | [TOOL-007 Tool Set Design Audit](backlog/active/TOOL-007-tool-set-design-audit.md) | Run after TOOL-004; include WEBFETCH-001 Phase 2+ in the holistic tool family and permission/progressive-loading design |
| TOOL-008 Tree-sitter On-Demand Loading | Planned | [TOOL-008 Tree-sitter On-Demand](backlog/active/TOOL-008-tree-sitter-on-demand.md) | Phase 1: LazyLock, Phase 2: feature-gated (~8 default langs, 25-30 MB), Phase 3: WASM runtime loading |
| MEM-005 Context Compaction Policy | Planned (Phase 1 selected into I047) | [MEM-005 Context Compaction Policy](backlog/active/MEM-005-context-compaction-policy.md) | I047 selects boundary-aware layers 1-3 policy, manual control, status visibility, and failure fallback; MEM-003 LLM layers remain separate |
| I020 Exploration Library | Planned | [RES-001 Exploration Library](backlog/active/RES-001-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
| AGENT-002 dotagentsprotocol Shared Config | Research | [AGENT-002 dotagentsprotocol.com Support](backlog/active/AGENT-002-dotagents-protocol-support.md) | Three sub-areas: A) `models.json` import, B) `skills/` discovery (needs SKILL-002 gate), C) `mcp.json` import (needs server opt-in ADR) |
| WEB-001 Embedded Web Control | Research (→ P2) | [WEB-001](backlog/active/WEB-001-embedded-web-control-surface.md) | Product differentiation track; study omp.sh/EXT-002 patterns, then define loopback-only embedded UI MVP without permission/auth bypass |
| GOV-003 Built-in Project Governance | Planned (Phase 1 selected into I047) | [GOV-003](backlog/active/GOV-003-builtin-project-governance.md) | I047 selects read-only status/context only; gate enforcement and PM web UI remain later phases |
| TOOL-009 Internet Search Tool | I039 Complete | [TOOL-009](backlog/active/TOOL-009-internet-search-tool.md) | web_search with DDG/Tavily/SearXNG/Wikipedia landed in I039 |
