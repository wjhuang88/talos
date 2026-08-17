# Iteration I203: v0.8.0 GitHub And Crates.io Publication

> Document status: Complete / Closed (2026-08-17)
> Published plan date: 2026-08-14
> Planned objective: publish one validated v0.8.0 GitHub Release before publishing the authorized
> Cargo package closure, then prove external CLI installation and runtime SDK consumption.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a user can obtain v0.8.0 from the completed GitHub Release or install the `talos`
> binary from crates.io, while an embedder can resolve the documented v0.8.0 `talos-runtime`
> contract outside the workspace.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | REL-003 only: GitHub v0.8.0 release first, then the reviewed Cargo closure and external install/SDK verification. |
| Claimed At | 2026-08-16 |
| Source Issue | None |
| Governance Claim PR | #262 |
| Authorization Mode | Independent review |
| Authorization Evidence | Implementation PR #264 exact head `d5de4a65` passed CI `31952144946`, independent approval `5307931428`, and merge-time CAS before merging as `f425e7bc`. |
| Implementation PR | #264 |
| Last Updated | 2026-08-17 |
| Handoff / Release Condition | Closed after GitHub Release `v0.8.0`, all 20 Cargo packages, external CLI installation, and the registry-only runtime fixture passed. RUNTIME-006/#234 and REL-002 remain separate residual owners. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| REL-003 | ARCH-031 | Blocked | I159-I162 Complete and I162 GO | GitHub v0.8.0 Release precedes the 20-package Cargo publication and external verification. |

### Ordered Delivery

1. Reconfirm current `main`, registry state, package closure, credentials and v0.8.0 availability.
2. Prepare the release commit and pass `./scripts/release_preflight.sh v0.8.0`.
3. Push the release commit and immutable annotated tag once.
4. Wait for the GitHub Release, five archives and checksum to succeed.
5. Publish Cargo packages in the I162 metadata-derived dependency waves, checking registry
   visibility between waves.
6. Run clean external `cargo install` and runtime SDK fixtures.
7. Close REL-003/I203 and the task with exact evidence.

### Authorized Cargo Closure

I162 must recompute the final order. The intended set is every workspace package required by
`talos-cli` or `talos-runtime`, excluding only quarantined `talos-models`: 20 packages total.
Because the published `talos-cli` binary depends on `talos-dashboard`, `talos-evolution`, and
`talos-tui`, all 20 closure members must be registry-enabled for `cargo install talos-cli --bin
talos` to resolve. `talos-models` remains the only `publish = false` workspace member.
Current metadata permits these dependency waves:

1. `talos-core`, `talos-skill`, `talos-memory`, `talos-exploration`, `talos-dashboard`;
2. `talos-config`, `talos-permission`, `talos-session`, `talos-sandbox`;
3. `talos-plugin`, `talos-provider`, `talos-tools`;
4. `talos-conversation`, `talos-evolution`, `talos-mcp`, `talos-rpc`, `talos-agent`;
5. `talos-tui`, `talos-runtime`;
6. `talos-cli` last.

Dev-dependency resolution and registry visibility may further serialize packages inside a wave.
I162's exact packet supersedes this planning-time order if current metadata changes.

### Non-Terminal Inventory At Planning

| Iteration(s) | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Selected prerequisite chain; execute in order through separate claims. |
| I188, I189, I195, I196 | Planned / Claimed | Keep unactivated; I196 is explicitly held until this release sequence closes. |
| I197-I201 | Planned / Unclaimed | Proposed only in PR #227; no target-branch ownership or activation. |
| I159-I162 historical gates | Blocked | Preserve published baselines; append readiness/execution facts only. |
| I164 | Paused | Superseded; do not resume. |

No Active or Review iteration is imported into this release plan. I188 PR #228 and Dashboard PR
#233 remain independent and must reconcile their own exact heads.

### Forbidden Changes

- no Cargo publication before GitHub Release success;
- no `talos-models` publication;
- no RUNTIME-006 API work;
- no v1/REL-002 claim;
- no tag movement, force push, hidden package skip or status-only self-certification.

### Planned Validation

- `./scripts/release_preflight.sh v0.8.0`;
- `bash scripts/check_publish_guard.sh .` (only `talos-models` remains quarantined);
- I162 package/dry-run matrix for all 20 packages;
- exact GitHub workflow/artifact/checksum inspection;
- per-wave crates.io visibility checks;
- clean external `cargo install talos-cli --version 0.8.0 --bin talos --locked`;
- clean external current-contract `talos-runtime = "0.8.0"` fixture;
- both governance validators and `git diff --check`.

### Risks And Recovery

- GitHub Release failure: stop all Cargo publication and release a new patch after source repair;
  never move v0.8.0.
- Partial crates.io publication: published versions remain immutable. Verify visibility, repair only
  unpublished dependents, and use a patch version if the source must change.
- Rate limiting: honor the registry retry time and checkpoint the last visible package.
- Credential/ownership failure: keep I203 Blocked with the exact package and registry response.

## Verification Evidence

- Release preflight: `./scripts/release_preflight.sh v0.8.0` passed on merged release commit
  `f425e7bc`; main CI `31952994218` also passed.
- GitHub Release and assets: annotated tag `v0.8.0` points to `f425e7bc`; Release workflow
  `31953951828` completed successfully with five platform archives and `checksum.sha256`.
- Cargo publication: all 20 authorized `0.8.0` packages were published in the recorded dependency
  waves and confirmed visible through the Cargo registry. `talos-models` remains excluded.
- External install/SDK fixtures: isolated crates.io installation returned `talos 0.8.0`; a
  registry-only `talos-runtime = "0.8.0"` fixture passed in default and `coding` modes.
- Governance validation: recorded by the closeout PR after owner-first synchronization.

## Completion Evidence

- Completion Commit: `b0354ae6b7c349ccbc101a046ded1d8aafdda3ff`,
  `d8e1aa268d3419ee957b78a46a57c68bad50c3f5`, and
  `d5de4a6573e8f3b77fbfc80c5dc1504f078f1ee7`.
- Release/tag/workflow evidence: merge `f425e7bc`, immutable annotated tag `v0.8.0`, GitHub Release
  workflow `31953951828`, and published Release timestamp `2026-08-16T15:23:21Z`.
- Cargo registry evidence: all 20 closure packages resolve at `0.8.0`; `talos-cli` exposes binary
  `talos`, and `talos-runtime` resolves for an independent Cargo root.
- This closeout cites pre-existing implementation commits and external release evidence; the
  status-only closeout commit does not certify itself.

## 2026-08-16 Claim Preparation Checkpoint

I203 is proposed as `Planned / Claimed` through governance PR #262 from exact main
`8eaa22a2`. The Work Slice covers the reviewed v0.8.0 candidate, `models.toml` refresh, release
preflight, immutable GitHub-first release, dependency-wave Cargo publication, and external CLI/SDK
acceptance. It does not authorize implementation, version changes, tags, GitHub Release, or real
Cargo publication until the claim is effective and the implementation head is independently
reviewed.

## 2026-08-16 Activation Checkpoint

I203 is `Active / In Progress / Claimed` after claim PR #262 merged as `f6b2d243`. The implementation
branch must be fresh from this activation head. The release implementation may include the reviewed
`models.toml` refresh and v0.8.0 alignment, but no irreversible action is authorized until its exact
head has independent review, green CI, and merge-time CAS. GitHub Release remains a hard predecessor
to every real Cargo publication.

## Variance And Residuals

- Issue #234 / RUNTIME-006 is intentionally excluded and remains Refinement/Unclaimed.
- REL-002 remains NO-GO and is not a v0.8.0 gate or claimed outcome.

## 2026-08-16 Publication Closure Correction

The I162/I204 readiness packets intentionally preserved four product `publish = false` guards at
their historical candidate-only boundaries. The I203 implementation rechecked the actual
`talos-cli` dependency closure and found those guards incompatible with the committed MVP,
`cargo install talos-cli --bin talos`. The implementation therefore removes the guards from
`talos-cli`, `talos-dashboard`, `talos-evolution`, and `talos-tui`; this is a release-scope
correction, not a rewrite of either historical packet. `talos-models` remains quarantined and is
the only excluded workspace member. The final package count remains 20 and must be recomputed
from exact-head metadata before publication.

## Retrospective

- Outcome: GitHub-first ordering, 20-package publication, CLI installation, and runtime SDK
  consumption all completed.
- Documentation: v0.8.0 release notes, README surfaces, Cargo manifests, SDK contract, and owner
  evidence are synchronized.
- Lessons: crates.io enforced new-crate rate limits before `talos-runtime` and `talos-cli`; the
  release honored each retry timestamp and retried only the unpublished package.

## 2026-08-17 Release Closure Checkpoint

Implementation PR #264 merged as `f425e7bc` after exact-head approval and CI. The annotated
`v0.8.0` tag triggered GitHub Release workflow `31953951828`; the Release completed before the
first Cargo upload and contains all five platform archives plus `checksum.sha256`. The 20-package
closure then published in dependency order. Crates.io rate limits temporarily rejected only the
still-unpublished `talos-runtime` and `talos-cli`; both were retried after their registry-specified
windows and are now visible at `0.8.0`. An isolated Cargo installation returned `talos 0.8.0`, and
the registry-only runtime fixture passed in default and `coding` modes. I203 is Complete/Closed;
RUNTIME-006/#234 and REL-002 remain unchanged.
