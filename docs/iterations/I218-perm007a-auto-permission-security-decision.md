# Iteration I218: Auto Permission Security Decision

> Document status: Complete / Closed
> Published plan date: 2026-08-22
> Planned objective: decide the bounded security contract for cross-surface model-assisted `auto`
> permission decisions without changing executable behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an independently reviewed threat matrix and accepted ADR make PERM-007-B/C/D
> separately runnable while PERM-006 remains the authoritative implementation prerequisite.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline governance session 2026-08-22 |
| Work Slice | Decide only PERM-007-A / I218: threat model and one ADR revising or superseding ADR-011, including eligible decisions, maximum authority, mode precedence, privacy, validation, audit, deadline, circuit-breaker, migration, rollback and bounded B-D implementation children. No Rust/Cargo/config schema, `/auto`, model request, prompt, grant, approval, runtime, sandbox, TOOL-024, Desktop, release or publication implementation. |
| Claimed At | 2026-08-22 |
| Source Issue | #188 |
| Governance Claim PR | #352 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #352 merged as `ca30081a`. Decision PR #353 exact head `a289a07f` passed CI `32505438495`, independent Agent-role security review `5372825090` and merge-time CAS, then merged as `c129d4a5`. |
| Implementation PR | #353 (decision documentation only) |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Closed at Completion Commit `a289a07f`; ADR-064 is Accepted. Implementation waits for all PERM-006 A-C gates and separate child claims. |

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
| 2026-08-22 | Claim proposed | PR #352 atomically proposes Claimed/Active state. It remains ineffective before target-branch merge and authorizes no executable behavior. |
| 2026-08-22 | Claim effective | Exact head `13ecbdfa` passed CI `32503441611`, independent Agent-role review `5372605087` and CAS; PR #352 merged as `ca30081a`. |
| 2026-08-22 | Decision execution | PR #353 carries the current-path/threat matrix and Proposed ADR-064: default-on attempted assistance, a narrow one-shot atomic no-clobber text-create class, deterministic exclusions, redaction, timeout/circuit, cross-surface and migration boundaries. No executable behavior changes. |
| 2026-08-22 | Security correction | Exact-head review `5372757905` found a same-path modification TOCTOU. The first eligible class is now create-only with absent-target/path/parent binding and atomic no-clobber at the mutation point; existing-file modification requires a later portable CAS decision. |
| 2026-08-22 | Decision accepted | Corrected exact head `a289a07f` passed CI `32505438495`, independent Agent-role security review `5372825090` and CAS; PR #353 merged as `c129d4a5`. ADR-064 is Accepted and supersedes ADR-011 without authorizing executable behavior. |

## Verification Evidence

- Claim: exact head `13ecbdfa`, CI `32503441611`, independent review `5372605087`, merge
  `ca30081a`.
- Decision evidence: `docs/reference/I218-AUTO-PERMISSION-THREAT-MATRIX.md` and Accepted ADR-064 at
  exact head `a289a07f`; CI `32505438495`, independent security review `5372825090`, CAS and PR
  #353 merge `c129d4a5` passed.

## Completion Evidence

- Completion Commit: `a289a07ff97746d877f3a422d15f8044bbf50ab6`
- This pre-existing reviewed decision commit, not the status-only closeout, certifies completion.

## Variance And Residuals

No behavior variance is authorized. PERM-006-A/B/C and PERM-007-B/C/D retain separate ownership.
