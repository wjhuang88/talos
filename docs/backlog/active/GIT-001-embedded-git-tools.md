# GIT-001: Embedded Git Tools

## Outcome

Talos exposes a small read-only Git tool surface using a `gix`-first implementation strategy.

## Status

Planned. Selected into I017.

## Priority

P2.

## Required Reads

- `docs/iterations/I017-embedded-git-tools.md`
- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/proposals/builtin-workspace-search-tools.md`

## Acceptance Criteria

- [ ] Read-only Git tools use structured arguments, not raw `git(args)` passthrough.
- [ ] `gix` is the primary implementation target where API coverage is practical.
- [ ] Any host `git` fallback is direct-process, allowlisted, bounded, and documented.
- [ ] No `git2`/libgit2 dependency is added in this slice.
- [ ] Write-capable Git operations remain out of scope until permission behavior is reviewed.

## Residual Work Destination

Write-capable Git operations require a separate activation and permission review.

