# Built-in Workspace Search Tools

## Status

Proposal. Captured 2026-06-05 for I012 Portable Tools as the detailed design
behind `#I012-S3` and the search side of `#I012-S4`.

## Motivation

Talos needs search tools that are reliable in locked-down environments and
better shaped for agent use than raw shell commands:

- path search should return bounded, ranked, workspace-relative results;
- content search should skip binaries and very large files by default;
- results should be deterministic enough for model consumption and replay;
- search should not depend on `rg`, `find`, `grep`, or shell features being
  available on the host;
- future Git-aware ranking should help the agent prioritize actively changed
  files while preserving Talos' self-contained-first dependency principle.

The `fff` project is the main reference. Its design combines a long-running
file picker, background scan, optional watcher, typo-resistant fuzzy search,
content grep, frecency, Git status annotations, and optional content prefilters.
Talos should borrow the architecture and ranking ideas, but implement a smaller
first version that fits Talos' dependency and permission constraints.

## Reference Design Notes from `fff`

Useful ideas:

- **Long-running index**: scan once, query many times.
- **Separate path search and grep**: file name/path matching and content search
  have different result shapes and budgets.
- **Workspace-relative metadata**: results carry relative path, file name,
  size, modified time, binary flag, and optional Git status.
- **Fuzzy fallback**: exact or plain search first; fuzzy/typo-tolerant behavior
  is useful when exact results are empty.
- **Git-aware ranking**: modified/untracked/staged files are often more useful
  to an agent.
- **Frecency**: recently and frequently opened files deserve a boost in
  interactive workflows.
- **Candidate prefiltering**: bigram or similar content prefilters can reduce
  grep cost on large repositories.

Ideas to defer:

- Filesystem watcher. It creates hard edge cases around ignored files,
  symlinks, directory deletion, and workspace root changes.
- Persistent frecency/query databases. They introduce write behavior and
  storage paths that need security and privacy review.
- Bigram content index. It is valuable for scale but has a complex memory model
  and, in fff, includes `unsafe`-heavy implementation details.
- Native Git library dependency beyond the ADR-010 `gix` direction.
  `git2`/libgit2 remains rejected for I012's first slices.

## Proposed Tool Surface

### `find_files`

Read-only structured tool.

Input:

```json
{
  "query": "state tui",
  "path": "crates/talos-tui/src",
  "glob": "*.rs",
  "limit": 50,
  "mode": "auto"
}
```

Fields:

- `query`: required. Matched against workspace-relative path and file name.
- `path`: optional workspace-relative directory/file prefix.
- `glob`: optional include glob. Excludes can be added later.
- `limit`: optional, capped by config/default.
- `mode`: `auto | exact | fuzzy`. `auto` tries exact-ish matching first and may
  fall back to fuzzy when empty.

Output:

```json
{
  "items": [
    {
      "path": "crates/talos-tui/src/state.rs",
      "name": "state.rs",
      "score": 832,
      "size": 12345,
      "modified": 1780627200,
      "git_status": "modified"
    }
  ],
  "truncated": false
}
```

### `grep`

Read-only structured tool.

Input:

```json
{
  "pattern": "ApprovalState",
  "path": "crates",
  "glob": "*.rs",
  "mode": "auto",
  "context": 2,
  "limit": 100
}
```

Fields:

- `pattern`: required.
- `path`: optional workspace-relative path.
- `glob`: optional include glob.
- `mode`: `auto | literal | regex | fuzzy`.
- `context`: optional context lines, capped.
- `limit`: optional match cap.

Output:

```json
{
  "matches": [
    {
      "path": "crates/talos-tui/src/state.rs",
      "line": 12,
      "column": 10,
      "text": "pub enum ApprovalState {",
      "before": [],
      "after": []
    }
  ],
  "files_searched": 42,
  "files_skipped": 3,
  "truncated": false
}
```

### Later Tool Candidates

- `search_symbols`: definition-first search once parsing/heuristics stabilize.
- `search_recent`: frecency-backed path search after privacy/storage review.
- `rescan_workspace`: explicit refresh when a session-level index exists.

## Architecture

### R1: Stateless Search

No persistent index, no watcher.

Implementation shape:

- Walk from `workspace_root / path`.
- Reject path escape before walking.
- Respect `.gitignore` and common ignore rules.
- Skip binary files and files over a configured max size.
- Use deterministic scoring:
  - exact file-name match
  - path segment match
  - substring match
  - extension/path proximity bonus
  - optional Git modified/untracked/staged boost when available
- Cap output by both item count and approximate byte budget.

R1 can use small Rust crates if needed:

- `ignore` for `.gitignore`-aware walking;
- `globset` for include/exclude patterns;
- `regex` for regex grep.

These are Rust crates already common in the ecosystem and do not create the
native dependency issue that `git2` creates. If they are added, run
`cargo deny` or equivalent dependency review before merge if available.

### R2: Session-Local Index

In-memory only. Built lazily per workspace and discarded on process exit.

Adds:

- file metadata cache;
- invalidation by explicit rescan;
- optional query time budget;
- stable pagination cursor.

No watcher, no persistent database.

### R3: Git-Aware Ranking

Read Git status through the `#I012-S4` structured Git provider. The primary
implementation target is `gix`; host `git` is a fallback/bridge only.

Rules:

- Search still works if Git is missing or the workspace is not a repository.
- Git status is a ranking hint only, not a filter unless user requests it.
- Git calls must be bounded and must not shell out via `sh -c`.

### R4: Advanced Indexing

Only after benchmarks justify it:

- frecency storage;
- query history;
- background watcher;
- content prefilter such as bigram index.

Each persistent or watcher feature requires a fresh review before
implementation.

## Git Dependency Judgment

Do not add `git2` for the first search/Git slices.

Initial Git functionality should target `gix` because it:

- avoids `git2`/libgit2 native dependency expansion;
- avoids requiring a host `git` binary for local repository reads;
- fits the self-contained-first project principle;
- exposes feature-level control for local capabilities such as status and
  revision traversal.

Host `git` may still exist as a structured fallback because it:

- keeps command semantics identical to user expectations when fallback is used;
- lets each operation be permission classified;
- can run with explicit timeout and workspace root;
- keeps a temporary bridge available for operations whose `gix` mapping is not
  yet implemented.

The Git implementation evaluation compares:

- `gix` / gitoxide pure-Rust crates;
- `git2` / libgit2 bindings;
- continuing with structured host-`git`.

ADR-010 records the result: `gix` is the preferred first-slice target, host
`git` is a fallback/temporary bridge, and `git2`/libgit2 is rejected.

## Git Dependency Spike Result (2026-06-05)

The Spike has been completed for the first I012 decision point.

Result:

- Use **`gix`** as the preferred implementation target for initial read-only Git tools.
- Keep **structured host-`git`** only as a compatibility fallback or temporary bridge.
- Reject **`git2`/libgit2** for the first slice because it adds a native dependency and requires a
  separate ADR/security review.

Why this is enough for search:

- `find_files` and `grep` must not depend on Git metadata for correctness.
- Git status can be added as an optional ranking hint through the structured Git provider.
- If Git metadata is unavailable, search degrades to non-Git ranking.
- No persistent Git cache is needed in R1.

Validated `gix` evidence:

- `cargo info gix` reported `gix 0.84.0`, license `MIT OR Apache-2.0`, Rust version `1.85`.
- Feature inspection showed `status`, `revision`, and `sha1` can be selected without default
  network-oriented features.
- A temporary Spike crate compiled with `default-features = false` and read Talos `head_name`,
  `head_id`, and `is_dirty`.

Validated host-`git` fallback command shapes:

- `git status --porcelain=v1`
- `git diff --name-status`
- `git log -3 --format=... --date=iso-strict`
- `git show --stat --oneline --no-renames HEAD`

Implementation implication:

- Git-aware search ranking should consume a small normalized status map from the structured Git
  provider, not call Git directly from inside the search implementation.
- The search implementation must treat Git status lookup failure as a warning/degraded metadata
  state, not as search failure.

## Error-Prone Design Points

### Workspace Boundaries

All input paths must be resolved against `workspace_root` and rejected if the
canonical target escapes. Symlink handling must be explicit:

- R1 default: do not follow symlinks during search walk.
- If following symlinks is ever enabled, it needs loop detection and canonical
  workspace-boundary checks.

### Ignore Semantics

Search defaults should respect `.gitignore`, `.ignore`, and common hidden build
directories. Users need explicit opt-in to search ignored files. Ignored file
search can expose secrets or enormous generated content.

### Binary and Large Files

Use a binary sniff threshold similar to existing file tools. Grep must skip
binary and oversized files by default and report counts so the model understands
what was omitted.

### Regex Safety

Use Rust `regex`, not backtracking PCRE. Reject empty wildcard-only patterns
that would flood context. Enforce `limit`, `max_file_size`, and time budget.

### Ranking Stability

Ranking should be deterministic for the same repo state. Use explicit
tiebreakers such as path lexicographic order after score, then file size or
modified time only where documented.

### Git Status Coupling

Git status must never be required for basic search. Status failures should
degrade to non-Git ranking with a warning field, not fail the entire search.

### Output Budget

Tool results go into model context. Every search tool must enforce:

- max result count;
- max bytes/chars per result line;
- max total output bytes/chars;
- explicit `truncated: true` when clipped.

### Permission Boundary

Search is read-only, but persistent indexes, frecency databases, query history,
and exports are writes. Those features are not read-only just because their
primary user-facing command is "search".

## Acceptance for the First Implementation Slice

- `find_files` and `grep` are structured `AgentTool`s.
- Both are workspace-root bounded and reject path escape.
- Both enforce output budgets and report truncation/skipped files.
- `grep` skips binary/oversized files.
- `.gitignore` is respected by default.
- Search works without host `find`, `grep`, `rg`, or `git`.
- Git-aware ranking is either absent in R1 or degraded cleanly when Git is not
  available.
- Unit tests cover path escape, ignored files, binary skip, output truncation,
  and deterministic ordering.
- `cargo test -p talos-tools` and `cargo test --workspace` pass.

## References

- `fff` repository: <https://github.com/dmtrKovalenko/fff>
- `fff` core file picker: <https://raw.githubusercontent.com/dmtrKovalenko/fff/main/crates/fff-core/src/file_picker.rs>
- `fff` scoring pipeline: <https://raw.githubusercontent.com/dmtrKovalenko/fff/main/crates/fff-core/src/score.rs>
- `fff` bigram filter: <https://raw.githubusercontent.com/dmtrKovalenko/fff/main/crates/fff-core/src/bigram_filter.rs>
- ADR-010: [Git and Search Tool Dependency Boundary](../decisions/010-git-search-tool-dependency-boundary.md)
