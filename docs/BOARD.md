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
| I024 Conversation Context | Active | [I024 Conversation Context](iterations/I024-conversation-context.md) | P0: wire session JSONL history + compaction into agent turn loop; all modes receive conversation history; multi-turn conversations verified |

## Review

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I023 TUI State Model | Complete | [I023 TUI State Model](iterations/I023-tui-state-model.md) | Review remediation closed: broadcast→mpsc non-lossy delivery, agent abort-on-cancel, SIGINT fallback, engine-owned mutation verified (pub(crate)), workspace verification clean |
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
| I016 Portable File And Search Tools | Planned | [TOOL-001 Portable File/Search](backlog/active/TOOL-001-portable-file-search.md) | Start when native file/search portability becomes release-critical |
| I018 Observability and Prompt Assets | Planned | [OBS-001 Observability and Prompt Assets](backlog/active/OBS-001-observability-prompt-assets.md) | Start before memory/exploration prompt expansion; follow ADR-014/ADR-015 |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I017 Embedded Git Tools | Planned | [GIT-001 Embedded Git Tools](backlog/active/GIT-001-embedded-git-tools.md) | Start after I016 or explicit Git priority; follow ADR-010 |
| I019 Layered Memory Foundation | Planned | [MEM-001 Layered Memory Foundation](backlog/active/MEM-001-layered-memory-foundation.md) | Start after I018 or explicit memory priority; follow ADR-016 |
| I020 Exploration Library | Planned | [RES-001 Exploration Library](backlog/active/RES-001-exploration-library.md) | Start after I019 or explicit research priority; follow ADR-017 |
