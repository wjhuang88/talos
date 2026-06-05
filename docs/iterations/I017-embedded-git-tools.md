# I017: Embedded Git Tools

**User can**: Inspect repository status, diffs, history, and branches through structured Git tools
without requiring a host `git` binary for the first read-only slice.

## Status: PLANNED

## Decision Gate

Follow ADR-010. `gix` is the primary implementation target. Host `git` may be used only as a
documented fallback/temporary bridge. `git2`/libgit2 remains rejected unless a later ADR approves
that dependency.

## Selected Stories

- [ ] #I012-S4: Self-contained structured Git tools

## Scope

- Implement read-only `git_status`, `git_diff`, `git_log`, `git_show`, and `git_branch_list`.
- Use structured arguments and bounded outputs.
- Expose normalized status metadata for search ranking.
- Verify behavior on a deliberately minimal `PATH`.

## Non-Goals

- No write-capable Git operations.
- No generic `git(args)` passthrough.
- No host `git` as the primary implementation path.

## Acceptance Criteria

- [ ] Read-only Git tools have stable schemas and reject unsupported flags.
- [ ] Minimal-`PATH` verification passes through the `gix` provider for covered operations.
- [ ] Any operation-level fallback records rationale and replacement trigger.
- [ ] Search can consume Git status hints without failing when Git metadata is unavailable.
- [ ] `cargo test -p talos-tools --features git-tools` or the equivalent workspace test passes.
