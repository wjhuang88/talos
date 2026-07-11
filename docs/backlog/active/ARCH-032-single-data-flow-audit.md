# ARCH-032: Single Data Flow Audit

| Field | Value |
|---|---|
| Story ID | ARCH-032 |
| Priority | P1 |
| Status | Complete (I108 SBT121, 2026-07-09) |
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

## ARCH-032 Audit Results (I108 SBT121, 2026-07-09)

### Scope
All src/ files in 12 crates: `talos-agent`, `talos-cli`, `talos-conversation`, `talos-core`, `talos-evolution`, `talos-mcp`, `talos-memory`, `talos-permission`, `talos-plugin`, `talos-runtime`, `talos-session`, `talos-tui`.

### Findings
The workspace is **fully compliant with ADR-006**. No deviations found.

- **Zero `broadcast::channel` usages** across the entire workspace.
- **All mpsc channels** are single-consumer (one `.recv()` site per channel).
- **Three `watch::channel` instances** in `talos-cli/src/mode_runners.rs:702-704` carry state snapshots (`Session`, `mpsc::Sender<SessionOp>`, `ModelInfo`), not event streams. This is the "deterministic fan-out from single consumer" pattern ADR-006 §73-75 endorses.
- **Hook system** (`HookRegistry` in `talos-plugin/src/registry.rs`) uses sequential trait-method dispatch via `HashMap<HookEventKind, Vec<Arc<dyn HookHandler>>>`. Not a channel. `EvolutionHookHandler` and `LoggingHandler` are registered per-Agent. No global event bus.
- **MCP transport** uses `oneshot::channel` for JSON-RPC request/response correlation — bounded, single-consumer.
- **WASM watchdog** uses `std::sync::mpsc::channel<()>` scoped to one `execute_inner` call — function-local, single-consumer.
- **Sync crates** (`talos-session`, `talos-permission`, `talos-memory`) have zero channels — pure synchronous.
- **Dashboard** has zero channels — serves pre-computed snapshots via HTTP.
- **No global/static channels** or singleton patterns found.

### Channel Classification Summary
| Category | Count | ADR-006 |
|---|---|---|
| SQ/EQ session seam (mpsc) | 2 | Adopted (A+B) |
| Per-turn agent event channel (mpsc) | 1 | Compliant (A) |
| Per-turn result oneshot | 1 | Compliant |
| L1 UI event loop (mpsc) | 1 | Adopted (A) |
| Conversation bridge (mpsc) | 5 | Compliant (A) |
| Session lifecycle (mpsc) | 1 | Compliant (A) |
| Watch state distribution | 3 | Compliant — state cache, not event broadcast |
| MCP request/response (oneshot) | 5 | Compliant — bounded request/response |
| WASM watchdog (std::sync::mpsc) | 1 | Compliant — function-local, single-consumer |
| Sync crates (no channels) | 0 | N/A — pure sync |
| Dashboard (no channels) | 0 | N/A — HTTP snapshots |

No deviations. No remediation required. `docs/reference/ARCHITECTURE.md` updated with the "Channel Topology Audit (ARCH-032)" section.

## Post-Completion Correction (2026-07-11)

The result above is retained as the published channel-topology audit. It proved ADR-006's absence of
a global broadcast bus, but “No deviations” was too broad for semantic single-data-flow behavior.
ARCH-033 subsequently found independent UI ordering domains, split provider/session lifecycle
authority, multiple persistence writers, and cross-mode divergence. Remediation is owned by
`docs/backlog/active/ARCH-033-runtime-event-semantic-convergence.md`, I115, and ADR-039; this note
does not rewrite the completed ARCH-032 baseline.

### REL-002 Classification
Runtime: glm-5.2 via zai-coding-plan (external, NOT Talos). Per REL-002 criterion 7, this is NON-QUALIFYING evidence. The audit is useful for future Talos-primary sessions, but this session does not prove self-bootstrap capability.
