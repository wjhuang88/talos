# SEC-002: Dashboard Opt-In Token Delivery Boundary

**Status**: Refinement — Unclaimed; Selected Iteration: None
**Priority**: P2 — security/usability residual for a non-default mode
**Type**: Security decision Story
**Parent Epic**: WEB-001

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Select only after an ADR-backed threat model chooses a supported token-delivery or mode-deprecation contract; implementation requires a separate iteration and effective claim. |

## Identity / Goal / Value

An operator who explicitly sets `[dashboard] loopback_only = false` needs a supported way to obtain
the per-process bearer token without violating ADR-031's prohibition on config, logs, sessions and
history. Today the token is generated and enforced but has no compliant operator-delivery channel.

The goal is to decide whether Talos should provide a bounded ephemeral delivery mechanism or
deprecate/remove the opt-in mode. This owner prevents the pre-existing ADR-031 usability gap found
during I202 from dissolving into closeout prose.

## Scope

- Inventory the token generation, middleware, lifecycle and all potential operator-facing surfaces.
- Threat-model local process access, terminal/clipboard exposure, browser handoff, replay, lifetime,
  revocation and accidental persistence.
- Choose and accept one explicit contract: a safe ephemeral delivery channel, a different
  authentication design, or deprecation/removal with migration guidance.
- Define compatibility, rollback, documentation and deterministic security-test requirements for a
  later implementation Story.

## Exclusions

- No token display, logging, persistence, clipboard write, browser opener or route change in this
  Refinement Story.
- No remote/LAN/tunnel access, write-capable Dashboard action, session mutation or permission-model
  change.
- No reuse of I202/TUI-037 or WEB-001-A/I195 authorization.

## Dependencies

- ADR-031's accepted loopback Dashboard and memory-only token constraints.
- I202/TUI-037 Completion Commit
  `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772`, which removed the non-compliant token log without
  inventing a replacement.
- WEB-001 remains the parent Dashboard product direction; WEB-001-A/I195 is an independent
  read-only visual-shell slice and does not own this security contract.

## Decision Links And Constraints

- `docs/decisions/031-web-loopback-dashboard-boundary.md`: tokens must remain absent from config,
  logs, session files and history unless a future accepted decision explicitly revises the model.
- `docs/decisions/023-inline-api-key-boundary.md`: credential display masking is relevant evidence
  but does not itself authorize token delivery.
- Any public API, config compatibility or persistence change needs the normal ADR and migration
  treatment before implementation.

## Uncertainty And Validation Path

- It is not yet established that a terminal, clipboard, browser fragment, local file descriptor or
  IPC handoff can satisfy the threat model without creating a worse disclosure path.
- Refinement must compare alternatives and rejection reasons before the Story can become Ready.
- A later runnable implementation must prove authorized access, rejection without the token,
  lifetime/restart behavior and absence from every forbidden persistence/logging surface.

## State / Status Owners

- Security decision and readiness: this `SEC-002` owner.
- Existing Dashboard authentication behavior: ADR-031 / `talos-dashboard`.
- Product Dashboard direction: WEB-001; WEB-001-A remains separately governed.
- Selection and implementation evidence: a future dedicated iteration, not I202 or I195.

## User-Facing Documentation

- No current behavior documentation changes beyond truthfully recording that the opt-in mode has no
  supported token-delivery channel.
- A selected implementation or deprecation slice must update both `README.md` and
  `README.zh-CN.md` with the accepted operator workflow.

## Required Reads

- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/WEB-001-A-dashboard-read-only-visual-shell.md`
- `docs/backlog/active/TUI-037-dashboard-logo-link.md`
- `docs/iterations/I202-tui037-dashboard-logo-link.md`
- `crates/talos-dashboard/src/lib.rs`
- `crates/talos-cli/src/mode_runners.rs`

## Acceptance For Decision Work

- [ ] An accepted ADR-backed threat model selects delivery, redesign or deprecation and records why
  rejected alternatives fail.
- [ ] Compatibility, migration, rollback and user-documentation impacts are explicit.
- [ ] A runnable/testable implementation child is separately owned and selected before production
  changes begin.
- [ ] No implementation claim reuses I202 or WEB-001-A authorization.
