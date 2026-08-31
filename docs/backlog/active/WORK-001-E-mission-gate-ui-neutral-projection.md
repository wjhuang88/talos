# WORK-001-E: Mission Final Gate And UI-Neutral Projection

| Field | Value |
|---|---|
| Story ID | WORK-001-E |
| Type | Runtime / State / Projection Story |
| Parent Epic | WORK-001 |
| Priority | P0 |
| Status | Ready |
| Source | GitHub Issue #29; DESKTOP-001 prerequisite chain section 20.5 |
| Selected Iteration | I240 Planned |
| Depends On | WORK-001-D / I239 Complete; WORK-001-C / I238; DESKTOP-001 direction; ADR-061 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned — proposed P4 boundary only |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | To be recorded on the atomic claim PR; claim and activation are ineffective until merge |
| Implementation PR | Not started |
| Last Updated | 2026-08-31 |
| Handoff / Release Condition | Establish effective claim and I240 activation before implementation; complete only with end-to-end evidence and independent review |

## Identity / Goal / Value

Provide the shared runtime gate that turns independently evaluated Goals into a separately
evaluated Mission and a Delivery-eligible result. Later Desktop and Dashboard clients must consume
this projection rather than inventing a second Mission or evaluation authority.

## Scope

- Add a Mission-level final-evaluation gate over the existing Work Domain, Completion Claim and
  independent evaluator contracts.
- Ensure Goal `PASS` results alone never create Delivery; required Goals, current revisions and a
  Mission evaluation must all be valid before Delivery becomes eligible.
- Represent rework and stale child results explicitly and fail closed when a required result is
  missing, stale, failed, inconclusive or conflicting.
- Expose the minimum UI-neutral state and event projection needed by later clients, reconciling
  existing `talos-conversation` and CLI `tui_bridge` ownership.
- Add a non-GPUI end-to-end fixture/walkthrough covering WorkUnit completion, Completion Claim,
  independent Evaluation, rework/staleness, Mission evaluation, Delivery gating and TUI
  compatibility.
- Update shared runtime/API documentation with the projection contract and migration boundary.

## Exclusions

- No `talos-desktop`, GPUI/native renderer, localization, windowing or Desktop UI.
- No Dashboard files or Dashboard behavior.
- No SESSION-009 attach/reconnect/multi-client implementation.
- No `/auto` or permission-policy expansion, sandbox fallback, or new authority class.
- No new durable persistence schema unless a separately accepted ADR and migration contract exists;
  the initial slice must remain storage-neutral or use an explicitly documented adapter.
- No release, version, tag, publication or dependency work.
- No generic workflow scheduler, global event bus or unrestricted multi-agent framework.

## Dependencies And Decision Constraints

- Reuse P1 Work Domain/Todo compatibility and P2 Completion Claim/Evaluation types; do not create
  parallel state or duplicate verdict semantics.
- Reuse P3 independent evaluator admission and evidence boundary; executor, validator and evaluator
  remain distinct authorities.
- Preserve stable identity/revision, permission, credential, transcript and session boundaries.
- UI-neutral events must be serializable and presentation-independent; locale/layout/cursor state is
  not Mission identity.
- Existing TUI behavior remains available through an adapter; GPUI types cannot enter shared crates.
- Any public API or persistent behavior change requires an ADR/change-control record before coding.

## Acceptance For Behavior

- Given every required Goal has a current criterion-level `PASS`, when Mission evaluation has not
  passed, then Delivery remains ineligible.
- Given a Goal is mutated after evaluation, when Mission gating runs, then its prior result is stale
  and Delivery is denied until a new claim/evaluation completes.
- Given a required Goal is missing, failed, inconclusive, conflicting or bound to another revision,
  when the Mission gate runs, then it returns an explicit non-deliverable outcome without mutation.
- Given all required Goals and the Mission evaluation pass at the same revision, when the projection
  is emitted, then it exposes a deterministic Delivery-eligible state and ordered UI-neutral events.
- Given the existing CLI/TUI bridge consumes the projection, when a non-GPUI walkthrough runs, then
  current TUI-compatible behavior is preserved without a Desktop dependency.

## Acceptance For Technical Work

- [ ] Mission gate and Delivery eligibility are implemented against existing P1-P3 contracts.
- [ ] Criterion/revision/staleness and fail-closed cases have focused tests.
- [ ] Projection state/events have serialization and ordering tests and one documented adapter path.
- [ ] A runnable non-GPUI end-to-end test or transcript covers the full P4 sequence.
- [ ] Public/API documentation names the projection, compatibility and migration boundary.
- [ ] Locked focused/workspace validation, governance validators, `git diff --check`, exact-head CI
      and independent runtime/security review pass before Complete.

## State / Status Owners

- Story scope, claim and completion: this owner document.
- Iteration execution and baseline: `docs/iterations/I240-work001-e-mission-gate-projection.md`.
- Parent dependency order: `WORK-001-goal-oriented-work-evaluation-foundation.md`.
- Product direction: `DESKTOP-001-desktop-product-direction.md`.
- Derived views: Board, Product Backlog, iterations README and manifest only.

## User-Facing Documentation

Update `docs/reference/WORK-EVALUATION-API.md` and the relevant runtime/TUI integration guidance;
do not describe Desktop behavior as shipped.

## Required Reads

- `docs/proposals/talos-desktop-goal-oriented-workspace.md` section 20
- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/WORK-001-C-completion-claim-evaluation-state.md`
- `docs/backlog/active/WORK-001-D-independent-evaluator-runtime.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/decisions/061-canonical-work-domain-and-todo-migration.md`

## Residual Destination

Desktop real binding, durable Mission/session storage, multi-client attachment and localized UI
remain separately governed downstream work. Any newly discovered authority or persistence gap is
recorded as a new child or change-control item rather than folded into P4.
