# VALIDATION-002: Deferred Human Review And Acceptance Batch

| Field | Value |
|---|---|
| Story ID | VALIDATION-002 |
| Type | Governance / Human Validation Story |
| Priority | P0 within the mainline long-task closeout |
| Status | Ready - I211 Planned / Unclaimed |
| Source | [GitHub Issue #302](https://github.com/wjhuang88/talos/issues/302) |
| Selected Iteration | I211 - Planned / Unclaimed |
| Depends On | Terminal dispositions for I200, I197, I201, I212, I210 and I198; exact implementation heads recorded in Issue #302 |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #302 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | None - evidence-only validation slice |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | After I200, I197, I201, I212, I210 and I198 have terminal implementation dispositions, establish an effective evidence-only I211 claim and execute every Issue #302 row before the mainline long task can close. |

## Identity / Goal / Value

Provide one scheduled human validation phase for long-running work so unavailable natural-person
review or device-dependent acceptance does not idle the implementation queue, while preserving a
truthful record that those gates have not yet passed.

## Scope

- Bind each source iteration's exact implementation head, merge commit, CI and Agent technical
  review before accepting a deferred row.
- Obtain independent natural-person review for I200, I197, I201, I212, I210 and I198 against their recorded
  implementation heads.
- Run I200's mouse-wheel and touchpad terminal matrix on the integrated `main` build.
- Run the owner-defined human/manual matrices added by I197 and I201 when their implementations
  exist; record any applicable I198 compatibility inspection.
- Record OS, terminal, input device, integrated `main` head, result and corrective owner for every
  manual row.

## Exclusions And Hard Boundaries

- No product implementation, bug repair, permission-policy change, release, tag or publication.
- No deferral of exact-head CI, locked checks, independent Agent technical review, governance
  validation or merge-time CAS required before a child implementation merge.
- No deferral of independent security review required by `AGENTS.md` for sandbox,
  `talos-permission`, process-hardening or permission-policy changes.
- I197 remains presentation/layout-only. Any permission semantic or policy change stops and needs a
  separately authorized security-reviewed iteration.

## Acceptance

- Issue #302 identifies every included source iteration and exact implementation head.
- A natural-person reviewer records a per-head conclusion with shared-account identity limits
  disclosed where applicable.
- Every device/manual row records the integrated `main` head, environment and result.
- Passed rows are synchronized owner-first into their source Story/iteration before derived views.
- Failed rows leave the source owner in Review and name a separately governed corrective owner.
- The long-task closeout does not claim Complete while any required Issue #302 row is open.

## Change Control - 2026-08-19 MODEL-013 Priority Advance

The maintainer advanced MODEL-013/#312 into I212 before I198. Add one I212 natural-person
custom-provider walkthrough after its exact implementation head exists: verify an exact catalog
match visibly supplies a catalog-derived context window, an explicit value remains unchanged, and
an ambiguous/unknown identity stays manual/unknown without a network request. This extends only the
deferred validation inventory; it does not authorize I212 implementation or rewrite I211's
published baseline.

## Change Control - 2026-08-19 Provider Reconnect Status

After I210 has an implementation head, add a natural-person live-provider/mock-provider walkthrough
that observes initial `Connecting…`, at least one truthful `Reconnecting… (attempt n/m)` transition,
and clearing on success, terminal failure and cancellation. Record the integrated `main` head and
confirm the displayed counts match the structured retry facts. This row does not waive I210's ADR,
claim, exact-head CI, technical review or CAS gates.

## Residual Destination

Implementation defects found during the batch use new corrective Stories/iterations. Automation of
this scheduling mode belongs to a separately claimed governance improvement; I211 changes no SOP
validator or CI behavior.
