# SOP: Release Workflow

## Purpose

Ensure every agent uses the same reproducible build and release procedure.

## Required Inputs

- `rust-toolchain.toml` is the only toolchain source of truth.
- `Cargo.lock` is committed and must not be regenerated or removed to bypass validation.
- `scripts/release_preflight.sh` is the shared local/CI preflight entrypoint.

## Procedure

1. Confirm the worktree is clean and inspect the latest release tag.
2. Converge version, manifest, release-note, installer and publication-plan changes locally. Do not
   push a partial release candidate merely to use remote CI as an editing loop.
3. Synchronize `[workspace.package] version` and every internal path dependency version in
   component `Cargo.toml` files. In the same release commit, update `README.md`,
   `README.zh-CN.md`, and the paired EN/zh-CN public-site release surfaces (`site/`),
   including the Documentation hubs. Do not leave site publication as a post-tag follow-up.
4. Review the complete local branch diff, run applicable preflight checks, and commit the stable
   release candidate with the required model marker.
5. Submit that exact candidate commit for CI and independent release review. Any substantive
   correction requires local reconvergence, a new stable head and fresh evidence; protected release
   review is never replaced by the local loop.
6. After merge and merge-time CAS, synchronize the exact reviewed target-branch commit and run
   `./scripts/release_preflight.sh vX.Y.Z`. The script validates tag/version alignment, all
   Talos package versions, README/site release truth, public-site links/accessibility contracts,
   installer instructions, formatting, locked dependency resolution, check, Clippy, and tests.
7. Create and push an annotated tag at that validated target-branch commit. The tag-driven GitHub
   workflow owns GitHub Release artifact creation; complete and verify the GitHub Release before
   any Cargo publication starts.
8. Append the commit, tag, validation output, workflow result and later Cargo publication result to
   the release closeout task and synchronize `docs/BOARD.md`.

## Failure Rules

- A failed preflight blocks tagging.
- A README/site version mismatch or a failed public-site/installer check blocks tagging; correct
  the release commit and rerun the preflight.
- A failed workflow does not authorize moving or force-pushing the tag. Correct the source and use
  a new patch version/tag.
- `--locked` failures require fixing the committed lockfile or dependency declaration; deleting
  `Cargo.lock` or dropping `--locked` is not an acceptable workaround.
