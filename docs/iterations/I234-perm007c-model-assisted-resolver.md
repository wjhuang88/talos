# Iteration I234: Bounded Model-Assisted Permission Resolver

> Document status: Active / Claimed (proposed; ineffective until this governance record merges)
> Published plan date: 2026-08-28
> Planned objective: deliver PERM-007-C under Accepted ADR-064 without allowing model output to bypass the authoritative permission pipeline.
> MVP deliverable: a runnable, fail-closed resolver path that evaluates only the ADR-064 create-only allowlist and returns a one-shot authorization or human fallback.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline permission session |
| Work Slice | PERM-007-C/I234 only: deterministic create-only eligibility, redacted evaluator request/output schemas, isolated single-call resolver, digest/revision/mode/session binding, audit report and circuit breaker at the PERM-006-C seam. No cross-surface D, existing-file modification, grants, sandbox fallback, Execute/Network, Dashboard, Desktop, release or publication. |
| Claimed At | 2026-08-28 |
| Source Issue | #188 |
| Governance Claim PR | #431 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-064 Accepted at `c129d4a5`; PERM-006-A/B/C and PERM-007-B are complete. Protected permission/security scope requires exact-head independent review before implementation merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-28 |
| Handoff / Release Condition | Require independent permission/security review, exact-head CI and merge-time CAS. PERM-007-D remains separately blocked. |

## Current Nonterminal Inventory And Disposition

This selection was checked against the current `main@c352435e` before claim preparation:

| State | Iterations | Disposition |
|---|---|---|
| Active | None | I234 is only proposed on this branch; it is ineffective until claim merge. |
| Review | None | No unresolved review iteration blocks this selection. |
| Planned | I207, I208 | Preserve as unclaimed steering children; do not activate or overlap. |
| Blocked | None with a current iteration document status | PERM-007-D and other backlog blockers retain their own owners; no authority transfers here. |
| Paused | I164 | Superseded startup target; do not restore. |

Historical terminal iterations and archival Draft PRs #120/#121 were inspected and do not own this
scope. Existing worktrees outside this branch are preserved and untouched.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-C | PERM-007 / Issue #188 | Ready / Unclaimed | ADR-064/I218; PERM-006-A/B/C; PERM-007-B/I233 | One bounded model-assessment seam that can only suggest `AllowOnce` for an eligible new text-file creation. |

### Scope

- Add a deterministic eligibility predicate for exactly one native workspace-local Write creating an absent structured text file under a typed managed-workspace lease.
- Add closed, versioned, redacted evaluator input/output schemas and validate digest, policy revision, mode generation, session and confidence before admission.
- Add one tool-free, non-recursive, no-retry evaluator request with an eight-second default deadline and thirty-second hard maximum.
- Return `AllowOnce` only for ADR-064's `bounded_workspace_text_create`; route uncertainty, malformed output, timeout, provider failure and ineligible requests to human approval or headless Deny.
- Bind execution to an open-parent capability and atomic no-clobber creation; preserve Deny precedence and existing human/headless adapters.
- Record redacted audit outcomes and open the session circuit after two technical/validation failures or three `HumanRequired` outcomes; `/auto on` is the only reset.

### Non-Goals

- No automatic modification, delete, rename, chmod, binary write, grant creation or policy generation.
- No Execute, Network, shell, sandbox fallback, external path, credential, plugin or MCP eligibility.
- No PERM-007-D cross-surface rollout/conformance, `/auto` command changes, Dashboard/Desktop, dependency, version, release or publication work.

### Acceptance

- Given policy `Deny` or a hard boundary, model output cannot produce execution authorization.
- Given an eligible absent-target create request and valid high-confidence response bound to the exact request, one `AllowOnce` authorization is admitted and consumed once.
- Given any ineligible operation, stale/replayed digest, changed policy/mode/session, malformed or injected output, timeout or provider error, no automatic authorization is produced.
- Given an interactive surface, fallback reaches the existing human resolver; headless and unsupported surfaces fail closed.
- Audit reports contain only bounded IDs/digests, classifications, versions, latency and outcome; no secrets, raw prompts, reasoning or full arguments.
- Circuit thresholds and explicit `/auto on` reset are deterministic and tested.
- The real `talos` write path exercises the resolver and proves atomic no-clobber behavior; unit-only types are insufficient.

### Planned Validation

- Focused permission-pipeline, resolver schema, redaction, timeout, replay, race and circuit tests.
- CLI/TUI/runtime compatibility tests proving existing `ApprovalResolver` behavior remains valid.
- A binary/integration fixture for eligible create, target-appears-after-assessment and headless fallback.
- `cargo fmt --all -- --check`, locked workspace check/Clippy/tests, release preflight, both governance validators and `git diff --check`.
- Exact-head CI and independent permission/security/API review; merge-time CAS immediately before merge.

### Documentation To Update

- PERM-007-C story and parent, ADR-064 implementation evidence, README permission/auto behavior, Board, backlog, iteration index, manifest and Issue #188.

### Risks And Rollback

- Risk: evaluator context leaks secrets, a stale result is replayed, or the write path races with a parent/target swap.
- Rollback: disable `auto.enabled`, issue `/auto off`, or remove the resolver; the existing human/headless-deny path remains authoritative and no grants require migration.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-28 | Governance preparation | Claim and Active state are proposed in this governance branch and have no effect until the finalized claim merges to `main`. Implementation must start from that merge or later. |
| 2026-08-28 | Remote reconciliation | Issue #188 owner/status reconciliation comment `5453556697` records PERM-007-C/I234 as proposed Active/Claimed and ineffective until claim merge. |
| 2026-08-28 | Remote reconciliation correction | Comment `5453584031` records the matrix status `In Progress` and exact relative owner path required by the reconciliation validator. |
| 2026-08-29 | I235 handoff accepted | I235/PERM-007-C0 decision closeout merged as `55acce9b` with Completion Commit `71acbe0c`; ADR-064 now permits a separately reviewed `cap-std` implementation. |
| 2026-08-29 | Local implementation checkpoint | Local commits add the `cap-std` 4.0.3 directory-capability creator, atomic no-clobber tests, shared `WriteTool` capability injection, and optional Runtime shared-composition injection. Candidate remains local; no implementation PR or production model assessor wiring has been submitted. |
| 2026-08-29 | Local TUI composition checkpoint | Local commits `d5bb9d7c` and `b3500cd4` wire the provider-backed resolver and the same optional atomic-create capability into TUI product composition when `auto.enabled` is true. Runtime SDK, MCP, print/headless defaults, Dashboard and Desktop remain unchanged. These commits are local evidence only; exact-head CI, independent security review and merge remain pending. |

## Verification Evidence

- Preflight source audit: current `ApprovalResolver` returns legacy `ApprovalChoice`; `WriteTool` uses check-then-write and therefore requires capability-bound atomic creation before entering the allowlist.
- Local candidate validation: `cargo test -p talos-agent --lib --locked` (298 passed), `cargo test -p talos-tools --features file-write --lib --locked` (107 passed), `cargo check -p talos-tools --features file-write --locked`, and `cargo check -p talos-runtime --features shared-composition --locked` passed. A concurrent full workspace run was interrupted by `ENOSPC`; it must be rerun after the candidate is complete.
- Stable-candidate validation: workspace `cargo check --workspace --locked -j2`, workspace Clippy with `-D warnings`, `./scripts/release_preflight.sh`, both governance validators and `git diff --check` passed. `cargo test --workspace --locked -j2` completed all runnable suites; only the two macOS Seatbelt tests failed because this restricted execution environment denies `sandbox-exec` (`Operation not permitted`). A low-debug, single-job `cargo test -p talos-cli --locked -j1` completed 358 unit tests and all CLI integration suites successfully.

## Completion Evidence

- Completion Commit: Pending. A status-only governance commit cannot self-certify implementation.

## Variance And Residuals

- PERM-007-D remains separately blocked until C is complete and requires its own claim and cross-surface evidence.
