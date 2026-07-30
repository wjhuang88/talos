# Talos Four-Month v0.6 Execution Package

> Document status: In Progress
> Execution window: 2026-07-28 through 2026-11-28
> Baseline commit: `2bb2b6185f2f9ca35af269efa63c618076f4a32e`
> Branch mode: direct `main` for I168 under the maintainer's no-parallel-task direction;
> reassess release-managed/on-demand worktree mode before I158
> Current implementation authority: I168 / RUNTIME-003 provider terminal-outcome integrity only
> Program owner: `docs/tasks/2026-07-26-v0.6-runtime-productization-program.md`

## Outcome

Deliver the remaining v0.6 runtime-productization mainline as a reviewable sequence:

1. close the provider-removal lifecycle gap;
2. obtain an explicit ADR-053 decision;
3. consolidate built-in tool registration;
4. establish real `talos-tools` feature boundaries;
5. share CLI/runtime internal composition without coupling runtime to product UI;
6. introduce a fail-closed sandbox fallback policy and official coding preset;
7. produce an external SDK fixture and publication-readiness packet.

The four-month window is a capacity envelope, not permission to bypass a dependency, security
review, acceptance gate, or release authorization. Only the currently Active iteration is
implementation authority.

## In Scope

- I157 through I162 in their published order and unchanged owner-defined scope;
- the ADR-053 maintainer/architecture decision gate;
- required focused, workspace, runtime, documentation, and governance evidence;
- a bounded reserve queue for periods when a human gate blocks the mainline;
- per-phase commits and durable recovery checkpoints.

## Out Of Scope

- Desktop or GPUI work;
- v1.0 claims or REL-002 qualification;
- real `cargo publish`, tags, GitHub Releases, product releases, or version changes;
- a new composition crate;
- a second renderer or global event bus;
- permission-default relaxation;
- sandbox implementation changes outside an activated, independently reviewed I161;
- parallel implementation on `main`;
- speculative work from Refinement, Research, or Planned stories.

## Capacity Model And Schedule

The plan assumes one primary developer/agent working sequentially. It allocates sixteen
implementation weeks plus decision, review, and recovery margin inside the four-calendar-month
window.

| Window | Package | Planned Result | Gate To Leave Window |
|---|---|---|---|
| Weeks 1-2 | P1 / I157 | Provider removal and credential clear | MODEL-010 acceptance, runtime evidence, locked workspace gates |
| Week 3 | G1 / ADR-053 | Architecture decision | ADR-053 explicitly Accepted or mainline formally Blocked |
| Weeks 3-5 | P2 / I158 | One explicit tool contribution/composition model | Registry-set, collision, wrapper, print/TUI/MCP equivalence |
| Weeks 6-7 | P3 / I159 | Real lightweight `talos-tools` feature boundary | Feature matrices compile/test; default remains product-compatible |
| Weeks 8-10 | P4 / I160 | Shared internal CLI/runtime composition | Separate public entrypoints use one tested internal implementation |
| Weeks 11-13 | P5 / I161 | Sandbox fallback policy and coding preset | Independent security review plus fail-closed runtime evidence |
| Weeks 14-16 | P6 / I162 | External SDK fixture and readiness packet | External fixture, metadata closure, explicit GO/NO-GO evidence |
| Weeks 17-18 | Margin | Review fixes, platform replay, documentation closeout | Every required owner synchronized; residuals have owners |

Dates are forecasts. A package starts only after its dependency owner records the gate as
satisfied. Finishing early does not authorize pulling a blocked package forward.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| P0 | Publish this execution baseline and activate the first eligible iteration | I157 Active; MODEL-010 In Progress; Board/program/index synchronized | I166 Complete; no other Active/Review implementation iteration | Governance validation and clean staged review | Stop if inventory contradicts owner docs | Complete |
| P1 | Execute I157 / MODEL-010 | provider-unset | P0 | stale-snapshot concurrency acceptance correction | one locked current-state semantic update path across Talos writers | Complete — `5aac6756` |
| U1 | Execute I168 / RUNTIME-003 P0 terminal-outcome correction | Unknown/missing provider terminal signals cannot become normal success; MaxTokens is explicit; bounded terminal cause survives interactive TLOG outside transcript/model/export | Explicit maintainer resumption; P1 Complete; no competing Active/Review iteration | I168 acceptance, two-protocol fixture matrix, TLOG round trip/compaction/exclusion, canonical bridge tests, rebuilt-binary evidence, full locked validation | Stop on public break/ADR-042 conflict; do not degrade to intent heuristics or silently fold into OBS-002 | Active — maintainer resumed 2026-07-30 |
| G1 | Review ADR-053 and ARCH-034-R01 readiness | Explicit accepted/rejected/revision-required decision and synchronized story state | P1 Complete | ADR-053 Accepted and ARCH-034-R01 Ready before I158 activation | Select a reserve packet through a new iteration; never implement I158 around the gate | Planned |
| P2 | Execute I158 / ARCH-034-R01 | Explicit contribution contract, deterministic collisions, equivalent tool sets/wrappers across modes | G1 passed | I158 acceptance and full equivalence/runtime evidence | Block and use reserve queue if ADR or API contract remains unresolved | Blocked |
| P3 | Execute I159 / ARCH-031-A | Optional dependencies and gated modules/re-exports with a lightweight read-only boundary | P2 Complete | Feature/build matrix and unchanged CLI product behavior | Keep existing default intact and record unsupported feature split | Blocked |
| P4 | Execute I160 / ARCH-031-B | One shared internal composition implementation with separate CLI/runtime entrypoints | P3 Complete | Cross-entrypoint equivalence and dependency-direction checks | Preserve previous builders until equivalence passes | Blocked |
| G2 | Schedule I161 independent security review | Named reviewer, review packet, threat model, evidence commands | P4 approaching completion | Reviewer and acceptance protocol recorded before I161 activation | Pause mainline and use reserve queue | Planned |
| P5 | Execute I161 / ARCH-031-C | Explicit fail-closed sandbox fallback and `RuntimePreset::coding()` | P4 Complete; G2 passed | Independent security acceptance; Deny/Ask/AllowUnsandboxed matrix; workspace gates | Revert to existing fail-closed behavior and mark Blocked | Blocked |
| G3 | Obtain I162 readiness/version authorization | Maintainer authorizes readiness evaluation at the current workspace version | P5 Complete | Written authorization in I162/program owner; no publish permission implied | Keep I162 Blocked | Planned |
| P6 | Execute I162 / ARCH-031-D | External SDK fixture and evidence-backed v0.6 publication GO/NO-GO packet | P5 Complete; G3 passed | Fixture runs outside workspace; metadata-derived closure; all locked gates | Emit NO-GO with exact residual owners | Blocked |
| P7 | Four-month closeout | Owners, Board, program, decisions, residuals, and recovery record synchronized | P1-P6 terminal | Every Complete owner cites pre-existing implementation SHA; no unresolved unowned residual | Keep package Partial or Blocked | Planned |

## Reserve Queue

Reserve work prevents idle time while a human decision is pending; it is not automatic scope.
Before starting a reserve item, the current Active iteration must be formally Paused or Blocked,
the reserve Story dependencies must be rechecked, and a dedicated iteration must be selected and
activated through the normal SOP.

| ID | Story | Capacity | Eligibility | Completion Gate | Current State |
|---|---|---:|---|---|---|
| R1 | PROVIDER-004 text tool-call ID collision | 1-2 weeks | Mainline blocked; story still Ready; no conflicting provider protocol work | Unique IDs on text paths, pairing/replay regression, native path unchanged | Ready reserve |
| R2 | TOOL-023-A bash timeout fix | 1 week | Mainline blocked; story still Ready | One absolute deadline survives continuous output; kill/drain behavior preserved | Ready reserve |
| R3 | TOOL-023-B configurable 300-second default | 1-2 weeks | R2 Complete and story revalidated | per-call > config > built-in precedence; 600-second clamp; docs | Dependency-blocked reserve |
| R4 | TOOL-023-C Windows PowerShell path | 2 weeks plus review | R2 Complete; new ADR accepted; Windows host/reviewer available | Windows automated evidence and real Windows walkthrough | External-gate reserve |

Do not use Refinement stories such as OBS-002 or SESSION-007 as reserve implementation work until
their decisions and Ready transition are complete.

## Dependencies And Human Gates

- **ADR-053**: architecture/maintainer owns acceptance. An executor may prepare evidence and review
  notes but cannot self-accept the decision.
- **I161 security review**: must be independent of the primary implementation author and scheduled
  before activation.
- **I162 version/readiness gate**: requires explicit maintainer authorization. It is readiness only;
  it does not authorize version mutation, publish, tag, or release.
- **Manual/runtime evidence**: each behavior-facing owner defines its real CLI/runtime/platform
  matrix. Unit tests cannot substitute for a required manual gate.

## Artifacts And State Owners

Update in this order:

1. selected Story owner;
2. active iteration owner;
3. parent Story/ADR when its actual state changes;
4. `docs/iterations/README.md`;
5. v0.6 program owner;
6. `docs/backlog/PRODUCT-BACKLOG.md`;
7. `docs/BOARD.md`.

The Board is a mirror only. This package coordinates capacity and recovery; it does not replace
Story acceptance or iteration completion evidence.

## Validation And Acceptance Evidence

Every implementation package runs its owner-specific focused tests plus:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

Run `scripts/assess_project_scale.sh .` when the branch/worktree profile or governance depth
changes. Record command, exit code, test counts, warnings, runtime fixture, and relevant platform;
do not record only “green”.

## Branch, Worktree, And Checkpoint Plan

- Work directly on `main` for I168 under the maintainer's existing no-parallel-task direction.
- The 2026-07-28 scale assessment recommends `high-risk`, `release-managed`, and
  `on-demand` worktrees. Reassess and record the branch decision before I158, where the
  architecture/security sequence begins.
- Recheck a clean worktree and `origin/main` relationship at every dispatch.
- If parallel work starts, stop and establish a dedicated worktree/branch before further edits.
- Make one conventional commit per logical change and inspect `git diff --cached` first.
- The maintainer authorized one push at every completed phase boundary on 2026-07-28.
- After the phase completion gate and staged-diff review pass, commit, push `main`, and verify that
  `origin/main` resolves to the pushed phase commit. Never force-push.
- If a phase is Blocked, push only its intentional evidence/checkpoint commit after validation;
  do not push an uncommitted or known-broken implementation state.
- Append a checkpoint to this file and the active iteration at every phase boundary.

Checkpoint template:

```text
Completed task items:
Current state and artifacts:
Commands/checks and actual results:
Open risks or deviations:
Next task item:
Recovery or resume instruction:
```

## Allowed Permissions And External Actions

Authorized by this package:

- read and edit in-repository files within the Active iteration scope;
- run local focused/workspace validation;
- create intentional local commits after staged-diff review;
- push the validated phase commit to `origin/main` once per completed or formally blocked phase.

Not authorized:

- force-push, tag, publish, release, deploy, or version changes;
- paid APIs, cloud resources, remote mutations, or new credentials;
- destructive cleanup, history rewriting, migration of user data, or changing permissions;
- new dependencies unless the active owner explicitly permits them and the dependency review passes.

## Time, Cost, And Resource Limits

- One implementation iteration Active at a time.
- One primary executor; no parallel agents or worktrees by default.
- No paid external service usage.
- Retry a failing deterministic validation once after diagnosing the cause. A repeated unexplained
  failure is a blocker, not permission to weaken the gate.
- Keep reserve work within its stated capacity band; return scope expansion to Refinement.

## Failure, Retry, And Fallback Policy

Stop and record a blocker when:

- an owner dependency or decision is not satisfied;
- a public API break lacks an accepted ADR and migration plan;
- a security-sensitive change lacks the required independent review;
- baseline tests fail for an unexplained reason;
- implementation requires a forbidden dependency, `unsafe`, permission relaxation, or release action;
- three bounded approaches fail.

When blocked, preserve the runnable path, record evidence and the exact maintainer decision needed,
then either wait or activate one eligible reserve item through a new iteration. Do not silently
skip to a later mainline package.

## Default Decisions For Foreseeable Ambiguity

- Choose the smallest owner-compliant implementation.
- Preserve current public behavior unless acceptance explicitly changes it.
- Prefer additive migration and equivalence tests before removing an old path.
- Treat dates as planning ranges, never as a reason to weaken tests or security.
- Classify publication readiness as NO-GO when evidence is incomplete.
- Leave ambiguous external or destructive actions unperformed.

## Residual-Work Destination

- Product behavior gaps: the owning Story or a new backlog Story after requirement intake.
- Architecture decisions: ADR-053 or a new decision record.
- Security findings: ARCH-031-C/I161 and its independent review record.
- Publication findings: ARCH-031-D/I162 readiness packet.
- Optional reserve leftovers: their existing Story owners.
- Reusable lessons or failed assumptions: `EVOLUTION.md` via
  `docs/sop/EVOLUTION-FEEDBACK.md`.

## Current Checkpoint

- Completed task items: P0 (baseline inventory + package publication) and P1 (I157/MODEL-010,
  including the stale-snapshot concurrency correction in `5aac6756`).
- Completion Commit: `5aac675690bf6528d5977521db3dfb6f2abb486d`.
- Current state and artifacts: I157/MODEL-010 and I167/TUI-040 are Complete;
  I168/RUNTIME-003 is the sole Active implementation authority; I164 is Paused; I158-I162 are
  Blocked; ADR-053 is Proposed.
- Commands/checks and actual results for the I157 correction: locked workspace check, Clippy,
  tests, formatting, governance validation, and diff checks passed on 2026-07-30. The owner
  records `5aac6756` as the Completion Commit.
- External-action authorization: validated `main` push at each phase boundary completed on
  2026-07-28. Tag, publish, release, deploy, force-push, and version changes remain unauthorized.
- Open risks or deviations: ADR-053 acceptance, I161 independent reviewer, and I162 maintainer
  authorization are external gates. I158 remains Blocked until ADR-053 is Accepted.
- Next task item: execute U1 / I168 / RUNTIME-003 under its published provider terminal-outcome
  acceptance. G1 remains deferred.
- Recovery or resume instruction: read RUNTIME-003, I168, OBS-002's scope split, this package, and
  the v0.6 program; verify the current Git state; write the required failing provider fixtures
  before production changes; keep I158 blocked.
