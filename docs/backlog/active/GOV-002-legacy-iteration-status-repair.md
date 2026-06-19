# GOV-002: Legacy Iteration Status And Supersession Repair

| Field | Value |
|---|---|
| Type | Governance Story |
| Priority | P1 |
| Status | Complete (2026-06-19) |
| Depends On | None |

## Failure Mode And Value

The iteration index still labels I010 Active and leaves I012/I016/I017 Planned even though later
records delivered, split, or superseded substantial portions of those baselines. This can make an
Agent select stale work or bypass the mandatory non-terminal inventory.

The maintainer needs the original baselines and actual later execution mapped without rewriting
history, so iteration selection and dependencies become trustworthy again.

## Scope

- Recover and preserve the published objectives/dependencies for I010, I012, I016, and I017.
- Record actual replacement/delivery work and explicit Complete, Deferred, or Superseded status.
- Revisit downstream dependencies and active backlog rows affected by those dispositions.
- Synchronize iteration files, iteration index, Board, Product Backlog, Manifest, and any roadmap
  entries that still treat stale states as current.

## Exclusions

- Implementing residual feature work from those iterations.
- Renumbering historical iterations or deleting their plans.

## Acceptance

- [x] Git history/current owner docs establish the original baseline and actual later execution for
      each of I010, I012, I016, and I017.
- [x] Every item receives an explicit disposition without erasing its published target.
- [x] Dependent plans and backlog rows no longer assume an unmet or already-superseded prerequisite.
- [x] `.agent-governance/manifest.yaml` returns to `conformant` only after validator and semantic
      consistency review pass.
- [x] `scripts/validate_project_governance.sh .` passes and the non-terminal inventory is updated.
- [x] Any remaining uncertainty is registered rather than guessed.

## Verification

- Git history established I010 R2/R3 completion commits and the I012 split decision.
- I025/I026 owner records established later native-tool and Git delivery without retroactively
  claiming I016/I017 execution.
- Original iteration plans remain intact; only status headers and appended disposition records changed.
- `scripts/validate_project_governance.sh .`: passed with 0 warnings after Manifest synchronization.

## State Owners

- Iteration owner files and `docs/iterations/README.md`
- `docs/backlog/PRODUCT-BACKLOG.md`
- `docs/BOARD.md`
- `.agent-governance/manifest.yaml`

## Required Reads

- `docs/iterations/I010-polished-agent.md`
- `docs/iterations/I012-portable-tools.md`
- `docs/iterations/I016-portable-file-search.md`
- `docs/iterations/I017-embedded-git-tools.md`
- `docs/iterations/I025-tool-pipeline-completion.md`
- `docs/iterations/I026-approval-ux-doc-validation.md`
- `docs/backlog/active/TOOL-001-portable-file-search.md`
- `docs/backlog/active/GIT-001-embedded-git-tools.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/CHANGE-CONTROL.md`
