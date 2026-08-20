# GitHub Issue / Owner Document Status Reconciliation — 2026-08-06

**Repository baseline**: `main@685d3b4f4088a172551f8c844a89f5dee9469430`
**Closeout PR**: #137
**Remote scope**: all 34 open GitHub Issues after Issues #119, #134 and #104 completion, Issue #136
registration, intake registration of Issues #141–#143, #146, #155, #188, and #199, and
RUNTIME-006 registration for Issue #234
**Authority rule**: owner document first, then Product Backlog / Board, then remote Issue.

## Result

- Every open Issue has one explicit owner Story/Epic/Spike document.
- TUI-046/I186 is Complete and Issue #134 is closed after implementation, exact-head two-terminal
  acceptance, CI, independent review, merge-time CAS and governance closeout.
- TUI-044/I169 is Complete, ADR-056 is Accepted and Issue #119 is closed as completed after merged
  implementation PR #131 and accepted exact-head evidence.
- Issue #136 is independently owned by TUI-047 and remains open as a non-blocking Ready diagnostic
  correction; it does not reopen TUI-044/I169 or ADR-056.
- Issue #155 is independently owned by SKILL-004 as unclaimed Intake compatibility work; its
  registration does not expand or block I175.
- Issue #188 is independently owned by PERM-007 as unclaimed Refinement security work; its
  registration does not authorize model-assisted permission decisions or alter PERM-006 ordering.
- Issue #234 is independently owned by RUNTIME-006 as unclaimed Refinement SDK/API work; its
  registration does not expand the v0.8.0 publication scope or authorize API implementation.
- Issue #245 is owned by I161/ARCH-031-C as a reviewer-assignment request; it records the
  independent security-review gate and does not authorize implementation or publication.
- Recovery PRs #120/#121 and their branches remain immutable archival evidence.
- Deferred, Refinement, Ready, Partial and Blocked remain open states; registration does not imply
  selection or implementation authorization.

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
| [#49](https://github.com/wjhuang88/talos/issues/49) | bounded graceful shutdown | [RUNTIME-005](../backlog/active/RUNTIME-005-bounded-graceful-shutdown.md) | Refinement | SESSION-008-A/B then RUNTIME-005-A/B/C; TOOL-024 is a consumer, not a prerequisite. |
| [#52](https://github.com/wjhuang88/talos/issues/52) | permission pipeline convergence | [PERM-006](../backlog/active/PERM-006-permission-pipeline-convergence.md) | Refinement | Epic registered; children staged. |
| [#53](https://github.com/wjhuang88/talos/issues/53) | structured permission decisions | [PERM-006-A](../backlog/active/PERM-006-A-structured-permission-decisions.md) | Refinement | First additive child; unclaimed. |
| [#54](https://github.com/wjhuang88/talos/issues/54) | scoped grant stores | [PERM-006-B](../backlog/active/PERM-006-B-scoped-grant-store.md) | Blocked | Blocked by PERM-006-A. |
| [#55](https://github.com/wjhuang88/talos/issues/55) | agent-owned permission pipeline | [PERM-006-C](../backlog/active/PERM-006-C-agent-owned-permission-pipeline.md) | Blocked | Blocked by PERM-006-A/B. |
| [#56](https://github.com/wjhuang88/talos/issues/56) | typed effects/resources | [PERM-006-D](../backlog/active/PERM-006-D-typed-effects-and-resources.md) | Blocked | Blocked by PERM-006-C and ADR migration. |
| [#57](https://github.com/wjhuang88/talos/issues/57) | cross-surface permission conformance | [PERM-006-E](../backlog/active/PERM-006-E-cross-surface-conformance.md) | Blocked | Completion blocked by C/D. |
| [#59](https://github.com/wjhuang88/talos/issues/59) | supervised background command jobs | [TOOL-024](../backlog/active/TOOL-024-background-command-jobs.md) | Refinement | A/I188 is Complete and ADR-060 Accepted at `245eddeb`; B waits for RUNTIME-005/PERM-006-C, C owns `process` controls, and D owns cross-platform acceptance. |
| [#79](https://github.com/wjhuang88/talos/issues/79) | no-op mouse scroll layout shift | [TUI-042](../backlog/active/TUI-042-noop-history-scroll-stability.md) | Complete / Closed | Completion Commit `3afeeb28`; PR #301 merge gates and the maintainer-approved touchpad substitute passed. Close the remote Issue only after owner closeout reaches main. |
| [#111](https://github.com/wjhuang88/talos/issues/111) | hide Calling tools placeholder | [TUI-043](../backlog/active/TUI-043-tool-placeholder-suppression.md) | Review / Claimed | I201 implementation PR #309 merged as `7f5a6df2`; human suppression-safety validation remains in Issue #302/I211. |
| [#114](https://github.com/wjhuang88/talos/issues/114) | user-only global-memory admission | [MEM-010](../backlog/active/MEM-010-user-origin-memory-admission.md) | Ready P0 | Narrow safety correction; iteration/claim required. |
| [#116](https://github.com/wjhuang88/talos/issues/116) | extensible memory scopes/migration | [MEM-011](../backlog/active/MEM-011-extensible-memory-scopes.md) | Refinement | ADR and migration fixtures required. |
| [#124](https://github.com/wjhuang88/talos/issues/124) | custom-model capability probe | [MODEL-011](../backlog/active/MODEL-011-custom-model-capability-probe.md) | Refinement | Probe decision, evidence precedence, cost UX and persistence schema remain unclaimed. |
| [#125](https://github.com/wjhuang88/talos/issues/125) | permission prompt layout anchor stability | [TUI-045](../backlog/active/TUI-045-permission-prompt-layout-anchor.md) | Refinement | Layout ownership and real-terminal acceptance remain unclaimed. |
| [#132](https://github.com/wjhuang88/talos/issues/132) | non-API-key provider authentication | [PROVIDER-003](../backlog/active/PROVIDER-003-dynamic-provider-credentials.md) | Refinement Epic | Architecture/decomposition owner only; no child is selected. |
| [#136](https://github.com/wjhuang88/talos/issues/136) | executable recovery commands in delete cleanup failure | [TUI-047](../backlog/active/TUI-047-delete-cleanup-recovery-diagnostics.md) | Ready | Independent non-blocking correction; preserve accepted ADR-056 cleanup semantics. |
| [#141](https://github.com/wjhuang88/talos/issues/141) | storage topology and runtime ownership | [DATA-002](../backlog/active/DATA-002-storage-topology-and-runtime-ownership.md) | Intake | P0 architecture intake; ADR and owner/handoff refinement required before implementation. |
| [#142](https://github.com/wjhuang88/talos/issues/142) | serve/connect protocol adapter architecture | [SERVER-001](../backlog/active/SERVER-001-serve-connect-protocol-adapters.md) | Intake | P1 architecture intake; dependency and single-runtime boundary refinement required. |
| [#143](https://github.com/wjhuang88/talos/issues/143) | RTK-derived semantic shell output filters | [TOOL-025](../backlog/active/TOOL-025-rtk-derived-semantic-output-filters.md) | Intake | P1 source/provenance and behavior-boundary refinement required before extraction. |
| [#146](https://github.com/wjhuang88/talos/issues/146) | optional utility model role and bounded routing | [MODEL-012](../backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md) | Intake | P2 model-role, routing, TUI, compatibility, and evaluation refinement required before implementation. |
| [#155](https://github.com/wjhuang88/talos/issues/155) | SkillLoader rejects `SKILL.md` without triggers | [SKILL-004](../backlog/active/SKILL-004-optional-skill-triggers-compatibility.md) | Review / Claimed | PR #325 final head `b2d5adaf` passed exact-head CI/review/CAS and merged as `15a3d424`; Issue #302 human validation remains open. |
| [#188](https://github.com/wjhuang88/talos/issues/188) | model-assisted cross-surface `auto` permission decisions | [PERM-007](../backlog/active/PERM-007-model-assisted-goal-permission-decisions.md) | Refinement | ADR-011 revision/supersession, threat model, PERM-006 dependencies and bounded child decomposition required before any implementation claim. Default-on is a proposed target, not an accepted policy. |
| [#199](https://github.com/wjhuang88/talos/issues/199) | shared retry and circuit-breaker policy | [NET-001](../backlog/active/NET-001-network-resilience-policy.md) | Intake | Inventory and ADR-backed decomposition required; no implementation or replay authority. |
| [#234](https://github.com/wjhuang88/talos/issues/234) | single-direct-dependency runtime SDK facade | [RUNTIME-006](../backlog/active/RUNTIME-006-single-dependency-sdk-facade.md) | Refinement | Provider strategy, compatibility treatment, external fixture, iteration and claim required before API implementation. |
| [#245](https://github.com/wjhuang88/talos/issues/245) | I161 independent security review: sandbox fallback and coding preset | [I161](../iterations/I161-sandbox-fallback-and-coding-preset.md) | Blocked | ARCH-031-C reviewer-assignment request bound to `main@b570ac27`; no implementation, release, tag, GitHub Release, or Cargo publication authorization. |

## Closed In This Reconciliation

| Issue | Owner | Final State | Completion Evidence |
|---|---|---|---|
| [#119](https://github.com/wjhuang88/talos/issues/119) | TUI-044 / I169 / ADR-056 | Completed | PR #131 merged exact Head `90165cace4625c0f27616b3e1b9871bcb6a10186` at `685d3b4f4088a172551f8c844a89f5dee9469430`; CI `31010166558` and rebuilt real-terminal acceptance passed. |
| [#134](https://github.com/wjhuang88/talos/issues/134) | TUI-046 / I184 / I186 | Completed | Completion Commits `f98488277803ee26180100089a48ef850939234b` and `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc`; final PR #193 head `313e47e5` passed CI `31481069023`, independent review `4905391760` and merge-time CAS; exact runtime head `70b51e28` passed both terminal acceptance rows. |
| [#104](https://github.com/wjhuang88/talos/issues/104) | TUI-037 / I202 | Completed | Implementation Completion Commit `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772`; PR #230 merged as `e0cc782a475c2e5baceb31f2a125f1e268af7ecf` after exact-head CI `31775126382`, independent approval `5290402214`, terminal acceptance and merge-time CAS; closeout PR #231 merged as `1c4e29292ffcbc9e53a9dfaeed125ab5c5697e9f`. SEC-002 owns the separate token-delivery residual. |
| [#272](https://github.com/wjhuang88/talos/issues/272) | TUI-051 / I209 | Completed | Completion Commit `2eff6285688a7ecc8842197c09fb81d89678d728`, source implementation `7b82fea6`/`7d90def8`, CI `32025371877`, real-terminal evidence and independent agent audit `5316533941`; owner-first closeout PR #281 merged as `57a068eb3f29ac9532b070652b2a10d402f0123a`. Retry progress remains separately owned by #278. |
| [#69](https://github.com/wjhuang88/talos/issues/69) | TUI-041 / I199 | Completed | Completion Commits `938c9edb`/`558b76d3`/`14bf4e60`/`add84074`/`de24bffd`; implementation PR #297 merged as `5fc814b5` after exact-head CI `32138003207`, maintainer terminal acceptance `5328282375`, independent review `5328531254` and merge-time CAS; owner-first closeout PR #299 merged as `4acb896e`. TUI-056/#298 remains a separate unclaimed residual. |

## Closure Rule

An Issue may be closed only after its owner is Complete with implementation/acceptance evidence and
no unresolved residual remains inside that owner. A separately owned follow-up does not keep the
completed source Issue open when the boundary and disposition are explicit.

## 2026-08-15 Current Checkpoint: Issue #245

Issue #245 is now the formal pre-implementation security-review record for I161 / ARCH-031-C,
accepted by the maintainer on 2026-08-15. The complete ARCH-031-C security chapters and nine-row
Security Test Matrix are normative. The recorded review requires permission `Deny` precedence,
fail-closed headless `Ask`, scoped and typed fallback approval that cannot be widened by ordinary
`AlwaysApprove`, non-bypass behavior for `AllowUnsandboxed`, coding-preset security equivalence,
policy neutrality in `talos-sandbox`, and path/network/execute variants. The reviewer role is
separate from implementation, with shared-account identity limits disclosed.

This checkpoint supersedes the earlier reviewer-assignment disposition for current status only;
the historical matrix and its original dated row remain unchanged. It does not mark I161 complete,
authorize a merge, or authorize release/tag/GitHub Release/Cargo publication. A finalized I161
implementation head still requires independent exact-head security approval against the complete
normative matrix before merge.

## 2026-08-17 Open Issue Intake Addendum

This append-only addendum registers Issues opened after the published matrix. Registration is not
selection, activation or implementation authority.

| Issue | Summary | Owner | Owner Status | Disposition |
|---|---|---|---|---|
| [#266](https://github.com/wjhuang88/talos/issues/266) | Todo rendering and muted-text readability | [TUI-052](../backlog/active/TUI-052-todo-rendering-muted-readability.md) | Intake | Decompose presentation coalescing and style inventory before selection. |
| [#267](https://github.com/wjhuang88/talos/issues/267) | Steering follow-up sequence | [TUI-048](../backlog/active/TUI-048-steering-esc-activation.md) | Planned | I206-I208 remain separately Planned/Unclaimed; no implementation authority. |
| [#268](https://github.com/wjhuang88/talos/issues/268) | Numeric permission approval shortcuts | [TUI-053](../backlog/active/TUI-053-numeric-permission-shortcuts.md) | Intake | Resolve TUI-045 overlap and protected permission-surface claim before selection. |
| [#269](https://github.com/wjhuang88/talos/issues/269) | Native key-repeat routing | [TUI-054](../backlog/active/TUI-054-native-key-repeat-routing.md) | Intake | Inventory repeat-safe and one-shot actions before selection. |
| [#278](https://github.com/wjhuang88/talos/issues/278) | Bounded provider retry progress contract | [PROVIDER-006](../backlog/active/PROVIDER-006-bounded-retry-progress-contract.md) | Review / Claimed | ADR-062 accepted; PR #323 final head `c984ec48` passed exact-head CI/review/CAS and merged as `9d5c8a71`. Issue #302 human validation and no-release boundary remain open. |
| [#280](https://github.com/wjhuang88/talos/issues/280) | Narrow Markdown table layout integrity | [TUI-055](../backlog/active/TUI-055-narrow-markdown-table-layout.md) | Intake / Unclaimed | Characterize renderer ownership and choose a deterministic narrow-width strategy before iteration selection; no I209 implementation authority. |
| [#285](https://github.com/wjhuang88/talos/issues/285) | Prompt authority architecture and model-behavior harness | [PROMPT-001](../backlog/active/PROMPT-001-prompt-authority-architecture.md) | Refinement / Unclaimed | Accept the authority/precedence architecture and decompose bounded decision, harness and migration children before implementation. |
| [#298](https://github.com/wjhuang88/talos/issues/298) | Collapsible reasoning history | [TUI-056](../backlog/active/TUI-056-collapsible-reasoning-history.md) | Refinement / Unclaimed | Separate completed-history interaction intake; I199 remains limited to the live transient preview. |
| [#302](https://github.com/wjhuang88/talos/issues/302) | Deferred human review and terminal acceptance batch | [VALIDATION-002](../backlog/active/VALIDATION-002-deferred-human-validation-batch.md) | Review / I211 Claimed | Every row passed or has a separate corrective owner. I200 and I212 are Complete; four source iterations remain Review. PR #331 and evidence-bound closeout remain. |
| [#308](https://github.com/wjhuang88/talos/issues/308) | Presets as global Session Environment templates | [DESKTOP-002](../backlog/active/DESKTOP-002-preset-session-environment-templates.md) | Blocked / Unclaimed | MODEL-012/#146 and shared Session authority must resolve before child decomposition; no Desktop or runtime implementation authority. |
| [#310](https://github.com/wjhuang88/talos/issues/310) | Dynamic line counts for live thinking and tool activity | [TUI-057](../backlog/active/TUI-057-live-activity-status-headers.md) | Refinement / Unclaimed | Refine display-row counting and typed tool lifecycle projection before a separate iteration or claim; no I201 implementation authority. |
| [#312](https://github.com/wjhuang88/talos/issues/312) | Infer custom-model context window from catalog metadata | [MODEL-013](../backlog/active/MODEL-013-catalog-context-window-inference.md) | Complete / Closed | Completion Commit `5a1709cb`; PR #318 merge gates and integrated exact/prefix/override/unknown no-request walkthrough passed. Close remote Issue only after this owner closeout reaches main. |
| [#316](https://github.com/wjhuang88/talos/issues/316) | Isolate process HOME mutations in parallel tests | [TEST-001](../backlog/active/TEST-001-process-home-test-isolation.md) | Ready / Unclaimed | Test-infrastructure-only Story; select an iteration and effective claim before changing test environment ownership. |
| [#317](https://github.com/wjhuang88/talos/issues/317) | Progressive workspace intelligence from Tree-sitter queries | [CODE-003](../backlog/active/CODE-003-tree-sitter-usage-pattern-analysis.md) | Refinement / Unclaimed | Reuses the existing CODE-003 Epic; linked child CODE-003-A is Ready/Unclaimed. Only contract characterization is runnable; query, index, resolver, storage and integration children remain unclaimed. |
| [#329](https://github.com/wjhuang88/talos/issues/329) | Permission-mediated marker and unnamed outcome correlation | [TUI-058](../backlog/active/TUI-058-permission-mediated-tool-activity-correlation.md) | Ready / Unclaimed | I211 corrective owner; separate iteration and effective claim required before implementation. |
| [#330](https://github.com/wjhuang88/talos/issues/330) | Permission prompt composer-relative docking | [TUI-059](../backlog/active/TUI-059-permission-prompt-composer-docking.md) | Ready / Unclaimed | I211 corrective owner for new-session physical-bottom placement and incomplete terminal matrix. |
| [#332](https://github.com/wjhuang88/talos/issues/332) | Initial connection and queue status sequencing | [TUI-060](../backlog/active/TUI-060-initial-connection-and-queue-status-sequencing.md) | Ready / Unclaimed | I211 corrective owner for fleeting `Connecting...` and the false idle-first-turn queue hint. |
| [#333](https://github.com/wjhuang88/talos/issues/333) | Invalid Skill diagnostic visibility | [SKILL-005](../backlog/active/SKILL-005-invalid-skill-diagnostic-visibility.md) | Ready / Unclaimed | I211 corrective owner for malformed local Skill activation losing its actionable `triggers` diagnostic. |
| [#334](https://github.com/wjhuang88/talos/issues/334) | History continuation padding regression | [TUI-061](../backlog/active/TUI-061-history-continuation-padding-regression.md) | Ready / Unclaimed | I211 corrective owner for ordinary wrapped history losing the documented blank three-column prefix. |
