# GIT-001: Built-in Git Tools — Native Git Support

## Current corrective delivery — 2026-10-26

I300/T7 is Complete / Closed at #735, merge `ee84889fd19c590fdcc125cb971143dffd3cfb94`.
Completion Commit: ee84889fd19c590fdcc125cb971143dffd3cfb94

The [I300 closure](../../iterations/I300-structured-git-repair.md) records the
original acceptance mapping, full local locked preflight, final all-success
three-platform CI 40251434249 and independent exact-head/base review.
Prepared Add, Commit, nonrecursive Checkout, limited Push and native Pull are
merged; configuration discovery and final mutation/whole-command authority stay
separate. Explicit Deny/human policy and process supervision remain binding.
Default Windows git:// Push remains the accepted host limitation; configured
Push passed natively, and production does not alter sideband configuration.
T8/PERM-007-H and advanced P3 scope remain incomplete. Earlier candidate and
blocker sections below are historical, superseded by this delivery record.

### Standing gix tracking — 2026-10-26

The live [crates.io sparse index](https://index.crates.io/3/g/gix) reports latest
non-yanked **gix 0.90.0**, published 2026-10-22, MSRV 1.88; Talos remains locked
to 0.87.1. The [exact upstream release](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.90.0)
adds linked-worktree lock/unlock and separate registration/initial checkout,
submodule resolved URLs, global attribute-path selection, absolute-path helpers
and offline default-branch lookup. Fixes cover linked-worktree submodule gitdirs,
replacement refs/commit-graph and clone config/ref parity. Upstream also cites
GHSA-q54r-jf2p-rfr2 for configuration section-trust filtering.

This is release-note/API discovery, not an upgrade or Talos security-applicability
assessment. **Open residual owned here:** assess the advisory against Talos 0.87.1
and evaluate operation-specific parity before selecting an upgrade/replacement
iteration. Release additions do not demonstrate complete native Push or Pull
merge/rebase orchestration replacement; source and runtime parity verification
remain required. No date for removing host Git is established. This tracking
supersedes the older 0.89 availability checkpoint without changing Cargo.lock.


2026-10-26 merge blocker: PR #735 CAS exposed an I307 protected-mount overlap
finding for broad/canonical workspace roots. Independent merge readiness was
withdrawn; green narrow-root CI does not cover it. I300's top checkpoint owns
this prerequisite and the required sandbox follow-up. T7 is not Complete.


Tracking recheck (2026-10-17): the live crates.io sparse index at
`https://index.crates.io/3/g/gix` still ends at non-yanked `gix 0.89.0`
(MSRV 1.88), while the Talos lockfile remains at 0.87.1. This is a release
availability check, not a new API or parity review. The 0.89.0 source audit
below remains the capability evidence: fetch, merge primitives and some
worktree operations are available, but it did not establish complete
checkout/switch, push, or Pull merge/rebase orchestration. Keep the existing
host-Git bridges and their separate Read/Write/Execute and network admission
until each replacement passes operation-specific parity, security, unavailable-
host, and runtime tests. No Git-removal date can be inferred from this recheck.

## Outcome

Talos provides a comprehensive set of built-in Git tools powered by the `gix` crate (pure Rust
Git implementation), eliminating the need for a host `git` binary for common repository
operations. The agent can inspect, navigate, and (with explicit approval) modify Git state
through structured tool calls.

## Historical I300 corrective candidate — before delivery

Historical I026/I094 acceptance below does not prove closure of the current
permission and diff defects. [I300](../../iterations/I300-structured-git-repair.md)
owns the active repair; no implementation merge or completion SHA exists yet.

2026-10-25 evidence: pushed candidate `a51d44b27` (PR #735) has the original
two-document paths in the production Agent regression, including preservation
of unrelated staged and unstaged bytes (3/3 approved/denied cases pass).
I300's top audit maps the complete ADR diff matrix and distinguishes shared
proof replay protection from MCP protocol request-ID reservation. Independent
whole-T7 source review found no new concrete permission/process/API blocker;
final full validation, platform CI, governance/CAS and merge remain required.
Prior native Windows configured MCP Push passed; default git:// Push remains
an accepted host limitation, and production preserves the user's sideband setting.
The previous full preflight passed Tools 934/934 before the last test correction;
current executable source now also passes the full authorized standard preflight
(exit 0; Tools 934/934 and both SDK consumers). Current macOS/Linux CI passes;
Windows also passed; this run is terminal success. The sandboxed attempt failed EPERM and one Search
classification case; no intermittent Search cause or repair is claimed. No completion
or exhaustive cross-entry test coverage is claimed.

The local `git_add` candidate interprets selected paths literally and separately
authorizes fixed Git configuration-chain discovery and the resulting exact
repository mutation plan. Configuration values remain internal. Unsupported
filters/hooks/fsmonitor and object-alternate/split-index layouts refuse with an
explicit separately authorized fallback; there is no silent helper bypass.
Ordinary custom Git installations require support for the fixed attribute-path
queries. The unpublished candidate also implements prepared Commit,
nonrecursive Checkout, limited Push and native Pull. Pull retains separate
configuration discovery, followed by one whole-command authorization; dedicated
Auto review can approve the final command, while explicit Deny and human-only
policies remain authoritative. This grants transitive command effects rather
than per-path/endpoint confinement, and failures may leave partial effects.
Unix process-group and Windows Job supervision are connected; affected MSVC
test targets cross-compile with cargo xwin. Native Windows execution, exact-head
CI and the complete acceptance mapping remain open; see the
[migration inventory](../../reference/I300-GIT-AUTHORIZATION-MIGRATION.md).

Local diff repairs compare HEAD/index for staged changes and index/worktree for
unstaged changes, preserve additions/deletions/binary/mode/symlink distinctions,
and render staged gitlinks from object IDs without loading child commit objects.
`core.fileMode=false` suppresses only ordinary worktree executable-bit differences;
staged mode changes remain visible. Unstaged submodule OID/dirty/ignore behavior
is covered by native gix state and host comparison fixtures. Absent/empty checkout and ordinary-file replacement fixtures also pass; broader
deinitialization, cross-platform/publication validation remain open. Current
focused Git tests pass 23/23; this bounded evidence does not reopen or replace historical acceptance.

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

Switch branches. The `branch` parameter does not select files to restore.

| | |
|---|---|
| **Nature** | Write (changes working tree) |
| **Parameters** | `branch` (string, required), `create` (boolean, default false) |
| **Output** | `switched to branch {branch}` |
| **Implementation** | `gix` checkout API (if supported) or host `git checkout` fallback |

Delivered in I026 via host `git checkout` fallback because `gix` does not provide porcelain
checkout/switch orchestration.

The I300 development candidate explicitly separates the branch argument from
pathspecs: a missing branch matching a tracked filename fails without restoring
that file. Previous-branch `-`, branch creation and unique remote-tracking branch
guessing remain supported. Broader prepared authorization/lifecycle migration
is still incomplete.

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

### gix API Coverage (registry and release rechecked 2026-10-13; latest published 0.89.0)

Registry recheck on 2026-10-13 against `https://index.crates.io/3/g/gix`
still reports 0.89.0, non-yanked, with Rust 1.88 as its MSRV. The
[0.89.0 release notes](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0)
add safe linked-worktree creation, checkout and removal, including branch
protection and rollback. That is a narrower workflow than switching the current
worktree, which remains unchecked in upstream's development-status survey.
No dependency upgrade or new host-Git removal date is implied.

Talos currently requests and locks **0.87.1**. The registry lists **0.89.0** as
the latest published, non-yanked version; 0.88/0.89 require Rust 1.88, below our
pinned 1.99.0. This survey does not upgrade dependencies. The table describes
upstream capability status, not a successful Talos integration of 0.89.0.

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
| Linked worktree add/remove | ✅ in 0.89.0 | New `worktree-mutation` APIs create and check out a *new linked worktree* or remove one; Talos has not integrated them and they do not replace current-worktree branch switching. |
| **Stash** | ❌ | `gix-stash` is placeholder |
| **Reset** | ❌ | Not implemented |
| **Rebase** | ❌ | `gix-rebase` is placeholder |
| **Merge** (full workflow) | ⚠️ | 3-way merge ✅, but no MERGE_HEAD orchestration |
| Config write | ❌ | Can read, cannot persist |
| Hooks | ❌ | Not implemented |

### Phase 1: gix Read-Only (P0)

Use `gix` with minimal features for all read-only operations. No host `git` dependency.

```toml
gix = { version = "0.87.1", default-features = false, features = [
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

Native write migration targets (not the current implementation):
- `git_add` — investigate `gix::index` manipulation
- `git_commit` — investigate `gix::Repository::commit()` and required hook/signing parity

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

2026-10-15 release/source recheck: `cargo info gix` reports locked 0.87.1
and latest 0.89.0; `cargo info gix@0.89.0` successfully downloaded the
published crate and reports MSRV 1.88. The annotated `gix-v0.89.0` tag
dates to 2026-10-08 and targets `0115e902fd0334b8f370766993eb17babed3157f`.
Its release notes confirm linked-worktree addition/checkout/removal and the
breaking public `gix::Result` migration. These are not ordinary branch-switch
or push/rebase workflow parity. Development sources inspected at
`8dc7afdeed215558e89f0c49fd87621d46a98307` (2026-10-14) still mark push
client/server plumbing, rebase, and checkout/switch orchestration incomplete.
The config include implementation still documents section-end insertion order
and hasconfig URL-prohibition deviations and directly opens included files.
Thus upgrading alone does not replace I300's exact-admission target evaluator.
Keep existing host bridges pending operation-specific parity/security gates;
no dependency upgrade or removal date is claimed. The HTTP registry API returned
403; availability evidence is Cargo plus the matching annotated release tag.
Sources: [0.89.0 release](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0),
[pinned development status](https://github.com/GitoxideLabs/gitoxide/blob/8dc7afdeed215558e89f0c49fd87621d46a98307/crate-status.md),
[pinned include implementation](https://github.com/GitoxideLabs/gitoxide/blob/8dc7afdeed215558e89f0c49fd87621d46a98307/gix-config/src/file/includes/mod.rs).

Published-crate source audit (2026-10-15): inspected the downloaded `gix 0.89.0`
crate source in addition to its release notes. Its public remote API exposes
fetch under explicit blocking/async network features; merge exposes tree and
commit merge primitives; and linked-worktree add/remove are behind
`worktree-mutation`. `fetch_then_checkout` is present in the clone workflow, but
it constructs a clone checkout continuation and does not implement pull's
current-branch reconciliation, index/worktree safety, merge/rebase state, or
conflict continuation. The public checkout module supplies checkout options,
not porcelain branch-switch orchestration. No push workflow was found in this
crate audit. These findings refine the earlier broad phrase “fetch + update”:
fetch and low-level merge/worktree ingredients exist, while Talos-equivalent
Pull and branch-switch workflows remain absent. The release crate's network and
worktree mutation features also expand dependency effects and are not enabled by
this audit. Talos remains on locked 0.87.1; 0.89.0 requires Rust 1.88 and has
the recorded breaking `gix::Result` migration. No upgrade or host-Git removal
date follows from API presence alone.

Audit sources: `cargo info gix@0.89.0` (published version/MSRV/features);
downloaded `gix-0.89.0/src/remote/fetch.rs`, `src/repository/merge.rs`,
`src/repository/checkout.rs`, `src/clone/fetch/mod.rs`, `src/worktree/add.rs`,
and `src/worktree/remove.rs`; current `Cargo.lock` resolves 0.87.1.

2026-10-14 registry recheck after Checkout changes: crates.io sparse index still
reports latest non-yanked stable `gix 0.89.0` (Rust 1.88); Talos remains locked to
0.87.1. No newer published version was found. This registry check establishes
release availability only, not new checkout/push parity or a Git-removal date.
Source: https://index.crates.io/3/g/gix; local evidence
`/private/tmp/i300-gix-index-current.jsonl`.

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

Tracking recheck (2026-10-15): successful live sparse-index response still
reports non-yanked gix 0.89.0, Rust 1.88 minimum; Talos remains locked to 0.87.1.
The upstream development `crate-status.md`, retrieved through GitHub API, still
marks branch checkout/switch orchestration, merge/rebase workflows, push,
self-contained file/SSH clone/fetch, hooks and signed commits incomplete.
Low-level checkout and fetch being checked does not establish full command
parity. No new release or capability changes the operation replacement gates
below, and no supported full Git-removal date is available. The crates.io HTTP
API returned 403 again; the version claim uses the successful sparse index.
Sources: [registry](https://index.crates.io/3/g/gix),
[upstream status](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md).
No dependency upgrade or background monitoring service was introduced.

Tracking recheck (2026-10-21): the live [sparse index](https://index.crates.io/3/g/gix)
still reports non-yanked gix 0.89.0 / MSRV 1.88; Cargo.lock remains 0.87.1.
The [0.89.0 release](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0)
is dated 2026-10-08 and adds linked-worktree creation/removal, already tracked
below; no newer top-level gix release was found in the registry. The immutable
[upstream status at 59ba38b8](https://github.com/GitoxideLabs/gitoxide/blob/59ba38b89a4dcaa829f21db437306facbddd7ae7/crate-status.md)
still leaves push protocol, rebase orchestration, Git-compatible hooks,
switch/restore orchestration and persistent merge continuation/abort unchecked.
It is development-tree evidence, not a released parity test. No new evidence
supports removing host Git or assigning a removal date. Keep the operation-level
replacement gates below; no upgrade or background monitor was introduced.
Evidence retained in /private/tmp/i300-gix-index-20261021.jsonl,
/private/tmp/i300-gix-status-20261021.md and
/private/tmp/i300-gix-release-089-20261021.json. The crates.io HTTP API again
returned 403; the version claim uses the successful sparse-index response.

Tracking recheck (2026-10-14): the live crates.io sparse index still reports
non-yanked stable gix 0.89.0 (MSRV 1.88), versus Cargo.lock 0.87.1. The crates.io
HTTP API returned 403; version evidence comes from the successful sparse-index
response, not that API. Current upstream README and crate-status still list
push, rebase and checkout/switch orchestration as incomplete, including fully
self-contained file/SSH clone/fetch. These development-tree observations do
not establish release parity. No newly available replacement or removal date
is established; no dependency versions or background monitor were changed.
Sources: [registry](https://index.crates.io/3/g/gix),
[status](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md).

Tracking recheck (2026-10-13, maintainer request): the sparse registry index
and [gix 0.89.0 release](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.89.0)
report non-yanked gix 0.89.0 with Rust 1.88 minimum; Talos still locks 0.87.1.
0.89.0 adds linked-worktree add/checkout/remove with safety checks, while 0.88.0
introduced configuration-path discovery and return values for branch deletion
([release notes](https://github.com/GitoxideLabs/gitoxide/releases/tag/gix-v0.88.0)).
The new linked-worktree workflow could support a future dedicated tool, but
cannot replace Talos's current `git_checkout` branch-switch contract. The
0.89.0 public `gix::Result` migration is breaking for downstream APIs, so a
Talos dependency upgrade needs its own compile and parity review.
The upstream [development status](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md)
was fetched again on this date: fetch, low-level commit/ref/index writes and
checkout primitives are checked, while push, checkout/switch orchestration,
client hooks, signed commits and persistent merge continuation/abort remain
unchecked. The release notes establish the new linked-worktree capability;
the development survey describes broader outstanding workflows, not released
0.89.0 compatibility. This confirms the blockers below.
There is no defensible Git-removal date. Track release changes against these
operation gates and migrate individual operations only after parity and
permission tests; no automatic background monitor or dependency upgrade was
created by this recheck.

Tracking update (2026-10-12, I300 follow-up):

- Evidence: [crates.io sparse index](https://index.crates.io/3/g/gix), current
  Talos manifest/lockfile, and upstream
  [crate-status at cd317d380ea10385aad4d87fbf995f0bdd70375c](https://github.com/GitoxideLabs/gitoxide/blob/cd317d380ea10385aad4d87fbf995f0bdd70375c/crate-status.md).
  The status is a development-tree survey, not a release-specific compatibility
  test. Component release tags must not be mistaken for top-level gix releases.
- Confirmed upstream inventory: low-level checkout, index/ref/object writing,
  commit creation, fetch and merge primitives exist. Push protocol/workflow,
  checkout/switch orchestration, hook execution, and persistent merge
  continuation/abort workflows remain unchecked. A `push` module or a merge API
  does not establish equivalent porcelain behavior.
- Decision: keep native local read tools; retain the structured host Git bridge
  for add/commit (configuration, filters, hooks and signing semantics), checkout
  (dirty files, branch/ref/index/worktree coordination), push (transport and
  remote updates), and pull (fetch plus merge/rebase/worktree state). I300 push
  transport admission remains unfinished; this survey does not claim delivery.
- Dependency surface: retain `basic`, `status`, `revision`, `blob-diff`, `index`,
  `sha1`, with defaults disabled. Fetch/network and `worktree-mutation` would
  require explicit feature/dependency review. Native library APIs alone do not
  prove subprocess freedom: credentials, filters, hooks and SSH may execute
  helpers, and upstream still tracks self-contained file/SSH fetch separately.
- Replacement gates: test each replacement against Git for configuration and
  attributes, dirty/conflicted worktrees, linked worktrees/submodules, hooks and
  helper effects, credential/URL routing, denied/replayed approval, cancellation
  and unavailable-host behavior. Preserve read/write/execute/network permission
  effects even when the backend becomes Rust-native.
- No supported removal date is known. Reassess on every relevant gix release
  and Git-tool change using the standing triggers above; push and branch-switch
  orchestration are the primary blockers, followed by behavior and authorization
  parity. Partial migrations can precede full removal. Merely upgrading to
  0.89.0 does not remove the host Git dependency.
- Follow-up owner remains GIT-001; no new iteration activation, completion claim,
  dependency upgrade or background monitoring service is implied by this update.
- Validation: project governance and collaboration-claim validators passed with
  zero warnings; `git diff --check` passed. Existing candidate tests passed:
  `talos-tools --features git --lib` 177/177 and MCP `git_prepared` 3/3, both
  with `--locked`. These tests validate the current 0.87.1 candidate, not 0.89.0.

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

Tracking update (2026-07-06, I101 closeout):

| Item | Observation | Decision |
|---|---|---|
| Current Talos dependency | `Cargo.lock` still resolves `gix 0.85.0`; `crates/talos-tools` still requests `gix = "0.85"` with `default-features = false` and features `basic`, `status`, `revision`, `blob-diff`, `index`, and `sha1`. | No dependency update in I101. Keep the I094 feature posture. |
| Feature surface | `cargo tree --invert gix@0.85.0 -e features` shows the same local read/index/diff/status/revision surface. No network or worktree-mutation feature was enabled by this phase. | Do not expand features just to reduce host prompts. Feature expansion still requires a scoped Git-tool iteration. |
| Runtime host Git leak | Source scan found no current `std::process::Command::new("git")` or `git status --porcelain` use in `crates/talos-cli/src/governance.rs`; the identity prompt still says to prefer built-in Git tools for read-only inspection. | Keep the runtime leak fix closed. Any future direct host-`git` CLI/runtime call outside `talos-tools/src/git.rs` is a regression. |
| Push/pull/checkout/add/commit | No new evidence in this pass that `gix 0.85.0` supplies Talos-ready porcelain push, pull, branch switch, or behavior-equivalent staging/commit flows under the current feature set. | Retain structured host-`git` fallbacks for write/publication workflows. Replacement remains future GIT-001 work with unavailable-host and permission regressions. |
| REL-002 dependency | Publication workflows remain dependent on explicit host fallback behavior. | REL-002 cannot treat Git publication as fully self-contained until push/pull/checkout fallback decisions are replaced or qualified with stronger evidence. |

I101 tracking evidence:

- `cargo tree --invert gix@0.85.0 -e features`: passed.
- Source scan for `Command::new("git")`, `git status --porcelain`, prompt `bash git`, and `git_status`
  confirmed the current runtime leak fix remains in place.

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
- [x] No permission default changes are introduced.

2026-07-05 revalidation evidence:

- `cargo test -p talos-cli governance` passed: 13 tests.
- `cargo test -p talos-tools git` passed: 4 tests, including
  `host_git_unavailable_returns_actionable_error`.
- `cargo test -p talos-agent prompt::tests -- git_status` passed: 33 prompt tests, including
  `test_identity_prompt_prefers_builtin_git_tools_over_bash_git`.
- Source scan found no `std::process::Command::new("git")` or `git status --porcelain` call in
  `crates/talos-cli/src/governance.rs`.
- Source scan confirmed the identity prompt says to use built-in Git tools for read-only
  inspection instead of `bash git ...`.
- No permission policy or default permission table was changed in this revalidation.

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
