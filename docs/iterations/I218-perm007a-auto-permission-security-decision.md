# Iteration I218: Auto Permission Security Decision

> Document status: Planned / Unclaimed
> Published plan date: 2026-08-22
> Planned objective: decide the bounded security contract for cross-surface model-assisted `auto`
> permission decisions without changing executable behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed threat matrix and accepted ADR make PERM-007-B/C/D
> separately runnable while PERM-006 remains the authoritative implementation prerequisite.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-22 |
| Work Slice | Decide only PERM-007-A / I218: threat model and one ADR revising or superseding ADR-011, including eligible decisions, maximum authority, mode precedence, privacy, validation, audit, deadline, circuit-breaker, migration, rollback and bounded B-D implementation children. No Rust/Cargo/config schema, `/auto`, model request, prompt, grant, approval, runtime, sandbox, TOOL-024, Desktop, release or publication implementation. |
| Claimed At | Not applicable |
| Source Issue | #188 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Pending exact-head independent security review and target-branch merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Claim and activation must merge before decision work. ADR acceptance requires independent exact-head security review; implementation waits for all PERM-006 A-C gates and separate child claims. |

## Published Baseline

Planning target: `main@20cfcce4e72be3da4e3efc1190ee498975e7476b`.

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I218 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I189 | Active / Claimed | Continue independently; its structured-decision implementation does not decide `auto` policy. |
| I197, I198, I201, I210 | Review | Preserve their corrective owners; no authority transfer. |
| I206-I208 | Planned / Unclaimed | Preserve the steering sequence. |
| I213 | Planned / Claimed | Dashboard lane remains independent and unactivated. |
| I218 | Planned / Unclaimed | Prepare only the decision/threat-model claim. |

PRs #120/#121 remain archival Drafts. The maintainer explicitly authorizes this non-overlapping
decision track early so later unattended work does not wait on the ADR; this does not authorize
behavior or relax PERM-006 dependencies.

### Selected Story And Scope

PERM-007-A produces a current-path/threat matrix, one ADR revising or superseding ADR-011, and
bounded B-D implementation boundaries. It changes no Rust, Cargo, dependency, config schema,
command, model request, permission result, sandbox, product UI, release or publication behavior.

### Acceptance And Validation

- Every eligible and ineligible risk class has an explicit maximum authority and fail-closed path.
- Global/session/headless precedence, privacy, output validation, audit, timeout, circuit-breaker,
  migration and rollback semantics are deterministic.
- PERM-007-B/C/D are runnable/testable but remain blocked on PERM-006 and their own claims.
- Run both governance validators with an explicit `main` base, parse the manifest YAML, run
  `git diff --check`, preserve this Published Baseline, and obtain exact-head security review.
- CI validates the stable documentation candidate; no Rust validation is claimed as behavior
  evidence for this decision-only slice.

### Risks And Rollback

- Risk: model judgment becomes a second authority or converts a scoped Ask into a broad grant.
- Risk: prompts/audit expose secrets or untrusted tool text injects the classifier.
- Rollback: reject the ADR, leave ADR-011 authoritative and PERM-007-B/C/D blocked; runtime behavior
  remains unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-22 | Selection | The maintainer requested early ADR completion for later unattended work. I218 is decision-only and non-overlapping with Active I189; this draft creates no authority before merge. |

## Verification Evidence

Pending claim merge, decision execution, exact-head CI and independent security review.

## Completion Evidence

- Completion Commit: pending
- A later status-only closeout cannot certify itself; it must reference the pre-existing reviewed
  decision commit.

## Variance And Residuals

No behavior variance is authorized. PERM-006-A/B/C and PERM-007-B/C/D retain separate ownership.
