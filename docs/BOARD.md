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
| I010-S9 TUI Clipboard Copy/Export | Planned | [Product Backlog](backlog/PRODUCT-BACKLOG.md) | Start after current TUI module split is committed or an explicit priority change |
| ARCH-S8 Production Logging | Planned | [Product Backlog](backlog/PRODUCT-BACKLOG.md) | Start after I010 R3 or explicit priority change |
| I012 Portable Tools | Planned | [I012 Portable Tools](iterations/I012-portable-tools.md) | Start after I010 or when environment-dependency reduction becomes release-critical |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I012 Portable Tools | Planned | [I012 Portable Tools](iterations/I012-portable-tools.md) | Start after I010/R3 or when environment-dependency reduction becomes release-critical |
