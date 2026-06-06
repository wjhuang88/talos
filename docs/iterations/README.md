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
| I014 | TUI Completion | Planned | Finish TUI provenance/plugin visibility and copy/export workflows. See `I014-tui-completion.md`. |
| I015 | Provider Schema | Planned | Implement schema-only provider config foundation under ADR-013. See `I015-provider-schema.md`. |
| I016 | Portable File And Search Tools | Planned | Split I012 file/tool-pack/search work into a smaller native tools iteration. See `I016-portable-file-search.md`. |
| I017 | Embedded Git Tools | Planned | Split I012 Git work into a dedicated `gix`-first read-only Git tools iteration. See `I017-embedded-git-tools.md`. |
| I018 | Observability and Prompt Assets | Planned | Bounded file-log retention and compile-time embedded prompt assets. See `I018-observability-prompt-assets.md`. |
| I019 | Layered Memory Foundation | Planned | Four-layer memory foundation under ADR-016. See `I019-layered-memory-foundation.md`. |
| I020 | Exploration Library | Planned | Local research library, source/claim/synthesis storage, and vector/graph storage Spike under ADR-017. See `I020-exploration-library.md`. |
| I021 | Evolution MenteDB Realignment | Planned | Root-cause fix for the 5MB knowledge.db bloat and `400 Bad Request` loop. Realigns `talos-evolution` data structure with the MenteDB blueprint (`Signal.context` becomes a small window, `TurnObservation` aggregates per-turn, `Pattern` gets `key`/`value`/`contradicting_count`/`source_sessions`). Defense layer from commit 7470ac5 stays as belt-and-suspenders. See `I021-evolution-mentedb-realignment.md`; EVOLUTION.md lesson #19. |

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
| R12: I021 Evolution MenteDB Realignment | Before EVOL-001 cognitive rigor stories; independent of R5–R11 ordering | `I021-evolution-mentedb-realignment.md` | `talos-evolution` data structure aligned with MenteDB blueprint; `knowledge.db` cannot grow past 1MB during 20-turn stress test; 5MB bloat and `400 Bad Request` loop cannot recur |
