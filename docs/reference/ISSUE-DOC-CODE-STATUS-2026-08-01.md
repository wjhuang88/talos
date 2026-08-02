# GitHub Issue / Owner Document Status Reconciliation — 2026-08-01

**Original repository baseline**: `main@455bfbd5c5316862675aa68c62f1b62bff2e5cc7`
**Post-I170 reconciliation baseline**: `main@592254d73a98166df48da0139a02df67e9cd2cd6`
**Remote scope**: all 27 open GitHub Issues observed through 2026-08-02
**Authority rule**: owner document first, then Product Backlog / Board, then remote Issue.

## Result

- Every open Issue has one explicit owner Story/Epic/Spike document.
- No open Issue is closed by this reconciliation: none has a Complete owner that also matches the remaining remote scope.
- Deferred, Refinement, Ready, Partial, and Blocked remain open states; “registered” does not mean “scheduled”.
- Recovered Issue #119 is assigned to TUI-044 because current main already assigns TUI-041 to Issue #69.
- I170 completed through PR #126 and clears Issue #119's Windows/current-main prerequisite, but TUI-044/I169 remains open in Draft PR #131.
- New Intake Issues #124 and #125 are registered as unclaimed Refinement owners MODEL-011 and TUI-045.
- Issue #132 is assigned to existing PROVIDER-003 because that Story already owns dynamic provider credentials; this synchronization does not authorize provider-auth implementation in I169.
- REL-001 and DATA-001 historical owner drift is corrected to Complete; they are not currently open GitHub Issues.

## Open Issue Matrix

| Issue | Summary | Owner | Owner Status | Disposition |
|---|---|---|---|---|
| [#22](https://github.com/wjhuang88/talos/issues/22) | feat: 设计权限沙箱模型 | [PERM-004](../backlog/active/PERM-004-workspace-trust-sandbox.md) | Partial | Keep open: broader sandbox/security residuals. |
| [#29](https://github.com/wjhuang88/talos/issues/29) | proposal: 构建 talos-desktop | [DESKTOP-001](../backlog/active/DESKTOP-001-desktop-product-direction.md) | Deferred | Directional proposal; no implementation authorization. |
| [#30](https://github.com/wjhuang88/talos/issues/30) | proposal: 多 Agent 体系架构设计思考 | [AGENT-003](../backlog/active/AGENT-003-multi-agent-architecture.md) | Deferred | Architecture decision required. |
| [#32](https://github.com/wjhuang88/talos/issues/32) | proposal: 独立健康检查线程 | [RUNTIME-004](../backlog/active/RUNTIME-004-session-health-monitoring.md) | Refinement | Recovery authority and false-positive boundaries unresolved. |
| [#38](https://github.com/wjhuang88/talos/issues/38) | feat: 长程任务执行引擎 | [TASK-001](../backlog/active/TASK-001-persistent-task-runtime-spike.md) | Deferred via ADR-043 | Decision spike retained; no runtime authorization. |
| [#40](https://github.com/wjhuang88/talos/issues/40) | 多 talos 实例自动发现和通信 | [A2A-001](../backlog/active/A2A-001-multi-instance-discovery-spike.md) | Deferred via ADR-044 | Trust/identity/transport decision required. |
| [#45](https://github.com/wjhuang88/talos/issues/45) | persist interrupted-turn partial results | [SESSION-008](../backlog/active/SESSION-008-interrupted-turn-partial-persistence.md) | Refinement | ADR, durable shape, and iteration required. |
| [#46](https://github.com/wjhuang88/talos/issues/46) | multi-client session architecture | [SESSION-009](../backlog/active/SESSION-009-multi-client-session-architecture.md) | Refinement | New owner; architecture boundary required. |
| [#47](https://github.com/wjhuang88/talos/issues/47) | ACP-compatible Agent server | [ACP-001](../backlog/active/ACP-001-agent-client-protocol-server.md) | Blocked | Blocked by SESSION-009. |
| [#49](https://github.com/wjhuang88/talos/issues/49) | bounded graceful shutdown | [RUNTIME-005](../backlog/active/RUNTIME-005-bounded-graceful-shutdown.md) | Refinement | Lifecycle/finalizer contract required. |
| [#52](https://github.com/wjhuang88/talos/issues/52) | permission pipeline convergence | [PERM-006](../backlog/active/PERM-006-permission-pipeline-convergence.md) | Refinement | Epic registered; children staged. |
| [#53](https://github.com/wjhuang88/talos/issues/53) | structured permission decisions | [PERM-006-A](../backlog/active/PERM-006-A-structured-permission-decisions.md) | Refinement | First additive child; unclaimed. |
| [#54](https://github.com/wjhuang88/talos/issues/54) | scoped grant stores | [PERM-006-B](../backlog/active/PERM-006-B-scoped-grant-store.md) | Blocked | Blocked by PERM-006-A. |
| [#55](https://github.com/wjhuang88/talos/issues/55) | agent-owned permission pipeline | [PERM-006-C](../backlog/active/PERM-006-C-agent-owned-permission-pipeline.md) | Blocked | Blocked by PERM-006-A/B. |
| [#56](https://github.com/wjhuang88/talos/issues/56) | typed effects/resources | [PERM-006-D](../backlog/active/PERM-006-D-typed-effects-and-resources.md) | Blocked | Blocked by PERM-006-C and ADR migration. |
| [#57](https://github.com/wjhuang88/talos/issues/57) | cross-surface permission conformance | [PERM-006-E](../backlog/active/PERM-006-E-cross-surface-conformance.md) | Blocked | Completion blocked by C/D. |
| [#59](https://github.com/wjhuang88/talos/issues/59) | supervised background command jobs | [TOOL-024](../backlog/active/TOOL-024-background-command-jobs.md) | Refinement | Issue linked to existing Epic; children remain gated. |
| [#69](https://github.com/wjhuang88/talos/issues/69) | thinking preview wrapping/height | [TUI-041](../backlog/active/TUI-041-thinking-preview-wrap-and-height.md) | Refinement | New owner; layout and PTY gates required. |
| [#79](https://github.com/wjhuang88/talos/issues/79) | no-op mouse scroll layout shift | [TUI-042](../backlog/active/TUI-042-noop-history-scroll-stability.md) | Refinement | New owner; state-transition regressions required. |
| [#104](https://github.com/wjhuang88/talos/issues/104) | Dashboard link in Logo region | [TUI-037](../backlog/active/TUI-037-dashboard-logo-link.md) | Refinement P1 | First post-I158 disposition; design gates unresolved. |
| [#111](https://github.com/wjhuang88/talos/issues/111) | hide Calling tools placeholder | [TUI-043](../backlog/active/TUI-043-tool-placeholder-suppression.md) | Ready | Bounded fix; iteration/claim still required. |
| [#114](https://github.com/wjhuang88/talos/issues/114) | user-only global-memory admission | [MEM-010](../backlog/active/MEM-010-user-origin-memory-admission.md) | Ready P0 | Narrow safety correction; iteration/claim required. |
| [#116](https://github.com/wjhuang88/talos/issues/116) | extensible memory scopes/migration | [MEM-011](../backlog/active/MEM-011-extensible-memory-scopes.md) | Refinement | ADR and migration fixtures required. |
| [#119](https://github.com/wjhuang88/talos/issues/119) | transactional batched steering recovery | [TUI-044](../backlog/active/TUI-044-transactional-batched-steering-turn.md) | Active | Draft PR #131 implements I169; keep open until complete acceptance, ADR-056 review, exact-head CI and merge evidence. |
| [#124](https://github.com/wjhuang88/talos/issues/124) | custom-model capability probe | [MODEL-011](../backlog/active/MODEL-011-custom-model-capability-probe.md) | Refinement | Intake registered; probe decision, evidence precedence, cost UX and persistence schema remain unclaimed. |
| [#125](https://github.com/wjhuang88/talos/issues/125) | permission prompt layout anchor stability | [TUI-045](../backlog/active/TUI-045-permission-prompt-layout-anchor.md) | Refinement | Intake registered; layout ownership and real-terminal acceptance remain unclaimed. |
| [#132](https://github.com/wjhuang88/talos/issues/132) | non-API-key provider authentication | [PROVIDER-003](../backlog/active/PROVIDER-003-dynamic-provider-credentials.md) | Refinement | Existing dynamic-credential owner broadened; ADR, threat model and bounded provider-specific slices required before implementation. |

## Status Corrections

- `PERM-004`: corrected from Complete to Partial; Issue #22 explicitly remains open for residual sandbox/security scope.
- `REL-001`: corrected from Planned to Complete with v0.1.2 completion commit `89bbcbbf221cc383bd974d24837013a9bc5f3c33`.
- `DATA-001`: corrected from Active/Deferred to Complete with I049/I053 commits `20f9b3e63b482b81b0639b916bae0d58e131c13a` and `e745e2c906737403a5af8e6238e353cc00993c99`.
- `TOOL-024`: linked explicitly to Issue #59 and to RUNTIME-005/PERM-006 lifecycle and permission prerequisites.
- `TUI-044`: added for recovered Issue #119; historical TUI-041 steering ownership is not restored because TUI-041 currently belongs to Issue #69.
- `I170`: completed in merged PR #126; this clears only TUI-044's prerequisite and does not complete or close Issue #119.
- `MODEL-011` and `TUI-045`: registered from Intake Issues #124/#125 as unclaimed Refinement owners only.
- `PROVIDER-003`: linked to Issue #132 and broadened from the Copilot driving example to the architecture owner for dynamic provider authentication; remains Refinement and unselected.

## Closure Rule

An Issue may be closed only after its owner is Complete with implementation/acceptance evidence and the remote Issue has no separately owned residual. This audit intentionally leaves all 27 observed Issues open.
