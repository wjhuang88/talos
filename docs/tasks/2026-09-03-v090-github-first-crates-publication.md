# v0.9.0 GitHub-First Crates Publication Task

**Status**: Complete / Closed (2026-09-04)
> Created: 2026-09-03
> Candidate release: v0.9.0
> Current base: claim merge `main@c6e453a49dff12397d80335242e8291bff239938`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release session` |
| Work Slice | Coordinate only REL-005/I245 from release candidate through GitHub-first publication, Cargo publication, external verification and owner-first closeout. |
| Claimed At | 2026-09-03 |
| Source Issue | None |
| Governance Claim PR | #479 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly requested a complete GitHub and crates.io release on 2026-09-03; independent exact-head release review is retained for irreversible execution. |
| Implementation PR | #480 |
| Last Updated | 2026-09-04 |
| Handoff / Release Condition | Closed after GitHub-first v0.9.0 publication, Cargo closure and external acceptance. |

## Closure Ledger

Requested outcome: complete one GitHub and crates.io release.

Artifacts to update: REL-005/I245/task owners, workspace/internal versions, README/site release
surfaces, v0.9.0 notes, Cargo lockfile, Board/backlog/index/manifest and final evidence.

Existing assets to preserve: v0.8.0 immutable artifacts and historical owners, I164 paused state,
I207/I208 planned scope, `talos-models` quarantine, RUNTIME-006 and REL-002 residual ownership.

Validation required: versioned release preflight, exact-head CI, independent release review,
merge-time CAS, GitHub asset/checksum completion, registry visibility and external CLI/runtime tests.

Residual-work destination: patch-release owner for immutable failures; RUNTIME-006 and REL-002 for
their existing outcomes.

## Ordered Task Items

| ID | Task | Completion Gate | Status |
|---|---|---|---|
| V090-00 | Establish claim and activation | Finalized claim PR, validators, exact-head CI and CAS | Done — #479 merge `c6e453a4` |
| V090-10 | Prepare stable candidate locally | Version/release surfaces aligned; versioned preflight passes | Done — local preflight passed |
| V090-20 | Review and merge candidate | Exact-head CI, independent release review and CAS | Done - PR #480 merge `fad6e24e` |
| V090-30 | Publish GitHub Release | Immutable tag; five archives plus checksum | Done - tag `v0.9.0`, workflow `33825995467` |
| V090-40 | Publish Cargo closure | GitHub Release complete; all registry-enabled packages visible | Done - 20 packages at `0.9.0` |
| V090-50 | External acceptance and closeout | CLI install, runtime fixture and owner-first evidence pass | Done - CLI/runtime acceptance passed |

## Irreversible Action Rules

- Never push the tag before exact-head release review and merged-main versioned preflight pass.
- Never publish Cargo before the GitHub Release is complete.
- Never publish `talos-models`.
- Never move or force-push a failed tag; use a new patch version after a source-changing failure.
- Never overwrite or automatically yank a published crate version.

## 2026-09-03 Activation And Local-Convergence Checkpoint

Claim PR #479 merged as `c6e453a4` after exact-head CI `33766270164`, both governance validators
and single-maintainer CAS. The implementation branch starts from that merge. The first
`BUILD_MODELS=1 cargo build --locked -p talos-config` exposed that the refresher could not parse its
own omitted-false capability fields and silently discarded local variants. The local correction
defaults omitted capabilities to false and makes prior-catalog parse/read errors preserve the old
file. A clean rerun imported 6,473 upstream entries and retained the `k2p7` compatibility alias plus
all five reasoning variants, producing 6,474 packaged models. Normal builds remain offline. No
candidate has been pushed and no tag or package has been published.

## 2026-09-03 Stable Candidate Checkpoint

The complete versioned release preflight passed in the standard user filesystem environment after
the restricted agent sandbox correctly denied the shared-skill fixture's dedicated
`~/.agents/skills` write. All 21 package versions are `0.9.0`; 20 remain registry-enabled and
`talos-models` remains excluded. The model catalog contains 6,474 entries, five variants and the
`k2p7` compatibility alias. The candidate is ready for one stable PR; exact-head CI and independent
release review remain pending, and no immutable publication action has occurred.

## 2026-09-04 Closeout Checkpoint

PR #480 merged as `fad6e24e4b716b90b776b358d92ff69015688adf` after CI `33774242560` and
independent release review `5534058458` approved exact head `9f6a22a2`. Tag `v0.9.0`
points to the implementation commit; GitHub workflow `33825995467` published all five archives
and checksum. Cargo publication then completed for all 20 registry-enabled packages, with
`talos-models` excluded. External CLI installation returned `talos 0.9.0`; registry-only
runtime default and `coding` fixtures passed. Completion Commit:
`fad6e24e4b716b90b776b358d92ff69015688adf`.

## Completion Evidence

- Completion Commit: `fad6e24e4b716b90b776b358d92ff69015688adf`.
- The closeout documentation commit does not self-certify completion.
