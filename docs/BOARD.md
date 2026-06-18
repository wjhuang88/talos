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
| I026 Approval UX Fix + Doc Validation | Review | [I026 Approval UX + Doc Validation](iterations/I026-approval-ux-doc-validation.md) | All 7 stories implemented; clippy, workspace tests, and governance validation pass |

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
| TUI-008 Approval Dialog UX | Planned | [TUI-008 Approval Dialog UX](backlog/active/TUI-008-approval-dialog-ux.md) | Move approval from bottom-right to prominent position; easy to miss currently |
| ARCH-002 Architecture Optimization | Planned | [ARCH-002 Architecture Optimization](backlog/active/ARCH-002-architecture-optimization.md) | Prompt template/cache Phase A complete; remaining: full doc audit → architecture audit → ACL → decomposition → traits |
| I016 Portable File And Search Tools | Planned | [TOOL-001 Portable File/Search](backlog/active/TOOL-001-portable-file-search.md) | Residual scope beyond TOOL-003 (persistent indexes, extra native deps) |
| I018 Observability and Prompt Assets | Planned | [OBS-001 Observability and Prompt Assets](backlog/active/OBS-001-observability-prompt-assets.md) | Start before memory/exploration prompt expansion; follow ADR-014/ADR-015 |

## Done This Cycle

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| TOOL-003 P0 POSIX Tool Set | P0 Complete | [TOOL-003 POSIX Tool Set](backlog/active/TOOL-003-posix-tool-set.md) | grep/glob/ls/delete + read offset/limit + ls long format; 42 new tests; permission rules; TUI summaries; prompt optimization; clippy clean |
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
| I017 Embedded Git Tools | P0-P2 Complete | [GIT-001 Embedded Git Tools](backlog/active/GIT-001-embedded-git-tools.md) | Read/write Git tools delivered in I026; future scope starts from P3 advanced tools |
| I019 Layered Memory Foundation | Planned | [MEM-001 Layered Memory Foundation](backlog/active/MEM-001-layered-memory-foundation.md) | Start after I018 or explicit memory priority; follow ADR-016 |
| I020 Exploration Library | Planned | [RES-001 Exploration Library](backlog/active/RES-001-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
