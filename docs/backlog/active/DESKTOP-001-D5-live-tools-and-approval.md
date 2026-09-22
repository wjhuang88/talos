# DESKTOP-001-D5: Desktop Tools Approval And Cancellation

> Document status: Planned

| Field | Value |
|---|---|
| Story ID | DESKTOP-001-D5 |
| Parent Epic | DESKTOP-001 |
| Type | Desktop integration / behavior Story |
| Priority | P1 |
| Status | Planned / Unclaimed |
| Selected Iteration | I283 |
| Source | #29; four-week Desktop task |
| Depends On | I282 implementation merged and technical gates passed; deferred human rows tracked; current permission/Auto contracts |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | DESKTOP-001-D5 / I283: Desktop Tools Approval And Cancellation; planned scope only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable until activation is effective |
| Authorization Mode | Not applicable |
| Authorization Evidence | I282 implementation merged as `aaa4c015`; activation remains proposed in #601 |
| Implementation PR | Not started |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Establish an effective serial child claim before implementation; no release or new permission authority |

## Outcome

A Desktop user can execute real file/shell work, understand Auto decisions, allow or deny exact requests, and cancel safely.

## Scope

- Compose existing Runtime tools and permission policy; show call identity, progress, output and real terminal outcome without duplicating execution authority.
- Bridge existing scoped ApprovalHandler and explanation surfaces to an accessible localized UI; preserve existing choice semantics and explicit Deny.
- Expose existing Auto decision/reason when provided, including model-not-consulted and human-needed states. Do not expand Auto or add a second classifier.
- Bind UI responses to one pending request and task; stale, duplicate, closed-channel or cancelled replies cannot authorize later work.
- Make explicit cancel work during model, tool and approval states; distinguish interruption requested from confirmed stopped, with no silent replay.

## Acceptance

- Given a read-only tool call, display its true call/result association and exactly one terminal result.
- Given an Ask request, show tool, bounded scope and available explanation; Once/Session/Deny produce existing Runtime semantics without UI-invented grants.
- Given a denied write or cancelled prompt, no write occurs; late/double replies cannot approve another request.
- Given Auto evaluates an eligible request, the UI exposes the actual decision status without claiming a model was consulted when it was not.
- Given a running tool or pending approval, cancel and shutdown terminate observation/request lifetimes safely; tests prove no replay of side effects.

## Validation And Risks

Deterministic fake tools/provider tests for Allow/Deny/Once/Session, Auto visibility, concurrent same-name IDs, stale replies, cancellation and shutdown; protected permission/security/API review; native rows H2-H3.

Approval identity/cancellation races are protected blockers. Keep tool mode unavailable if failed; do not fall back to unconditional allow.

## Required Reads

- [Four-week task](../../tasks/2026-09-22-desktop-four-week-delivery.md)
- [Desktop parent](DESKTOP-001-desktop-product-direction.md)
- [Renderer boundary](../../decisions/059-desktop-renderer-host-motion-boundary.md)
- [Visual baseline](../../design/talos-desktop/DESIGN.md)
- [Localization](../../design/talos-desktop/I18N.md)
- [Shared work foundation](WORK-001-goal-oriented-work-evaluation-foundation.md)
- [Runtime SDK migration](../../reference/I280-RUNTIME-FACADE-MIGRATION.md)

## Documentation And Residuals

Update the Desktop crate README (created in I282), bilingual user-facing instructions as behavior
lands, and this owner before derived views. Preserve #29 and the four-week task acceptance ledger.
New permission policy, shell safety heuristics, sandbox fallback changes, CLI/TUI behavior changes, multi-client approvals or persistent broad grants.

## Completion Evidence

Completion Commit: pending.
No implementation or acceptance evidence exists for this planned child.
