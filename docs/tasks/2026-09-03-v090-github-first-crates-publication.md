# v0.9.0 GitHub-First Crates Publication Task

> Status: Active / Claimed (proposed; ineffective until claim PR merge)
> Created: 2026-09-03
> Candidate release: v0.9.0
> Current base: `main@336c65e08c49c0e95334a8d967bb8a1648aa3a8f`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | `@wjhuang88` |
| Executing Agent | `Codex / GPT-5 mainline release session` |
| Work Slice | Coordinate only REL-005/I245 from release candidate through GitHub-first publication, Cargo publication, external verification and owner-first closeout. |
| Claimed At | 2026-09-03 |
| Source Issue | None |
| Governance Claim PR | #479 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly requested a complete GitHub and crates.io release on 2026-09-03; independent exact-head release review is retained for irreversible execution. |
| Implementation PR | Not started |
| Last Updated | 2026-09-03 |
| Handoff / Release Condition | Effective only after claim PR merge; complete only after GitHub Release, Cargo closure and external acceptance all pass. |

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
| V090-00 | Establish claim and activation | Finalized claim PR, validators, exact-head CI and CAS | In progress |
| V090-10 | Prepare stable candidate locally | Version/release surfaces aligned; versioned preflight passes | Blocked on V090-00 |
| V090-20 | Review and merge candidate | Exact-head CI, independent release review and CAS | Blocked on V090-10 |
| V090-30 | Publish GitHub Release | Immutable tag; five archives plus checksum | Blocked on V090-20 |
| V090-40 | Publish Cargo closure | GitHub Release complete; all registry-enabled packages visible | Blocked on V090-30 |
| V090-50 | External acceptance and closeout | CLI install, runtime fixture and owner-first evidence pass | Blocked on V090-40 |

## Irreversible Action Rules

- Never push the tag before exact-head release review and merged-main versioned preflight pass.
- Never publish Cargo before the GitHub Release is complete.
- Never publish `talos-models`.
- Never move or force-push a failed tag; use a new patch version after a source-changing failure.
- Never overwrite or automatically yank a published crate version.
