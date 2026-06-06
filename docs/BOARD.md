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
| (no active work) | — | — | — |

## Review

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I013 Boundary Control | Complete | [I013 Boundary Control](iterations/I013-boundary-control.md) | Boundary ADRs recorded; #ARCH-S8 R1 implemented |
| I010 R3 Product Polish | Complete | [I010 Polished Agent](iterations/I010-polished-agent.md) | All 5 stories done; 567 tests, clippy clean |
| I010 R2 Architecture Convergence | Complete | [I010 Polished Agent](iterations/I010-polished-agent.md) | All acceptance criteria met; 532 tests, clippy clean |

## Blocked / Paused

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I011 S2 Provider Plugin Architecture | Paused | [PROV-001 Provider Schema](backlog/active/PROV-001-provider-schema.md) | Resume as I015 schema-only work under ADR-013 |
| I009-S6 TUI Provenance Markers + /plugins | Deferred | [EXT-001 TUI Provenance](backlog/active/EXT-001-tui-provenance-plugins.md) | Resume through TUI-001/I014 |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I021 Evolution MenteDB Realignment | Planned | [I021 Evolution MenteDB Realignment](iterations/I021-evolution-mentedb-realignment.md) | Start when user requests I021 implementation; defense layer (commit `7470ac5`) already shipped; `knowledge.db` 221MB → 13MB after manual cleanup |
| I014 TUI Completion | Planned | [TUI-001 TUI Completion](backlog/active/TUI-001-completion.md) | Start when next product-facing TUI slice is selected |
| I015 Provider Schema | Planned | [PROV-001 Provider Schema](backlog/active/PROV-001-provider-schema.md) | Start after I014 or explicit provider priority; follow ADR-013 |
| I016 Portable File And Search Tools | Planned | [TOOL-001 Portable File/Search](backlog/active/TOOL-001-portable-file-search.md) | Start when native file/search portability becomes release-critical |
| I018 Observability and Prompt Assets | Planned | [OBS-001 Observability and Prompt Assets](backlog/active/OBS-001-observability-prompt-assets.md) | Start before memory/exploration prompt expansion; follow ADR-014/ADR-015 |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I017 Embedded Git Tools | Planned | [GIT-001 Embedded Git Tools](backlog/active/GIT-001-embedded-git-tools.md) | Start after I016 or explicit Git priority; follow ADR-010 |
| I019 Layered Memory Foundation | Planned | [MEM-001 Layered Memory Foundation](backlog/active/MEM-001-layered-memory-foundation.md) | Start after I018 or explicit memory priority; follow ADR-016 |
| I020 Exploration Library | Planned | [RES-001 Exploration Library](backlog/active/RES-001-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
