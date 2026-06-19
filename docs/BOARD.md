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

## Review

| Item | State | Owner Doc | Gate |
|---|---|---|---|
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
| I033 Runtime Skill Activation | Planned | [I033 Runtime Skill Activation](iterations/I033-runtime-skill-activation.md) | Discover skills at startup; inject Level 0 metadata; define Level 1/2 activation |
| I034 MCP Session Integration | Planned | [I034 MCP Session Integration](iterations/I034-mcp-session-integration.md) | MCP startup discovery, tool registry injection, permission routing, provenance/status |
| I035 Agent Protocol Compatibility Foundation | Planned | [I035 Agent Protocol Compatibility Foundation](iterations/I035-agent-protocol-compatibility-foundation.md) | Survey + ADR + read/import-first shared Agent config plan |
| I036 Research Consolidation | Planned | [I036 Research Consolidation](iterations/I036-research-consolidation.md) | End-of-plan research-only pass turns REMOTE/WEB/WEBFETCH/PLUGIN/OKF/MEM/MODEL/STORE items into decisions or follow-up stories |
| I028 SCHED-001 Delayed and Scheduled Tasks | Planned | [SCHED-001 Delayed/Scheduled Tasks](backlog/active/SCHED-001-delayed-scheduled-tasks.md) | 4 tools (`delay`, `schedule`, `cancel_scheduled_task`, `list_scheduled_tasks`); session-scoped message injection; start after I029 architecture cleanup |
| SKILL-001 Runtime Skill Activation | Planned | [SKILL-001](backlog/active/SKILL-001-runtime-skill-activation.md) | Discover skills at startup; inject Level 0 metadata; define Level 1/2 activation |
| MCP-001 MCP Session Integration | Planned | [MCP-001](backlog/active/MCP-001-session-mcp-integration.md) | Discover configured MCP tools at startup; route through permissions and provenance |
| ARCH-011 Architecture Watchlist | Planned | [ARCH-011](backlog/active/ARCH-011-architecture-watchlist.md) | Track tests.rs, prompt.rs, scrollback.rs as observation-only module-growth risks |
| TUI-008 Approval Dialog UX | Planned | [TUI-008 Approval Dialog UX](backlog/active/TUI-008-approval-dialog-ux.md) | Move approval from bottom-right to prominent position; easy to miss currently |
| TUI-009 Input Clear And Session Exit Summary | Planned | [TUI-009 Input Clear And Session Exit Summary](backlog/active/TUI-009-input-and-session-exit-polish.md) | Ctrl+C clears idle input instead of Esc; clean exit prints compact session usage summary |
| TUI-010 Slash Command Menu Below Input | Planned | [TUI-010 Slash Command Menu Below Input](backlog/active/TUI-010-slash-command-menu.md) | Typing `/` opens a Codex-like command menu below the composer with filtering and keyboard navigation |
| I016 Portable File And Search Tools | Planned | [TOOL-001 Portable File/Search](backlog/active/TOOL-001-portable-file-search.md) | Residual scope beyond TOOL-003 (persistent indexes, extra native deps) |
| I018 Observability and Prompt Assets | Planned | [OBS-001 Observability and Prompt Assets](backlog/active/OBS-001-observability-prompt-assets.md) | Start before memory/exploration prompt expansion; follow ADR-014/ADR-015 |

## Done This Cycle

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I029 Architecture Cleanup Completion | Complete | [I029 Architecture Cleanup](iterations/I029-architecture-cleanup-completion.md) | ARCH-004/007/006/005 all landed. 10345→2082 lines across 4 god modules (-80%). |
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

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| REMOTE-001 Remote Session Protocol | Research | [REMOTE-001 Remote Session Protocol](backlog/active/REMOTE-001-remote-session-protocol.md) | Research transport, auth, and minimum viable slice before any implementation |
| PLUGIN-001 WASM Runtime Plugin Protocol | Research | [PLUGIN-001 WASM Runtime Plugin Protocol](backlog/active/PLUGIN-001-wasm-runtime-plugins.md) | Design protocol spec for WASM plugins providing tools, hooks, and filters; ADR required before runtime implementation |
| I017 Embedded Git Tools | P0-P2 Complete | [GIT-001 Embedded Git Tools](backlog/active/GIT-001-embedded-git-tools.md) | Read/write Git tools delivered in I026; future scope starts from P3 advanced tools |
| I019 Layered Memory Foundation | Planned | [MEM-001 Layered Memory Foundation](backlog/active/MEM-001-layered-memory-foundation.md) | Start after I018 or explicit memory priority; follow ADR-016 |
| MODEL-001 Model Catalog And Reasoning Capability Foundation | Planned | [MODEL-001 Model Catalog And Reasoning Capability Foundation](backlog/active/MODEL-001-model-catalog-and-reasoning.md) | Built-in model dataset, models.dev import/cache, reasoning/thinking support, and compaction limit source |
| MODEL-002 Local Micro-Model Decision Layer | Research | [MODEL-002 Local Micro-Model Decision Layer](backlog/active/MODEL-002-local-micro-model-decision-layer.md) | Evaluate local small-model hints for intent/routing/title/tool candidates; no permission authority before ADR |
| DIST-001 Optional Runtime Asset Distribution | Research | [DIST-001 Optional Runtime Asset Distribution](backlog/active/DIST-001-optional-runtime-asset-distribution.md) | Keep default release light; optional large assets install post-install with consent, verification, and offline/mirror support |
| WEBFETCH-001 Web And Document Fetch Tools | Research | [WEBFETCH-001 Web And Document Fetch Tools](backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md) | Design context fetch, URL auto-detection, HTML extraction, link storage, document conversion, and separate URL save tool |
| STORE-001 Zvec Storage Evaluation | Research | [STORE-001 Zvec Storage Evaluation](backlog/active/STORE-001-zvec-storage-evaluation.md) | Decide whether Zvec is rejected, watch-only, optional derived index, or ADR-ready; no dependency before Spike |
| MEM-005 Context Compaction Policy | Planned | [MEM-005 Context Compaction Policy](backlog/active/MEM-005-context-compaction-policy.md) | Define automatic/manual compaction triggers, pre-turn ordering, status visibility, and failure fallback |
| I020 Exploration Library | Planned | [RES-001 Exploration Library](backlog/active/RES-001-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
