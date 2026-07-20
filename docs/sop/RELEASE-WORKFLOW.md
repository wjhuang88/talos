# SOP: Release Workflow

## Purpose

Ensure every agent uses the same reproducible build and release procedure.

## Required Inputs

- `rust-toolchain.toml` is the only toolchain source of truth.
- `Cargo.lock` is committed and must not be regenerated or removed to bypass validation.
- `scripts/release_preflight.sh` is the shared local/CI preflight entrypoint.

## Procedure

1. Confirm the worktree is clean and inspect the latest release tag.
2. Synchronize `[workspace.package] version` and every internal path dependency version in
   component `Cargo.toml` files. In the same release commit, update `README.md`,
   `README.zh-CN.md`, and the paired EN/zh-CN public-site release surfaces (`site/`),
   including the Documentation hubs. Do not leave site publication as a post-tag follow-up.
3. Run `./scripts/release_preflight.sh vX.Y.Z`. The script validates tag/version alignment, all
   Talos package versions, README/site release truth, public-site links/accessibility contracts,
   installer instructions, formatting, locked dependency resolution, check, Clippy, and tests.
4. Review `git diff --cached`, commit with the required model marker, and create an annotated tag.
5. Push the commit and tag. The tag-driven GitHub workflow owns release artifact creation.
6. Append the commit, tag, validation output, and workflow result to the release closeout task and
   synchronize `docs/BOARD.md`.

## Failure Rules

- A failed preflight blocks tagging.
- A README/site version mismatch or a failed public-site/installer check blocks tagging; correct
  the release commit and rerun the preflight.
- A failed workflow does not authorize moving or force-pushing the tag. Correct the source and use
  a new patch version/tag.
- `--locked` failures require fixing the committed lockfile or dependency declaration; deleting
  `Cargo.lock` or dropping `--locked` is not an acceptable workaround.
