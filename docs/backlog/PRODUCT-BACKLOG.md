# Product Backlog

This file is the compact current-state backlog entrypoint. Story scope, acceptance criteria,
implementation evidence, and residual ownership live in the linked owner documents.

The complete pre-closeout index is preserved unchanged at
[`PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md`](PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md).
That snapshot is historical evidence, not current activation authority.

## Current Priorities

| Priority | Focus | Current State / Gate | Required Reads |
|---|---|---|---|
| 0 | v0.8.0 GitHub-first crates publication | Complete / Closed | I203 Completion Commits `b0354ae6`/`d8e1aa26`/`d5de4a65`; PR #264 merged as `f425e7bc`. Tag/Release workflow `31953951828` completed with five archives plus checksum before all 20 crates published. External install returned `talos 0.8.0`; registry-only runtime fixture passed. | `docs/tasks/2026-08-14-v080-github-first-crates-publication.md`; `docs/backlog/active/ARCH-031-crate-publication-boundary.md`; `docs/backlog/active/ARCH-031-E-v080-release-candidate-readiness.md`; `docs/iterations/I204-v080-release-candidate-readiness.md`; `docs/backlog/active/REL-003-v080-github-and-crates-publication.md`; `docs/iterations/I203-v080-github-and-crates-publication.md` |
| 0 | v0.8.0 release-candidate registry readiness | ARCH-031-E / I204 Complete/Closed | Completion Commit `f46094e3`; PR #260 merged as `7c10afe3`; reviewed conditional GO for preparing I203 claim only. The candidate-only boundary remains historical; I203 subsequently completed the authorized release and publication. | `docs/backlog/active/ARCH-031-E-v080-release-candidate-readiness.md`; `docs/iterations/I204-v080-release-candidate-readiness.md`; `docs/reference/I204-V080-READINESS-2026-08-16.md`; `docs/reference/I162-PUBLICATION-READINESS-2026-08-15.md`; `docs/iterations/I203-v080-github-and-crates-publication.md` |
| 0 | Goal-oriented work and evaluation foundation | WORK-001-A / I196 P0 is Complete / Closed through Completion Commit `779a4c71` and PR #291 merge `1467a561`; P0 remained decision/documentation-only and non-overlapping with Dashboard I195. P1-P4 remain separate and unactivated. | `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`; `docs/backlog/active/WORK-001-A-work-domain-decision-migration-contract.md`; `docs/iterations/I196-work-domain-decision-migration-contract.md`; DESKTOP-001; RUNTIME-001; SESSION-009; TODO-001; TODO-002; VALIDATION-001 |
| 0 | Ordered mainline priority requirements | Complete/Closed with owned residuals. T1-T10 have terminal dispositions and T11 is Complete; #59 remains Blocked only on separate TOOL-024 authority, while I197/I198/I201/I210 remain Review under corrective owners. | `docs/tasks/2026-08-14-mainline-priority-requirements-sequence.md`; I205; I196; I188; I199; I200; I197; I201; I212; I210; I198; I211; I214; I216; I217 |
| 0 | Deferred human review and acceptance batch | VALIDATION-002 / I211 is Complete/Closed at evidence commits `b7d55a0d`/`7c333d98`; every Issue #302 row passed or has a separate corrective owner. | `docs/backlog/active/VALIDATION-002-deferred-human-validation-batch.md`; `docs/iterations/I211-deferred-human-validation-batch.md`; Issues #302/#329/#330/#332/#333/#334 |
| 0 | PR workflow throughput simplification | GOV-007 / I205 is Complete / Closed through Completion Commit `2e2cf04b`, PR #287 merge `0394e264`, exact-head CI `32094384772` and independent technical audit `5323234878`. The evidence packet selects atomic claim activation as a separately claimable follow-up; no executable governance change is included. | `docs/backlog/active/GOV-007-pr-workflow-throughput-simplification.md`; `docs/iterations/I205-pr-workflow-throughput-simplification.md`; `docs/reference/I205-PR-WORKFLOW-THROUGHPUT-AUDIT.md`; collaboration/Git/DOC-CHECK/CI SOPs |
| 0 | Local convergence and stable-stage validation | GOV-008 / I215 is Complete/Closed at Completion Commit `06e61e3c`; PR #341 merged as `81a603b4` after exact-head CI, independent Agent-role review and CAS. No product/runtime scope. | `docs/backlog/active/GOV-008-local-convergence-stage-validation.md`; `docs/iterations/I215-local-convergence-stage-validation.md`; Issue #339 |
| 1 | Prompt authority architecture | PROMPT-001 / Issue #285 is Refinement / Unclaimed. Define authority/precedence and behavioral-evaluation contracts, then decompose bounded children before implementation. | `docs/backlog/active/PROMPT-001-prompt-authority-architecture.md`; ADR-033; Runtime SDK contract; prompt/context/evolution/memory/plugin owners |
| 1 | Parallel-test HOME isolation | TEST-001 / Issue #316 is Ready / Unclaimed. Isolate process-wide home-directory mutation in configuration and CLI tests without changing product paths or weakening sandbox coverage. | `docs/backlog/active/TEST-001-process-home-test-isolation.md`; `docs/sop/TESTING.md`; config and CLI test owners |
| 1 | Tombstone-pruning fixture performance | TEST-002 / Issue #396 is Ready and selected into I227, which remains Planned / Unclaimed. Preserve permanent idempotency coverage and production retention limits while removing the Windows 60-second fixture delay. | `docs/backlog/active/TEST-002-tombstone-pruning-fixture-performance.md`; `docs/iterations/I227-test002-tombstone-pruning-fixture-performance.md`; `docs/sop/TESTING.md`; pending-submission storage/tests; Windows CI |
| 1 | Progressive workspace code intelligence | CODE-003 / Issue #317 is Refinement / Unclaimed; CODE-003-A is the only Ready/Unclaimed child. Characterize current symbol-tool claims before query packs, indexing, resolution or storage work. | `docs/backlog/active/CODE-003-tree-sitter-usage-pattern-analysis.md`; `docs/backlog/active/CODE-003-A-code-intelligence-contract-characterization.md`; ADR-020; CODE-001/002; ARCH-034-R04-AG5 |
| 0 | Resumed Session interactivity under provider delay | TUI-051 / I209 is Complete/Closed at Completion Commit `2eff6285`, with source implementation `7b82fea6`/`7d90def8`, exact-head CI `32025371877`, real-terminal acceptance and independent agent audit `5316533941`. The separate PROVIDER-006/I210 owner remains Review/Claimed after PR #323; TUI-060/#332 owns its unresolved sequencing defects. | `docs/backlog/active/TUI-051-resumed-session-interactivity.md`; `docs/iterations/I209-resumed-session-interactivity.md`; Issue #272; PR #279 |
| 1 | Bounded provider retry progress contract | PROVIDER-006 / I210 is Review/Claimed. Retry ordinals and cleanup passed, while TUI-060/#332 owns fleeting initial connection status and false idle-first-turn queue messaging. | `docs/backlog/active/PROVIDER-006-bounded-retry-progress-contract.md`; `docs/iterations/I210-provider-retry-progress-contract.md`; `docs/backlog/active/TUI-060-initial-connection-and-queue-status-sequencing.md`; Issues #278/#332 |
| 0 | Internal validation service | Validation must become an internal callable, language-neutral service; governance must not depend on shell scripts and project adapters must be detected before guidance is injected. | `docs/backlog/active/VALIDATION-001-internal-validation-service.md`; `docs/backlog/active/GOV-003-builtin-project-governance.md`; `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` |
| 0 | Change-aware CI routing | I190/GOV-005 is Complete at implementation merge `a69ffa30` plus reduced-probe merge `01721f68`. Case-variant SOP exclusion matching remains separately unclaimed as GOV-006. | `docs/backlog/active/GOV-005-change-aware-ci-routing.md`; `docs/iterations/I190-change-aware-ci-routing.md`; `docs/backlog/active/GOV-006-ci-doc-path-case-normalization.md` |
| 0 | Emergency terminal containment | I191/TOOL-026 is Complete at `512ff32f`; final head `6b2dbdb5` passed CI `31587076213`, independent natural-person review `5274917099` and merge-time CAS. | `docs/backlog/active/TOOL-026-noninteractive-terminal-containment.md`; `docs/iterations/I191-noninteractive-terminal-containment.md`; ADR-007 |
| 0 | Emergency Session recovery closure | I192/SESSION-010 is Complete at `512ff32f`; matching-target resume, filtered picker and strict zero-byte cleanup passed CI `31587076213` and independent review `5274917099`. Historical/forced-kill cleanup remains excluded. | `docs/backlog/active/SESSION-010-runtime-resume-empty-artifact-closure.md`; `docs/iterations/I192-session-runtime-recovery-closure.md` |
| 0 | Permission pipeline convergence | A/I189, B/I219, I220 and C/I221 are Complete/Closed. I221 Completion Commit `49d1546c` reached main through PR #376 merge `f9e6706d`; D is Ready/Unclaimed and E remains Blocked/Unclaimed on D. `/auto` behavior and TOOL-024 remain separately governed. | `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`; `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md`; `docs/iterations/I221-perm006c-agent-owned-pipeline.md`; `docs/iterations/I220-perm006c-agent-owned-pipeline-decision.md`; `docs/decisions/067-agent-owned-permission-pipeline.md` |
| 0 | Memory admission safety | MEM-010 is Ready but unselected. A bounded correction iteration must prove only user-authored episodes enter new global memory. | `docs/backlog/active/MEM-010-user-origin-memory-admission.md`; Issue #114 |
| 0 | TUI Native Selection And Copy | TUI-046/Issue #134 is Complete through I184 policy merge `f9848827` and I186 implementation merge `a5115f5c`; exact code head `70b51e28` passed both terminal rows. | `docs/backlog/active/TUI-046-native-text-selection-copy.md`; `docs/iterations/I186-tui-visible-cell-selection.md`; ADR-054; `docs/reference/TUI-NATIVE-SELECTION-MATRIX.md` |
| 1 | Dashboard read-only visual shell | WEB-001-A / I195 is Complete / Closed through Completion Commit `490503db` and PR #233 merge. Exact-head CI `32087223234`, accepted independent AI review, human browser acceptance and merge-time CAS passed; excluded live/write/control/remote capabilities remain separate. | `docs/backlog/active/WEB-001-A-dashboard-read-only-visual-shell.md`; `docs/iterations/I195-dashboard-read-only-visual-shell.md`; `docs/backlog/active/WEB-001-embedded-web-control-surface.md`; ADR-031; three-track baseline |
| 1 | Dashboard live activity and log viewer | WEB-001-B / I213 is Complete / Closed at implementation merge `9f963d0ca662f334fe007d0fcfc857640a2a5bd6`; exact-head CI `32680547034` and independent security review `5390029881` passed. | `docs/backlog/active/WEB-001-B-dashboard-live-activity-log-viewer.md`; `docs/iterations/I213-dashboard-live-activity-log-viewer.md`; `docs/backlog/active/WEB-001-embedded-web-control-surface.md`; ADR-006; ADR-031; OBS-001/ADR-014 |
| 1 | Dashboard availability in Logo prefix | TUI-037 / I202 is Complete at Completion Commit `6d3f85ea`; PR #230 merged as `e0cc782a` after exact-head CI `31775126382`, independent security approval `5290402214`, real-terminal acceptance and CAS `5290414997`. One display-only token-free Logo entry replaces the success Tip and token log; failures remain Tips. | `docs/backlog/active/TUI-037-dashboard-logo-link.md`; `docs/iterations/I202-tui037-dashboard-logo-link.md`; ADR-031; ADR-054; Issue #104; PR #230 |
| 1 | TUI regression intake | TUI-041/I199 and TUI-042/I200 are Complete/Closed. I197/I201 remain Review under TUI-059/#330 and TUI-058/#329; TUI-061/#334 separately owns the continuation-padding regression. | `docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md`; `docs/backlog/active/TUI-042-noop-history-scroll-stability.md`; `docs/backlog/active/TUI-043-tool-placeholder-suppression.md`; `docs/backlog/active/TUI-045-permission-prompt-layout-anchor.md`; `docs/backlog/active/TUI-058-permission-mediated-tool-activity-correlation.md`; `docs/backlog/active/TUI-059-permission-prompt-composer-docking.md`; `docs/backlog/active/TUI-061-history-continuation-padding-regression.md`; `docs/backlog/active/VALIDATION-002-deferred-human-validation-batch.md` |
| 1 | Steering follow-up sequence | TUI-048/I206, TUI-049/I207 and TUI-050/I208 are Planned / Unclaimed in ordered sequence. They do not reopen I169 and have no implementation authorization. | `docs/tasks/2026-08-17-steering-follow-up-sequence.md`; `docs/backlog/active/TUI-048-steering-esc-activation.md`; `docs/backlog/active/TUI-049-steering-wrap-padding.md`; `docs/backlog/active/TUI-050-steering-insertion-boundary.md` |
| 1 | Remaining TUI interaction intake | TUI-052/#266, TUI-053/#268, TUI-054/#269 and TUI-055/#280 remain Intake / Unclaimed; TUI-056/#298 and TUI-057/#310 are Refinement / Unclaimed. None has a selected iteration, and each requires decomposition or boundary checks before selection. TUI-051/#272 is Complete through I209. | `docs/backlog/active/TUI-052-todo-rendering-muted-readability.md`; `docs/backlog/active/TUI-053-numeric-permission-shortcuts.md`; `docs/backlog/active/TUI-054-native-key-repeat-routing.md`; `docs/backlog/active/TUI-055-narrow-markdown-table-layout.md`; `docs/backlog/active/TUI-056-collapsible-reasoning-history.md`; `docs/backlog/active/TUI-057-live-activity-status-headers.md`; `docs/backlog/active/TUI-051-resumed-session-interactivity.md` |
| 1 | Desktop Preset Session templates | DESKTOP-002/#308 is Blocked / Unclaimed. It defines Presets as one-time Session Environment templates and must wait for MODEL-012/#146, SESSION-009 and DESKTOP-001 shared prerequisites before child decomposition or implementation. | `docs/backlog/active/DESKTOP-002-preset-session-environment-templates.md`; `docs/backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md`; `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`; `docs/backlog/active/DESKTOP-001-desktop-product-direction.md` |
| 1 | Custom-model context-window catalog inference | MODEL-013/I212 is Complete/Closed at Completion Commit `5a1709cb`; PR #318 merge gates and the integrated exact/prefix/override/unknown no-request walkthrough passed. | `docs/backlog/active/MODEL-013-catalog-context-window-inference.md`; `docs/iterations/I212-model013-catalog-context-window-inference.md`; `docs/backlog/active/MODEL-011-custom-model-capability-probe.md`; ADR-022 |
| 1 | Desktop D0 renderer/host boundary | `DESKTOP-001-D0` / I194 Complete at `0a47208c` after PR #215 merge `1beaca68`; GPUI/Iced source snapshots and crates.io metadata are recorded, while the transitive graph probe remains negative evidence. The parent DESKTOP-001 remains Deferred / Unclaimed / Selected Iteration None; D0 does not authorize GPUI, i18n, native dependencies or Desktop implementation. | `docs/backlog/active/DESKTOP-001-D0-renderer-host-boundary.md`; `docs/iterations/I194-desktop-renderer-host-boundary.md`; `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`; ADR-042; ADR-052; ADR-059; `docs/reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md` |
| 1 | Runtime session and protocol foundations | SESSION-008 and RUNTIME-005 A/B/C are Complete/Closed; ADR-063 is Accepted. Issue #49 closed after evidence-bearing closeout merge `1503ca21`; Issue #45 remains separately open. | `docs/backlog/active/SESSION-008-interrupted-turn-partial-persistence.md`; `docs/iterations/I193-session008b-durable-partial-finalization.md`; `docs/decisions/058-partial-turn-durable-finalization.md`; `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md`; `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`; `docs/backlog/active/RUNTIME-005-A-shutdown-contract-decision.md`; `docs/backlog/active/RUNTIME-005-B-shutdown-coordinator-admission.md`; `docs/backlog/active/RUNTIME-005-C-ordered-finalizer-durable-closure.md`; `docs/iterations/I217-runtime005c-ordered-finalizer-durable-closure.md` |
| 1 | Runtime single-direct-dependency SDK facade | RUNTIME-006 / Issue #234 is Refinement / Unclaimed. It will make `talos-runtime` the only direct Talos dependency for the supported provider/tool/event/permission/sandbox surface, but is explicitly outside v0.8.0 and needs its own iteration/claim. | `docs/backlog/active/RUNTIME-006-single-dependency-sdk-facade.md`; `docs/reference/RUNTIME-SDK-CONTRACT.md`; ARCH-031; ADR-024; ADR-052; Issue #234 |
| 1 | Supervised background command jobs | TOOL-024-B/I222 and C/I224 are Complete/Closed. D1-A/I225 is Active/Claimed after #388 merge `2afcdc3e`; D1-B, D2 and I223 remain separate, and Windows stays fail-closed. | `docs/backlog/active/TOOL-024-background-command-jobs.md`; `docs/backlog/active/TOOL-024-D1-A-windows-job-object-decision.md`; `docs/iterations/I225-tool024d1a-windows-job-object-decision.md`; `docs/iterations/I223-issue59-deferred-human-validation-cleanup.md`; `docs/tasks/2026-08-23-issue59-supervised-background-jobs.md` |
| 1 | Memory scope architecture | MEM-011 remains Refinement. Accept schema/migration and legacy-fixture decisions before implementation. | `docs/backlog/active/MEM-011-extensible-memory-scopes.md`; Issue #116 |
| 1 | Provider/runtime reliability follow-ups | Preserve explicit terminal outcomes, usage accounting and bounded request/stream behavior before dependent status/cost UX. | `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`; `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`; `docs/backlog/active/PROVIDER-004-text-tool-call-id-collision.md` |
| 0 | Provider streaming UTF-8 containment | PROVIDER-005 is Complete/Closed at Completion Commit `1d31847a`; PR #271 merged as `89523dbc` after exact-head CI `32002811484`, independent approval `5313112992` and CAS. Valid multibyte characters survive transport splits; invalid bytes remain fail-closed. | `docs/backlog/active/PROVIDER-005-streaming-utf8-chunk-boundary.md`; Issue #270; PR #271 |
| 1 | Network resilience policy | NET-001 / Issue #199 is Intake and Unclaimed. Inventory all Talos-owned outbound paths and accept replay-safety, retry, cancellation, streaming-commit and per-target breaker decisions before selecting implementation children. | `docs/backlog/active/NET-001-network-resilience-policy.md`; Issue #199 |
| 1 | Structured diagnostics and error fidelity intake | OBS-002 / Issue #395 is Intake / Unclaimed. Characterize current logging, correlation and source-preserving error boundaries before selecting an ADR-backed implementation child; no implementation authority exists. | `docs/backlog/active/OBS-002-structured-diagnostics-contract.md`; Issue #395; OBS-001; ADR-014 |
| 1 | `/delete` cleanup-failure actionability | Issue #136 is Open and independently owns executable recovery-command wording. It must preserve the accepted transcript-last, retryable, no-false-success behavior from I169. | `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`; ADR-056; Issue #136 |
| 2 | Dashboard opt-in token delivery boundary | SEC-002 is Refinement / Unclaimed / Selected Iteration None. Decide a threat-modeled ephemeral delivery, authentication redesign or mode deprecation before any production change; do not reuse I202 or WEB-001-A authorization. | `docs/backlog/active/SEC-002-dashboard-token-delivery-boundary.md`; ADR-031; ADR-023; WEB-001 |
| 2 | Dynamic provider authentication program | PROVIDER-003 / Issue #132 is a Refinement Epic only. PROVIDER-003-A must accept the capability ADR and threat model before any child can be selected; B/C own shared lifecycle/request contracts and D-G own bounded provider/acquisition slices. | `docs/backlog/active/PROVIDER-003-dynamic-provider-credentials.md`; ADR-013; ADR-023; ADR-057; Issue #132 |
| 2 | TOOL-023 residual configuration work | TOOL-023-A/C are Complete through I170. TOOL-023-B alone owns the 300-second default/configuration proposal and is not implemented by I170. | `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md`; `docs/backlog/active/TOOL-023-B-configurable-timeout-default.md` |
| 3 | Deferred product architecture | DESKTOP-001 remains a Deferred/Unclaimed parent while its separately governed D0 child is proposed above; multi-agent, health monitoring, persistent tasks and A2A remain Deferred/Refinement until explicitly reprioritized and ADR-gated. | DESKTOP-001; DESKTOP-001-D0; AGENT-003; RUNTIME-004; TASK-001; A2A-001 |

## I169 Closeout

| ID | Title | Final State | Evidence / Residual |
|---|---|---|---|
| I169 | Transactional Batched Steering Turn | **Complete (2026-08-06)** | PR #131 merged at `685d3b4f4088a172551f8c844a89f5dee9469430`; exact accepted Head `90165cace4625c0f27616b3e1b9871bcb6a10186`; CI `31010166558`; rebuilt real-terminal acceptance passed. |
| TUI-044 | Transactional Batched Steering Turn | **Complete** | A/B/C remain distinct FIFO messages in one later Turn; durable receipt, restart, replay, fork isolation, complete delete and retryable failure behavior accepted. |
| ADR-056 | Transactional Steering Submission And Turn Ownership Boundary | **Accepted** | Durable custody, lost-Ack reconciliation, generation-safe lifecycle, Actor arbitration, transcript-before-journal finalization and exact request-plan boundaries are authoritative. |
| Issue #119 | Recovered transactional steering feature | **Completed** | Acceptance matrix satisfied; implementation and governance reconciliation complete. |
| Issue #136 | Direct `/delete` recovery-command wording | Open, non-blocking | Separately owns only diagnostic/actionability wording; does not reopen TUI-044/I169 or ADR-056 semantics. |

Required reads:

- `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`
- `docs/iterations/I169-batched-steering-turn.md`
- `docs/decisions/056-transactional-steering-submission-boundary.md`
- PR #131 and Issues #119/#136

## I170 Closeout

| ID | Title | Final State | Evidence / Residual |
|---|---|---|---|
| I170 | Windows Workspace Validation Unblocker | Complete (2026-08-01) | PR #126 merged at `592254d73a98166df48da0139a02df67e9cd2cd6`; exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`; CI `30705366763`; walkthrough artifact `8820174164`. |
| TOOL-023-A | Absolute shell timeout | Complete | One pinned operation deadline, partial-output preservation and direct-child cleanup are verified on Windows and Unix/macOS. Full descendant process-tree supervision remains separate. |
| TOOL-023-C | Windows-native PowerShell shell | Complete | One authoritative platform shell contribution, child-local environment scrub, portable output and fail-closed Windows reusable-template allowlist are verified. |
| ADR-057 | Windows PowerShell Process Boundary | Accepted | Accepted direct-child process/timeout and conservative permission boundary. No Job Object, parser, PowerShell 7 or full process-tree claim. |
| TOOL-023 | Cross-platform shell and timeout Epic | Partial | A/C Complete; B remains Ready and separately unimplemented. |

Required reads:

- `docs/iterations/I170-windows-workspace-validation-unblocker.md`
- `docs/backlog/active/TOOL-023-A-bash-timeout-fix.md`
- `docs/backlog/active/TOOL-023-C-windows-powershell.md`
- `docs/decisions/057-windows-powershell-process-boundary.md`
- `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`

## Active / Selectable Items

| ID | State | Selection / Exit Gate |
|---|---|---|
| I219 / PERM-006-B | Complete / Closed | Completion Commits `56436027`/`d0c96048`; exact-head CI `32579790496`, review `5381051760`, CAS and PR #368 merge `de79ad46`. C-E remain separately blocked. |
| I213 / WEB-001-B | Complete / Closed | Completion Commit `9f963d0ca662f334fe007d0fcfc857640a2a5bd6`; PR #372 merged after CI `32680547034` and independent review `5390029881`. |
| I220 / PERM-006-C decision prerequisite | Complete / Closed | Completion Commits `c21bb7f3`/`820586ea`; PR #373 merged as `5d2d2dcf`, ADR-067 Accepted. I221 separately completed C implementation through commit `49d1546c` and PR #376 merge `f9e6706d`. |
| I221 / PERM-006-C implementation | Complete / Closed | Completion Commit `49d1546c`; PR #376 merged as `f9e6706d` after exact-head CI `32640691772`, independent permission/security/API approval `5386153429` and CAS. |
| I222 / TOOL-024-B managed background core | Complete / Closed | Completion Commit `8671edf45c168612bfa4a4bbb65a9847026e1b96`; PR #382 merged after exact-head CI `32690533253`, independent protected-scope review and final governance review. |
| I223 / Issue #59 deferred validation cleanup | Planned / Unclaimed | Evidence-only cleanup for Issue #378 after B/C/D implementation heads exist; no product behavior authority. |
| I225 / TOOL-024-D1-A Windows Job Object decision | Active / Claimed | Decision-only ADR-068/current-path prerequisite activated by #388 merge `2afcdc3e`; no Windows implementation authority until a later D1-B claim. |
| ARCH-031-A / I159 | Complete / Closed | Completion Commit `d886917e` plus cfg follow-up `34c09b14`; PR #236 merged as `f79c1ead` after CI `31801484313`, approval `5293622712` and CAS. |
| ARCH-031-B / I160 | Complete / Closed | Completion Commit `0524e82f`; PR #240 merged as `97556149`, closeout PR #241 merged as `2d48bd2c`; I161 and I162 are Complete/Closed, with I162 readiness Completion Commit `077b347d` and PR #255 merge `16564ba0`. |
| WORK-001-A / I196 | Complete / Closed | Completion Commit `779a4c71`; PR #291 merged as `1467a561` after exact-head CI `32101943484` and independent architecture review. P0 was documentation-only; P1-P4 remain separately governed. |
| ARCH-034-D / I171 | Complete | Completion Commit `56f419f7`; source current v0.7.0 audit/register evidence `c88c1d1a`; bounded owner creation validated; no production refactor. |
| MEM-010 | Ready | Select one bounded safety iteration and preserve existing session/memory behavior. |
| TUI-043 / I201 | Review / Claimed | Implementation merged as `7f5a6df2`; permission-mediated suppression failed terminal validation and is now owned by Ready/Unclaimed TUI-058/#329. Preserve Review until that separately governed correction is delivered. |
| TOOL-023-B | Ready | Separate timeout-default/configuration change; do not reopen completed A/C behavior. |
| PROVIDER-004 | Ready | Force unique text-path tool-call IDs and prove cross-turn pairing; keep separate from shell/runtime timeouts. |
| Issue #136 | Open follow-up | Implement and validate exact `/delete <uuid>` and `talos storage maintenance --reconcile` guidance without changing accepted cleanup ownership. |

## Review / Partial Items

| ID | State | Required Disposition |
|---|---|---|
| I158 / ARCH-034-R01 | Complete | Completion Commit `c88c1d1a`; scheduler/status exceptions and final architecture/extension/finding documentation accepted. |
| ARCH-034-R02 / I172 | Complete | Completion Commit `4084138dc0652d3200045847d42518d9ecb66231`; PR #144 merged at `c1dc67ae`; exact-head CI `31137882248` passed. |
| ARCH-034-R05 / I174 | Complete | Completion Commit `e4248bfedd17c91aebb24c80c60580fcbcebec62`; PR #152 merged at `62b09c277713bea8404ed7ef9c7f50354e5a2e17`; exact-head CI `31148908291` passed. |
| ARCH-034-R06 / I175 | Complete | Completion Commit `5c45322245788e12316dffbe1f9cfacef390eff8`; PR #156 merged at `73898bdba0d072886c79023c048250190a3b5e04`; exact-head CI `31152972959` passed. R04 is tracked by its own current row. |
| ARCH-034-R07 / I176 | Complete | Completion Commit `1de3243d`; PR #159 merged at `37c557271b906664022476bd2775c5cd77f2b8ea`; exact-head CI `31160309818` passed. R04 is tracked by its own current row. |
| ARCH-034-R08 / I177 | Complete | Completion Commit `f505eea8` (squash merge of implementation `786aa571`); PR #162; exact-head CI `31166594367` passed. |
| ARCH-034-R09 / I178 | Complete | Completion Commit `f92634803560dc50e0b15ca8d7d511e9928c983f` (squash merge of source implementation `c662a7e6`); PR #165; exact-head CI `31180591881` passed. |
| ARCH-034-R10 / I179 | Complete | Completion Commit `dafc9be08736aee91e0f9cdd92e5226930808061` (squash merge of source implementation `63d494c5`); PR #168; exact-head CI `31189425069` passed. |
| ARCH-034-R11 / I180 | Complete | Completion Commit `10cceec6aeb9089fe9c830355992c8fc60430d63` (squash merge of source implementation `fd8ac75d`); PR #171; exact-head CI `31238721507` passed. R04 is tracked by its own current row. |
| ARCH-034-R03 / I173 | Complete | Completion Commit `e4818e34c1e047c41d41abc1f7859c7984008e83`; PR #149 merged as `506311dc`; exact-head CI `31143057387` passed. |
| ARCH-034-R04 / I181 / I182 / I183 / I185 | Partial / I185 Complete | Review-only matrix completed at `aea26ad0`; AG-4 at `ae31242b`; AG-7/I183 at `edf903aa`; [AG-12](active/ARCH-034-R04-AG12-sqlite-validator-integrity.md) / [I185](../iterations/I185-sqlite-validator-policy-integrity.md) at `af978322`. AG-1/2/3/5/6/8/9/10/11 remain separate. |
| DATA-002 | Intake — Issue #141 | Storage topology and runtime ownership require ADR-backed filesystem policy, cross-runtime ownership, and fail-safe reconciliation refinement before implementation. [Owner](active/DATA-002-storage-topology-and-runtime-ownership.md) |
| SERVER-001 | Intake — Issue #142 | Serve/connect protocol adapter architecture requires one authoritative runtime/session/permission path and dependency refinement before implementation. [Owner](active/SERVER-001-serve-connect-protocol-adapters.md) |
| TOOL-025 | Intake — Issue #143 | RTK-derived shell filtering requires bounded source selection, provenance review, and preservation of Talos execution authorities before implementation. [Owner](active/TOOL-025-rtk-derived-semantic-output-filters.md) |
| MODEL-012 | Intake — Issue #146 | Optional Utility Model role requires additive configuration, explicit task routing, canonical provider ownership, and evaluation refinement before implementation. [Owner](active/MODEL-012-utility-model-role-and-bounded-routing.md) |
| SKILL-004 | Review / Claimed — I198 implementation merged | Omitted/empty/list compatibility passed; malformed-input CLI diagnostic failed and is owned by Ready/Unclaimed SKILL-005/#333. [Owner](active/SKILL-004-optional-skill-triggers-compatibility.md) |
| SKILL-005 | Ready / Unclaimed | Issue #333 owns real-CLI visibility of invalid local Skill `triggers` diagnostics; separate iteration/claim required. [Owner](active/SKILL-005-invalid-skill-diagnostic-visibility.md) |
| TUI-061 | Ready / Unclaimed | Issue #334 owns ordinary wrapped-history continuation padding; separate iteration/claim required. [Owner](active/TUI-061-history-continuation-padding-regression.md) |
| ARCH-034-R04 AG-8/9/10 | Refinement — unclaimed review residuals | Independent review `5230395611` approved #177 head `4b968823`; [AG-8](active/ARCH-034-R04-AG8-symbol-workspace-path-containment.md), [AG-9](active/ARCH-034-R04-AG9-symbol-decoding-consistency.md) and [AG-10](active/ARCH-034-R04-AG10-symbol-notice-admissibility.md) own the non-blocking path/decoding/notice findings without expanding I182. |
| ARCH-034-R04-AG11 | Refinement — unclaimed SQLite containment residual | [AG-11](active/ARCH-034-R04-AG11-sqlite-containment-evidence.md) owns five-consumer corrupt/busy/locked/panic/deadline evidence classification discovered by I183; no runtime remediation is authorized. |
| ARCH-034-R04-AG12 / I185 | Complete | Completion Commit `af9783229bfc8ee592813440ecfcdb6efc90a3c2`; exact head `45f70802`, CI `31556720252`, independent review `5261491057`. No consumer or runtime change. |
| GOV-004 | Refinement — shared-account reviewer attestation | Preserve independent natural-person review while defining explicit shared-account disclosure and mechanically honest validation. [Owner](active/GOV-004-shared-account-review-attestation.md) |
| GOV-005 / I190 | Complete | Trusted-base fail-closed routing merged at `a69ffa30`; real reduced probe merged at `01721f68` after run `31564461023` proved stable retained/skipped checks. [Story](active/GOV-005-change-aware-ci-routing.md) / [Iteration](../iterations/I190-change-aware-ci-routing.md) |
| GOV-006 | Refinement — unclaimed CI path-case residual | Normalize the `docs/sop/` full-route exclusion across path-letter case and add adversarial fixtures without broadening the completed GOV-005 allowlist. [Owner](active/GOV-006-ci-doc-path-case-normalization.md) |
| GOV-008 / I215 | Complete / Closed | Completion Commit `06e61e3c`; PR #341 merge `81a603b4`, CI `32442052401`, review `5365129718` and CAS. [Story](active/GOV-008-local-convergence-stage-validation.md) / [Iteration](../iterations/I215-local-convergence-stage-validation.md) / Issue #339 |
| TOOL-026 / I191 | Closed / Delivered | Owner evidence `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213` and independent review `5274917099`. TOOL-024-A/I188 later completed independently at `245eddeb`; production children remain separately blocked. [Story](active/TOOL-026-noninteractive-terminal-containment.md) / [Iteration](../iterations/I191-noninteractive-terminal-containment.md) |
| SESSION-010 / I192 | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213` and independent review `5274917099`. Its SESSION-008-B residual was completed later by I193; RUNTIME-005 remains independently governed. [Story](active/SESSION-010-runtime-resume-empty-artifact-closure.md) / [Iteration](../iterations/I192-session-runtime-recovery-closure.md) |
| TOOL-023 | Partial | A/C Complete; decide TOOL-023-B independently. |
| PERM-004 | Partial | Issue #22 stays open for broader sandbox/security residuals. |
| PROVIDER-001 | Review | Complete accurate OpenAI-compatible streaming usage evidence. |
| TUI-017 / TUI-018 / TOOL-015 / TUI-019 | Review | Reconcile with their owner docs and dependent provider/tool evidence before closure. |

## Refinement / Blocked Items

The authoritative open-Issue mapping and dispositions are maintained in
[`docs/reference/ISSUE-DOC-CODE-STATUS-2026-08-01.md`](../reference/ISSUE-DOC-CODE-STATUS-2026-08-01.md).
Key chains include:

- PERM-006-A → B → C → D → E;
- SESSION-009 → ACP-001;
- I158 disposition → TUI-037 disposition → I159 → I160 → I161 → I162;
- SESSION-008-A → SESSION-008-B → RUNTIME-005-A → B → C;
- PERM-006-A → B → C and RUNTIME-005 Complete before TOOL-024-B may spawn a background process;
- TOOL-024-A decision → B supervisor → C `process` tool → D1-A Windows decision → D1-B Windows ownership → D2 projection/acceptance → I223 validation cleanup;
- ADR/migration acceptance before MEM-011;
- PROVIDER-003-A before B/C, then one bounded D-G provider/acquisition child at a time;
- TUI-046-A and TUI-046-B are Complete; future terminal interaction changes require separate owners;
- DESKTOP-001-D0 / I194 decision acceptance before any Desktop renderer/i18n implementation dependency; P0-P4 remain required before real Mission/runtime/work/evaluation binding;
- architecture decisions before DESKTOP-001, AGENT-003, RUNTIME-004, TASK-001 or A2A-001 implementation.

## Completed Programs And History

Completed programs, release closeouts, prior active-item detail and the full historical inventory remain available in:

- [`PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md`](PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md)
- `docs/iterations/`
- `docs/tasks/`
- `docs/releases/`
- `docs/reference/`

Historical entries are evidence only. They do not override current owner documents or authorize new work.

## Reading Rules

1. Read this file to identify current state and priority.
2. Read every linked owner document before planning, implementation or status change.
3. Re-read current GitHub Issues, PRs, branches and `main` before activation.
4. Never infer implementation authorization from a Ready/Planned entry alone.
5. Do not continue work on archival recovery branches or PRs.

## Issue Sync Rule

When an Issue-backed owner transitions to Active, Review, Complete, Blocked or Cancelled, synchronize
the remote Issue with the causing PR/commit and one-line disposition. Close an Issue only when its
owner is Complete/Cancelled and no separately owned residual remains.

Issue #119 is completed and closed by the I169 post-merge governance closeout. Issue #136 remains
open under its own diagnostic scope and does not block or reopen I169.
Issue #132 remains open under the PROVIDER-003 Refinement Epic; no child implementation is authorized
until PROVIDER-003-A and a separately claimed child owner establish the required boundary.
Issue #49 closed after the RUNTIME-005 evidence-bearing closeout merged as `1503ca21`; SESSION-008 durable
partial persistence and RUNTIME-005-A/B/C form the completed non-circular closure chain. Issue #59 remains open under TOOL-024;
TOOL-024 consumes completed RUNTIME-005 finalization and PERM-006-C instead of blocking either;
production spawn now waits for a separate TOOL-024-B owner, iteration and effective claim. Issue #134 closes only after the reviewed governance closeout reaches
`main`; TUI-046-A and I186/TUI-046-B are Complete with two-terminal acceptance and existing merge
evidence.
