# GIT-001: Built-in Git Tools — Native Git Support

## Outcome

Talos provides a comprehensive set of built-in Git tools powered by the `gix` crate (pure Rust
Git implementation), eliminating the need for a host `git` binary for common repository
operations. The agent can inspect, navigate, and (with explicit approval) modify Git state
through structured tool calls.

## Status

P0-P2 complete in I026. P3 advanced operations remain planned. I094 completed the `gix 0.85.0`
dependency upgrade and fallback audit. `gix` capability tracking is a standing requirement for this
story because host-`git` fallbacks are accepted only as documented bridges, not as the desired end
state.

I094 activation note (2026-07-04): selected for `gix 0.84.0 -> 0.85.0` upgrade attempt and
operation-by-operation host fallback audit. No Git permission-default change, destructive Git
operation, tag, publish, release, or issue-sync behavior is authorized by activation.

I094 closeout note (2026-07-04): `talos-tools` now requests `gix = "0.85"` and `Cargo.lock`
resolves `gix 0.85.0`. The explicit feature list remains `basic`, `status`, `revision`,
`blob-diff`, `index`, and `sha1`; no network, worktree-mutation, native Git, destructive Git,
permission-default, tag, publish, release, or issue-sync behavior was added.

Runtime leak note (2026-07-04): maintainer review found Talos runtime paths can still trigger host
Git outside the structured Git tool boundary. `crates/talos-cli/src/governance.rs` calls
`std::process::Command::new("git").args(["status", "--porcelain"])` for governance status output,
and `crates/talos-agent/prompts/identity.txt` still names `git` as a `bash` exception even though
read-only `git_status` is a built-in `gix` tool. This is a product bug, not an acceptable fallback:
read-only runtime status must use internal/gix-backed capability or omit Git state, and prompt
guidance must prefer built-in Git tools over host shell Git.

Runtime leak closeout (2026-07-05): `talos governance status` now uses
`talos_tools::git_dirty_count()`, which is backed by the native `gix` status API, and no longer
spawns host `git status --porcelain`. The identity prompt now names built-in Git tools for
read-only Git inspection and treats host shell Git as an explicit fallback only when no structured
Git tool covers the operation and the user approves that command.

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

Additional runtime leak:

- `talos governance status` currently shells out to host `git status --porcelain` for a dirty-tree
  count instead of using the internal Git tool/provider boundary.
- The agent identity prompt can still induce `bash git ...` by listing `git` as an example where
  bash is appropriate, despite the presence of built-in Git tools.

## Proposed Tool Set

### P0 — Read-Only (Auto-Allow in Workspace) — Complete

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

### P1 — Navigation (Ask) — Partial Complete

#### 6. `git_checkout`

Switch branches or restore files.

| | |
|---|---|
| **Nature** | Write (changes working tree) |
| **Parameters** | `branch` (string, required) |
| **Output** | `switched to branch {branch}` |
| **Implementation** | `gix` checkout API (if supported) or host `git checkout` fallback |

Delivered in I026 via host `git checkout` fallback because `gix` does not provide porcelain
checkout/switch orchestration.

#### 7. `git_stash_list`

List stashed changes.

| | |
|---|---|
| **Nature** | Read |
| **Parameters** | None |
| **Output** | One line per stash: `stash@{0}: WIP on main: a1b2c3d message` |
| **Implementation** | `gix` stash reading or host fallback |

Not delivered in I026. Remains future scope.

### P2 — Write Operations (Ask, Destructive) — Complete

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

### gix API Coverage (v0.85.0, verified 2026-07-04)

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
gix = { version = "0.85", default-features = false, features = [
    "basic", "status", "revision", "blob-diff", "index", "sha1"
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
- `git_stash_list` — gix stash is placeholder (not delivered in I026)

Operations using native `gix`:
- `git_add` — via `gix::index` manipulation
- `git_commit` — via `gix::Repository::commit()`

I026 implementation note: `git_add` and `git_commit` are delivered through the same structured
host-git adapter as the other write operations while native gix write orchestration remains under
evaluation. The externally visible contract stays explicit, permission-gated, and shell-free.

### Phase 3: gix Migration (Future)

As `gix` matures its write API surface (push, checkout, stash), migrate host-`git` fallbacks
to `gix` implementations. Track gix releases for:
- Push support (currently the #1 gap)
- Checkout/switch orchestration
- Stash support

### Standing gix Tracking Requirement

Track `gix` continuously until every host-`git` fallback has an explicit keep/replace decision.
This is not an implementation authorization by itself; each replacement still needs a scoped
iteration with permission and behavior regressions.

Update this owner doc when any of these triggers occur:

- A `gix` or gitoxide release changes push, checkout/switch, stash, reset, merge/rebase, index,
  commit, config-write, hook, or network support.
- A Talos Git tool feature touches `git_push`, `git_pull`, `git_checkout`, `git_add`,
  `git_commit`, stash, reset, merge, rebase, tags, or remotes.
- A release-readiness or self-bootstrap packet depends on Git publication behavior.
- A host-`git` fallback fails on a supported environment or becomes a portability blocker.
- A Talos CLI/runtime path invokes host `git` directly outside `crates/talos-tools/src/git.rs`.
- Agent prompt guidance presents host shell Git as preferred or normal when a built-in Git tool
  exists.

Each tracking update must record:

- `gix` version inspected and source of the capability claim.
- Feature flags needed and whether they add network, native, unsafe, or large dependency surface.
- Operation-by-operation decision: keep host fallback, replace with `gix`, defer, or reject.
- Permission classification impact for read/write/execute paths.
- Tests required before replacement, including unavailable-host behavior for any retained fallback.

Current tracking baseline:

| Operation family | Current Talos posture | Tracking decision |
|---|---|---|
| Read-only local status/diff/log/show/branches | Native `gix` direction is accepted and implemented for common reads. | Keep watching for API/output improvements, but no urgent migration gap. |
| Governance/runtime dirty-tree status | Fixed 2026-07-05. `talos governance status` uses `talos_tools::git_dirty_count()` backed by `gix::status`, and reports unavailable state instead of spawning host `git`. | Keep using the shared internal/gix-backed path; do not reintroduce direct host `git` calls in CLI/runtime status surfaces. |
| Agent prompt Git guidance | Fixed 2026-07-05. Identity prompt prefers built-in Git tools for read-only inspection and scopes host shell Git to explicit fallback only. | Keep prompt snapshots/tests aligned when Git tool names change. |
| Add/commit | Structured tool surface exists; native `gix` write orchestration remains under evaluation. | Re-evaluate on next Git write-tool work. |
| Push/pull | Host `git` fallback. | High-priority tracking item because REL-002 publication workflows depend on this boundary. |
| Checkout/switch | Host `git` fallback. | Track porcelain/worktree orchestration maturity before replacing. |
| Stash/reset/merge/rebase/tags/remotes | Future scope. | Do not implement without fresh `gix` coverage review and destructive-operation tests. |

Tracking update (2026-07-04, I094 closeout):

| Item | Observation | Decision |
|---|---|---|
| Current Talos dependency | `Cargo.lock` resolves `gix 0.85.0`; `crates/talos-tools` requests `gix = "0.85"` with `default-features = false` and features `basic`, `status`, `revision`, `blob-diff`, `index`, and `sha1`. | Upgrade landed in I094 with no feature expansion. |
| Latest available release | `cargo info gix` and docs.rs reported `gix 0.85.0`, license `MIT OR Apache-2.0`, Rust version `1.85`, during the I094 audit. | Keep 0.85 line; continue standing tracking for future releases. |
| Feature surface | `gix 0.85.0` keeps network features separate (`async-network-client`, `blocking-network-client`, HTTP transport features) and keeps `worktree-mutation` as an explicit feature. `cargo tree --invert gix@0.85.0 -e features` showed Talos still uses only the accepted local feature surface plus transitive features required by those selected features. | Do not enable network or worktree mutation features by default. Any future enablement needs permission and dependency-surface review. |
| Push | `gix 0.85.0` has a `push` module, but local source inspection shows it exposes `push.default` configuration values rather than a complete push workflow for Talos's `git_push` contract. | Keep `git_push` on structured host-`git` fallback. Revisit only when `gix` exposes a tested push workflow. |
| Checkout/switch | `gix 0.85.0` exposes low-level checkout options and worktree-state plumbing behind `worktree-mutation`, but not a Talos-ready branch switch/restore workflow. | Keep `git_checkout` on structured host-`git` fallback. Replacement requires branch switch, dirty-worktree, conflict, and permission regressions. |
| Pull/fetch/update | `gix` continues to expose fetch/remote plumbing, but Talos's `git_pull` contract includes worktree update/merge-conflict behavior. | Keep `git_pull` on structured host-`git` fallback until fetch + update/merge workflow is mapped and tested. |
| Add/commit | The Talos surface remains permission-gated and shell-free, but the current implementation uses the structured host-`git` adapter. | Keep host fallback in I094. Native `gix` write orchestration replacement needs a separate behavior-equivalence slice. |
| Stash/reset/merge/rebase | No new evidence in this pass that these are ready for Talos P3 user-facing tools. | Keep future scope; require a fresh coverage review before implementation. |

I094 validation evidence:

- `cargo fmt --all -- --check`: passed.
- `cargo check -p talos-tools`: passed.
- `cargo test -p talos-tools git`: passed.
- `cargo test -p talos-tools`: passed, 226 unit tests plus 18 integration tests and doctests.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace -- -D warnings`: passed.
- `cargo test --workspace`: passed.
- `cargo tree --invert gix@0.85.0 -e features`: passed.
- `scripts/validate_project_governance.sh .`: passed, 0 warnings.
- `git diff --check`: clean.

New regression: retained host-`git` fallbacks now have a unit test proving that an unavailable
host `git` executable returns `git not installed. Install git or use read-only tools.`

Sources inspected: `cargo info gix`, docs.rs `gix 0.85.0`, local registry source
`gix-0.85.0/Cargo.toml`, `src/push.rs`, `src/repository/checkout.rs`,
`src/repository/remote.rs`, and `src/worktree/mod.rs`.

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

## Delivered in I026

- `git_status`
- `git_diff`
- `git_log`
- `git_show`
- `git_branch_list`
- `git_add`
- `git_commit`
- `git_push`
- `git_pull`
- `git_checkout`

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

### Runtime Host-Git Leak Fix

- [x] `talos governance status` does not invoke host `git`.
- [x] Governance dirty-tree status uses a shared internal/gix-backed read path or is omitted with
      an explicit unavailable message.
- [x] Agent prompt guidance says to use built-in Git tools for Git inspection before `bash`.
- [x] Tests prove governance status works when host `git` is unavailable.
- [x] Tests or prompt snapshots prove `git` is no longer listed as a normal bash exception while
      `git_status` is available.
- [ ] No permission default changes are introduced.

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
- `crates/talos-tools/src/git.rs` (current gix/host fallback boundary)
- `crates/talos-cli/src/governance.rs` (current direct host `git status` leak)
- `crates/talos-agent/prompts/identity.txt` (current prompt guidance leak)

## Open Questions

1. **gix vs host git for write operations**: Does gix 0.84+ support commit, push, and checkout
   reliably? Or should we use host `git` for all write operations in Phase 2?
2. **Auto-commit behavior**: Should Talos auto-commit after write/edit operations (like Aider)?
   Or leave all commits to explicit `git_commit` calls?
3. **Git config**: Should Talos respect `.gitconfig` for user.name/user.email, or require
   explicit config in Talos config?
4. **Partial clone / sparse checkout**: Should we support large repos with partial clone?

## Reference: Other Agent Git Designs

Survey of how major coding agents handle Git operations (2026-06-17):

### Aider: Auto-Commit Model

- Git is a **first-class citizen**, handled internally by `GitRepo` class
- **Auto-commits after each AI edit** (default ON, configurable via `--no-auto-commits`)
- **Dirty-commit before AI edits**: commits current dirty state before LLM touches files
  (creates undo checkpoint)
- Conventional Commits format for auto-generated messages
- Author attribution: appends `(aider)` to committer name
- No explicit per-commit approval — fully automatic
- Slash commands: `/diff`, `/undo` (reset --hard HEAD~1), `/commit`, `/git`

### Claude Code: Explicit-Only Model

- **No auto-commit** — Git is driven through Bash tool with slash commands
- `/commit` slash command: restricted to `git add`, `git status`, `git commit` only
- Safety constraints: never amends, never skips hooks, never commits secrets
- `/commit-push-pr`: full workflow (branch → commit → push → PR) as a single command
- Permission model: `allowed-tools` frontmatter restricts which git operations each command
  can use; users can configure deny rules like `Bash(git push --force)`

### OpenCode: No Dedicated Git Tools

- **No built-in Git tools** — Git operations go through `bash` tool
- Internal Git service exists but is read-only (status, diff, branch detection) — not exposed
  to LLM
- Community plugin (`@slorenzot/git-plugin-opencode`) provides 19 Git tools as MCP tools
- Permission model: `BashArity` system allows separate permission rules for `git push`
  (high-risk) vs `git status` (low-risk)

### Recommended Design for Talos

Based on proven patterns:

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Auto-commit | ❌ Default OFF | Claude Code model — explicit is safer; opt-in config possible later |
| Commit message | LLM-generated, Conventional Commits | All agents converge on this |
| Permission tiers | Read → Allow, Write → Ask, Destructive → Ask | ToolNature::Read/Write/Execute |
| Secret protection | Block `.env`, `credentials.*`, `*.key` | Claude Code safety pattern |
| Safe git config | `--no-optional-locks`, `core.autocrlf=false` | OpenCode pattern |
| Force push | Always Ask (never auto-allow) | Universal across all agents |
| Commit attribution | No forced attribution | Unlike Aider — user's commits are theirs |
