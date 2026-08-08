# ARCH-034-R11: Architecture Documentation Truth

> Document status: Complete

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Finding | ARCH-034-F26 |
| Status | Complete |
| Priority | P1 |
| Selected Iteration | I180 (Complete; implementation PR #171) |
| Preserved behavior | Documentation-only; historical evidence remains immutable |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Reconcile `docs/reference/ARCHITECTURE.md` current-state workspace, crate, CLI, tool-contribution, extension, and composition descriptions against root `Cargo.toml` and current source; explicitly distinguish historical iteration-era snapshots from current facts; update directly affected architecture indexes/registers with non-semantic factual status only; preserve every runtime/API/dependency/decision/security behavior and route any ADR-007/R0 semantic or process-hardening conclusion to ARCH-034-R04. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #170 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, scale assessment, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #171 |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Closed after implementation PR #171 merged; any later architecture fact correction needs its own bounded owner, while ADR-007/R0 semantic or process-hardening changes remain blocked on independent R04 security review. |

## Problem And Boundary

Current-state architecture text omits the implemented tool contribution model and retains stale CLI
boundary wording. ADR-007/R0 status text also needs factual reconciliation, but security semantics
cannot be changed without R04 review.

## Scope

- Update current-state crate/composition/extension documentation from source evidence.
- Label historical snapshots as historical and link the August audit.
- Route ADR/process-hardening semantic edits through R04; make only non-semantic factual fixes here.

## Exclusions

- No decision reversal, history rewrite, production edit, behavior claim, or security policy change.

## Acceptance And Validation

- Architecture extension paths match current source and R01 exception verdicts.
- Historical audit/iteration baselines remain intact and current facts are not backdated.
- DOC-CHECK, governance/claim validators, scale assessment, link/search checks, and diff checks pass.

## Rollback / Residual

Revert inaccurate prose. Any decision change belongs to a new ADR or R04 security review.

## Review Evidence

- Source-to-document trace covers the root workspace member list, CLI composition modules,
  `talos-tools`/`talos-session` contribution sources, plugin manifest/WASM carrier, MCP session
  startup, direct internal dependency edges, and the R01 scheduler/status exceptions.
- `docs/reference/ARCHITECTURE.md` was updated with factual current-state boundaries and explicit
  historical labels. No production, test, dependency, API, permission, sandbox, or process-
  hardening files changed.
- ADR-007/R0 semantics were not edited; security interpretation remains the R04 residual.
- Standard release preflight, both governance validators, scale assessment, architecture audit
  harness, stale-claim search, scope check, and `git diff --check` passed locally on 2026-08-08.
- Source implementation commit `fd8ac75d` was submitted in PR #171 before the accepted exact head
  and squash merge recorded below.

## Completion Evidence

- Completion Commit: `10cceec6aeb9089fe9c830355992c8fc60430d63`
- Implementation PR #171 squash-merged at `10cceec6aeb9089fe9c830355992c8fc60430d63`
  from source implementation `fd8ac75d` and accepted exact Head
  `b9f4716862940d6e125854ed3a70a7742f922f61`.
- Exact-head CI `31238721507` passed Unix release preflight, Windows workspace and focused checks,
  governance, remote owner reconciliation, rebuilt CLI smoke, and installer fixture jobs.
- Merge-time CAS confirmed base `1ca03fdf8d262eba4d1de2374e43f2c1a94882dd`, exact head
  `b9f4716862940d6e125854ed3a70a7742f922f61`, no blocking feedback or overlapping claim or
  implementation PR, and unchanged recovery PR #120/#121 heads.
