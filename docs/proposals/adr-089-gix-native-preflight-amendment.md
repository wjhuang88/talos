# ADR-089 修订提案：使用 gix 原生前置分析替换 Git CLI 发现

> 2026-11-12。**Status: Proposed — NOT Accepted / NOT implementation authority.**
> Issue [#919](https://github.com/wjhuang88/talos/issues/919), I324 / governance PR #920; supersedes the "host probe" option in the initial 2026-11-10 gix proposal.
> Existing [ADR-089](../decisions/089-structured-git-effect-authorization.md) remains Accepted and binding until an explicit reviewed amendment is accepted.

## Why ADR review is necessary

ADR-089's accepted configuration-chain capability is expressly limited to fixed Git CLI inspection, `git var GIT_ATTR_SYSTEM/GLOBAL`, `git var GIT_CONFIG_SYSTEM/GLOBAL`, and commit's `git --exec-path` inspection. I324's maintainer requires gix to do preflight, not a version-specific CLI fallback. Replacing those operations changes the **implementation of discovery authority**, not the already-accepted effects/facets. It must be reviewed as a bounded change; the old fixed-command grant does not automatically become a general filesystem-read or execute permission for a library.

## Proposed amendment (not yet in force)

1. **One approach for all supported Git versions:** gix 0.90.0 (and bounded Talos-owned adapters of its pure primitives) performs repository/configuration/attributes/ignore/path/URL/branch/tracking/effect *preflight analysis* using one frozen invocation context. There is no `git --version` dispatch or OS/package-manager-specific hardcoded installation prefix.
2. **No Git CLI in preflight:** no `git var`, `git config`, `git --exec-path`, or unbound gix-internal Git subprocess during discovery or pre-execution revalidation. Auditing gix's call graph for hidden process execution, process-wide lazy caches, ambient PATH/env, panic and file-system accesses is a prerequisite. Calling `gix::config_path()` merely because it is a Rust API does *not* satisfy this criterion: relevant gix-path installation discovery can launch Git.
3. **Keep staged authorization:** initial metadata-only path discovery uses only the narrowly approved capability and selected bound executable/repository/cwd/environment identity. Config file bytes and discovered include/attributes/hooks/filter files remain within existing authorized configuration-chain scope or receive fresh exact Path Read facets before access; no direct gix file loading that bypasses Talos authority. gix parses captured admitted bytes, does not independently open new files.
4. **Bind the final writer:** the selected system Git executable remains the final write executor, with its identity and all operation effects/auxiliary sources revalidated before mutation. gix's interpretation must be proven equivalent for the relevant supported installation, not presumed. Failure to determine compiled default config/attribute/exec paths, missing trust metadata, or unsupported semantics results in a specific fail-closed diagnostic and separate acceptance residual.
5. **Keep all invariants:** request-specific snapshots, stage limits, budgets, symlink evidence, explicit Deny, hook/filter/maintenance/transport executions, exact command authority, cancellation, one-use proofs and replay rejection must not weaken. No new generic Read/Execute capability, default auto-approval or hidden fallback.
6. **Environment:** do not blanket permit `GIT_*`; test whether `GIT_INSTALL_ROOT` is semantically inert for the selected host binary before considering a narrow exception. Preserve current `GIT_PAGER` behavior and dangerous redirect/config/trace refusal.

## Proof / acceptance before switching implementation

- Map every production Git CLI preflight call site and its current admitted facet/cumulative budget to an equivalent gix-native function/explicit staged evidence or an unresolved fail-closed row.
- Instrument a deliberately hostile/recording Git executable and test that no discovery stage invokes it; the final mutation may invoke exactly the selected admitted binary.
- Fixture matrix: Git 2.36.1.windows.1 (original Scoop environment) and newer Git; multiple Git installs, quoted/non-ASCII paths, HOME/XDG, config includes/includeIf, absent/empty files, system attributes, URL rewriting/tracking, hook/filter/maintenance, source/executable replacement, old-plan invalidation and >16MiB prepared Checkout/Commit.
- If native Git 2.36 cannot be supported without a compiled-default-path oracle, do not add hardcoded platform/version paths or declare compatibility achieved. Record an explicit Unsupported profile and return to the maintainer with a constrained evidence-based design choice.
- Exact-head locked workspace/preflight and native Windows CI, governance validators, independent authorization/security/API review, and merge-time CAS. No implementation branch until governance claim is effective and this semantic amendment accepted.

## Non-goals

No replacement of actual Git write execution; no permission policy expansion; no I300 or GIT-SNAPSHOT-001 reopen; no arbitrary host probing; no stealthy version fallback. This proposal records a technical decision request, **not** acceptance or a claim of implementation/test evidence.
