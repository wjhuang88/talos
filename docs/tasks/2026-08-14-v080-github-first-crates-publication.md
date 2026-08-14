# v0.8.0 GitHub-First Crates Publication Long Task

> Status: Planned
> Created: 2026-08-14
> Candidate release: v0.8.0
> Current base: `main@453d1fba97470639835468664c58397770db384c`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Coordinate the ordered I159-I162 readiness chain and I203 release without inheriting any child implementation authority. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Maintainer requested this release before I196 implementation, required CLI and runtime Cargo publication, and fixed GitHub-before-Cargo ordering. Each child still needs its own effective claim. |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Coordinate only after this record reaches `main`; implementation and irreversible actions remain gated by each child owner and claim. |

## Closure Ledger

Requested outcome: release Talos before I196 implementation, with GitHub Release completed before
Cargo publication, `talos-cli` installable as the `talos` binary, and `talos-runtime` published as
the SDK facade.

Artifacts to create or update: ARCH-031/A-D, I159-I162, REL-003/I203, release surfaces, package
manifests/guards, publication matrix, runtime SDK contract and external fixtures.

Existing assets to preserve: I159-I162 published baselines, I196 effective P0 claim, I188/I189/I195
independent claims, #227 planning, Dashboard/Desktop lane boundaries, `talos-models` quarantine,
stashes, recovery PRs #120/#121 and all user worktrees.

State/status owners to synchronize: child Story then iteration, ARCH-031, this task, Product
Backlog, iteration index, Board, manifest and relevant Issues.

Validation required: each child acceptance matrix; locked workspace/release preflight; governance;
GitHub assets/checksums; crates.io visibility; external Cargo install/runtime fixtures.

Evidence and uncertainty: v0.8.0 is the next minor after v0.7.0 and is selected for the new public
distribution surface. Registry ownership/rate limits and the final metadata graph must be
reconfirmed immediately before irreversible execution.

Residual-work destination: RUNTIME-006/#234 for the stronger single-direct-dependency facade;
REL-002 for stable qualification; a new patch release task for any immutable release failure.

## Outcome

Complete I159-I162 in their published order, then execute I203 so GitHub owns the first public
v0.8.0 release event and crates.io publication follows only after the GitHub Release is complete.
After closure, resume I196 from its effective claim on then-current `main` through a fresh exact-head
inventory.

## In Scope

- I159 feature boundary;
- I160 shared composition;
- I161 sandbox fallback/coding preset with independent security review;
- I162 external fixture, metadata closure, publish guards/package readiness and GO packet;
- I203 release commit/tag/GitHub workflow, then Cargo publication and external verification;
- owner-first status, Issue and derived-view synchronization.

## Out Of Scope

- I196 P0 implementation until this task closes;
- RUNTIME-006/#234 implementation;
- Desktop or Dashboard work;
- `talos-models` publication;
- v1/REL-002 qualification;
- activation of I188, I189, I195 or proposed I197-I201.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| V080-00 | Establish release plan and first claim | Target-branch plan plus effective I159 claim | None | Exact-head governance/CI, independent review and CAS | Keep all implementation blocked | Done — claim merge `fa635b4e` |
| V080-10 | Execute I159 | Lightweight `talos-tools` default with product parity | V080-00 | I159 acceptance and Completion Commit | Record blocker; do not skip to I160 | Active |
| V080-20 | Execute I160 | One shared internal CLI/runtime composition | V080-10 | I160 acceptance and Completion Commit | Record blocker; do not skip to I161 | Planned |
| V080-30 | Execute I161 | Fail-closed fallback and explicit coding preset | V080-20 | Security review, runtime matrix and Completion Commit | Record blocker; do not skip to I162 | Planned |
| V080-40 | Execute I162 | External fixtures, 20-package dry-runs and GO/NO-GO packet | V080-30 | I162 Completion Commit and explicit GO | Stop before release on NO-GO | Planned |
| V080-50 | GitHub v0.8.0 release | Immutable tag, five assets and checksums | V080-40 | GitHub workflow and Release complete | No Cargo publish; use patch after repair | Planned |
| V080-60 | Cargo publication | 20 visible crates.io packages in dependency order | V080-50 | Per-package visibility and no omitted closure package | Checkpoint partial state; never overwrite | Planned |
| V080-70 | External acceptance and closeout | Cargo install, runtime fixture and owner evidence | V080-60 | External tests plus owner-first closeout | Keep Review/Blocked with exact residual | Planned |

## Branch, Worktree And Checkpoint Plan

- Each child claim starts from the then-current `origin/main` in its own isolated worktree.
- Each implementation starts only after its claim is effective on `main`.
- Append a checkpoint after every task item and after every registry publication wave.
- Never reuse #227, I188 or Dashboard worktrees for release implementation.

## Allowed Permissions And External Actions

The maintainer authorized a release, including GitHub and crates.io publication, and later required
GitHub completion first. Network reads, branches, PRs, commits, pushes, the annotated tag, GitHub
workflow observation and dependency-ordered `cargo publish` are within the long-task outcome only
after their owner gates pass. This authorization does not waive exact-version/package confirmation,
credentials, independent review, CI, CAS or the GitHub-first barrier.

## Destructive Or Irreversible Operations

- Tag and crates.io versions are immutable.
- Never force-push or move a tag.
- Never publish `talos-models`.
- Stop before the first real publish if GitHub v0.8.0 is incomplete or package scope differs.
- After a source-changing failure, use a new patch version rather than retrying the same tag/version.

## Time, Cost And Resource Limits

- No paid external service is authorized.
- Respect GitHub/crates.io rate limits; checkpoint rather than busy retry.
- Keep build artifacts bounded and clean only task-owned generated output when space is constrained.

## Default Decisions For Foreseeable Ambiguity

- Version: v0.8.0, the next minor after v0.7.0 for a new distribution surface.
- Scope: 20 packages in the CLI/runtime closure; exclude only `talos-models`.
- Order: GitHub Release complete first; Cargo dependency waves second; external acceptance last.
- Failure: stop at the failed gate, preserve immutable evidence, and create a patch recovery owner if
  source changes are required.

## Checkpoints

### 2026-08-14 Planning Checkpoint

Completed task items: Issue #234 created; current facade limitation verified; latest main and open
PR/worktree inventory confirmed; existing I159-I162 chain identified as mandatory rather than
duplicated.

Current state and artifacts: planning/claim PR preparation only. No Rust, Cargo manifest, version,
publish guard, tag or release mutation.

Commands/checks and actual results: `git fetch origin` confirmed
`main@453d1fba97470639835468664c58397770db384c`; workspace branch clean; metadata currently yields
the intended 20-package closure excluding `talos-models`.

Open risks or deviations: I159 Story decisions must be finalized and its claim merged before
implementation. Registry ownership/version availability requires fresh external confirmation.

Review correction: PR #235 review at head `4cd5d6868b42f7efafccf117c78e30173addef01`
found that the proposed default `file-read` ownership for `document_extract` omitted its
unconditional existing `scraper 0.27` dependency. ARCH-031-A change control now assigns the whole
tool to a default-off `document` feature requiring `file-read` and included by `coding`; related
`tree`, `search_engine`, and `browser_page` dependency attributions were also corrected. This is a
planning-only correction and does not activate I159 or modify Cargo/Rust code.

Next task item: obtain exact-head CI and independent review for finalized I159 claim PR #235, then
repeat merge-time CAS before it reaches `main`.

Recovery or resume instruction: read this task, ARCH-031-A, I159 and the claim PR exact head; do not
start code until the claim reaches current `main`.

### 2026-08-14 I159 Activation Checkpoint

Completed task items: V080-00. PR #235 head
`11619e13ca6c854b4db737a9978767436a19ab9f` passed CI `31789567122`, independent approval
`5292115807`, both governance validators and merge-time CAS, then merged as
`fa635b4eaadd4b55939322f89acfda4522489ab7`.

Current state and artifacts: I159/ARCH-031-A is the sole Active iteration and Draft implementation
PR #236 starts from the claim merge on `feat/tools-I159-feature-boundary`. I188/I189/I195 remain
Planned/Claimed, I196 remains on release priority hold, I160-I162/I203 remain Blocked, and I164
remains Paused. No other open PR owns the I159 slice.

Commands/checks and actual results: post-merge project governance and Collaboration Claim
validators passed with 0 warnings; `origin/main` and the implementation branch baseline were both
`fa635b4eaadd4b55939322f89acfda4522489ab7`.

Open risks or deviations: the feature skeleton must be reconciled with the exact dependency graph,
including shared `scraper` ownership and the pre-existing 0.22/0.27 duplication. Do not treat a
dependency-version cleanup as implicitly authorized by I159.

Next task item: capture the exact Cargo/module/downstream baseline, create the draft implementation
PR, then implement the minimum feature gates and run the full build/product-parity matrix.

Recovery or resume instruction: use only `/private/tmp/talos-i159-impl`; do not touch I188,
Dashboard, I196, recovery branches, stashes, tags or registries.
