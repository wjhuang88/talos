# 2026-07-15 Four-Month Product And Risk Plan

**Status**: Ready for assignment; no iteration activated.
**Timebox**: 2026-08-01 through 2026-11-30.
**Execution package**: `docs/tasks/2026-07-15-product-risk-execution-package.md`.

## Objective

Turn WEB-001's loopback snapshot API into a useful read-only browser surface, deliver the smallest TUI input-history improvement, and convert the newly synchronized high-risk issues into reviewed decisions or evidence-backed follow-up work. This plan does not authorize release, remote control, permission-policy broadening, autonomous execution, or multi-instance networking.

## Pre-Activation Inventory

| Item | Recorded state | Disposition |
|---|---|---|
| I018, I019, I020 | Historical Planned baselines | Deferred; do not rewrite or bypass them. Record their disposition again in any activation record. |
| WEB-001 | Partial | Select first for read-only rendered pages only. |
| TUI-030 | Refinement | Run a code-level readiness review before selecting the in-memory slice. |
| TOOL-021 | Refinement | Evidence-first audit; any repair receives a new owner story. |
| TASK-001, A2A-001 | Refinement, ADR-gated | Decision packages only; no runtime/protocol implementation. |
| REL-002 | Planned, NO-GO | Not selected; this plan cannot qualify a release. |

## Delivery Matrix

| Month | Package | Deliverable | Exit gate |
|---|---|---|---|
| August | P100 / WEB-001 | Loopback pages render status, history, governance, and masked config from existing snapshot routes. | Browser/runtime evidence; redaction regressions; locked validation ladder. |
| September | P110 / TUI-030; P120 / TOOL-021 | In-memory composer history with draft restoration; reviewed tool-error flow matrix and fixtures. | TUI runtime evidence; audit disposition and targeted fixtures. |
| October | P130 / TASK-001 | ADR or explicit defer/reject on task/session ownership, recovery, retention, and fresh permission evaluation. | Security/architecture review; no task engine. |
| November | P140 / A2A-001; P150 closeout | ADR or explicit defer/reject on inter-instance boundaries; synced residuals and next-selection packet. | Threat-model review; owner/Board/index/issue synchronization. |

## Package Contracts

### P100 — WEB-001 rendered dashboard

- Render existing loopback snapshot data with accessible navigation and deterministic empty/error states.
- Preserve loopback-only operation, redaction-before-serialization, and existing runtime lifecycle.
- Do not add config/session mutation, approvals, WebSocket/SSE, remote binding, or a new web dependency without a new owner/ADR.
- Acceptance: a real browser sees pages, not only JSON/plain text; secrets, headers, tokens, and raw provider responses are absent from HTML and route payloads.

### P110 — TUI-030 composer input history

- Inspect composer, approval, and slash-command key dispatch before activation; append exact module findings to the owner doc.
- Scope: process-local in-memory Up/Down history, duplicate policy, boundaries, and exact draft restoration.
- Acceptance: semantic-buffer or runtime evidence proves navigation and draft restoration without changing transcript/session persistence or input priority.

### P120 — TOOL-021 error-propagation audit

- Trace tool result/error data through execution, agent messages, context, and supported provider serializers.
- Fixture matrix: expected non-zero, execution error, paired/orphan result, retry, and resume.
- Acceptance: every route has an observed outcome; silent loss is a finding, never success. A fix needs a separately selected story.
- This is an explicit infrastructure/security-evidence exception to the normal vertical-slice rule.

### P130 — TASK-001 persistent-task spike

- Decide task/turn/session identity, Talos-owned checkpoints, crash recovery, cancellation, retention, cleanup, and permission re-authorization after resume.
- Acceptance: ADR or explicit defer/reject reviewed against ADR-006 and permission constraints.
- Non-goals: scheduler, daemon, direct tool path, permission reuse, and multi-agent orchestration.

### P140 — A2A-001 multi-instance spike

- Threat model: identity, authentication, authorization, discovery, capability advertisement, retention, revocation, credentials, and transcript exposure.
- Compare explicit host-managed connections with automatic discovery.
- Acceptance: ADR or explicit defer/reject; no global event bus or implicit authority.
- Non-goals: automatic discovery, network service, protocol implementation, credential transfer, and multi-agent orchestration.

### P150 — Closeout

- Synchronize owner docs, Board, iteration index, README when user behavior changed, issue comments, ADR index, and explicit residual owners.
- Publish one next-selection recommendation: bounded TOOL-021 repair, bounded task slice, explicit A2A defer/reject, or another ready backlog item.

## Common Validation

Code-bearing packages record actual results for `cargo fmt --all -- --check`, `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`, `cargo test --workspace --locked`, `./scripts/release_preflight.sh`, `scripts/validate_project_governance.sh .`, and `git diff --check`. User-visible work also requires real binary/browser evidence. Decision-only packages run governance and targeted fixture checks and explicitly record why no runtime behavior changed.

## Success At 16 Weeks

- WEB-001 is a rendered read-only loopback dashboard, not merely an API/link page.
- TUI-030 is delivered with runtime evidence or explicitly rejected with a recorded reason.
- TOOL-021 has reviewed evidence and any repair has its own owner.
- TASK-001 and A2A-001 have an architecture/security disposition.
- No credentials, raw provider payloads, approvals, or remote-control capabilities were added to dashboard or transcript surfaces.
