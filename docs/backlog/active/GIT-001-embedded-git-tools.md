# GIT-001: Built-in Git Tools — Native Git Support

## Outcome

Talos provides a comprehensive set of built-in Git tools powered by the `gix` crate (pure Rust
Git implementation), eliminating the need for a host `git` binary for common repository
operations. The agent can inspect, navigate, and (with explicit approval) modify Git state
through structured tool calls.

## Status

Planned. Selected into a future iteration.

## Priority

P2.

## Related

- ADR-010 (Git dependency boundary: gix preferred, git2/libgit2 rejected)
- TOOL-003 (POSIX tool set — same tool registration and permission patterns)
- I017 (original read-only Git tools iteration — superseded by this expanded scope)

## Problem

The agent currently relies on `bash git ...` for all Git operations. This has three problems:

1. **Host dependency**: Requires `git` installed on the host. Violates self-contained-first.
2. **Unstructured output**: The agent must parse raw `git` output, wasting tokens and risking
   misinterpretation.
3. **No permission granularity**: Every `git` command goes through bash approval. Read-only
   Git inspection (status, log, diff) should be auto-allowed; write operations (commit, push,
   reset) should require explicit approval.

## Proposed Tool Set

### P0 — Read-Only (Auto-Allow in Workspace)

#### 1. `git_status`

Show working tree status.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | `path` (optional, defaults to workspace root), `porcelain` (bool, default true for compact output) |
| **Output** | One line per changed file: `M src/main.rs` / `?? new.txt` / `A staged.rs` |
| **Implementation** | `gix` status API |

#### 2. `git_diff`

Show unstaged or staged changes.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | `staged` (bool, default false), `path` (optional, file filter), `max_lines` (optional, default 200) |
| **Output** | Unified diff text |
| **Implementation** | `gix` diff API |

#### 3. `git_log`

Show commit history.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | `max_count` (optional, default 20), `oneline` (bool, default true), `path` (optional, file filter), `author` (optional) |
| **Output** | One line per commit: `a1b2c3d - message (author, date)` |
| **Implementation** | `gix` revision walk API |

#### 4. `git_show`

Show details of a specific commit.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | `revision` (string, required, e.g. "HEAD" or "a1b2c3d"), `stat` (bool, default true for file list) |
| **Output** | Commit message, author, date, changed files, optional diff |
| **Implementation** | `gix` object reading API |

#### 5. `git_branch_list`

List branches.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | `remote` (bool, default false, include remote branches), `all` (bool, default false) |
| **Output** | One branch per line, `*` marks current: `* main` / `  feature/x` |
| **Implementation** | `gix` reference API |

### P1 — Navigation (Ask)

#### 6. `git_checkout`

Switch branches or restore files.

| | |
|---|---|
| **Nature** | Write (changes working tree) |
| **Parameters** | `branch` (string, required) |
| **Output** | `switched to branch {branch}` |
| **Implementation** | `gix` checkout API (if supported) or host `git checkout` fallback |

#### 7. `git_stash_list`

List stashed changes.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | None |
| **Output** | One line per stash: `stash@{0}: WIP on main: a1b2c3d message` |
| **Implementation** | `gix` stash reading or host fallback |

### P2 — Write Operations (Ask, Destructive)

#### 8. `git_add`

Stage files.

| | |
|---|---|
| **Nature** | Write |
| **Parameters** | `paths` (array of strings, required) |
| **Output** | `staged N file(s)` |
| **Implementation** | `gix` index manipulation or host `git add` fallback |

#### 9. `git_commit`

Create a commit.

| | |
|---|---|
| **Nature** | Write |
| **Parameters** | `message` (string, required), `all` (bool, default false, stage all tracked files) |
| **Output** | `committed: {short_sha} {message_first_line}` |
| **Implementation** | `gix` commit API or host `git commit` fallback |

#### 10. `git_push`

Push to remote.

| | |
|---|---|
| **Nature** | Write (network + destructive) |
| **Parameters** | `remote` (string, default "origin"), `branch` (string, optional, defaults to current), `force` (bool, default false) |
| **Output** | `pushed {branch} to {remote}` or error |
| **Implementation** | `gix` push API (if network features supported) or host `git push` fallback |

#### 11. `git_pull`

Pull from remote.

| | |
|---|---|
| **Nature** | Write (network + modifies working tree) |
| **Parameters** | `remote` (string, default "origin"), `branch` (string, optional) |
| **Output** | `pulled {branch} from {remote}` or merge conflict report |
| **Implementation** | `gix` fetch + merge or host `git pull` fallback |

### P3 — Advanced (Future)

- `git_merge` — merge branches
- `git_rebase` — rebase commits
- `git_reset` — reset HEAD (destructive)
- `git_clean` — remove untracked files (destructive)
- `git_tag` — create/list tags
- `git_remote` — manage remotes
- `git_blame` — line-level history

## Implementation Strategy

### gix API Coverage (v0.84.0, verified 2026-06-17)

Based on the gitoxide crate-status survey:

| Operation | gix Support | Notes |
|-----------|-------------|-------|
| Open/discover repo | ✅ | `gix::open()`, `gix::discover()` |
| Status (index vs worktree) | ✅ | `gix::status` |
| Diff (blob + tree) | ✅ | `gix::diff` via `imara-diff` |
| Rev-walk / log | ✅ | `gix::Repository::rev_walk()` |
| Object reading (commit/tree/blob/tag) | ✅ | `gix::Repository::find_object()` |
| Branch listing (refs) | ✅ | `gix::refs` |
| Blame | ✅ | `gix::blame` |
| Describe / merge-base | ✅ | `gix::Repository::describe()`, `merge_base()` |
| Init repo | ✅ | `gix::init()` |
| Clone | ✅ | `gix::prepare_clone()` |
| Fetch | ✅ | `gix::protocol::fetch()` |
| Create commit | ✅ | `gix::Repository::commit()` — no hooks, no signing |
| Index read/write | ✅ | `gix::index` — can stage/unstage via index manipulation |
| **Push** | ❌ | `gix-protocol` push not implemented |
| **Checkout / switch** | ❌ | No porcelain orchestration |
| **Stash** | ❌ | `gix-stash` is placeholder |
| **Reset** | ❌ | Not implemented |
| **Rebase** | ❌ | `gix-rebase` is placeholder |
| **Merge** (full workflow) | ⚠️ | 3-way merge ✅, but no MERGE_HEAD orchestration |
| Config write | ❌ | Can read, cannot persist |
| Hooks | ❌ | Not implemented |

### Phase 1: gix Read-Only (P0)

Use `gix` with minimal features for all read-only operations. No host `git` dependency.

```toml
gix = { version = "0.84", default-features = false, features = [
    "status", "revision", "blob-diff", "sha1", "index", "blame"
] }
```

### Phase 2: Host Git Bridge (P1-P2)

For operations `gix` doesn't support (push, checkout, stash, reset), use a structured
host `git` adapter:
- Direct process invocation (no `sh -c`)
- Allowlisted command shapes
- Structured arguments (not raw command strings)
- Timeout and output bounding
- Per-operation permission classification

Operations using host `git` fallback:
- `git_checkout` — no gix checkout orchestration
- `git_push` — gix has no push support
- `git_pull` — gix fetch works, but no worktree update after fetch
- `git_stash_list` — gix stash is placeholder

Operations using native `gix`:
- `git_add` — via `gix::index` manipulation
- `git_commit` — via `gix::Repository::commit()`

### Phase 3: gix Migration (Future)

As `gix` matures its write API surface (push, checkout, stash), migrate host-`git` fallbacks
to `gix` implementations. Track gix releases for:
- Push support (currently the #1 gap)
- Checkout/switch orchestration
- Stash support

## Permission Model

| Tool | Nature | Default Permission | Rationale |
|------|--------|-------------------|-----------|
| git_status | Read | Allow | No mutation |
| git_diff | Read | Allow | No mutation |
| git_log | Read | Allow | No mutation |
| git_show | Read | Allow | No mutation |
| git_branch_list | Read | Allow | No mutation |
| git_stash_list | Read | Allow | No mutation |
| git_checkout | Write | Ask | Changes working tree |
| git_add | Write | Ask | Modifies index |
| git_commit | Write | Ask | Creates history |
| git_push | Execute | Ask | Network + destructive |
| git_pull | Execute | Ask | Network + modifies working tree |

All tools will use the `ToolNature` attribute (from I025 S5) for permission classification.

## Design Constraints

- **No raw git passthrough**: All tools use structured arguments. No `git(args)` string.
- **No `sh -c`**: Host git invoked directly with arg vector.
- **Workspace root bounded**: All operations limited to the workspace repository.
- **Output bounding**: All outputs truncated at configurable limits to prevent context blowup.
- **Permission pipeline**: Write/Execute operations go through the standard approval flow.
- **AGENTS.md compliance**: Follows ADR-010 (gix preferred, git2 rejected, host git as
  documented fallback only).

## Acceptance Criteria

### P0 (Read-Only)
- [ ] `git_status` shows working tree status in porcelain format
- [ ] `git_diff` shows unified diff with staged/unstaged filter
- [ ] `git_log` shows commit history with count limit
- [ ] `git_show` shows commit details
- [ ] `git_branch_list` lists local and remote branches
- [ ] All P0 tools registered in all 4 registry builders
- [ ] All P0 tools auto-allowed in workspace (ToolNature::Read)
- [ ] Unit tests for each tool
- [ ] `gix` dependency added with minimal feature set
- [ ] No host `git` required for P0 operations

### P1-P2 (Navigation + Write)
- [ ] `git_checkout` switches branches
- [ ] `git_add` stages files
- [ ] `git_commit` creates commits
- [ ] `git_push` pushes to remote
- [ ] `git_pull` pulls from remote
- [ ] All write tools require explicit approval (ToolNature::Write/Execute)
- [ ] Host `git` fallback documented with rationale per operation

## Required Reads

- `docs/decisions/010-git-search-tool-dependency-boundary.md`
- `docs/backlog/active/TOOL-003-posix-tool-set.md` (tool registration pattern)
- `crates/talos-tools/src/lib.rs` (AgentTool trait, ToolNature)
- `crates/talos-permission/src/lib.rs` (permission engine)

## Open Questions

1. **gix vs host git for write operations**: Does gix 0.66+ support commit, push, and checkout
   reliably? Or should we use host `git` for all write operations in Phase 2?
2. **Auto-commit behavior**: Should Talos auto-commit after write/edit operations (like Aider)?
   Or leave all commits to explicit `git_commit` calls?
3. **Git config**: Should Talos respect `.gitconfig` for user.name/user.email, or require
   explicit config in Talos config?
4. **Partial clone / sparse checkout**: Should we support large repos with partial clone?
