# Iteration I277: Desktop Mock Visual and Localization Slice

> Document status: Active
> Published plan date: 2026-09-16
> Planned objective: Deliver a fixture-backed, mock-only Desktop visual surface with bilingual localization.
> Baseline rule: preserve this scope; changed objectives use a new iteration ID.
> MVP deliverable: a runnable mock Desktop surface that renders the same fixture in `zh-CN` and `en-US`.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | DESKTOP-001-D3 mock-only visual/i18n slice |
| Claimed At | 2026-09-16 |
| Source Issue | #29 |
| Governance Claim PR | #570 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Proposed atomic claim+activation in #570; ineffective until merged to main. |
| Implementation PR | Not started |
| Last Updated | 2026-09-16 |
| Handoff / Release Condition | Atomic claim+activation required before code or Cargo changes. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| DESKTOP-001-D3 | DESKTOP-001 / #29 | Planned / Unclaimed | D0/I194 and WORK-001 P0-P4 | Mock-only bilingual visual surface |

### Scope

- Fixture-backed visual state, localization catalogs, fallback, keyboard/focus and IME validation.

### Non-Goals

- No real runtime/session/permission/evaluation binding, browser automation, or release packaging.

### Acceptance

- Given the same fixture, when locale changes, then visible labels change but fixture identity and state do not.
- Given an unavailable locale, then deterministic fallback is shown without network access.

### Planned Validation

- Bilingual snapshot/manual visual walkthrough and IME/focus checks.
- `./scripts/validate_project_governance.sh .` and `bash scripts/validate_collaboration_claims.sh .`.
- Exact-head CI and independent review after implementation.

### Documentation To Update

- Desktop owner, Board, backlog and user-facing setup documentation for the selected mock surface.

### Risks And Rollback

- Risk: renderer or localization dependency leaks into shared runtime crates.
- Rollback: remove the isolated Desktop slice while preserving all shared runtime contracts.

## Activation Gate

This iteration is proposed Active / Claimed. A governance-only PR must establish an effective claim
and activation before any implementation branch or Cargo/production change.
