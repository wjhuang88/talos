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
| R1 Review Closure | Active | [R1 Review Closure](iterations/R1-review-closure.md) | I008/I009 close or residual work is moved through change control; I011 S2 remains paused |

## Review

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I008 Learning Agent | Review | [I008 Learning Agent](iterations/I008-learning-agent.md) | Fresh print/mock and TUI/mock runtime evidence is recorded, then status is synchronized |
| I009 Extensible Agent | Review | [I009 Extensible Agent](iterations/I009-extensible-agent.md) | TUI provenance marker and `/plugins` work lands, or moves to a numbered follow-up through change control |

## Blocked / Paused

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I011 S2 Provider Plugin Architecture | Paused | [I011 Open Providers](iterations/I011-open-providers.md) | Resume after R1/I010 or an explicit priority-change update |

## Next

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I010 R2 Architecture Convergence | Planned | [I010 Polished Agent](iterations/I010-polished-agent.md) | Activate after R1 closes |

## Later

| Item | State | Owner Doc | Gate |
|---|---|---|---|
| I010 R3 Product Polish | Planned | [I010 Polished Agent](iterations/I010-polished-agent.md) | Start after I010 R2 verification evidence is recorded |
| I012 Portable Tools | Planned | [I012 Portable Tools](iterations/I012-portable-tools.md) | Start after I010/R3 or when environment-dependency reduction becomes release-critical |
