# PERM-007-H: Auto Review Across All Built-In Tools

| Field | Value |
|---|---|
| Story ID | PERM-007-H |
| Type | Permission / Security Story |
| Status | Refinement / Unclaimed |
| Source | Maintainer request 2026-10-11: automatic approval must cover all built-in tools, not only script execution |
| Selected Iteration | Remaining full built-in coverage after I300; not activated |
| Depends On | I300 truthful Git effects; existing Agent-owned permission pipeline and shared evaluator |

## Outcome

Auto mode evaluates built-in tool requests consistently by their actual effects,
user intent and bound resources. Do not force an otherwise assessable operation
to manual approval solely because its tool name is absent from a shell-specific
allowlist. This is an implementation requirement, not only an inventory task.
The maintainer accepted this full native-tool scope. An operation requiring a
human-only boundary remains visible in the inventory with its precise reason;
the acceptance does not authorize a model to override explicit Deny, an
unenforceable effect boundary, or a hard safety constraint.

## Current Evidence

`talos-agent/src/auto_resolver.rs::project_auto_request` currently dispatches only
to `eligible`, `eligible_exec` and `eligible_bash`. The first permits a narrow
native `write` text-create case. Therefore Auto is not exclusively scripts today,
but it does not provide general built-in coverage. Inventory production tool
registration and all approval entrypoints before final design.

## Acceptance And Required Design

- Enumerate every registered built-in family and operation, including feature-gated
  tools. Every operation has a tested disposition: existing policy allow/deny,
  model-assisted decision, or an explicit mandatory-human boundary with rationale.
  An arbitrary name allowlist or generic “not eligible” is not complete coverage.
- Reuse one shared model-review flow with operation-specific typed evidence,
  rather than copying shell logic or adding command/tool-name special cases.
  Cover file create/edit/delete, Git, process, network and other native tools
  according to their actual schemas/effects; exact inventory controls completeness.
- Existing deterministic allow decisions need no extra model call. For requests
  that require approval and permit automatic resolution, supply relevant user
  intent, arguments, resource scope and bounded effect evidence to the model.
  Assess multi-facet operations as a whole; partial approval cannot execute them.
- Model allow is invocation-bound, scope-limited and subject to authoritative
  admission/execution checks. Explicit Deny and hard safety constraints remain
  non-overridable. All built-in coverage does not mean unconditional approval.
- Uncertainty, insufficient evidence, unsupported enforcement or model failure
  produces a clear reason and concrete human decision points. Preserve headless
  behavior truthfully; do not invent an interactive approval capability.
- Display whether the model was consulted, its decision and useful rationale.
  Preserve shared evaluator isolation, untrusted-data handling, secret redaction,
  cancellation and configured model/request behavior; no per-tool extra model
  round-trip for requests already allowed by policy.
- Verify production registration → permission → model review → execution, with
  allow/deny/uncertain/error/cancel/stale-evidence tests, across applicable hosts.
  Include representative non-shell operations and a coverage check detecting new
  built-ins that lack a declared/tested disposition.
- Treat coverage as a registry-level invariant, including feature-gated production
  registrations: a newly registered built-in without review evidence/disposition
  must fail the coverage check, not silently fall back to a shell-only allowlist.
  Existing policy-allowed read-only calls must remain free of extra model latency.
- Update Auto user documentation and tool behavior/migration notes; independent
  permission/security/API review and normal exact-head gates remain mandatory.

## Related Scope And Compatibility

[PERM-007-G](PERM-007-G-higher-risk-auto-writes.md) / #563 already tracks
higher-risk diff-aware writes, backup/rollback and impact analysis. Reconcile that
owner before implementation: reuse its requirements and update ownership explicitly,
do not create duplicate implementation or silently waive its existing human gate.
All built-ins enter the coverage inventory; permanently excluding higher-risk
families simply to claim universal coverage is not acceptable.

Existing per-operation mandatory-human policies require an explicit reviewed
policy decision before model-assisted allowance can replace them. Record the
concrete policy/API delta after investigation, not a speculative blanket waiver.
MCP/plugin tools are not automatically included by this native-tool request.
No release, new dependency, new unsafe code or arbitrary permanent grant is
authorized by this intake. The complete handoff retains this added deliverable.

## I300 prerequisite sequencing amendment — 2026-10-23

I300 proposes ownership of only shared prepared-command assessment and native
Pull admission needed to satisfy its existing Auto activation acceptance. This
amendment takes effect only on target-branch merge of the governance candidate.
It removes the T7-before-T8 dependency cycle without reducing this Story's full
registry coverage, other built-in operations, or higher-risk policy requirements.
PERM-007-H remains Refinement / Unclaimed; reuse and verify I300's shared path
when this Story is activated. No generic all-built-in Auto success is claimed.
