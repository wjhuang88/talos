# Product Backlog

This file is the compact backlog entrypoint. It preserves priority, scope, dependency, and
decision context; executable details live in item files listed under Required Reads.

## Current Priorities

| Priority | Focus | Why Now | Required Reads |
| --- | --- | --- | --- |
| 1 | Evolution MenteDB realignment (root-cause fix) | The 7470ac5 defense layer caps the 5MB bloat, but the underlying `Signal.context` field semantics are wrong by construction. I021 realigns the data structure with the MenteDB blueprint so the bloat and 400-error loop cannot recur. | `docs/iterations/I021-evolution-mentedb-realignment.md`; `docs/reference/REFERENCE-PROJECTS.md` §17; EVOLUTION.md lesson #19 |
| 2 | TUI completion | Completes deferred user-facing extensibility and copy/export workflows before deeper runtime work. | `docs/backlog/active/TUI-001-completion.md`; `docs/iterations/I014-tui-completion.md` |
| 3 | Observability and prompt assets | Bounded logs and embedded prompts should land before memory/exploration increase log and prompt surface area. | `docs/backlog/active/OBS-001-observability-prompt-assets.md`; ADR-014; ADR-015 |
| 4 | Provider schema | Provider openness is paused until schema-only foundation follows ADR-013. | `docs/backlog/active/PROV-001-provider-schema.md`; ADR-013 |
| 5 | Portable file/search tools | Reduces host environment dependency without mixing Git implementation risk into the same slice. | `docs/backlog/active/TOOL-001-portable-file-search.md`; ADR-010 |
| 6 | Embedded Git tools | Dedicated `gix`-first slice after file/search baseline or explicit Git priority. | `docs/backlog/active/GIT-001-embedded-git-tools.md`; ADR-010 |
| 7 | Layered memory and exploration | Memory must precede durable research conclusions; research library follows memory or explicit research priority. | `docs/backlog/active/MEM-001-layered-memory-foundation.md`; `docs/backlog/active/RES-001-exploration-library.md`; ADR-016; ADR-017 |
| 8 | Evolution cognitive rigor (MenteDB Phase 2) | 6 SignalKinds, Bayesian confidence, time decay, cross-session provenance, outcome tracking. **Independent of I021** — does not solve the bloat / 400 / signal-loss problems. Schedule after I021 lands. | `docs/backlog/active/EVOL-001-evolution-cognitive-rigor.md`; `docs/iterations/I021-evolution-mentedb-realignment.md`; `docs/reference/REFERENCE-PROJECTS.md` §17 |

## Active Items

| ID | Title | Status | Priority | Decision Context | Required Reads |
| --- | --- | --- | --- | --- | --- |
| TUI-001 | TUI completion | Planned | P1 | Finishes deferred I009 provenance visibility and transcript copy/export before more runtime features add UI states. | `docs/backlog/active/TUI-001-completion.md`; `docs/iterations/I014-tui-completion.md`; ADR-009 |
| OBS-001 | Observability and prompt assets | Planned | P1 | #ARCH-S8 R2 must bound local log files; built-in prompts must be standalone compile-time assets before memory/exploration prompts grow. | `docs/backlog/active/OBS-001-observability-prompt-assets.md`; `docs/iterations/I018-observability-prompt-assets.md`; ADR-014; ADR-015 |
| PROV-001 | Provider schema foundation | Planned | P2 | Provider S2 resumes only as schema/config work under ADR-013; dynamic provider loading remains out of scope. | `docs/backlog/active/PROV-001-provider-schema.md`; `docs/iterations/I015-provider-schema.md`; `docs/proposals/provider-plugin-architecture.md`; ADR-013 |
| TOOL-001 | Portable file/search tools | Planned | P2 | Native file/search tools reduce host utility dependence; persistent indexes and extra native deps remain deferred. | `docs/backlog/active/TOOL-001-portable-file-search.md`; `docs/iterations/I016-portable-file-search.md`; `docs/proposals/builtin-workspace-search-tools.md`; ADR-010 |
| GIT-001 | Embedded Git tools | Planned | P2 | Git work is split from file/search so `gix` API mapping and fallback behavior stay auditable. | `docs/backlog/active/GIT-001-embedded-git-tools.md`; `docs/iterations/I017-embedded-git-tools.md`; ADR-010 |
| MEM-001 | Layered memory foundation | Planned | P2 | Memory architecture must separate working, episodic, semantic, and procedural memory with explicit consolidation. | `docs/backlog/active/MEM-001-layered-memory-foundation.md`; `docs/iterations/I019-layered-memory-foundation.md`; ADR-016; ADR-002; ADR-008 |
| RES-001 | Exploration library | Planned | P2 | Research artifacts need local source/claim/synthesis provenance; vector/graph stores remain Spike-gated. | `docs/backlog/active/RES-001-exploration-library.md`; `docs/iterations/I020-exploration-library.md`; ADR-017; ADR-008 |
| EVOL-001 | Evolution cognitive rigor (MenteDB Phase 2) | Planned | P3 | 6 SignalKinds, Bayesian confidence, time decay, cross-session provenance, outcome tracking. **Independent of the I021 root-cause fix** — does not solve the 5MB bloat / 400 error / signal-loss problems. Schedule after I021 lands. | `docs/backlog/active/EVOL-001-evolution-cognitive-rigor.md`; `docs/iterations/I021-evolution-mentedb-realignment.md`; `docs/reference/REFERENCE-PROJECTS.md` §17 |

## Blocked Items

| ID | Title | Status | Priority | Decision Context | Required Reads |
| --- | --- | --- | --- | --- | --- |
| EXT-001 | TUI provenance markers and `/plugins` | Deferred | P1 | Backend provenance exists, but user-facing TUI consumer work was deferred from I009 and now rolls into TUI-001/I014. | `docs/backlog/active/EXT-001-tui-provenance-plugins.md`; `docs/iterations/I009-extensible-agent.md`; ADR-009 |
| PERM-001 | Guardian and exec approval policy | Deferred | P2 | AI approval and exec DSL can change permission boundaries; implementation must follow ADR-011/ADR-012 and cannot start as polish work. | `docs/backlog/active/PERM-001-guardian-exec-policy.md`; ADR-011; ADR-012 |
| PROV-001 | Provider plugin architecture | Paused | P2 | S1 gateway support shipped; S2 resumes as schema-only I015 work under ADR-013. | `docs/backlog/active/PROV-001-provider-schema.md`; `docs/iterations/I011-open-providers.md`; ADR-013 |

## Archived Index

| ID / Source | Status | Decision Context | Required Reads |
| --- | --- | --- | --- |
| I001-I013 historical backlog | Archived | Original monolithic backlog preserved for completed and superseded story detail. Active decision context has been copied into item files above. | `docs/backlog/archive/2026-Q2/PRODUCT-BACKLOG-monolith-2026-06-05.md` |
| GOV-001 backlog compaction | Complete | Completed the transition from monolithic backlog to compact entrypoint + item files. | `docs/backlog/archive/2026-Q2/GOV-001-backlog-compaction.md`; `docs/sop/EVOLUTION-FEEDBACK.md` |

## Reading Rules

For backlog-related work:

1. Read this file first.
2. Find the target row.
3. Read every path listed under Required Reads before implementation or prioritization.
4. Read the archive only when a Required Reads entry points to it, the user asks for history, or a
   story explicitly depends on archived rationale.
5. Do not add long acceptance criteria or execution logs here; put them in the item file or
   iteration record.
