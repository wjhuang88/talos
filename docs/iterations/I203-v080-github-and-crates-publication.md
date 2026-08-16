# Iteration I203: v0.8.0 GitHub And Crates.io Publication

> Document status: Active / In Progress
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release-governance session` |
| Work Slice | REL-003 only: GitHub v0.8.0 release first, then the reviewed Cargo closure and external install/SDK verification. |
| Claimed At | 2026-08-16 |
| Source Issue | None |
| Governance Claim PR | #262 |
| Authorization Mode | Independent review |
| Authorization Evidence | I204 closeout is effective at `main@8eaa22a2`; claim PR #262 merged as `f6b2d243`, and this activation checkpoint records the now-effective claim. |
| Implementation PR | Not started |
| Last Updated | 2026-08-16 |
| Handoff / Release Condition | Start from `main@f6b2d243`, preserve all I204/I162 Published Baselines, and keep irreversible release actions behind this activation plus an independently reviewed implementation PR. GitHub Release precedes Cargo publication. |

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

- Release preflight: pending
- GitHub Release and assets: pending
- Cargo publication: pending
- External install/SDK fixtures: pending
- Governance validation: pending

## Completion Evidence

- Completion Commit: pending
- Release/tag/workflow evidence: pending
- Cargo registry evidence: pending
- A status-only closeout commit cannot certify implementation or publication.

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

## Retrospective

- Outcome: pending
- Documentation: pending
- Lessons: pending
