# Iteration I316: Model-Selectable Process Memory Limit

> Document status: Active (effective only on claim PR #854 merge)
> Published plan date: 2026-11-05
> Planned objective: complete handoff T6 across existing shell/exec execution modes.
> Baseline rule: preserve full acceptance; platform gaps cannot be closed by wording.
> MVP deliverable: model-provided per-process limits with tested inheritance, enforcement/failure evidence and unchanged 2048 MiB default policy.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6-sol |
| Work Slice | PROCESS-001 / handoff T6: typed process memory policy, shell/exec foreground/background/composition propagation, native enforcement and acceptance |
| Claimed At | 2026-11-05 |
| Source Issue | #853 |
| Governance Claim PR | #854 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer authorized the complete ordered handoff; finalized independent native/API review and effective target-branch claim still required |
| Implementation PR | Not started |
| Last Updated | 2026-11-05 |
| Handoff / Release Condition | No implementation before finalized claim/ADR merge; no release authority |

## Published Baseline

Selected Story [PROCESS-001](../backlog/active/PROCESS-001-model-process-memory-limit.md)
owns every T6 acceptance item. [ADR-098](../decisions/098-model-process-memory-limits.md)
defines proposed native policy and next-minor migration; independent acceptance is
required before implementation. No model/conversation-memory scope is included.

### Inventory and dependencies

Baseline `cc36fb1bc5f2d495ddaea472642896701e98dca5`: I315 Complete/Closed,
implementation #847 `add669403`, closeout #852, source #844 closed. Full predecessors
I309 → I308 → MODEL-014 → T4 → T5 → T1 are complete; only T6 remains in this queue.

| Owner | State | Disposition |
|---|---|---|
| I310 | Active | Independent Gemini wire/authentication; retain ownership |
| I302 | Review | Self-only Windows PSS diagnostics, no production launcher claim; retain |
| I277 | Review | Deferred human/device acceptance; retain |
| I249 | Planned / Unclaimed | Deferred dependency pilot; no dependency upgrade |
| I164 | Paused | Retain historical superseded layout disposition |
| I315 | Complete / Closed | Consume merged lifecycle/API evidence; no reopening |

All iteration status headers were inventoried; no Blocked header was found.
Open search intake PRs #849/#851 remain Proposed/Unclaimed and non-overlapping;
their temporary registration in #852 grants no implementation authority. Refresh
inventory and open PR/Issue overlap during claim CAS, not from this snapshot alone.

### Implementation and validation

Map actual final spawn owners before coding: Unix shell, direct exec (currently
missing hardening), background duplicate hardening, and Windows Job-based paths.
Propagate one resolved typed policy with omission/explicit evidence and per-step
inheritance. Reuse lifecycle owners; no new parallel launcher or permission bypass.

Run schema/invalid input, actual permission proof, default/override/inheritance,
foreground/background/sequential/parallel/pipeline, native allocation denial,
parent isolation and cancellation/cleanup regressions from PROCESS-001. Pinned
locked workspace check/Clippy/tests and standard preflight with both SDK modes;
native Linux/Windows CI, macOS exact-host evidence, independent security/API review,
exact-head CAS/merge and owner-first Completion Commit are required.

### Documentation / risks / rollback

Update README en/zh, tool descriptions/schemas, SDK input migration, ADR-098,
story/iteration/index/Board/handoff and #853. Explicitly distinguish RLIMIT_AS and
Windows committed memory, omitted best-effort policy and confirmed enforcement.
Windows default enforcement tightens historical behavior; next-minor migration
must disclose it. Roll back policy via reviewed commit, never by removing permission
or cleanup safeguards. No release, provider probing or aggregate tree quota.

## Actual activation and evidence

Proposed atomic activation in #854; ineffective until target-branch merge. Independent
API/native-security design review approved the complete contract on 2026-11-05,
including explicit inheritance and pre-exec safe evidence requirements. Exact-head
protected review and CI remain required. No code implementation, tests or Completion
Commit exists. Full handoff remains Active.
