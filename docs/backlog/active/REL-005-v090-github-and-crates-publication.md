# REL-005: v0.9.0 GitHub And Crates.io Publication

**Status**: Review / Claimed
**Type**: Release / Distribution Story
**Parent Epic**: ARCH-031

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | `Codex / GPT-5 mainline release session` |
| Work Slice | `REL-005 / I245` only: prepare and review the v0.9.0 candidate, publish the immutable GitHub tag and Release first, then publish the metadata-derived Cargo closure and verify external CLI/runtime consumption. |
| Claimed At | 2026-09-03 |
| Source Issue | None |
| Governance Claim PR | #479 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer explicitly requested a complete GitHub and crates.io release on 2026-09-03. Independent exact-head release review remains mandatory before the immutable tag or any Cargo publish. |
| Implementation PR | #480 |
| Last Updated | 2026-09-03 |
| Handoff / Release Condition | Candidate publication remains blocked until exact-head CI, independent release review, merge-time CAS and merged-main versioned preflight pass. |

## Identity / Goal / Value

Release the accumulated post-v0.8.0 product, runtime, SDK, permission and reliability work as
v0.9.0 through the existing five-platform GitHub distribution path, then publish the corresponding
Cargo package closure so users can install the `talos` binary and embed `talos-runtime`.

## Scope

- synchronize the workspace and all Talos path-dependency versions to `0.9.0`;
- refresh the packaged offline model catalog through the explicit `BUILD_MODELS=1` release path;
- update paired English/Chinese README and site release surfaces plus `docs/releases/v0.9.0.md`;
- run the exact versioned release preflight and obtain exact-head CI and independent release review;
- merge the reviewed candidate and create one immutable annotated `v0.9.0` tag on that main commit;
- wait for the GitHub Release, five platform archives and checksum to succeed;
- only then publish every registry-enabled Talos package at `0.9.0` in dependency order;
- verify external `cargo install talos-cli --version 0.9.0 --bin talos --locked` and a
  registry-only `talos-runtime = "0.9.0"` fixture.

## Exclusions

- `talos-models` remains quarantined with `publish = false`;
- no unrelated product, Desktop, Dashboard, dependency-upgrade or API redesign work;
- no completion of RUNTIME-006 or REL-002;
- no tag move, force push, crate overwrite or automated yank.

## Dependencies And Ordering

- The prior v0.8.0 GitHub-first publication is Complete/Closed.
- No Active or Review iteration exists at selection; I164 remains Paused and I207/I208 remain
  Planned/Unclaimed and are not activated.
- GitHub Release success is a hard predecessor of the first real `cargo publish`.
- A substantive failure after tagging requires a new patch version rather than moving `v0.9.0`.

## Acceptance

- [ ] `./scripts/release_preflight.sh v0.9.0` passes on the exact release commit.
- [ ] Exact-head CI and independent release review approve the candidate.
- [ ] Annotated `v0.9.0` points at the reviewed main commit and the GitHub Release contains five
      platform archives plus `checksum.sha256`.
- [ ] No Cargo package is published before the GitHub Release succeeds.
- [ ] Every registry-enabled Talos package is visible at `0.9.0`; `talos-models` is absent.
- [ ] External Cargo installation returns `talos 0.9.0`.
- [ ] A registry-only runtime fixture passes in default and `coding` modes.
- [ ] The explicit model refresh preserves all local variants and compatibility aliases and fails
      without overwriting the committed catalog if its prior catalog cannot be parsed.
- [ ] Owner-first closeout records immutable commit, tag, workflow, package and external evidence.

## Residual Destination

RUNTIME-006 retains the stronger single-direct-dependency facade outcome; REL-002 retains v1
self-bootstrap qualification. Any source-changing release failure receives a new patch-release
owner rather than mutation of published artifacts.

## 2026-09-03 Activation Checkpoint

Claim PR #479 exact head `967c220a3af8c8f0df5bae443e996f4031b61e48` passed CI
`33766270164`, both governance validators and single-maintainer merge-time CAS, then merged as
`c6e453a49dff12397d80335242e8291bff239938`. The REL-005/I245 claim is effective on `main` and the
release candidate branch starts from that merge. No tag, GitHub Release or Cargo publication has
occurred. The pre-release catalog refresh is the previously requested explicit build-time refresh;
normal builds retain their no-network behavior.

## 2026-09-03 Stable Candidate Checkpoint

The locally converged candidate synchronizes all 21 Talos workspace packages and internal path
dependencies to `0.9.0`, keeps `talos-models` as the only non-published package, refreshes the
offline catalog to 6,474 entries while retaining five local variants and the `k2p7` compatibility
alias, and updates the paired release surfaces. `CARGO_INCREMENTAL=0
./scripts/release_preflight.sh v0.9.0` passed in the real user filesystem environment. The first
sandboxed run identified that `talos-skill` intentionally writes a dedicated shared-skill fixture
under `~/.agents/skills`; its `Operation not permitted` was a host sandbox restriction, and the
same test passed in the standard release environment. No tag, Release or crate publication has
occurred.
