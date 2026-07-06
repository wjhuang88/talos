# ARCH-032: Single Data Flow Audit

| Field | Value |
|---|---|
| Story ID | ARCH-032 |
| Priority | P1 |
| Status | Planned |
| Source | [GitHub Issue #35](https://github.com/wjhuang88/talos/issues/35) |
| Depends On | ADR-004, ADR-005, ADR-006 |

## Problem

Talos has added hooks, evolution, MCP, dashboard, memory, compaction, permissions, and runtime
surfaces after the original single-consumer event-loop decisions. The project needs an explicit audit
to verify whether these paths still obey the ADR-006 boundary: no global event bus, no uncontrolled
broadcast, and no multi-consumer tool/permission side channel.

## Acceptance

- Document all current producer/consumer channels for UI, agent, session, hooks, evolution, MCP,
  dashboard, permission, memory, and compaction paths.
- Classify every path as producer-to-single-consumer `mpsc`, SQ/EQ seam, bounded request/response,
  or deviation.
- Record deviations with risk, owner, and required follow-up story.
- Update `docs/reference/ARCHITECTURE.md` only with factual current-state diagrams; do not rewrite
  ADR history.
- No code changes in this audit story except tests/scripts needed to collect evidence.

## Required Reads

- `docs/decisions/004-tui-event-loop.md`
- `docs/decisions/005-agent-session-boundary.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/reference/ARCHITECTURE.md`
- `crates/talos-agent/src/lib.rs`
- `crates/talos-cli/src/tui_bridge.rs`
- `crates/talos-conversation/src/`
- `crates/talos-evolution/src/`
- `crates/talos-plugin/src/`

