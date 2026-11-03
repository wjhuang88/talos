# Iteration I315: Model Validation And Read-Only Agile Tools

> Document status: Active (proposed in #845; ineffective until target-branch merge)
> Published plan date: 2026-11-03
> Planned objective: complete handoff T1 using shared validation and supported read-only governance services.
> Baseline rule: preserve all published acceptance; no completion by documentation alone.
> MVP deliverable: a model discovers applicable tools, reads project status or plans checks, and executes supported validation with real permission, timeout, cancellation, bounded output and cleanup evidence.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 handoff closure session |
| Work Slice | VALIDATION-003 / handoff T1: shared model validation and read-only agile tools, CLI/Runtime host composition, bounded Unix/Windows execution and acceptance |
| Claimed At | 2026-11-03 |
| Source Issue | #844 |
| Governance Claim PR | #845 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized full ordered handoff delivery; independent API/process-security review, exact-head CI, validators and CAS required |
| Implementation PR | Not started |
| Last Updated | 2026-11-03 |
| Handoff / Release Condition | Claim and activation effective only after finalized governance PR merges; no release authority |

## Published Baseline

Selected child [VALIDATION-003](../backlog/active/VALIDATION-003-model-validation-agile-tools.md)
reuses VALIDATION-001 and GOV-003; it does not reopen the completed shared-service
slice or claim broader governance automation. All acceptance in that owner is binding.

### Preconditions and selection inventory

Target `58d8bf926beefe24df4e04c3d9c98f5ca62636cd`: I313/T5 implementation #837
is merged as `322574bc9707492ea6e9d5232794f73e687071d9`; closeout #843 merged,
Issue #390 closed. First five ordered handoff stages are complete. I315 was absent
from current owners and remote branch inventory. No overlapping validation PR found.

| Owner | State | Disposition |
|---|---|---|
| I310 / #838 | Active | Retain independent Gemini/authentication ownership; recheck composition overlap |
| I314 / #842 | Active | Retain independent Search ownership; no configuration/transport takeover |
| I302 | Review | Retain Windows self-only diagnostic owner; not launcher authorization |
| I277 | Review | Retain deferred human/device acceptance |
| I249 | Planned / Unclaimed | Retain deferred dependency pilot; no dependency upgrade |
| I164 | Paused | Retain superseded layout disposition |
| I313 | Complete / Closed | Consume merged T5 evidence; do not reopen |

No Blocked iteration header found. Nonstandard Status-table owners were also
checked; I263-I267/I274/I275 are terminal. Historical Active/Review references
inside completed owners are not new activation. T1 replaces this handoff's T5
execution slot and does not take over independently authorized I310/I314 work.

### Architecture and compatibility

Conversation retains project detection, plans, internal validation and evidence;
add a small injectable async host-check interface. Tools owns tool adapters and
process integration: Unix reuses sandbox OwnedCommandOperation; Windows reuses
existing Job-owned windows_foreground operation. No second subprocess runner or
conversation→tools dependency. Check tools→conversation dependency and feature
boundaries before implementation. Add a contribution factory without required
fields on public SharedToolContributions. Preserve bare RuntimeBuilder versus
explicit shared product composition. Migrate existing CLI host-run to the same
bounded adapter; internal /validate governance remains in-process. Record public
synchronous API compatibility; no silent loss of supported behavior. Any breaking
public API requires an accepted ADR/migration before coding that change.

### Validation and documentation

Pinned Rust 1.99.0, locked focused tests/check/Clippy, full standard
release_preflight.sh including external SDK modes, both governance validators,
independent API/process-security review, exact-head native CI and merge CAS.
Update README en/zh, tool reference, SDK/CLI usage, parent relationship notes,
owner/index/Board/handoff and Issue. Completion needs existing merged implementation
SHA and actual acceptance, not this claim. T6 remains next; full goal Active.

## Actual Activation And Execution

2026-11-03: governance-only draft from current main; no production implementation,
new dependency, implementation tests or effective activation in this record.

Finalized atomic proposal: #845 carries the actual PR number and proposed
Claimed/Active record. Both remain ineffective until this finalized head merges.
Independent review, validators, exact-head CI and CAS precede merge.
