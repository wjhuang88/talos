# Product Backlog

This file is the compact current-state backlog entrypoint. Story scope, acceptance criteria,
implementation evidence, and residual ownership live in the linked owner documents.

The complete pre-closeout index is preserved unchanged at
[`PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md`](PRODUCT-BACKLOG-pre-I170-closeout-2026-08-01.md).
That snapshot is historical evidence, not current activation authority.

## Current Priorities

| Priority | Focus | Current State / Gate | Required Reads |
|---|---|---|---|
| 0 | v0.6 Runtime Productization Program | I158 is Review, not Complete. Resolve scheduler/status contribution ownership and final architecture/tool-extension documentation. TUI-037 must be dispositioned after I158 reaches Complete or Paused; I159-I162 remain Blocked. | `docs/tasks/2026-07-28-four-month-v06-execution-package.md`; `docs/iterations/I158-tool-registration-composition.md`; `docs/backlog/active/TUI-037-dashboard-logo-link.md`; ADR-053 |
| 0 | Internal validation service | Validation must become an internal callable, language-neutral service; governance must not depend on shell scripts and project adapters must be detected before guidance is injected. | `docs/backlog/active/VALIDATION-001-internal-validation-service.md`; `docs/backlog/active/GOV-003-builtin-project-governance.md`; `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md` |
| 0 | Permission pipeline convergence | PERM-006 remains Refinement. Select and review A→E children sequentially; no child may broaden PERM-004/PERM-005 policy implicitly. | `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`; PERM-006-A/B/C/D/E |
| 0 | Memory admission safety | MEM-010 is Ready but unselected. A bounded correction iteration must prove only user-authored episodes enter new global memory. | `docs/backlog/active/MEM-010-user-origin-memory-admission.md`; Issue #114 |
| 1 | TUI-044 transactional batched steering | **Active in PR #131 review handoff.** Implement only on `feat/i169-tui-044-transactional-steering` synchronized with current `main`. ADR-056 remains Proposed; recovery PR #120 remains immutable. | `docs/backlog/active/TUI-044-transactional-batched-steering-turn.md`; `docs/iterations/I169-batched-steering-turn.md`; `docs/decisions/056-transactional-steering-submission-boundary.md`; Issue #119; PR #131 |
| 1 | TUI regression intake | TUI-043 is Ready; TUI-041, TUI-042, TUI-045 and TUI-046 remain Refinement and require bounded layout/state-transition or terminal-interaction evidence before selection. | `docs/backlog/active/TUI-041-thinking-preview-wrap-and-height.md`; `docs/backlog/active/TUI-042-noop-history-scroll-stability.md`; `docs/backlog/active/TUI-043-tool-placeholder-suppression.md`; `docs/backlog/active/TUI-045-permission-prompt-layout-anchor.md`; `docs/backlog/active/TUI-046-native-text-selection-copy.md` |
| 1 | Runtime session and protocol foundations | SESSION-009 and RUNTIME-005 remain Refinement; ACP-001 remains Blocked until session attachment, controller ownership and shutdown/finalization boundaries are accepted. | `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`; `docs/backlog/active/RUNTIME-005-bounded-graceful-shutdown.md`; `docs/backlog/active/ACP-001-agent-client-protocol-server.md` |
| 1 | Memory scope architecture | MEM-011 remains Refinement. Accept schema/migration and legacy-fixture decisions before implementation. | `docs/backlog/active/MEM-011-extensible-memory-scopes.md`; Issue #116 |
| 1 | Provider/runtime reliability follow-ups | Preserve explicit terminal outcomes, usage accounting and bounded request/stream behavior before dependent status/cost UX. | `docs/backlog/active/PROVIDER-001-openai-streaming-usage.md`; `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md`; `docs/backlog/active/PROVIDER-004-text-tool-call-id-collision.md` |
| 2 | Dynamic provider authentication program | PROVIDER-003 / Issue #132 is a Refinement Epic only. PROVIDER-003-A must accept the capability ADR and threat model before any child can be selected; B/C own shared lifecycle/request contracts and D-G own bounded provider/acquisition slices. | `docs/backlog/active/PROVIDER-003-dynamic-provider-credentials.md`; ADR-013; ADR-023; ADR-057; Issue #132 |
| 2 | TOOL-023 residual configuration work | TOOL-023-A/C are Complete through I170. TOOL-023-B alone owns the 300-second default/configuration proposal and is not implemented by I170. | `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md`; `docs/backlog/active/TOOL-023-B-configurable-timeout-default.md` |
| 3 | Deferred product architecture | Desktop, multi-agent, health monitoring, persistent tasks and A2A remain Deferred/Refinement until explicitly reprioritized and ADR-gated. | DESKTOP-001; AGENT-003; RUNTIME-004; TASK-001; A2A-001 |

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
| TUI-044 / I169 | Active — PR #131 review handoff | Remain Active until structured transaction/journal/lifecycle/request/replay implementation, exact-head CI, rebuilt real-TUI evidence and independent ADR-056 review reach Review. |
| MEM-010 | Ready | Select one bounded safety iteration and preserve existing session/memory behavior. |
| TUI-043 | Ready | Select a bounded compatibility-display iteration; preserve legitimate assistant text and ordered tool rows. |
| TOOL-023-B | Ready | Separate timeout-default/configuration change; do not reopen completed A/C behavior. |
| PROVIDER-004 | Ready | Force unique text-path tool-call IDs and prove cross-turn pairing; keep separate from shell/runtime timeouts. |

## Review / Partial Items

| ID | State | Required Disposition |
|---|---|---|
| I158 / ARCH-034-R01 | Review | Resolve scheduler/status contribution exceptions and final documentation before Complete/Paused. |
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
- RUNTIME-005 and PERM-006-C before background-job completion claims;
- ADR/migration acceptance before MEM-011;
- PROVIDER-003-A before B/C, then one bounded D-G provider/acquisition child at a time;
- TUI-046 interaction policy and ADR-054 disposition before any native-selection implementation;
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

Issue #119 remains open and Active in PR #131 review handoff. Completion still requires fresh
independent acceptance, maintainer merge authorization, merge evidence and recorded lifecycle closeout.
Issue #132 remains open under the PROVIDER-003 Refinement Epic; no child implementation is authorized
until PROVIDER-003-A and a separately claimed child owner establish the required boundary.
Issue #134 remains open under TUI-046 Refinement; terminal interaction policy, ADR-054 impact,
iteration selection and a real-terminal validation matrix are still required.

## I169 review synchronization (2026-08-04)

- TUI-044 / I169 remain **Active**; ADR-056 remains **Proposed**; Issue #119 remains **Open**.
- PR #131 now carries an atomic durable generation fence plus awaited old Scheduler/Actor retirement before G+1 publication, with production-path race, reconstruction, journal, Bridge, receipt-generation, stale-command, and Provider-call evidence.
- This synchronization records implementation and review evidence only. It does not claim Complete, Accepted, Approved, merge-ready, or merged status; exact-head CI and a new independent review remain required.
