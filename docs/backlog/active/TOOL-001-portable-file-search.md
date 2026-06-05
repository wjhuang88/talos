# TOOL-001: Portable File and Search Tools

## Outcome

Talos gains a native file/search tool baseline that works without host `find`, `grep`, `rg`, or
shell globbing.

## Status

Planned. Selected into I016.

## Priority

P2.

## Required Reads

- `docs/iterations/I016-portable-file-search.md`
- `docs/proposals/builtin-workspace-search-tools.md`
- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/decisions/009-tool-provenance.md`

## Acceptance Criteria

- [ ] Native file/search tools use structured parameters, not shell command strings.
- [ ] Workspace path bounds, ignore rules, binary/oversized-file skips, and output budgets are tested.
- [ ] Write-capable native tools remain permission-gated.
- [ ] Search works outside Git repositories and when host Git/search tools are absent.
- [ ] No persistent search DB, watcher, or extra native dependency lands without follow-up ADR.

## Residual Work Destination

Persistent indexes, frecency, watchers, and vector search belong in later items after baseline
performance is measured.

