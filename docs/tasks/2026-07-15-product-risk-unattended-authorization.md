# Unattended Authorization: Four-Month Product And Risk Plan

**Authority date**: 2026-07-15
**Authority source**: Maintainer direction to remove preconditions and delegate the four-month plan unattended.
**Applies to**: P100, P110, P120, P130, P140, and P150 in `2026-07-15-four-month-product-risk-plan.md`.

## Granted Authority

The assigned frontline developer may execute one package at a time, edit in-scope code/tests/docs, create and activate the next iteration only after its predecessor closes, commit logical changes, and push completed work to `main`. The developer must keep `Cargo.lock` locked and use the pinned toolchain.

## Pre-Authorized Defaults

| Situation | Default decision |
|---|---|
| P120 identifies a defect | Record an evidence-backed owner story and residual; do not implement a repair in P120. |
| P130 needs a design beyond existing boundaries | Publish an explicit Defer or Reject decision with research evidence; do not implement a task runtime. |
| P140 needs a design beyond existing boundaries | Publish an explicit Defer or Reject decision with the threat model; do not implement discovery or a protocol. |
| Browser evidence | Use the Codex in-app browser when available. If unavailable, `npx playwright install chromium` is authorized only for local test evidence; it must not alter Cargo dependencies or product artifacts. |
| A required gate fails twice unchanged | Record Blocked, preserve the worktree and checkpoint, and stop that package. |

## Still Prohibited

No new dependency, public semver-bound API, TLOG/session-format change, permission-policy change, remote/LAN bind, web write/action/approval route, release tag, publish, deployment, destructive Git operation, credential transmission, or external service account action is authorized. Any such need stops the current package and becomes a recorded residual.

## Completion Rule

P130 and P140 may complete with a documented Defer or Reject. The program may complete with those decisions and P120 residuals, provided P150 synchronizes all owner docs, Board, iteration index, GitHub issues, and recovery instructions.
