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
| I010 | Polished Agent | **Active** (R3 Complete 2026-06-04) | R2: AppServerSession convergence, TUI approval, inline mode. R3: Nord theme, markdown rendering, diff display, steering queues, slash commands. 567 tests. See `I010-polished-agent.md`. |
| I011 | Open Providers | **Paused** (S1 landed 2026-06-02; S2 deferred) | OpenAI-compatible `base_url` override + `OPENAI_COMPAT_API_KEY` env var shipped. S2 provider-plugin architecture is deferred until after R1/I010 or explicit priority change. See `I011-open-providers.md`. |
| I012 | Portable Tools | Planned | Native POSIX-style tool subset, embeddable tool-pack registration, Rust-native workspace search, and `gix`-first structured Git tools without first-slice `git2`/libgit2. See `I012-portable-tools.md` and ADR-010. |
| I013 | Boundary Control | **Complete** (2026-06-05) | Front-loaded high-risk boundary work: ADR-011 Guardian, ADR-012 exec DSL, ADR-013 provider schema, and #ARCH-S8 R1 centralized logging. See `I013-boundary-control.md`. |
| I014 | TUI Completion | **Complete** (2026-06-06) | Finish TUI provenance/plugin visibility and copy/export workflows. Two stories: #I009-S6 (provenance markers + `/plugins`) and #I010-S9 (clipboard copy/export) landed via 2 atomic commits. 652 tests pass workspace-wide (was 615; +37 from talos-tui). See `I014-tui-completion.md`; EXT-001 backlog; ADR-009. |
| I015 | Provider Schema | **Complete** (2026-06-08) | Schema types and built-in defaults landed 2026-06-06; one-way opencode import (`talos-config::opencode`) with 9 unit tests landed 2026-06-08. `cargo test -p talos-config -p talos-provider -p talos-cli` passes. See `I015-provider-schema.md`; `PROV-001-provider-schema.md`; ADR-013. |
| I016 | Portable File And Search Tools | Planned | Split I012 file/tool-pack/search work into a smaller native tools iteration. See `I016-portable-file-search.md`. |
| I017 | Embedded Git Tools | Planned | Split I012 Git work into a dedicated `gix`-first read-only Git tools iteration. See `I017-embedded-git-tools.md`. |
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
| I030 | Session Module Decomposition | **Planned** | Complete ARCH-008 by splitting `talos-session/src/lib.rs` into topology, JSONL, and session actor modules with no behavior change. See `I030-session-module-decomposition.md`. |
| I031 | Skill And CLI Module Cleanup | **Planned** | Complete ARCH-009 and the CLI slice of ARCH-010: split `talos-skill/src/lib.rs` and extract CLI mode runners. See `I031-skill-and-cli-module-cleanup.md`. |
| I032 | Tools Module Cleanup | **Planned** | Finish ARCH-010 tools cleanup by decomposing `talos-tools/src/file_tools.rs`; SCHED-001 remains in I028. See `I032-tools-module-cleanup.md`. |
| I033 | Agent Protocol Compatibility Foundation | **Planned** | Turn AGENT-001 into a dated protocol/config compatibility plan and read/import-first foundation for shared Agent config such as `~/.agent`. See `I033-agent-protocol-compatibility-foundation.md`. |

> Update this table whenever an iteration changes state. "Complete" requires runtime
> evidence, not only passing unit tests — see `docs/sop/ITERATION-WORKFLOW.md`.

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
| R17: Session Boundary Cleanup | Next architecture slice | `I030-session-module-decomposition.md`; `../backlog/active/ARCH-008-session-module-decomposition.md` | `talos-session/src/lib.rs` is decomposed without behavior change; session tests/clippy pass. |
| R18: Skill And CLI Cleanup | After R17 | `I031-skill-and-cli-module-cleanup.md`; `../backlog/active/ARCH-009-skill-module-decomposition.md`; `../backlog/active/ARCH-010-cli-tools-module-cleanup.md` | `talos-skill` and CLI mode-runner boundaries are decomposed; targeted tests/clippy pass. |
| R19: Tools Cleanup | After R18 | `I032-tools-module-cleanup.md`; `../backlog/active/ARCH-010-cli-tools-module-cleanup.md` | `file_tools.rs` is split before new tool growth; tool tests/clippy and workspace tests pass. |
| R20: Agent Protocol Compatibility | After R17-R19 or as research-only if cleanup slips | `I033-agent-protocol-compatibility-foundation.md`; `../backlog/active/AGENT-001-standard-agent-protocol-support.md`; `../proposals/standard-agent-protocol-support.md` | Dated protocol/config survey, ADR, and read/import-first plan for shared Agent config such as `~/.agent`. |
