# Iteration I307: Bubblewrap Workspace Visibility

> Document status: Complete
> Completion Commit: f643d87759189eed9d14cef965b7e9260a51d1ab
> Prior Implementation Evidence: 3beb011a77e0cc80422d915443e6caf9f8f74742
> Published plan date: 2026-10-24
> Planned objective: restore the authorized Linux workspace after temporary mounts without expanding writable scope.
> Baseline rule: preserve this bounded repair; broader sandbox redesign requires a separate owner.
> MVP deliverable: sandboxed commands in a /tmp child workspace run there while host temporary siblings remain isolated.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | SANDBOX-001: Linux Bubblewrap workspace mount order, explicit cwd, isolation tests and docs; no Git admission, fallback policy, dependencies or API changes. |
| Claimed At | 2026-10-24 |
| Source Issue | #757 |
| Governance Claim PR | #761 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested session-handoff and failing-CI closure; no distinct maintainer reviewer available. Independent Agent security review, exact-head CI, both validators and merge-time CAS remain mandatory. Prior claim #758 and implementation #759 are historical; follow-up claim #761 became effective on main before implementation #762. |
| Implementation PR | #762 (merged as f643d87759189eed9d14cef965b7e9260a51d1ab) |
| Last Updated | 2026-10-26 |
| Handoff / Release Condition | Protected-mount isolation acceptance complete at #762; I300 must integrate and validate its own final candidate. No release authorization. |

## Protected-mount closure — 2026-10-26

Requested outcome: close the reopened isolation defect without changing the published
acceptance. Updated artifacts: this owner, SANDBOX-001 and derived Board/backlog/iteration
indexes; preserve historical checkpoints below. No new contract or dependency decision.

PR #762 merged as `f643d87759189eed9d14cef965b7e9260a51d1ab`, containing
implementation `51245b28c` and fixture correction `48a6285de`. Exact-head CI
[40213017639](https://github.com/wjhuang88/talos/actions/runs/40213017639)
is terminal success across all jobs, including full macOS locked preflight and
native Windows workspace validation. Native Linux job 121231556761 logs were
inspected: 37 sandbox unit tests, all five Bubblewrap integration tests and two
doctests passed; Desktop passed 112 tests including Auto allow-once and both
running-shell cancellation cleanup cases. This covers the seven published
acceptance rows and reopened protected-root/alias rejection boundary. The older
run failed the historical basic fixture using /tmp itself; the final fixture
uses an owned temporary child, without weakening protection or skipping tests.

Independent Agent security review approved exact head `48a6285de`; both governance
validators passed with zero warnings. Merge-time CAS confirmed unchanged claim,
actor/slice and main, clean mergeability, no overlapping new PR or unresolved
review threads. The recorded Single-maintainer merge path applies; Agent technical
review is not a distinct-human GitHub approval. Native log:
`/private/tmp/i307-48a-linux-native.log`; final PR evidence comments retain review/CAS.
User documentation is `crates/talos-sandbox/README.md`.

No remaining I307 implementation or validation is deferred. Existing path-resolution
to spawn TOCTOU is not claimed fixed or race-free by this bounded repair. T7/I300
and the other handoff requirements remain separate and incomplete. Issue #757 and
these derived indexes follow this owner; closure records still require publication.

## Protected-mount implementation checkpoint — 2026-10-26

Follow-up claim #761 became effective on main `d8da359ad`; implementation branch
`fix/i307-protected-mount-overlap` starts there. The local patch canonicalizes
workspace and extra-read sources/destinations, checks lexical and canonical
protected roots, and rejects overlapping mounts before payload launch. Regressions
cover 16 direct/alias and workspace/read-overlay rejection combinations, plus an
ordinary workspace-alias success case; the original three native cases remain.
README describes canonical cwd and rejected roots. No API/dependency/unsafe change.

`cargo test -p talos-sandbox --locked` passed on macOS (41 unit tests and two
doctests); the Linux integration target ran zero tests and is not runtime evidence.
`cargo fmt --all` and `git diff --check` pass. Full standard preflight passed exit 0
at `/private/tmp/i307-protected-mount-preflight.log`, including workspace tests
and both external SDK fixtures. Both governance validators passed with zero warnings.
Implementation commit: `51245b28c`. Native Linux execution, final exact-head
security review, CI and merge remain unverified. Status is Review.

## Reopened Isolation Acceptance — 2026-10-26

The original objective and acceptance are unchanged. Prior narrow-workspace
implementation `3beb011a77` and successful CI remain historical evidence, but
PR #735 review exposed a missing input boundary: a late workspace bind at /,
/tmp, /dev or /proc can cover protected mounts. No native broad-root reproduction
has yet been run. This invalidates the full isolation completion claim.

Requested outcome: enforce the existing protected-mount isolation for all accepted
workspace/read overlays. Artifacts: sandbox implementation, native regressions,
sandbox README and these owners/derived indexes. Verification: original narrow
/tmp tests plus broad-root and symlink-alias non-execution/sentinel tests; locked
preflight, native Linux CI, independent escape review, both validators and CAS.
Residual destination: this owner and #757 until reviewed implementation merge.

Follow-up implementation plan:
1. Resolve workspace and extra-read mount paths before spawning bwrap; reject
   roots that equal or contain /tmp, /dev or /proc, including canonical aliases.
   Preserve ordinary /tmp child workspaces and read-only extra-path precedence.
2. Verify rejected configurations never execute the payload or alter a host
   sentinel; retain all original workspace/cwd, missing-path, outside-write,
   private-temp and read-only overlay regressions. Review overlap in both mount
   sets rather than guarding workspace string spelling alone.
3. Record actual native evidence; do not claim macOS compilation tests Linux
   namespace behavior. No public API, dependency, unsafe or Git contract change.

Selection inventory at main `238bae0f1`: I249 Planned, I277/I302 Review,
I293/I294/I300/I306 Active retain their owners; no new overlapping iteration
or open PR owns this sandbox correction. Existing #735/#749/#705 scopes remain
unchanged. Reuse the same I307 objective and #757; no new product requirement.
This proposed governance-only reactivation is ineffective before target merge.

## Published Baseline

Selected [SANDBOX-001](../backlog/active/SANDBOX-001-bubblewrap-workspace-visibility.md),
source #757. This independent prerequisite repairs a general sandbox defect uncovered
by I300 validation; it does not replace or narrow I300 acceptance.

### Selection Inventory And Disposition

Target main `d5e0d0627`, refreshed 2026-10-24. Owner status inventory:

- I249 Planned: retain dependency upgrade pilot; no overlap.
- I277 Review: retain deferred human/device acceptance.
- I293 and I294 Active: retain Search owners and their implementation.
- I300 Active: retain Git work; isolated dependency repair precedes its final Linux gate.
- I302 Review: retain Windows resource evidence follow-up.
- I306 Active: retain provider protocol work and #749.
- I162 Complete with recorded Review outcome is not an open iteration.
- No Blocked owner or other nonterminal iteration found owning this repair.

Open PRs #735 (Git), #749 (provider) and #705 (Windows diagnostic) do not own
Linux mount-order repair. Issue #245 and its comments concern completed I161
review assignment, not this new defect; I161 is not reopened.
I307 is serial prerequisite work by this actor, not a second autonomous stream.

### Scope And Acceptance

1. Preserve read-only host root, private /tmp, network policy and owned cleanup.
2. Bind only the authorized workspace after temporary/device mounts, preserve extra
   read paths' later read-only precedence, and select sandbox cwd explicitly.
3. Real Linux Bubblewrap reads an existing marker, creates a workspace file and
   reports exact cwd for a workspace below /tmp.
4. Host /tmp siblings remain invisible; a same-named sandbox temporary write must
   not mutate the host sentinel. Existing outside-workspace file writes fail.
5. An explicitly exposed /tmp extra-read file remains readable and not writable.
6. Missing workspace fails without executing the payload in HOME or another cwd.
7. Re-run Linux Desktop Auto allow-once and both cancellation readiness failures;
   available bwrap with namespace failure must not be misreported as test success.

### Non-Goals

No broad /tmp binding, unsandboxed fallback, new backend, dependency, unsafe code,
policy relaxation, public API change, process-supervisor rewrite or Git feature work.

### Validation And Documentation

Use pinned Rust 1.99.0 and --locked. Run real Linux bwrap regressions with
bwrap_installed, original Desktop failure cases, `./scripts/release_preflight.sh`,
exact-head CI and independent escape-vector review. Run both governance validators
and project-scale assessment for the isolated worktree. Update sandbox README,
this owner, backlog, Board and issue records. Rollback is a reviewed revert;
no test disablement or writable-scope enlargement.

## Evidence And Residuals

CI 40089763867 at f30b4df6f, Linux job 120859903922: current order produces
cwd /home/runner and absent marker; tmpfs-first/workspace-last plus --chdir
produces exact workspace cwd and visible marker. This proves the probe, not
a product fix. Independent Agent scope/security review requires the acceptance
above and separate effective claim before coding. No implementation or completion
is claimed; I300 and the full handoff remain incomplete.

## Proposed Atomic Activation

Claim and Active state above are proposed by #758 and become effective only
when its finalized reviewed head merges to main. Both validators passed before
PR creation; repeat them and target-branch CAS on the finalized candidate.

## Implementation Checkpoint — 2026-10-24

Claim #758 merged as `b29903196e182e63cc1eaffeb9f837918f7e5aa2` before
implementation branch `fix/i307-bubblewrap-workspace` was created. The candidate
moves workspace bind after temporary mounts, retains later extra-read read-only
priority and adds explicit sandbox cwd. Three native integration regressions
cover workspace visibility/writes, private temporary siblings, outside writes,
extra-read precedence and missing-workspace refusal. The previously referenced
`bwrap_installed` feature was absent from Cargo.toml; it is now declared and
explicitly invoked by Linux CI with Bubblewrap installed.

Independent Agent escape-vector review accepts this candidate for validation,
with no mandatory corrections, not as completed runtime acceptance. Local macOS
sandbox tests pass 41/41 plus two doctests; the Linux integration target runs
zero tests on macOS and supplies no Linux evidence. Standard preflight passed exit 0
(`/private/tmp/i307-release-preflight.log`), including workspace tests, Clippy
and external SDK fixtures. Native Linux regressions, original
Desktop failures, exact-head review/CI and merge remain outstanding. No Complete
status or Completion Commit is asserted. User documentation is now provided in
crates/talos-sandbox/README.md.

## Native Validation Checkpoint — 2026-10-25

Implementation cee63a8c8 passes CI 40113337467 on all jobs. Native Linux proves
37 sandbox tests, three new isolation regressions and two doctests, followed by
112 Desktop tests including the original Auto allow-once failure. Final candidate
81f3d6e1f changes only implementation-PR bindings; independent exact-head security
review accepts it. Its CI 40113538524 passes Linux and Windows, but macOS reports
EPERM in the existing delayed-descendant cleanup test. Same-head failed-job rerun passed; run 40113538524 is terminal success.
The initial EPERM cause remains unproven; rerun success does not claim a cleanup fix.
Both governance validators pass with zero warnings. Independent exact-head
security review reconfirmed ACCEPT. Merge-time CAS verified unchanged target
claim, mergeability and no overlapping PR. PR #759 merged as
3beb011a77e0cc80422d915443e6caf9f8f74742 on 2026-10-25.
The six published acceptance rows are covered by the three native Linux
regressions and unchanged-policy assertions in bubblewrap_workspace.rs.
User documentation is crates/talos-sandbox/README.md. No remaining I307
implementation or validation item is deferred; T7 and #705 remain separate. Logs retained at
/private/tmp/i307-native-linux-cee.log and /private/tmp/i307-final-ci-failure.log.
