# v0.8.0 GitHub-First Crates Publication Long Task

> Status: In Progress
> Created: 2026-08-14
> Candidate release: v0.8.0
> Current base: `main@1b129c951df22a7de63e14735e02b1e8a79a9cd7`

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
| Implementation PR | #250 merged; #251 matrix-closure follow-up in review |
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
| V080-10 | Execute I159 | Lightweight `talos-tools` default with product parity | V080-00 | I159 acceptance and Completion Commit | Record blocker; do not skip to I160 | Done — PR #236 merge `f79c1ead` |
| V080-20 | Execute I160 | One shared internal CLI/runtime composition | V080-10 | I160 acceptance and Completion Commit | Record blocker; do not skip to I161 | Done — Completion Commit `0524e82f`; PR #240 merged as `97556149`, closeout PR #241 as `2d48bd2c` |
| V080-30 | Execute I161 | Fail-closed fallback and explicit coding preset | V080-20 | Security review, runtime matrix and Completion Commit | Record blocker; do not skip to I162 | Done — PR #250 merged `d2b4bdd1`; matrix-closure PR #251 merged `da5a43a2`; Completion Commits `74c5502d`/`3ca2ec62` |
| V080-40 | Execute I162 | External fixtures, 20-package dry-runs and GO/NO-GO packet | V080-30 | I162 Completion Commit and explicit GO | Stop before release on NO-GO | Planned / claim PR #253 pending merge |
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

### 2026-08-14 I159 Completion Checkpoint

Completed task items: V080-10. Implementation commits
`d886917e45d5ca0f110e111b966cd379485e3580` and
`34c09b142766c70ac62ef24424ed035f2fa921a5` deliver the feature boundary. Accepted head
`33a2c6ffad0e5c473baf41c14e704dfd19fcd0c9` passed CI `31801484313` 5/5 and independent approval
`5293622712`, then PR #236 merged after CAS as `f79c1ead1cd3a547797dea3666295f510d88a13d`.

Current state and artifacts: ARCH-031-A/I159 is Complete/Closed. ARCH-031-B/I160 is
Ready/Planned/Unclaimed; I161-I162/I203 remain Blocked, I188/I189/I195 remain Planned/Claimed,
I196 remains on release priority hold, and I164 remains Paused.

Commands/checks and actual results: exact-head CI completed the full macOS/Windows Rust matrix and
both governance validators; merge-time CAS reconfirmed unchanged head/base/checks/review. The
closeout cites pre-existing implementation commits and does not self-certify.

Open risks or deviations: `scraper 0.22`/`0.27` duplication remains owned by I162. The
collaboration validator's unbound local `HEAD^` fallback remains a separate governance follow-up;
I159 records the required exact-base invocation.

Next task item: prepare and independently review a dedicated ARCH-031-B/I160 claim from current
`main`. Do not create an I160 implementation branch before that claim is effective.

Recovery or resume instruction: verify this closeout on current `main`, then use a new isolated I160
claim worktree. Do not reuse the I159 implementation branch or activate unrelated planned work.

### 2026-08-14 I160 Claim Preparation Checkpoint

Completed task items: V080-10 remains closed; V080-20 claim preparation has started from the
post-closeout main `1b129c951df22a7de63e14735e02b1e8a79a9cd7`.

Current state and artifacts: ARCH-031-B/I160 is Planned/Claimed through governance PR #238, which
is not effective until merged to `main`. No I160 implementation branch, Rust/Cargo change, version
bump, tag or publication action exists.

Next task item: obtain independent review for finalized PR #238, then repeat exact-head governance,
CI and merge-time CAS checks before merging the claim.

Recovery or resume instruction: treat the claim as ineffective until its finalized `Claimed` record
is merged to current `main`; after merge, create a fresh I160 implementation worktree from the claim
merge commit.

### 2026-08-15 I160 Activation Checkpoint

Completed task items: V080-10 remains closed; V080-20 is now active after the dedicated claim
merged. PR #238 exact head `edcbe47f81798480447962048fe4f50bb69fdba1` passed CI `31815122170`,
independent approval `5295372157`, both governance validators and merge-time CAS, then merged to
`main` as `71faf8440466668daeef0afd0e779be072978b01`.

Current state and artifacts: ARCH-031-B/I160 is In Progress / Active / Claimed. The implementation
worktree is `/private/tmp/talos-i160-impl` on `feat/runtime-I160-shared-composition`, based exactly
on the claim merge; no Rust/Cargo change, version bump, tag or publication action existed at
activation. I161-I162 remain blocked in order, I203 remains blocked, I188/I189/I195 remain
Planned/Claimed and unactivated, I196 remains on release priority hold, and I164 remains Paused.

Next task item: capture the exact CLI/runtime composition baseline, then implement only the bounded
ARCH-031-B acceptance. Do not begin I161, release/version/tag, or Cargo publication work.

Recovery or resume instruction: refresh `origin/main`, verify I160 remains Active and the claim
merge remains an ancestor, then continue from `/private/tmp/talos-i160-impl`. Preserve the published
baseline and append execution evidence only.

### 2026-08-15 I160 Implementation Merge Checkpoint

Completed task items: V080-20 implementation PR #240 merged as `97556149` after exact-head CI
`31824945312` and independent approval `5296616991`; the implementation is now in Review and its
Completion Commit remains pending.

Current state and artifacts: owner documents and derived views record I160 as Review; I161-I162 and
I203 remain blocked in order. No release, tag, GitHub Release, or Cargo publication action is
authorized by this checkpoint.

Recovery or resume instruction: refresh `origin/main`, verify I160 remains in Review with
implementation PR #240 merged as `97556149`, then prepare the owner-first closeout and Completion
Commit evidence. Do not resume the old implementation worktree or begin I161, release/version/tag,
or Cargo publication work. Preserve the published baseline and append execution evidence only.

### 2026-08-15 I160 Completion Checkpoint

Completed task items: V080-20 is Complete. Pre-existing implementation commit
`0524e82fa700892cb77bf378139c47b92a64693c` satisfies the Completion Commit requirement; PR #240
merged as `97556149` after exact-head CI `31824945312` and independent approval `5296616991`, and
owner-first closeout/derived synchronization merged as PR #241 `2d48bd2c`.

Current state and artifacts: I160/ARCH-031-B is Complete/Closed. I161 remains blocked until its own
claim, security review and implementation evidence; I162 and I203 remain blocked in order. No tag,
GitHub Release or Cargo publication is authorized by this checkpoint.

Next task item: prepare a fresh exact-main inventory and an independent claim for I161. Do not reuse
I160 authorization or implementation worktree.

Recovery or resume instruction: refresh `origin/main`, verify I160 Completion Commit
`0524e82f` and merge `2d48bd2c`, then follow I161's owner document and START-ITERATION gates. Keep
GitHub Release before Cargo publication and preserve the published baselines.

### 2026-08-15 I161 Security Review Gate Checkpoint

Completed task items: I160 remains Complete/Closed. The I161 governance claim is effective through
PR #244 merge `b570ac27`; no implementation authorization was created.

Current state and artifacts: I161/ARCH-031-C remains Blocked pending an independent security
reviewer. Issue #245 requests assignment and binds the review to `main@b570ac27` and the owner
security matrix. I162 and I203 remain blocked in order. No release, tag, GitHub Release, or Cargo
publication action is authorized.

Next task item: obtain a real independent security reviewer through issue #245, then run a fresh
exact-main inventory and activation gate. Do not begin Rust implementation while the reviewer is
unassigned; do not bypass I161 for release work.

Recovery or resume instruction: refresh `origin/main`, verify `b570ac27`, PR #244, and issue #245;
preserve all Published Baseline sections and append new execution evidence only. GitHub Release
must still precede Cargo publication.

### 2026-08-15 I161 Activation Checkpoint

Completed task items: the I161 security-review gate is formally recorded through Issue #245, with
the complete ARCH-031-C owner matrix treated as normative. I161 is activated from `main@cabb7fa1`;
the claim remains bounded to ARCH-031-C/I161 and does not authorize release work.

Current state and artifacts: I161 is the sole Active iteration. I159/I160 are Complete, I162 and
I203 remain blocked in order, I188/I189/I195/I196 remain Planned/Claimed and unactivated, and I164
remains Paused. The implementation branch must start from `main@cabb7fa1`. The security reviewer
role is separate from the implementation role with shared-account identity limits disclosed; exact
implementation-head security approval remains mandatory before merge. No tag, GitHub Release or
Cargo publication has occurred.

Next task item: establish the I161 implementation baseline, add focused security-matrix tests first,
then implement the smallest bounded API/runtime change. Keep all permission and sandbox stop
conditions active and submit the exact implementation head for independent security review.

Recovery or resume instruction: refresh `origin/main`, verify `cabb7fa1`, inspect the I161 owner and
ARCH-031-C matrix, and create the implementation worktree only from that exact main. Do not begin
I162 or any release/publish action until I161 has a pre-existing implementation Completion Commit
and exact-head security approval.

### 2026-08-15 I161 Completion Checkpoint

Completed task items: V080-30 is Complete. I161/ARCH-031-C is Complete/Closed with Completion
Commits `74c5502d8860316070182c0cf2366d5adf57ea6c` and
`3ca2ec62b3e91d88c345f5bba15e986cb31f606c`. PR #250 merged as
`d2b4bdd12f69f1eaffeade7e05625369a7d4f8aa` after exact-head security approval and CI
`31873172667`; PR #251 merged as `da5a43a244ee17902fb001b2445b4ec54cbf206c` after exact-head
security approval and CI `31878744293`. The matrix closure includes all nine rows and
path/network/execute variants. No release, tag, GitHub Release, or Cargo publication occurred.

Current state and artifacts: I159, I160, and I161 are Complete. I162 and I203 remain blocked in
order; I188/I189/I195/I196 remain Planned/Claimed and unactivated; I164 remains Paused. Non-blocking
I161 residuals M1/M4/N4/N5/N6 remain recorded in the I161 owner and are outside this release gate.

Next task item: prepare a fresh exact-main inventory and an independent claim for I162. I162 must
produce the external fixtures, 20-package dry-runs, metadata closure, and explicit GO/NO-GO packet;
do not begin GitHub Release or Cargo publication before I162 Completion Commit and GO evidence.

Recovery or resume instruction: refresh `origin/main`, verify `da5a43a2`, the I161 Completion
Commits, PR #250/#251, CI `31873172667`/`31878744293`, and their exact-head approvals. Inventory all
non-terminal iterations before creating the I162 claim. Preserve all Published Baseline sections;
GitHub Release remains a hard predecessor to Cargo publication.

### 2026-08-15 I162 Claim Preparation Checkpoint

Fresh exact-main inventory was performed at `main@2301434a` after I161 closeout PR #252 merged.
I159/I160/I161 are Complete; I162 and I203 were Blocked; I188/I189/I195/I196 remain Planned/Claimed
and unactivated; I164 remains Paused. ARCH-031-D is now Ready and I162 is Planned in the proposed
claim branch `docs/i162-claim` / PR #253. The proposed Work Slice is readiness-only: external SDK
fixture, metadata-derived publishable closure, per-crate package and `cargo publish --dry-run`
evidence, and an explicit GO/NO-GO packet for candidate v0.8.0. It excludes version bump, runtime
behavior, tag, GitHub Release, and real Cargo publication. The claim is ineffective until PR #253
merges; I162 is not Active and no implementation branch or release action is authorized yet.
