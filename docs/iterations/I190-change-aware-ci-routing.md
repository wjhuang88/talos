# Iteration I190: Change-Aware CI Routing

> Document status: Complete
> Published plan date: 2026-08-12
> Planned objective: introduce a deterministic fail-closed change classifier so narrowly allowlisted documentation-only pull requests keep documentation/governance gates without running the complete Unix and Windows Rust workspaces.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a real documentation-only PR proves stable required-check results with Rust workspace jobs skipped, while adversarial fixtures prove every ambiguous or control/executable change retains full validation.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-12 |
| Work Slice | Implement only I190/GOV-005: deterministic fail-closed changed-path classification, stable pull-request CI routing, adversarial fixtures and route documentation. Keep full validation for every code, control-plane, executable, schema, fixture, dependency, binary, ambiguous or mixed change. No product/runtime behavior, release authorization, branch-protection administration, unrelated CI optimization, closeout or I188/I189 activation. |
| Claimed At | 2026-08-12 |
| Source Issue | None |
| Governance Claim PR | #201 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | PR #202 exact head `13b288ec8670e2536a2d46ccda4e3240fb2b30cf` passed full CI `31560789644`, received independent approval `5262374485`, passed merge-time CAS and merged as `a69ffa30afed16271885d4ef3d11931ab3189673`. Reduced-route probe PR #203 passed run `31564461023` and merged as `01721f683d0c09ad5f5f9e98360da15cd5155c48`. |
| Implementation PR | #202 |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | None - implementation and reduced-route probe are merged; GOV-006 separately owns case-normalized SOP exclusion matching. |

Completion Commit: `a69ffa30afed16271885d4ef3d11931ab3189673`, `01721f683d0c09ad5f5f9e98360da15cd5155c48`

## Published Baseline

### Selected Story

| Story | Status At Selection | Outcome |
|---|---|---|
| GOV-005 | New P0 maintainer correction; no existing owner or overlapping PR | Trusted fail-closed changed-path classification plus stable CI routing and adversarial fixtures |

### Scope, Non-Goals And Acceptance

The complete frozen contract is owned by
`docs/backlog/active/GOV-005-change-aware-ci-routing.md`. I190 implements exactly that contract.

### Planned Validation

- Classifier fixture matrix including docs-only, Rust/Cargo, workflow, SOP/AGENTS, script, schema,
  fixture, binary, rename/delete, missing-base, malformed and mixed changes.
- Both governance validators and `git diff --check`.
- Full `./scripts/release_preflight.sh` on the implementation head.
- Exact-head CI plus a real allowlisted documentation-only probe that shows stable skipped workspace
  checks and passing documentation/governance gates.

## Non-Terminal Coordination Record

- I185, I186 and I187 have merged implementation/decision evidence but remain Review until separate
  closeout commits cite `af978322`, `a5115f5c` and `e288afb5`; this story does not alter them.
- I188 and I189 remain Planned with effective claims; this P0 CI correction does not activate their
  decision or permission implementation.
- I159-I162 remain Blocked, I164 remains Paused, and recovery PRs #120/#121 remain archival.
- Open PR #198 is non-overlapping desktop documentation and supplies no authority or evidence here.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-12 | Selection | Maintainer promoted pure-documentation CI routing to immediate priority because the current unconditional matrix delays every governance synchronization and exposes unrelated flaky Rust tests. Claim remains ineffective before target-branch merge. |
| 2026-08-12 | Activation | Claim PR #201 passed exact-head CI and merge-time CAS, then merged to `main` as `bb38c262`. I190 activated without changing its published objective, scope, acceptance or Planned Validation. |
| 2026-08-12 | Implementation | PR #202 implements the trusted-base classifier, fail-closed full fallback, stable workspace job names, reduced documentation gates and adversarial fixtures. It remains Review pending exact-head full CI, independent review, merge-time CAS and a post-merge reduced-path probe. |
| 2026-08-12 | Merge | PR #202 exact head `13b288ec` passed full CI `31560789644`, independent approval `5262374485` and merge-time CAS, then merged as `a69ffa30`. |
| 2026-08-12 | Reduced probe | PR #203 exact head `ecf4ca77` classified one allowlisted reference Markdown path as reduced in run `31564461023`; all retained gates passed, Unix Rust setup/preflight was skipped, Windows allocated no runner and concluded `SKIPPED`, and the PR remained `MERGEABLE/CLEAN` before merge `01721f68`. |

## Verification Evidence

- Claim PR #201 exact head `bc131f93407bc7298a20ac9f72d0c897223ce7dd` passed all four
  pre-routing CI jobs; the Windows Rust workspace alone took 11m39s. Merge-time CAS confirmed the
  unchanged head/base, `MERGEABLE/CLEAN`, non-overlapping scope and valid claim before squash merge
  `bb38c262bc7cdfa5c1690101d4fa48857fd5db64`.
- Local implementation validation passed the dependency-free classifier matrix: allowlisted prose
  is reduced, while Rust/Cargo/workflow/SOP/script/schema/fixture/binary/mixed changes, malformed
  inputs, rename/copy/delete/type changes, non-UTF-8/binary content, symlink/executable Markdown and
  mode changes all fail closed to full validation.
- `scripts/validate_project_governance.sh .`,
  `bash scripts/validate_collaboration_claims.sh .`, public-site and installer validation, workflow
  YAML parsing and `git diff --check` passed. The final implementation-tree
  `./scripts/release_preflight.sh` also exited 0 after the mode-change fixture was added; exact-head
  CI and independent review were then satisfied on the exact implementation head.
- Actions run `31560789644` passed all five full-route jobs for PR #202 exact head `13b288ec`; review
  comment `5262374485` independently exercised 11 adversarial repositories and approved that head.
- Probe run `31564461023` logged `CI change classification: reduced (allowlisted documentation only
  (1 path(s)))`. The reduced validation, remote Issue/owner reconciliation and Windows installer
  fixture passed; Rust toolchain/cache/release-preflight steps and the Windows Rust workspace were
  skipped. The stable skipped Windows check was accepted by the current repository protection state,
  with PR #203 still reported `MERGEABLE/CLEAN`.

## Completion Evidence

- Completion Commit: `a69ffa30afed16271885d4ef3d11931ab3189673` (PR #202 implementation)
  and `01721f683d0c09ad5f5f9e98360da15cd5155c48` (PR #203 reduced-route probe).

## Variance And Residuals

- Windows config-lock test reliability and general CI performance work remain separately owned.
- PR #202 review finding F1 is registered as unclaimed GOV-006: normalize case when matching the
  `docs/sop/` full-route exclusion and add case-variant fixtures. This does not reopen I190.

## Retrospective

- Trusted-base classifier execution plus full validation on every push are the central safety
  controls; the narrow PR allowlist is an optimization layered beneath those controls.
- The claim and implementation full-route Windows jobs each cost roughly 11 minutes. The real
  reduced probe completed its Unix validation path in about 40 seconds and allocated no Windows
  Rust runner, removing that cost from eligible governance closeouts.
