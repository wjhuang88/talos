# GOV-005: Change-Aware CI Routing

| Field | Value |
|---|---|
| Story ID | GOV-005 |
| Type | Governance / CI Reliability Story |
| Priority | P0 |
| Status | Complete |
| Source | Maintainer priority correction, 2026-08-12 |
| Selected Iteration | I190 |
| Depends On | Existing required CI checks and release preflight remain authoritative for non-documentation changes |

Status: Complete

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
| Authorization Evidence | PR #202 exact head `13b288ec8670e2536a2d46ccda4e3240fb2b30cf` passed full CI run `31560789644`, received independent approval in review comment `5262374485`, passed merge-time CAS and merged as `a69ffa30afed16271885d4ef3d11931ab3189673`. Probe PR #203 then passed reduced-route run `31564461023` and merged as `01721f683d0c09ad5f5f9e98360da15cd5155c48`. |
| Implementation PR | #202 |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | None - implementation and reduced-route probe are merged; residual case-normalization work is independently owned by GOV-006. |

Completion Commits: `a69ffa30afed16271885d4ef3d11931ab3189673`, `01721f683d0c09ad5f5f9e98360da15cd5155c48`

## Goal And Value

Make pull-request validation proportional to change risk. A mechanically proven documentation-only
change should not compile and test the complete Rust workspace on Unix and Windows, while any code,
build, release, CI, governance-rule, executable script, schema, fixture, dependency, or ambiguous
change must retain the full validation matrix.

## Scope

- Add one deterministic, dependency-free changed-path classifier with explicit output consumed by
  the CI workflow.
- Define a narrow documentation-only allowlist and a fail-closed full-validation fallback.
- Keep governance, Markdown/link, remote Issue-owner reconciliation, whitespace, and applicable
  cross-platform documentation/script checks on the reduced path.
- Gate Unix release preflight and Windows Rust workspace jobs on the classifier result without
  removing their required-check visibility.
- Add positive, negative, rename/delete, missing-base, malformed-input, and bypass-attempt fixtures.
- Document which paths force full validation and how maintainers reproduce classification locally.

## Exclusions

- No weakening of validation for Rust, Cargo, lockfile, build/release, CI/workflow, `AGENTS.md`,
  SOP, governance validator, executable script, schema, fixture, generated asset, or ambiguous changes.
- No product/runtime behavior, dependency, public API, security policy, release authorization, or
  branch-protection administration change.
- No attempt to infer safety from PR title, label, author text, commit message, or GitHub actor.
- No I185/I186/I187 closeout, I188/I189 activation, or Issue #49/#59/#134 status change.

## Acceptance

- Given only allowlisted prose/reference documentation changes, classification is reduced and the
  Rust workspace jobs are represented as successful skips while documentation/governance gates run.
- Given any non-allowlisted, control-plane, executable, schema, fixture, binary, renamed-across-
  boundary, deleted control, missing-base, or malformed change set, classification is full.
- Given a pull request from a fork or a branch with multiple commits, classification uses trusted
  base/head repository data rather than executing changed repository code.
- Required check names remain stable and a reduced path cannot make a failing documentation,
  governance, remote-owner, or installer/document portability gate disappear.
- Exact classifier fixtures, local governance checks and one real reduced-path PR prove the route;
  full release preflight passes on the implementation head before merge.

## Validation

- Classifier unit/fixture matrix on Linux and Windows-compatible path inputs.
- `git diff --check` and YAML/workflow semantic review.
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `./scripts/release_preflight.sh`
- Exact-head CI and merge-time CAS.

## Residuals

- PR #202 review finding F1 is registered as unclaimed GOV-006: case variants such as `docs/SOP/`
  currently bypass the lowercase `docs/sop/` exclusion and receive reduced validation. GOV-006 owns
  case-normalized matching and adversarial fixtures without reopening this completed routing slice.
- The independent Windows config-lock timeout observed on PR #195 is not fixed by this story; code
  changes continue to exercise it through full CI and any reliability repair needs a separate owner.
- Further test sharding, caching, runner selection and branch-protection changes remain separate.

## Completion Evidence

- PR #202 exact head `13b288ec8670e2536a2d46ccda4e3240fb2b30cf` passed all five jobs in
  CI run `31560789644`, received independent APPROVE in comment `5262374485`, passed merge-time CAS
  and merged as `a69ffa30afed16271885d4ef3d11931ab3189673`.
- Probe PR #203 exact head `ecf4ca775ae0e188f401042a5890ab9367fe0aec` classified one allowlisted
  Markdown path as reduced in run `31564461023`; governance, remote-owner and installer gates passed,
  Unix Rust toolchain/cache/release-preflight steps were skipped, the Windows Rust workspace allocated
  no runner and concluded `SKIPPED`, and the PR remained `MERGEABLE/CLEAN`. It merged as
  `01721f683d0c09ad5f5f9e98360da15cd5155c48`.
