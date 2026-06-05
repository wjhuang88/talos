# ADR-010: Git and Search Tool Dependency Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Iteration**: I012-S3 / I012-S4

## Context

Talos needs stronger built-in search and Git-oriented tool capabilities. The
`fff` project is a useful reference for long-running agent search: it combines
workspace indexing, typo-resistant path search, content grep, frecency ranking,
Git-aware annotations, and optional filesystem watching. Its `fff-search` crate
is MIT licensed and exposes a Rust library, but it also brings native and
stateful dependencies such as `git2`/vendored libgit2, LMDB via `heed`,
filesystem watchers, memory mapping, and optional native/SIMD-oriented
optimizations.

Talos has stricter dependency and safety constraints:

- Rust first; no arbitrary C/C++ bindings without an ADR-recorded exception.
- No `unsafe` without ADR.
- All write-capable tools must go through the permission pipeline.
- Search and Git tools must respect the workspace root and must not become a
  side channel around sandbox or permission policy.
- Talos should prefer self-contained capabilities over host environment
  assumptions. Host tools are acceptable only as compatibility fallbacks,
  temporary bridges, or explicitly documented escape hatches.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No arbitrary C/C++ bindings | Hard | AGENTS.md hard constraint #1 | Only by ADR |
| `git2` uses libgit2 bindings and may build bundled C code | Hard | `git2-rs` / `libgit2-sys` documentation | Only by ADR |
| Prefer self-contained runtime capabilities over host utilities | Soft | User/project operating principle; AGENTS.md dependency discipline | Yes, with recorded tradeoff |
| Git operations are useful for agent workflows | Soft | Product direction / I012 tools roadmap | Yes |
| Search should be fast in long-running sessions | Soft | fff reference project and Talos TUI/agent usage | Yes |
| First search implementation can be simpler than fff | Assumption | Current repo size and delivery priority | Validate with benchmarks |
| Persistent search/frecency databases are not required for the first slice | Assumption | Product scope | Validate after R1/R2 usage |

## Reasoning

`git2` is a poor default dependency for the first Git/search slice because it
would add libgit2 bindings to the core tool path. That conflicts with the
current Rust-first constraint unless we accept a new native dependency via ADR.
It also expands the security review surface before Talos has validated which
Git operations are actually needed.

The simplest path satisfying both the hard constraints and the self-contained
operating principle is:

1. Keep first-party search tools Rust-native and permission-aware.
2. Use lightweight pure-Rust crates for walking/ignore/glob/regex only when the
   standard library is insufficient.
3. Target a pure-Rust Git implementation for read-only Git tools where the API
   is proven by Spike, with host `git` reserved as a fallback/bridge rather than
   the default implementation target.
4. Use `gix` / gitoxide as the preferred candidate because it avoids libgit2,
   supports fine-grained feature selection, and does not require a host `git`
   binary for local repository reads.
5. Reject `git2`/vendored libgit2 for I012 unless a later ADR explicitly
   approves the native dependency and its security review.

This preserves portability progress for common file/search actions while not
pretending Talos can implement all of Git cheaply in the first pass.

## Decision

Talos will not introduce `git2`/libgit2 as a dependency for the first built-in
search or Git tool slices.

For I012:

- Workspace search tools will be implemented as Rust-native structured tools.
- Search may borrow algorithms and architecture from `fff`, but not copy code
  or depend on `fff-search` in the first slice.
- Git-aware ranking in search may read bounded Git status information through
  the structured Git provider, but search must work without Git metadata.
- Initial read-only Git tools should use `gix` / gitoxide as the primary
  implementation target when the required operation is covered by the validated
  API surface.
- Host `git` may be implemented only as a compatibility fallback or temporary
  bridge. It must be invoked directly without a shell, with allowlisted operation
  shapes, explicit workspace root, timeout, unavailable-host tests, and clear
  read/write permission classification.
- Any Git operation not covered by the initial `gix` Spike must either receive a
  bounded follow-up Spike or remain behind the host-`git` fallback with a
  documented replacement trigger.

## Spike Result (2026-06-05)

The I012-S4 dependency Spike compared three options:

| Option | Strengths | Risks / Costs | First-slice decision |
| --- | --- | --- | --- |
| `gix` / gitoxide | Pure Rust direction, avoids libgit2, does not require a host `git` binary for local repository reads, supports feature selection (`status`, `revision`, `sha1`) | Large dependency graph; APIs for exact `diff`/`log`/`show` output need implementation-level mapping; compile cost is higher than a process wrapper | **Preferred target for I012 read-only Git tools after bounded API mapping** |
| Structured host-`git` adapter | Small implementation, exact user-expected Git semantics, no new Cargo dependency, easy per-command permission classification | Requires host `git`; violates self-contained-first as a primary design; command output parsing must be bounded and tested | **Fallback/temporary bridge only** |
| `git2` / libgit2 | Mature bindings, broad local Git API, no host `git` binary required | C/native dependency via libgit2; docs.rs notes bundled/static libgit2 may be built when system libgit2 is unavailable or vendored feature is used; conflicts with Rust-first constraint without ADR | **Reject for I012 first slice** |

Local `gix` validation:

- `cargo info gix` reported `gix 0.84.0`, license `MIT OR Apache-2.0`, Rust
  version `1.85`, repository `https://github.com/GitoxideLabs/gitoxide`.
- `gix 0.84.0` feature inspection shows a non-default local-read subset can be
  composed from `status`, `revision`, and `sha1`; network features are separate.
- A temporary Spike crate at `/private/tmp/talos-gix-spike` compiled with
  `gix = { version = "0.84.0", default-features = false, features = ["status",
  "revision", "sha1"] }` and read the Talos repository:
  - `head_name=main`
  - `head_id=2339b35901d71e424644e716a74e2e466b596a8f`
  - `is_dirty=true`

Host command-shape validation remains useful only for fallback behavior. In the
Talos repo, these read-only surfaces can be expressed without shell
interpolation:

- `git status --porcelain=v1`
- `git diff --name-status`
- `git log -3 --format=... --date=iso-strict`
- `git show --stat --oneline --no-renames HEAD`

These map to structured fallback tools (`git_status`, `git_diff`, `git_log`,
`git_show`) with allowlisted flags and bounded output parsing. They do not
override the self-contained-first target.

## Design Guardrails

- Git command execution must not use `sh -c`.
- Git tool arguments must be structured fields, not raw command strings.
- Write-capable Git operations (`checkout`, `switch`, `merge`, `rebase`,
  `commit`, `reset`, `clean`, `apply`, `stash pop`, etc.) must be permission
  gated.
- Read-only Git operations (`status`, `diff`, `log`, `show`, `branch --list`)
  may be marked read-only only when arguments cannot mutate the repository.
- Search indexing must never follow paths outside the workspace root.
- Search must skip binary and oversized files by default.
- Any persistent index, frecency store, or query history is a write behavior and
  requires explicit path, config, and permission/security review.
- Filesystem watchers are deferred. They are easy to get wrong around deletes,
  symlinks, ignored paths, and workspace boundary changes.

## Reversal Trigger

Revisit this decision if one of these becomes true:

- Host `git` availability becomes a release blocker for supported environments.
- Talos needs Git object/database operations that cannot be implemented safely
  through the selected `gix` subset or structured host-`git` fallback.
- Benchmarks show pure Rust search without persistent indexing cannot meet
  target latency on expected repositories.
- A full security review accepts a native Git dependency and records the
  resulting build, licensing, sandbox, and audit implications.

## References

- `fff` repository: <https://github.com/dmtrKovalenko/fff>
- `fff-search` crate manifest: <https://raw.githubusercontent.com/dmtrKovalenko/fff/main/crates/fff-core/Cargo.toml>
- `fff` file picker design: <https://raw.githubusercontent.com/dmtrKovalenko/fff/main/crates/fff-core/src/file_picker.rs>
- `git2-rs` project: <https://github.com/rust-lang/git2-rs>
- `gix` docs: <https://docs.rs/gix>
