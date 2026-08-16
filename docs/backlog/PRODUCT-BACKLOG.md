# Product Backlog

This file is the compact current-state backlog entrypoint. Story scope, acceptance criteria,
implementation evidence, and residual ownership live in the linked owner documents.

The complete pre-closeout index is preserved unchanged at
[`PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md`](PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md).
That snapshot is historical evidence, not current activation authority.

## Current Priorities

| Priority | Focus | Current State / Gate | Required Reads |
|---|---|---|---|
| 0 | v0.8.0 GitHub-first crates publication | I159/I160/I161/I162/I204 Complete; I203 Planned / Claimed | I204 Completion Commit `f46094e3`, closeout merge `8eaa22a2`; reviewed conditional GO for preparing I203 claim. Claim PR #262 is proposed from exact main and ineffective until independently reviewed and merged. GitHub Release precedes Cargo publication; intended closure is 20 packages excluding only `talos-models`. | `docs/tasks/2026-08-14-v080-github-first-crates-publication.md`; `docs/backlog/active/ARCH-031-crate-publication-boundary.md`; `docs/backlog/active/ARCH-031-E-v080-release-candidate-readiness.md`; `docs/iterations/I204-v080-release-candidate-readiness.md`; `docs/backlog/active/REL-003-v080-github-and-crates-publication.md`; `docs/iterations/I203-v080-github-and-crates-publication.md` |
| 0 | v0.8.0 release-candidate registry readiness | ARCH-031-E / I204 Complete/Closed | Completion Commit `f46094e3`; PR #260 merged as `7c10afe3`; reviewed conditional GO for preparing I203 claim only. I203 remains Blocked/Unclaimed until a fresh claim; no release or publication was authorized. | `docs/backlog/active/ARCH-031-E-v080-release-candidate-readiness.md`; `docs/iterations/I204-v080-release-candidate-readiness.md`; `docs/reference/I204-V080-READINESS-2026-08-16.md`; `docs/reference/I162-PUBLICATION-READINESS-2026-08-15.md`; `docs/iterations/I203-v080-github-and-crates-publication.md` |
| 0 | Goal-oriented work and evaluation foundation | WORK-001-A / I196 has an effective claim through PR #226 merge `453d1fba` but remains Planned and unactivated. The maintainer placed it behind the v0.8.0 publication task; after release closure it must refresh exact-main inventory before P0 work. P1-P4 remain separate. | `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`; `docs/backlog/active/WORK-001-A-work-domain-decision-migration-contract.md`; `docs/iterations/I196-work-domain-decision-migration-contract.md`; DESKTOP-001; RUNTIME-001; SESSION-009; TODO-001; TODO-002; VALIDATION-001 |
| 0 | Internal validation service | Validation must become an internal callable, language-neutral service; governance must not depend on shell scripts and project adapters must be detected before guidance is injected. | `docs/backlog/active/VALIDATION-001-internal-validation-service.md`; `docs/backlog/active/GOV-003-builtin-project-governance.md`; `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` |
| 0 | Change-aware CI routing | I190/GOV-005 is Complete at implementation merge `a69ffa30` plus reduced-probe merge `01721f68`. Case-variant SOP exclusion matching remains separately unclaimed as GOV-006. | `docs/backlog/active/GOV-005-change-aware-ci-routing.md`; `docs/iterations/I190-change-aware-ci-routing.md`; `docs/backlog/active/GOV-006-ci-doc-path-case-normalization.md` |
| 0 | Emergency terminal containment | I191/TOOL-026 is Complete at `512ff32f`; final head `6b2dbdb5` passed CI `31587076213`, independent natural-person review `5274917099` and merge-time CAS. | `docs/backlog/active/TOOL-026-noninteractive-terminal-containment.md`; `docs/iterations/I191-noninteractive-terminal-containment.md`; ADR-007 |
| 0 | Emergency Session recovery closure | I192/SESSION-010 is Complete at `512ff32f`; matching-target resume, filtered picker and strict zero-byte cleanup passed CI `31587076213` and independent review `5274917099`. Historical/forced-kill cleanup remains excluded. | `docs/backlog/active/SESSION-010-runtime-resume-empty-artifact-closure.md`; `docs/iterations/I192-session-runtime-recovery-closure.md` |
| 0 | Permission pipeline convergence | I189/PERM-006-A is Planned/Claimed through claim merge `0df88638`; no implementation has started. B-E remain blocked in order, and PERM-007 cannot bypass PERM-004/PERM-005 safety. | `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`; `docs/backlog/active/PERM-006-A-structured-permission-decisions.md`; `docs/iterations/I189-perm006a-structured-permission-decisions.md`; PERM-006-B/C/D/E; `docs/backlog/active/PERM-007-model-assisted-goal-permission-decisions.md` |
| 0 | Memory admission safety | MEM-010 is Ready but unselected. A bounded correction iteration must prove only user-authored episodes enter new global memory. | `docs/backlog/active/MEM-010-user-origin-memory-admission.md`; Issue #114 |
| 0 | TUI Native Selection And Copy | TUI-046/Issue #134 is Complete through I184 policy merge `f9848827` and I186 implementation merge `a5115f5c`; exact code head `70b51e28` passed both terminal rows. | `docs/backlog/active/TUI-046-native-text-selection-copy.md`; `docs/iterations/I186-tui-visible-cell-selection.md`; ADR-054; `docs/reference/TUI-NATIVE-SELECTION-MATRIX.md` |
| 1 | Dashboard read-only visual shell | WEB-001-A / I195 is Planned / Claimed through PR #212 merge `f123e534`; it remains unactivated and independently owned by the Dashboard lane. The slice is user-facing IA/UX over existing GET-only loopback surfaces only; no release or mainline ownership is imported. | `docs/backlog/active/WEB-001-A-dashboard-read-only-visual-shell.md`; `docs/iterations/I195-dashboard-read-only-visual-shell.md`; `docs/backlog/active/WEB-001-embedded-web-control-surface.md`; ADR-031; three-track baseline |
| 1 | Dashboard availability in Logo prefix | TUI-037 / I202 is Complete at Completion Commit `6d3f85ea`; PR #230 merged as `e0cc782a` after exact-head CI `31775126382`, independent security approval `5290402214`, real-terminal acceptance and CAS `5290414997`. One display-only token-free Logo entry replaces the success Tip and token log; failures remain Tips. | `docs/backlog/active/TUI-037-dashboard-logo-link.md`; `docs/iterations/I202-tui037-dashboard-logo-link.md`; ADR-031; ADR-054; Issue #104; PR #230 |
| 1 | TUI regression intake | TUI-043 is Ready; TUI-041, TUI-042 and TUI-045 remain Refinement. Issue #134 is split into the separately prioritized TUI-046-A/B chain. | `docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md`; `docs/backlog/active/TUI-042-noop-history-scroll-stability.md`; `docs/backlog/active/TUI-043-tool-placeholder-suppression.md`; `docs/backlog/active/TUI-045-permission-prompt-layout-anchor.md` |
| 1 | Desktop D0 renderer/host boundary | `DESKTOP-001-D0` / I194 Complete at `0a47208c` after PR #215 merge `1beaca68`; GPUI/Iced source snapshots and crates.io metadata are recorded, while the transitive graph probe remains negative evidence. The parent DESKTOP-001 remains Deferred / Unclaimed / Selected Iteration None; D0 does not authorize GPUI, i18n, native dependencies or Desktop implementation. | `docs/backlog/active/DESKTOP-001-D0-renderer-host-boundary.md`; `docs/iterations/I194-desktop-renderer-host-boundary.md`; `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`; ADR-042; ADR-052; ADR-059; `docs/reference/DESKTOP-I194-DEPENDENCY-SECURITY-MATRIX.md` |
| 1 | Runtime session and protocol foundations | SESSION-008 is Complete: A decision `e288afb5`, B implementation `404d7a4b`, PR #216 merge `1b5461cd`, CI `31691761892`. I187 remains the pre-I193 baseline; ADR-058 is implemented on `main`. RUNTIME-005 remains Refinement/Unclaimed with A Ready/not selected, B awaiting A Accepted, and C awaiting B. Issues #45/#49 remain open. | `docs/backlog/active/SESSION-008-interrupted-turn-partial-persistence.md`; `docs/iterations/I193-session008b-durable-partial-finalization.md`; `docs/decisions/058-partial-turn-durable-finalization.md`; `docs/reference/I187-SESSION-008-PARTIAL-TURN-CHARACTERIZATION.md`; `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md` |
| 1 | Runtime single-direct-dependency SDK facade | RUNTIME-006 / Issue #234 is Refinement / Unclaimed. It will make `talos-runtime` the only direct Talos dependency for the supported provider/tool/event/permission/sandbox surface, but is explicitly outside v0.8.0 and needs its own iteration/claim. | `docs/backlog/active/RUNTIME-006-single-dependency-sdk-facade.md`; `docs/reference/RUNTIME-SDK-CONTRACT.md`; ARCH-031; ADR-024; ADR-052; Issue #234 |
| 1 | Supervised background command jobs | I188/TOOL-024-A is Planned/Claimed through claim merge `02a35588`; no decision implementation has started. Production work remains gated by accepted A, completed RUNTIME-005, PERM-006-C and TOOL-023-C; Issue #59 remains open. | `docs/backlog/active/TOOL-024-background-command-jobs.md`; `docs/backlog/active/TOOL-024-A-background-job-lifecycle-spike.md`; `docs/iterations/I188-tool024a-background-job-contract.md`; `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`; `docs/backlog/active/PERM-006-C-agent-owned-permission-pipeline.md` |
| 1 | Memory scope architecture | MEM-011 remains Refinement. Accept schema/migration and legacy-fixture decisions before implementation. | `docs/backlog/active/MEM-011-extensible-memory-scopes.md`; Issue #116 |
| 1 | Provider/runtime reliability follow-ups | Preserve explicit terminal outcomes, usage accounting and bounded request/stream behavior before dependent status/cost UX. | `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`; `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`; `docs/backlog/active/PROVIDER-004-text-tool-call-id-collision.md` |
| 1 | Network resilience policy | NET-001 / Issue #199 is Intake and Unclaimed. Inventory all Talos-owned outbound paths and accept replay-safety, retry, cancellation, streaming-commit and per-target breaker decisions before selecting implementation children. | `docs/backlog/active/NET-001-network-resilience-policy.md`; Issue #199 |
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
| ARCH-031-A / I159 | Complete / Closed | Completion Commit `d886917e` plus cfg follow-up `34c09b14`; PR #236 merged as `f79c1ead` after CI `31801484313`, approval `5293622712` and CAS. |
| ARCH-031-B / I160 | Complete / Closed | Completion Commit `0524e82f`; PR #240 merged as `97556149`, closeout PR #241 merged as `2d48bd2c`; I161 is Complete and I162 is Active/Claimed from `main@38127228`. |
| WORK-001-A / I196 | Ready / Planned / Claimed / priority hold | Claim PR #226 merged as `453d1fba`; resume only after v0.8.0 publication closure and a fresh exact-main inventory. |
| ARCH-034-D / I171 | Complete | Completion Commit `c88c1d1a`; current v0.7.0 audit/register and bounded owner creation validated; no production refactor. |
| MEM-010 | Ready | Select one bounded safety iteration and preserve existing session/memory behavior. |
| TUI-043 | Ready | Select a bounded compatibility-display iteration; preserve legitimate assistant text and ordered tool rows. |
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
| SKILL-004 | Intake — Issue #155 | Optional `SKILL.md` triggers compatibility requires a public-contract decision, parser fixtures, and a separately claimed runnable iteration. [Owner](active/SKILL-004-optional-skill-triggers-compatibility.md) |
| ARCH-034-R04 AG-8/9/10 | Refinement — unclaimed review residuals | Independent review `5230395611` approved #177 head `4b968823`; [AG-8](active/ARCH-034-R04-AG8-symbol-workspace-path-containment.md), [AG-9](active/ARCH-034-R04-AG9-symbol-decoding-consistency.md) and [AG-10](active/ARCH-034-R04-AG10-symbol-notice-admissibility.md) own the non-blocking path/decoding/notice findings without expanding I182. |
| ARCH-034-R04-AG11 | Refinement — unclaimed SQLite containment residual | [AG-11](active/ARCH-034-R04-AG11-sqlite-containment-evidence.md) owns five-consumer corrupt/busy/locked/panic/deadline evidence classification discovered by I183; no runtime remediation is authorized. |
| ARCH-034-R04-AG12 / I185 | Complete | Completion Commit `af9783229bfc8ee592813440ecfcdb6efc90a3c2`; exact head `45f70802`, CI `31556720252`, independent review `5261491057`. No consumer or runtime change. |
| GOV-004 | Refinement — shared-account reviewer attestation | Preserve independent natural-person review while defining explicit shared-account disclosure and mechanically honest validation. [Owner](active/GOV-004-shared-account-review-attestation.md) |
| GOV-005 / I190 | Complete | Trusted-base fail-closed routing merged at `a69ffa30`; real reduced probe merged at `01721f68` after run `31564461023` proved stable retained/skipped checks. [Story](active/GOV-005-change-aware-ci-routing.md) / [Iteration](../iterations/I190-change-aware-ci-routing.md) |
| GOV-006 | Refinement — unclaimed CI path-case residual | Normalize the `docs/sop/` full-route exclusion across path-letter case and add adversarial fixtures without broadening the completed GOV-005 allowlist. [Owner](active/GOV-006-ci-doc-path-case-normalization.md) |
| TOOL-026 / I191 | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213` and independent review `5274917099`. TOOL-024/I188 remains independent and unactivated. [Story](active/TOOL-026-noninteractive-terminal-containment.md) / [Iteration](../iterations/I191-noninteractive-terminal-containment.md) |
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
- TOOL-024-A decision → TOOL-024-B supervisor → TOOL-024-C `process` tool → TOOL-024-D cross-platform acceptance;
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
Issue #49 remains open under RUNTIME-005 Refinement; SESSION-008 durable partial persistence and
RUNTIME-005-A/B/C are the non-circular closure chain. Issue #59 remains open under TOOL-024;
TOOL-024 consumes completed RUNTIME-005 finalization instead of blocking it, and production spawn
also waits for PERM-006-C. Issue #134 closes only after the reviewed governance closeout reaches
`main`; TUI-046-A and I186/TUI-046-B are Complete with two-terminal acceptance and existing merge
evidence.
