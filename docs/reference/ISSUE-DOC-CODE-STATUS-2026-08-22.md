# GitHub Issue / Owner Document Status Reconciliation — 2026-08-22

**Repository baseline**: `main@781bb1122d2c323854d5d65aed354d35d045e383`
**Purpose**: refresh the latest open-Issue owner snapshot after server/S3 intake Issues #360–#362.
**Authority rule**: owner document first, then Product Backlog / Board, then remote Issue.

This is a synchronization snapshot only. Registering an Issue here does not select an iteration,
create a Collaboration Claim, or authorize implementation. Historical closed-Issue evidence remains
in earlier reconciliation snapshots.

## Open Issue Matrix

| Issue | Summary | Owner | Owner Status | Disposition |
|---|---|---|---|---|
| [#22](https://github.com/wjhuang88/talos/issues/22) | feat: 设计权限沙箱模型 | [PERM-004](../backlog/active/PERM-004-workspace-trust-sandbox.md) | Partial | Keep open: broader sandbox/security residuals. |
| [#29](https://github.com/wjhuang88/talos/issues/29) | proposal: 构建 talos-desktop | [DESKTOP-001](../backlog/active/DESKTOP-001-desktop-product-direction.md) | Deferred | Directional proposal; no implementation authorization. |
| [#30](https://github.com/wjhuang88/talos/issues/30) | proposal: 多 Agent 体系架构设计思考 | [AGENT-003](../backlog/active/AGENT-003-multi-agent-architecture.md) | Deferred | Architecture decision required. |
| [#32](https://github.com/wjhuang88/talos/issues/32) | proposal: 独立健康检查线程 | [RUNTIME-004](../backlog/active/RUNTIME-004-session-health-monitoring.md) | Refinement | Recovery authority and false-positive boundaries unresolved. |
| [#38](https://github.com/wjhuang88/talos/issues/38) | feat: 长程任务执行引擎 | [TASK-001](../backlog/active/TASK-001-persistent-task-runtime-spike.md) | Deferred via ADR-043 | Decision spike retained; no runtime authorization. |
| [#40](https://github.com/wjhuang88/talos/issues/40) | 多 talos 实例自动发现和通信 | [A2A-001](../backlog/active/A2A-001-multi-instance-discovery-spike.md) | Deferred via ADR-044 | Trust/identity/transport decision required. |
| [#45](https://github.com/wjhuang88/talos/issues/45) | persist interrupted-turn partial results | [SESSION-008](../backlog/active/SESSION-008-interrupted-turn-partial-persistence.md) | Refinement | Legacy error path is partial; A decides durable shape and B implements cancellation/error replay parity. |
| [#46](https://github.com/wjhuang88/talos/issues/46) | multi-client session architecture | [SESSION-009](../backlog/active/SESSION-009-multi-client-session-architecture.md) | Refinement | Architecture boundary required. |
| [#47](https://github.com/wjhuang88/talos/issues/47) | ACP-compatible Agent server | [ACP-001](../backlog/active/ACP-001-agent-client-protocol-server.md) | Blocked | Blocked by SESSION-009. |
| [#52](https://github.com/wjhuang88/talos/issues/52) | permission pipeline convergence | [PERM-006](../backlog/active/PERM-006-permission-pipeline-convergence.md) | In Progress | PERM-006-A/I189 is Complete/Closed; B is Ready/Unclaimed and C-E remain blocked in order. |
| [#54](https://github.com/wjhuang88/talos/issues/54) | scoped grant stores | [PERM-006-B](../backlog/active/PERM-006-B-scoped-grant-store.md) | Ready / Unclaimed | PERM-006-A is complete and ADR-066 is Accepted; a separate runnable iteration and effective protected-scope claim still gate implementation. |
| [#55](https://github.com/wjhuang88/talos/issues/55) | agent-owned permission pipeline | [PERM-006-C](../backlog/active/PERM-006-C-agent-owned-permission-pipeline.md) | Blocked | Blocked by PERM-006-A/B. |
| [#56](https://github.com/wjhuang88/talos/issues/56) | typed effects/resources | [PERM-006-D](../backlog/active/PERM-006-D-typed-effects-and-resources.md) | Blocked | Blocked by PERM-006-C and ADR migration. |
| [#57](https://github.com/wjhuang88/talos/issues/57) | cross-surface permission conformance | [PERM-006-E](../backlog/active/PERM-006-E-cross-surface-conformance.md) | Blocked | Completion blocked by C/D. |
| [#59](https://github.com/wjhuang88/talos/issues/59) | supervised background command jobs | [TOOL-024](../backlog/active/TOOL-024-background-command-jobs.md) | Refinement | A/I188 and RUNTIME-005 are Complete; B still waits for PERM-006-C and separate effective authority, C owns `process` controls, and D owns cross-platform acceptance. |
| [#111](https://github.com/wjhuang88/talos/issues/111) | hide Calling tools placeholder | [TUI-043](../backlog/active/TUI-043-tool-placeholder-suppression.md) | Review / Claimed | I201 implementation merged as `7f5a6df2`; permission-mediated suppression failed terminal validation and is now owned by Ready/Unclaimed TUI-058/#329. |
| [#114](https://github.com/wjhuang88/talos/issues/114) | user-only global-memory admission | [MEM-010](../backlog/active/MEM-010-user-origin-memory-admission.md) | Ready P0 | Narrow safety correction; iteration/claim required. |
| [#116](https://github.com/wjhuang88/talos/issues/116) | extensible memory scopes/migration | [MEM-011](../backlog/active/MEM-011-extensible-memory-scopes.md) | Refinement | ADR and migration fixtures required. |
| [#124](https://github.com/wjhuang88/talos/issues/124) | custom-model capability probe | [MODEL-011](../backlog/active/MODEL-011-custom-model-capability-probe.md) | Refinement | Probe decision, evidence precedence, cost UX and persistence schema remain unclaimed. |
| [#125](https://github.com/wjhuang88/talos/issues/125) | permission prompt layout anchor stability | [TUI-045](../backlog/active/TUI-045-permission-prompt-layout-anchor.md) | Review / Claimed | I197 implementation merged as `d98f37e7`; terminal layout/docking validation failed or remained incomplete and is now owned by Ready/Unclaimed TUI-059/#330. |
| [#132](https://github.com/wjhuang88/talos/issues/132) | non-API-key provider authentication | [PROVIDER-003](../backlog/active/PROVIDER-003-dynamic-provider-credentials.md) | Refinement Epic | Architecture/decomposition owner only; no child is selected. |
| [#136](https://github.com/wjhuang88/talos/issues/136) | executable recovery commands in delete cleanup failure | [TUI-047](../backlog/active/TUI-047-delete-cleanup-recovery-diagnostics.md) | Ready | Independent non-blocking correction; preserve accepted ADR-056 cleanup semantics. |
| [#141](https://github.com/wjhuang88/talos/issues/141) | storage topology and runtime ownership | [DATA-002](../backlog/active/DATA-002-storage-topology-and-runtime-ownership.md) | Intake | P0 architecture intake; ADR and owner/handoff refinement required before implementation. |
| [#142](https://github.com/wjhuang88/talos/issues/142) | serve/connect protocol adapter architecture | [SERVER-001](../backlog/active/SERVER-001-serve-connect-protocol-adapters.md) | Intake | P1 architecture intake; dependency and single-runtime boundary refinement required. |
| [#143](https://github.com/wjhuang88/talos/issues/143) | RTK-derived semantic shell output filters | [TOOL-025](../backlog/active/TOOL-025-rtk-derived-semantic-output-filters.md) | Intake | P1 source/provenance and behavior-boundary refinement required before extraction. |
| [#146](https://github.com/wjhuang88/talos/issues/146) | optional utility model role and bounded routing | [MODEL-012](../backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md) | Intake | P2 model-role, routing, TUI, compatibility, and evaluation refinement required before implementation. |
| [#155](https://github.com/wjhuang88/talos/issues/155) | SkillLoader rejects `SKILL.md` without triggers | [SKILL-004](../backlog/active/SKILL-004-optional-skill-triggers-compatibility.md) | Review / Claimed | Compatibility behavior passed after PR #325 merged as `15a3d424`, but the malformed-input CLI diagnostic failed and is now owned by Ready/Unclaimed SKILL-005/#333. |
| [#188](https://github.com/wjhuang88/talos/issues/188) | model-assisted cross-surface `auto` permission decisions | [PERM-007](../backlog/active/PERM-007-model-assisted-goal-permission-decisions.md) | In Progress; PERM-007-A/I218 Complete, B-D Blocked | [PERM-007-A](../backlog/active/PERM-007-A-auto-permission-security-decision.md) / [I218](../iterations/I218-perm007a-auto-permission-security-decision.md) closed at Completion Commit `a289a07f`; ADR-064 Accepted. No behavior authority exists before PERM-006-A/B/C and separate child claims. |
| [#199](https://github.com/wjhuang88/talos/issues/199) | shared retry and circuit-breaker policy | [NET-001](../backlog/active/NET-001-network-resilience-policy.md) | Intake | Inventory and ADR-backed decomposition required; no implementation or replay authority. |
| [#234](https://github.com/wjhuang88/talos/issues/234) | single-direct-dependency runtime SDK facade | [RUNTIME-006](../backlog/active/RUNTIME-006-single-dependency-sdk-facade.md) | Refinement | Provider strategy, compatibility treatment, external fixture, iteration and claim required before API implementation. |
| [#245](https://github.com/wjhuang88/talos/issues/245) | I161 independent security review: sandbox fallback and coding preset | [I161](../iterations/I161-sandbox-fallback-and-coding-preset.md) | Blocked | ARCH-031-C reviewer-assignment request bound to `main@b570ac27`; no implementation, release, tag, GitHub Release, or Cargo publication authorization. |
| [#360](https://github.com/wjhuang88/talos/issues/360) | server remote relational persistence profile | [SERVER-002](../backlog/active/SERVER-002-remote-relational-persistence-profile.md) | Intake / Unclaimed | New server persistence intake; separate refinement/ADR/iteration/claim required before database or schema work. |
| [#361](https://github.com/wjhuang88/talos/issues/361) | standalone `talos-server` host composition | [SERVER-001-C](../backlog/active/SERVER-001-C-standalone-server-host-composition.md) | Intake / Unclaimed | Reuse existing Talos runtime authorities; no production server/API implementation before separate governance. |
| [#362](https://github.com/wjhuang88/talos/issues/362) | optional S3-compatible object-storage workspace | [TOOL-027](../backlog/active/TOOL-027-s3-object-workspace-backend.md) | Intake / Unclaimed | Optional object-workspace tools only; no local-filesystem emulation, remote SQL authority, or implementation claim. |

## Synchronization Notes

- #360–#362 are registered as separate Intake / Unclaimed owners so SERVER-001 composition does not
  duplicate persistence or workspace-tool logic.
- Existing rows are copied from the previous latest reconciliation snapshot to preserve their remote
  synchronization status. Owner lifecycle truth remains in the owner documents themselves.
- This matrix is a remote-owner reconciliation surface, not an implementation backlog or activation
  mechanism.
