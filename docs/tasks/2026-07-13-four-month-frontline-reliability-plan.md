# 2026-07-13 Four-Month Frontline Reliability Plan

**Status**: Program plan; ready for assignment; no iteration activated
**Timebox**: 2026-07-13 through 2026-11-12 (16 weeks)
**Execution owner**: one frontline developer per active iteration
**Program owner**: maintainer for gates, review, and release decisions

## Objective

Give a frontline developer four consecutive, runnable product outcomes without requiring them to
invent architecture or permission policy: truthful diagnostics, visible TUI interaction, coherent
local extension/control diagnostics, and repeatable install/trial confidence.

## Governance Hierarchy

This file is the **program plan**, not an iteration and not the developer's resumable task owner.

```text
Long task owner: 2026-07-13-frontline-developer-execution-package.md
  ├─ Iteration I120 (month 1) ── stories F100-F103
  ├─ Iteration I121 (month 2) ── stories F110-F113
  ├─ Iteration I122 (month 3) ── stories F120-F123
  └─ Iteration I123 (month 4) ── stories F130-F133
```

- The long-task owner holds checkpoints, recovery state, authority, and the next exact action.
- An iteration is the only monthly activation/closeout unit; at most one is Active.
- An F-ID is a story and commit-sized acceptance unit. It is never activated independently and is
  never called an iteration.

## Success Criteria

- Each month ends with a user-runnable binary result and its own automated regression suite.
- A clean checkout plus the developer handoff file is sufficient to start and resume work.
- No story changes permission decisions, session format, provider protocol, public API, or release
  policy without a separately accepted change record.
- Workspace tests, strict Clippy, release preflight, and governance validation pass before each
  monthly closeout.

## Non-Terminal Inventory And Disposition

The iteration index had no Active or Review iteration at publication. Every non-terminal iteration
is disposed below; Board-level Review items are listed separately because they are not iterations.

| Iteration/item | State at inventory | Disposition before I120 activation |
|---|---|---|
| I018 | Planned | Deferred; bounded logs/prompts are not an I120 prerequisite. |
| I019 | Planned, dependency-blocked by I018 | Deferred; do not bypass its published dependency. |
| I020 | Planned, dependency-blocked by I019 | Deferred; do not bypass its published dependency. |
| I028 | Planned | Deferred; scheduling/permission prerequisites are outside this program. |
| I081 | Stale Planned header; superseded remainder | Reconcile to Superseded; preserve baseline and do not reactivate. |
| I082 | Stale Planned header; superseded remainder | Reconcile to Superseded; preserve baseline and do not reactivate. |
| I083 | Stale Planned header; superseded remainder | Reconcile to Superseded; preserve baseline and do not reactivate. |
| I120 | Planned | Ready for assignment; only Gate 0 may activate it. |
| I121 | Planned | Blocked on I120 Complete. |
| I122 | Planned | Blocked on I121 Complete. |
| I123 | Planned | Blocked on I122 Complete. |
| Issue / Doc / Code Status Audit | Board Review, not an iteration | Retain Review; it does not block I120, but F100 updates it for any owner drift found. |
| I046, I047, I057, I058, I079 | Stale owner headers vs Complete index/evidence | Reconciled to Complete on 2026-07-13; no code or acceptance claim added. |

Selected story disposition: TUI-008/TUI-024 enter I121; TUI-025/026/027 remain unselected;
PLUGIN-001/CMD-002/HOOK-001/WEB-001 contribute only bounded read-only diagnostics to I122;
PERM-004/PERM-005 are not selected; REL-002 remains NO-GO and I123 cannot qualify or release it.

## Four-Month Execution Matrix

| ID | Weeks | Iteration | Runnable deliverable | Validation |
|---|---:|---|---|---|
| F100 | 1 | I120 | Owner-state fixture and dynamic diagnostics contract | parser/fixture tests |
| F101 | 2 | I120 | `talos diagnostics status --json` uses `serde_json` and emits valid escaped JSON | CLI integration tests |
| F102 | 3 | I120 | Active iteration and residual gates come from shared/current sources with bounded fallback | clean/missing/malformed workspace fixtures |
| F103 | 4 | I120 | Diagnostics docs and smoke closeout | real binary + full gates |
| F110 | 5 | I121 | Approval overlay is prominent at 80 columns and narrow terminals | ratatui buffer snapshots |
| F111 | 6 | I121 | Keyboard approval flow and permission decisions remain unchanged | event/permission regressions |
| F112 | 7 | I121 | Thinking preview derives standalone bold section titles while retaining the `thinking` prefix | parser/render/export tests |
| F113 | 8 | I121 | Native-terminal visual packet and accessibility notes | real TUI walkthrough + full gates |
| F120 | 9 | I122 | One typed extension-diagnostics snapshot for MCP/plugins/hooks | schema/unit tests |
| F121 | 10 | I122 | `/mcp`, `/plugins`, `/hooks` show consistent status, provenance, collisions, and bounded failures | CLI/TUI fixture smoke |
| F122 | 11 | I122 | Dashboard exposes the same read-only extension/governance snapshot with redaction | loopback/auth/no-write-route tests |
| F123 | 12 | I122 | Extension/control documentation and failure matrix closeout | real binary/dashboard + full gates |
| F130 | 13 | I123 | Installer fixtures cover archive/checksum/offline/error behavior on supported script paths | POSIX/PowerShell fixture CI |
| F131 | 14 | I123 | Clean-HOME trial smoke covers setup, mock turn, session resume/export, Ask/Deny, diagnostics | real binary, no real credentials |
| F132 | 15 | I123 | Failure recovery and troubleshooting packet is replayable by a second operator | cold-start replay |
| F133 | 16 | I123 | Trial-readiness report and residual owners; no release action | full preflight + maintainer review |

## Monthly Exit Gates

1. I120: diagnostics JSON parses, contains no secret values, has no hardcoded stale iteration, and
   degrades safely when governance files are missing or malformed.
2. I121: approval is visibly prominent and thinking titles render without changing permission or
   persisted/exported reasoning semantics.
3. I122: CLI/TUI/dashboard agree on local extension state; every route remains read-only and
   loopback/auth boundaries still pass.
4. I123: a second operator can exercise install fixtures and the clean-HOME trial from written
   instructions; the result makes no release or REL-002 overclaim.

## Authority Boundary

Frontline developers may edit files named by the active iteration, add tests/fixtures, update
affected user docs, and create ordinary task-branch commits. They may not tag, publish, deploy,
push to `main`, change permission defaults, add remote/write dashboard routes, add plugin host
calls, introduce native dependencies, or amend an ADR. Stop and escalate if any acceptance item
appears to require one of those actions.

## Dependency And Fallback

```text
I120 -> I121 -> I122 -> I123
```

- If shared governance parsing would require a public API change, keep it crate-private and reuse
  existing parser helpers; otherwise stop for maintainer design review.
- If terminal snapshots vary by platform, assert semantic regions/styles in buffers and retain one
  named native-terminal walkthrough instead of golden ANSI streams.
- If extension state has no single owner, add a crate-private serializable view assembled from
  existing registries; do not add an event bus or duplicate mutable state.
- If Windows execution is unavailable, require static PowerShell parse plus Windows CI fixture; do
  not claim a local Windows install pass.

## Activation And Recovery

- I120 is Planned and may be activated only after the assignee completes Gate 0 in
  `2026-07-13-frontline-developer-execution-package.md`.
- Only one iteration may be Active. Later iterations remain Planned until the previous one is
  Complete.
- Resume by reading the execution package's latest checkpoint, this plan, the active iteration,
  its selected owner stories, and `docs/BOARD.md`.
- Push, PR, tag, publish, and release actions require a separate maintainer instruction.
