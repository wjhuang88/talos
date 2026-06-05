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
| I011 S2 Provider Plugin Architecture | Paused | [I011 Open Providers](iterations/I011-open-providers.md) | Resume after I010 or an explicit priority-change update |
| I009-S6 TUI Provenance Markers + /plugins | Deferred | [Product Backlog](backlog/PRODUCT-BACKLOG.md) | Resume in I010 R2/R3 or a dedicated follow-up |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I014 TUI Completion | Planned | [I014 TUI Completion](iterations/I014-tui-completion.md) | Start when next product-facing TUI slice is selected |
| I015 Provider Schema | Planned | [I015 Provider Schema](iterations/I015-provider-schema.md) | Start after I014 or explicit provider priority; follow ADR-013 |
| I016 Portable File And Search Tools | Planned | [I016 Portable File And Search Tools](iterations/I016-portable-file-search.md) | Start when native file/search portability becomes release-critical |
| I018 Observability and Prompt Assets | Planned | [I018 Observability and Prompt Assets](iterations/I018-observability-prompt-assets.md) | Start before memory/exploration prompt expansion; follow ADR-014/ADR-015 |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I017 Embedded Git Tools | Planned | [I017 Embedded Git Tools](iterations/I017-embedded-git-tools.md) | Start after I016 or explicit Git priority; follow ADR-010 |
| I019 Layered Memory Foundation | Planned | [I019 Layered Memory Foundation](iterations/I019-layered-memory-foundation.md) | Start after I018 or explicit memory priority; follow ADR-016 |
| I020 Exploration Library | Planned | [I020 Exploration Library](iterations/I020-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
