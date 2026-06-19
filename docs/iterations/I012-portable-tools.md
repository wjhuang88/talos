# I012: Portable Tools

**User can**: Use Talos on minimal or locked-down machines with fewer assumptions about
host POSIX utilities, while still exposing native tools through the same plugin/MCP/RPC
surfaces as external tools.

## Status: SUPERSEDED — split into I016/I017; later delivery mapped 2026-06-19

This iteration captures the requirement to reduce external environment dependency by
shipping a small Rust-native POSIX-style tool subset and agent-oriented workspace search
tools. It is linked to tool pluginization: built-in tool subsets should be packaged like
native tool packs so future plugin-provided tool packs can use the same registration path.

## Decision Gate

ADR-010 records the search/Git dependency boundary: do not introduce `git2`/libgit2 for
the first I012 search/Git slices; implement search Rust-natively and target `gix` for
self-contained read-only Git tools. Host `git` may be used only as a structured fallback or
temporary bridge. Create an additional ADR before implementation if this iteration changes any
public or long-lived boundary: `ToolPack`, `ToolProvenance`, `AgentTool`, tool listing schemas,
config toggles, MCP/RPC exposure, persistent search indexes, watcher behavior, or native
Git/search dependencies.

## Selected Stories

- [ ] #I012-S1: Built-in POSIX basic tools subset
- [ ] #I012-S2: Embeddable tool pack interface
- [ ] #I012-S3: Built-in workspace search tools
- [ ] #I012-S4: Structured Git tools and dependency spike

## Scope

- Implement a conservative set of POSIX/coreutils-like tools as structured `AgentTool`
  implementations.
- Initial read-only tools: `pwd`, `ls`, `cat`, `head`, `tail`, `wc`, `grep`.
- Initial write-capable tools: `mkdir`, `cp`, `mv`, `rm`.
- Initial search tools: `find_files`, `grep`.
- Initial Git tools: read-only `git_status`, `git_diff`, `git_log`, `git_show`; primary target is
  `gix` when the operation is covered by the validated API surface. Write-capable Git operations
  require separate activation after permission behavior is reviewed.
- Register the set as a native tool pack that can later share the same path as
  plugin-provided local tools.
- Preserve all existing permission and sandbox boundaries.
- Use `fff` as a reference for search architecture and ranking ideas, not as a direct dependency
  in the first slice.

## Non-Goals

- No general shell parser.
- No pipelines, redirects, glob expansion, or environment-variable expansion.
- No replacement for the existing `bash` tool.
- No arbitrary C/C++ bindings, Python FFI, Node.js runtime, or dynamic language runtime.
- No expansion beyond the initial tool subset without backlog change control.
- No `git2`/libgit2 dependency in the first search/Git slices.
- No persistent search database, filesystem watcher, frecency store, or bigram content index in
  the first search slice.

## Acceptance Criteria

- [ ] ADR recorded before code if public API, provenance, config, or listing schema changes.
- [ ] Native POSIX tools are available without relying on host `ls`, `cat`, `grep`, etc.
- [ ] `find_files` and `grep` work without relying on host `find`, `grep`, `rg`, or shell features.
- [ ] Search tools enforce workspace-root boundaries, ignore rules, binary/large-file skipping, and
      output budgets.
- [ ] Initial Git tools use a self-contained `gix` provider where viable; any host-`git` fallback
      uses no shell interpolation and has clear read/write permission classification.
- [ ] Every write-capable native tool is permission-gated by the existing pipeline.
- [ ] Read-only tools are marked read-only and can run concurrently with other read-only
      tools.
- [ ] Unsupported options fail with clear errors instead of falling back to host commands.
- [ ] Tool listing and provenance distinguish native built-in tools, native tool-pack
      tools, and MCP-remote tools.
- [ ] POSIX tool pack can be enabled by default and disabled by config.
- [ ] `cargo test --workspace` exits 0.

## Design Records

- ADR-010: [Git and Search Tool Dependency Boundary](../decisions/010-git-search-tool-dependency-boundary.md)
- Proposal: [Built-in Workspace Search Tools](../proposals/builtin-workspace-search-tools.md)

## Verification Notes

Append command outputs and test evidence here during execution. This iteration should
not move to Review until the tools are exercised on a deliberately minimal `PATH` to
prove the host utility dependency has actually been reduced.

### 2026-06-05: Git Dependency Spike

Completed the I012-S4 dependency Spike before implementation.

Decision:

- Use `gix` as the preferred target for the initial read-only Git tools.
- Keep structured host-`git` only as a compatibility fallback or temporary bridge.
- Do not add `git2`/libgit2 in the first I012 search/Git slices.

Evidence:

- `cargo info gix` -> `gix 0.84.0`, license `MIT OR Apache-2.0`, Rust version `1.85`.
- `gix 0.84.0` feature inspection shows local-read capabilities can be selected with
  `default-features = false` and `status`, `revision`, `sha1`.
- Temporary Spike crate `/private/tmp/talos-gix-spike` compiled and ran against this repo:
  `head_name=main`, `head_id=2339b35901d71e424644e716a74e2e466b596a8f`,
  `is_dirty=true`.
- `git --version` -> `git version 2.54.0`
- `git status --porcelain=v1` produced a parseable porcelain status list.
- `git diff --name-status` produced bounded path/status output.
- `git log -3 --format=%H%x09%an%x09%ad%x09%s --date=iso-strict` produced structured log rows.
- `git show --stat --oneline --no-renames HEAD` produced bounded commit/stat output.

Residual work:

- Map `git_status`, `git_diff`, `git_log`, `git_show`, and `git_branch_list` onto `gix` APIs where
  practical; record any host-`git` fallback as an explicit temporary bridge.
- Add tests for missing Git, non-repository workspace, unsupported flags, output truncation, and
  no-shell invocation.
- Run the eventual I012 tool verification on a deliberately minimal `PATH`; Git-dependent tests
  should pass through the `gix` path or assert clear fallback/unavailable-tool behavior.

## 2026-06-19 Supersession Record

I012 was a published umbrella plan. It was deliberately split before activation:

- file/search/tool-pack scope moved to I016;
- structured Git scope moved to I017;
- actual native tool delivery later landed through I025/TOOL-003;
- actual read/write Git delivery later landed through I026/GIT-001.

The umbrella itself must not be activated. Unfinished persistent-index/tool-pack portability scope
remains under TOOL-001; advanced/fallback Git scope remains under GIT-001.

Disposition: Superseded, not Complete. The original baseline remains visible above.
