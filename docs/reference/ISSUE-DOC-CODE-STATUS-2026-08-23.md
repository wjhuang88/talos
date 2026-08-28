# GitHub Issue / Owner Document Status Reconciliation — 2026-08-23

**Repository baseline**: `main@e1c375e6c38394336b2c69fcb6d2e17697fbc2e2`
**Purpose**: refresh the open-Issue owner snapshot after PERM-006-C/Issue #55 closeout and
Issue #59 deferred-validation tracker #378 intake.
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
| [#52](https://github.com/wjhuang88/talos/issues/52) | permission pipeline convergence | [PERM-006](../backlog/active/PERM-006-permission-pipeline-convergence.md) | In Progress | A-C are Complete/Closed; D is Ready/Unclaimed and E is Blocked on D. |
| [#56](https://github.com/wjhuang88/talos/issues/56) | typed effects/resources | [PERM-006-D](../backlog/active/PERM-006-D-typed-effects-and-resources.md) | Ready | C is Complete; D still requires its own iteration and effective claim. |
| [#57](https://github.com/wjhuang88/talos/issues/57) | cross-surface permission conformance | [PERM-006-E](../backlog/active/PERM-006-E-cross-surface-conformance.md) | Blocked | Completion blocked by D. |
| [#111](https://github.com/wjhuang88/talos/issues/111) | hide Calling tools placeholder | [TUI-043](../backlog/active/TUI-043-tool-placeholder-suppression.md) | Review / Claimed | I201 implementation merged as `7f5a6df2`; permission-mediated suppression was corrected by Complete/Closed I229/TUI-058/#329, while I201 retains its separate deferred natural-person acceptance. |
| [#114](https://github.com/wjhuang88/talos/issues/114) | user-only global-memory admission | [MEM-010](../backlog/active/MEM-010-user-origin-memory-admission.md) | Ready P0 | Narrow safety correction; iteration/claim required. |
| [#116](https://github.com/wjhuang88/talos/issues/116) | extensible memory scopes/migration | [MEM-011](../backlog/active/MEM-011-extensible-memory-scopes.md) | Refinement | ADR and migration fixtures required. |
| [#124](https://github.com/wjhuang88/talos/issues/124) | custom-model capability probe | [MODEL-011](../backlog/active/MODEL-011-custom-model-capability-probe.md) | Refinement | Probe decision, evidence precedence, cost UX and persistence schema remain unclaimed. |
| [#125](https://github.com/wjhuang88/talos/issues/125) | permission prompt layout anchor stability | [TUI-045](../backlog/active/TUI-045-permission-prompt-layout-anchor.md) | Review / Claimed | I197 implementation merged as `d98f37e7`; composer-relative docking was corrected by Complete/Closed I230/TUI-059/#330, while I197 retains its separate deferred natural-person acceptance. |
| [#132](https://github.com/wjhuang88/talos/issues/132) | non-API-key provider authentication | [PROVIDER-003](../backlog/active/PROVIDER-003-dynamic-provider-credentials.md) | Refinement Epic | Architecture/decomposition owner only; no child is selected. |
| [#136](https://github.com/wjhuang88/talos/issues/136) | executable recovery commands in delete cleanup failure | [TUI-047](../backlog/active/TUI-047-delete-cleanup-recovery-diagnostics.md) | Ready | Independent non-blocking correction; preserve accepted ADR-056 cleanup semantics. |
| [#141](https://github.com/wjhuang88/talos/issues/141) | storage topology and runtime ownership | [DATA-002](../backlog/active/DATA-002-storage-topology-and-runtime-ownership.md) | Intake | P0 architecture intake; ADR and owner/handoff refinement required before implementation. |
| [#142](https://github.com/wjhuang88/talos/issues/142) | serve/connect protocol adapter architecture | [SERVER-001](../backlog/active/SERVER-001-serve-connect-protocol-adapters.md) | Intake | P1 architecture intake; dependency and single-runtime boundary refinement required. |
| [#143](https://github.com/wjhuang88/talos/issues/143) | RTK-derived semantic shell output filters | [TOOL-025](../backlog/active/TOOL-025-rtk-derived-semantic-output-filters.md) | Intake | P1 source/provenance and behavior-boundary refinement required before extraction. |
| [#146](https://github.com/wjhuang88/talos/issues/146) | optional utility model role and bounded routing | [MODEL-012](../backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md) | Intake | P2 model-role, routing, TUI, compatibility, and evaluation refinement required before implementation. |
| [#155](https://github.com/wjhuang88/talos/issues/155) | SkillLoader rejects `SKILL.md` without triggers | [SKILL-004](../backlog/active/SKILL-004-optional-skill-triggers-compatibility.md) | Review / Claimed | Compatibility behavior passed after PR #325 merged as `15a3d424`, but the malformed-input CLI diagnostic failed and is now owned by Ready/Unclaimed SKILL-005/#333. |
| [#188](https://github.com/wjhuang88/talos/issues/188) | model-assisted cross-surface `auto` permission decisions | [PERM-007](../backlog/active/PERM-007-model-assisted-goal-permission-decisions.md) | In Progress; PERM-007-A/I218 Complete, B-D Blocked | ADR-064 is Accepted; no behavior authority exists before separate child claims. |
| [#199](https://github.com/wjhuang88/talos/issues/199) | shared retry and circuit-breaker policy | [NET-001](../backlog/active/NET-001-network-resilience-policy.md) | Intake | Inventory and ADR-backed decomposition required; no implementation or replay authority. |
| [#234](https://github.com/wjhuang88/talos/issues/234) | single-direct-dependency runtime SDK facade | [RUNTIME-006](../backlog/active/RUNTIME-006-single-dependency-sdk-facade.md) | Refinement | Provider strategy, compatibility treatment, external fixture, iteration and claim required before API implementation. |
| [#245](https://github.com/wjhuang88/talos/issues/245) | I161 independent security review: sandbox fallback and coding preset | [I161](../iterations/I161-sandbox-fallback-and-coding-preset.md) | Blocked | Historical reviewer-assignment record; remote Issue remains open. |
| [#266](https://github.com/wjhuang88/talos/issues/266) | Todo rendering and muted-text readability | [TUI-052](../backlog/active/TUI-052-todo-rendering-muted-readability.md) | Intake | Decompose presentation coalescing and style inventory before selection. |
| [#268](https://github.com/wjhuang88/talos/issues/268) | Numeric permission approval shortcuts | [TUI-053](../backlog/active/TUI-053-numeric-permission-shortcuts.md) | Intake | Resolve TUI-045 overlap and protected permission-surface claim before selection. |
| [#269](https://github.com/wjhuang88/talos/issues/269) | Native key-repeat routing | [TUI-054](../backlog/active/TUI-054-native-key-repeat-routing.md) | Intake | Inventory repeat-safe and one-shot actions before selection. |
| [#278](https://github.com/wjhuang88/talos/issues/278) | Bounded provider retry progress contract | [PROVIDER-006](../backlog/active/PROVIDER-006-bounded-retry-progress-contract.md) | Review / Claimed | Retry ordinals passed; Complete/Closed I231/TUI-060/#332 corrected initial connection and queue status, while I210 retains separate deferred acceptance. |
| [#280](https://github.com/wjhuang88/talos/issues/280) | Narrow Markdown table layout integrity | [TUI-055](../backlog/active/TUI-055-narrow-markdown-table-layout.md) | Intake / Unclaimed | Characterize renderer ownership and choose a deterministic narrow-width strategy before iteration selection. |
| [#285](https://github.com/wjhuang88/talos/issues/285) | Prompt authority architecture and model-behavior harness | [PROMPT-001](../backlog/active/PROMPT-001-prompt-authority-architecture.md) | Refinement / Unclaimed | Accept authority/precedence architecture and decompose children before implementation. |
| [#298](https://github.com/wjhuang88/talos/issues/298) | Collapsible reasoning history | [TUI-056](../backlog/active/TUI-056-collapsible-reasoning-history.md) | Refinement / Unclaimed | Separate completed-history interaction intake. |
| [#308](https://github.com/wjhuang88/talos/issues/308) | Presets as global Session Environment templates | [DESKTOP-002](../backlog/active/DESKTOP-002-preset-session-environment-templates.md) | Blocked / Unclaimed | MODEL-012/#146 and shared Session authority must resolve before decomposition. |
| [#310](https://github.com/wjhuang88/talos/issues/310) | Dynamic line counts for live thinking and tool activity | [TUI-057](../backlog/active/TUI-057-live-activity-status-headers.md) | Refinement / Unclaimed | Refine display-row counting and typed lifecycle projection before selection. |
| [#316](https://github.com/wjhuang88/talos/issues/316) | Isolate process HOME mutations in parallel tests | [TEST-001](../backlog/active/TEST-001-process-home-test-isolation.md) | Ready / Unclaimed | Select an iteration and effective claim before changing test environment ownership. |
| [#317](https://github.com/wjhuang88/talos/issues/317) | Progressive workspace intelligence from Tree-sitter queries | [CODE-003](../backlog/active/CODE-003-tree-sitter-usage-pattern-analysis.md) | Refinement / Unclaimed | Contract characterization is runnable; later children remain unclaimed. |
| [#333](https://github.com/wjhuang88/talos/issues/333) | Invalid Skill diagnostic visibility | [SKILL-005](../backlog/active/SKILL-005-invalid-skill-diagnostic-visibility.md) | Ready / Unclaimed | Corrective owner for malformed Skill diagnostics. |
| [#334](https://github.com/wjhuang88/talos/issues/334) | History continuation padding regression | [TUI-061](../backlog/active/TUI-061-history-continuation-padding-regression.md) | Ready / Unclaimed | Corrective owner for continuation padding. |
| [#360](https://github.com/wjhuang88/talos/issues/360) | server remote relational persistence profile | [SERVER-002](../backlog/active/SERVER-002-remote-relational-persistence-profile.md) | Intake / Unclaimed | Separate refinement/ADR/iteration/claim required. |
| [#361](https://github.com/wjhuang88/talos/issues/361) | standalone `talos-server` host composition | [SERVER-001-C](../backlog/active/SERVER-001-C-standalone-server-host-composition.md) | Intake / Unclaimed | Reuse existing runtime authorities; no implementation before separate governance. |
| [#362](https://github.com/wjhuang88/talos/issues/362) | optional S3-compatible object-storage workspace | [TOOL-027](../backlog/active/TOOL-027-s3-object-workspace-backend.md) | Intake / Unclaimed | Optional object-workspace tools only; no implementation claim. |
| [#390](https://github.com/wjhuang88/talos/issues/390) | architecture(memory): evolve context compaction into model-directed checkpoints with recoverable session history | [MEM-005](../backlog/active/MEM-005-context-compaction-policy.md) | Refinement | Intake reconciliation only; reconcile MEM-002/MEM-003/MEM-007 and select a separately governed implementation slice before production changes. |
| [#395](https://github.com/wjhuang88/talos/issues/395) | architecture(observability): establish R3 structured diagnostics, correlation, and error-fidelity contract | [OBS-002](../backlog/active/OBS-002-structured-diagnostics-contract.md) | Intake / Unclaimed | Intake owner only; architecture/characterization first. Do not activate OBS-002 or authorize implementation from the Issue alone. |

## Synchronization Notes

- Issues #59 and #378 closed after I223/TOOL-024 evidence and owner-first closeout merged as
  `0953f3b1`; they are removed from this open-Issue matrix while historical owner evidence remains.
- Issue #55 closed after I221/PERM-006-C implementation and owner-first closeout reached
  `main@e1c375e6`; its historical evidence remains in the 2026-08-22 snapshot and owner documents.
- Issue #395 is recorded under the unclaimed OBS-002 intake owner only; it has no iteration, claim,
  or implementation authority. Its registration is unrelated to I226 and does not activate
  observability work.
- This matrix is a remote-owner reconciliation surface, not an implementation backlog or activation
  mechanism.
