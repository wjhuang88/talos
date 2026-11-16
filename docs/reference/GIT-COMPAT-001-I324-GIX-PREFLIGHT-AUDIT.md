# GIT-COMPAT-001 / I324 — gix-native preflight T0 audit (2026-11-12)

> **Evidence status: T0 source inventory; NOT implementation, security acceptance or Git 2.36 conformance.**
> Baseline: `main@067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` (governance #920 merged, claim effective).
> Owner [GIT-COMPAT-001](../backlog/active/GIT-COMPAT-001-gix-preflight-host-git-compatibility.md) · [Issue #919](https://github.com/wjhuang88/talos/issues/919) · [I324](../iterations/I324-git-236-preflight-compatibility.md).
> Binding ADR: [089](../decisions/089-structured-git-effect-authorization.md); [change proposal](../proposals/adr-089-gix-native-preflight-amendment.md) remains **Proposed / not accepted**.

## Required architecture

Use **gix native interpretation for all preflight** across Git versions; retain the bound system Git **only for approved mutations**. Never dispatch by `git --version` nor add hardcoded Scoop/Windows installation paths, `git var` / `git config` CLI fallback, unbound gix-internal Git process, undocumented ambient `PATH` search or process-global host-state cache.

**A Rust API is not automatically an authorized pure computation.** It must consume frozen invocation data or specifically admitted snapshots, not independently read or execute. No expansion of `git:configuration-chain` into general filesystem read. If selected system Git installation defaults cannot be proven equivalent, this is a **real unsupported profile** requiring fail-closed refusal, not a silently skipped input.

## Code-truth inventory: CLI work still used by preflight

All locations below refer to `crates/talos-tools/src/git_write/prepared_add.rs` on the frozen baseline. This is an **inventory of the existing implementation**, not a list of authorized future subprocesses.

| Existing entry | Locations (approx.) | Preflight role | gix-native/authorization replacement requirement |
|---|---|---|---|
| `inspect_config_paths` | 3469; callers near 1017, 1223, 1372 | `git var GIT_CONFIG_SYSTEM/GLOBAL`, including missing and empty source roots | Select roots from frozen context and proven executable installation profile; retain absence/path identity; exact Read before bytes |
| `inspect_attribute_path_at` | 3515; callers near 1010, 1098, 1357, 1983 | `git var GIT_ATTR_SYSTEM/GLOBAL`, unsupported by Git 2.36 | gix-native selection, verify installed-system equivalence, include absent roots; never guess |
| `inspect_config_at` / `inspect_config` | 3432 / 3285 | `git config --show-origin --null --list --includes` | gix pure config parser over approved snapshots; ordered include/includeIf, hasconfig and negative evidence |
| `inspect_maintenance_program` | 794 | `git --exec-path`, used for maintenance/internal sender executable | Bound typed executable proof or explicit unsupported profile; `git --exec-path` command must not remain in preflight |
| `inspect_git_shell_path` | 2285 | `git var GIT_SHELL_PATH` for local push transport | Typed executable/transport discovery from accepted evidence; shell and children retained as effects |
| `inspect_checkout_start` / `inspect_checkout_branch` | 3343 / 3383 | `git rev-parse` / `git check-ref-format` | gix ref/branch validation and resolution on admitted source/refs |
| `inspect_commit_target` and previous-branch query | 3405 / 1174 | `git rev-parse --verify ...^{commit}` | gix revision/ref/object analysis; missing vs corrupt/ambiguous errors preserved |
| `inspect_receiver_config_at` / `inspect_child_bootstrap_config` | 2742 / 2765 | Git config inspection in receiver and child repositories | gix parser over separately admitted child/receiver roots and snapshots |
| Current/cached/target/child filter inspection | 3734, 3768, 4173–4300 | `git check-attr -z ... filter`, including target tree/cached index | gix attributes/ignore stack using approved source/index/tree evidence; no implicit unapproved file reads or filter executions |
| Push/target/metadata and other host probes | other `tokio::process::Command` uses; e.g. 3280–3420 and operation-specific code | Rev/remote/receiver detection | Enumerate **all** remaining process creation paths before declaring no-CLI preflight; retain hard source/frontier/child identities |

Existing final **mutation** execution is out of scope for the CLI-preflight ban. Calling a system Git executable only after the prepared mutation admission remains mandatory.

## Safe existing primitives that can be reused

- `git_write/target_config.rs`: `parse_records()` uses **gix config parse Events on byte buffers**, bounded with panic guard. `declared_paths()` computes include paths without I/O. `Sources::new/add/advance/ordered_records/revalidate` already implement staged **exact Path Read**, ancestor/symlink identities, cumulative source budgets, absence evidence, ordered include/includeIf and `hasconfig` evaluation. Expand this contract to host configuration; do not invent a second unreviewed include parser.
- `git_write/config_state.rs`: `gix::config::File::from_bytes_no_includes()` is used for admitted child startup configuration; reusing this pure byte-input API is materially different from opening an ambient Repository.
- `git_write/snapshot.rs`, `auxiliary_paths.rs`: retain separately admitted leaf/ancestor identities and revalidation. Do not use `PathIdentity::capture` as a substitute for exact Read or link metadata permission.
- Existing `gix 0.90.0` does expose `Repository::attribute_global_paths()` and returns even missing paths (gix v0.90 release notes; [API docs](https://docs.rs/gix/0.90.0/gix/struct.Repository.html#method.attribute_global_paths)). **Not approved for direct use**: repository opening may read system/global configuration or spawn Git before admission, so prove a safe construction from admitted snapshots or use lower-level pure path evaluation.
- `gix::config_path()` is **not an approved substitute** for `git var`: its API docs warn installation path discovery and Windows system path discovery **may invoke Git**; gix-path has possible process-global lazy installation caching. Any such path resolver must be audited against the lockfile and excluded from runtime preflight if unbound.

## Blocking equivalence/authority questions

1. **Compiled installation defaults.** A system Git binary can be built with configuration and attribute paths not derivable from user HOME/XDG or binary parent directory. A gix guess is not evidence about the selected system Git writer. How will a **verifiable, bounded per-executable installation profile** be obtained without invoking Git from preflight? No such source is currently accepted. Return Unsupported (fail closed) when missing; this prevents falsely declaring native Git 2.36 acceptance.
2. **ADR-089.** Current accepted `git:configuration-chain` authority names **fixed Git CLI discovery commands**; Rust file reads or broad `gix::open()` effects are not thereby authorized. Accept the proposed amendment defining path-only lexical planning + exact Path Read for file bodies **before switching production behavior**.
3. **Early vs later phases.** A scan of all `tokio::process::Command::new` shows Git CLI is used for more than the 2.36 attribute query (refs, filters, receiver and transport child). Do not label the entire prepared preflight gix-native until those calls are eliminated/replaced or explicitly declared outside the approved preflight scope.
4. **Environment.** `validate_environment` currently permits `GIT_PAGER` only; `GIT_INSTALL_ROOT` is not yet demonstrated semantically inert. No blanket `GIT_*` relaxation.
5. **Native verification.** Git 2.36.1.windows.1 baseline had reported 113 Git-enabled tool failures (259 passes), reduced to 80 with controlled environment; URL rewrite/tracking and GIT-SNAPSHOT-001 are separate, not universally fixed by path discovery.

## Red/green sequencing before implementation merge

1. **T0 / design gate:** ratify ADR-089 change (path-only discovery vs exact Read + subprocess ban) and choose verifiable installation-profile input; independent security review must approve actual permission semantics.
2. **T1a:** pure gix config-record/source graph over externally provided, already admitted byte snapshots; unit tests that cannot execute commands or access ambient filesystem.
3. **T1b:** wire staged host root and child/receiver discovery to the existing `Sources` admission machinery. Instrumented fake Git executable records **zero** invocations during all preflight stages. Declared missing/empty roots remain in evidence.
4. **T2:** gix revision/branch, filter/attributes, transport/maintenance subprogram and target planning. Preserve all current fail-closed cases, child identity, source-path revalidation, and operation-level budgets.
5. **T3:** differential native Git 2.36.1 and newer behavior for Checkout/Commit/Push; HOME/XDG, includeIf, Unicode/spaces, multiple installations, >16MiB sources, changed/linked config, URL rewrite/tracking. Disclose unsupported profiles.
6. **T4:** `cargo fmt`, locked tool tests, release preflight, exact-head CI, Windows-native gates, and **independent** authorization/security/API review. Only then consider merge/Complete.

**Current outcome:** This audit intentionally authorizes no new filesystem reads, subprocesses or mutations and makes no claim that the implementation already exists. It records the evidence needed to make the subsequent gix-native replacement safe.

## T0b: direct-process preflight inventory (2026-11-16; #928 snapshot)

**Evidence baseline:** implementation candidate 979c925aa4f17d3bae134a8f2cd09b359d1fc1a1. Enumerated all 46 Rust sources under crates/talos-tools/src/git_write*, including test modules, and inspected the 19 non-test entry/helper modules. This supplements T0's approximate discovery table with the explicit **14 host-Git command constructors** in production prepared_add.rs. This is a source-level inventory, **not** an actual runtime/hidden-subprocess trace or native Git 2.36 compatibility proof.

The shared AddPlan runner at prepared_add.rs:327–414 reserves up to 128 commands and 64 MiB of cumulative output capacity, counting both preflight and final mutation. Removing preflight CLI must preserve the rest of the operation's budgets, explicit facets and review phases; constructors alone do not prove which processes have run.

| Constructor line in prepared_add.rs | Existing command / inspection | Required pure-gix/authorized-data replacement |
|---|---|---|
| 795 | Git --exec-path, inspect_maintenance_program | Trustworthy selected-executable maintenance and internal sender profile; deny missing trusted provenance |
| 1174 | Git rev-parse --symbolic-full-name --verify | Previous branch and @{-n} checkout/reflog semantics from admitted sources, including symbolic and ambiguous state |
| 2286 | Git var GIT_SHELL_PATH | Push receiver shell and revalidation; bound executable plus independently authorized child effects |
| 2743 | Receiver Git config --show-origin --null --list --includes | Receiver-specific approved configuration roots, includes and physical origins |
| 2770 | Child-bootstrap Git config --show-origin --null --list --includes | Separate child Git-dir/worktree configuration frontier and exact source provenance |
| 3350 | Git rev-parse --symbolic-full-name --verify HEAD | Checkout create starting reference and post-review revalidation |
| 3384 | Git check-ref-format --branch | Short-branch validation, including leading-dash rejection and reflog shorthand distinction |
| 3406 | Git rev-parse --verify --quiet target^{commit} | Checkout target and remote-guess candidates, with missing/corrupt/ambiguous distinction |
| 3433 | Host Git config --show-origin --null --list --includes | Fully admitted config/includes/includeIf/hasconfig ordered records |
| 3470 | Git var GIT_CONFIG_SYSTEM/GLOBAL | Frozen selected-writer compiled system/global roots, with absence evidence; never infer from host version |
| 3520 | Git var GIT_ATTR_SYSTEM/GLOBAL | Same provenance for attributes; no Git 2.36 fallback CLI or guessed install prefix |
| 3734 | Git check-attr -z filter on selected current paths | Worktree attributes, macro precedence, configured-filter effect denial, exact source Read |
| 3768 | Git check-attr -z --source=target / --cached filter | Target tree and cached index attributes, fresh admitted object/index provenance |
| 4300 | Child Git check-attr -z filter with explicit --git-dir / --work-tree | Nested child attributes and separately admitted child/receiver identities |

**Callers and revalidation:** inspect_maintenance_program (~818/881/980/1143); Git config roots (~1018/1224/1373); attributes roots (~1012/1100/1359/1985); config inspection (~984/1022/1111/1401/3669/4238); checkout branch/start (~1153–1154 and 3840–3847); Push shell (~1030/1538/2315); receiver/child config (~2642–2645, 1870–1876); child filter (~4242). A replacement limited to initial discovery is insufficient: later read/revalidation stages must also be CLI-free before the one final mutation.

**Final system Git writer is excluded from the ban:** prepared_add/command.rs:164–165 materializes a FrozenCommand, and the operation dispatcher verifies retention and then invokes the approved native Add/Commit/Checkout/Push mutation at prepared_add.rs:4003. Its presence does not mean preflight still uses Git. Git Pull has separate staged execution and has not been characterized by this direct-constructor table; do not infer a complete Pull graph.

**Test oracles are not production preflight:** the Git constructor in git_write.rs near line 109 is behind cfg(test). Native Git calls in checkout_state, target_config, prepared_add/host_path, receiver_state, remote_state, repository_state and related test modules are in test-only fixtures. They may remain as differential oracles, but they do not establish a permitted runtime fallback.

**Open audit gaps:** direct runtime filesystem reads must be mapped independently; gix 0.90.0 dependency/feature paths need an independent source inspection for hidden process launches, ambient roots, process-global cache and path fallback. The 14-row table is not proof of zero child processes in the full product.

### T1a isolated PATH sentinel (candidate evidence only)

The candidate test file gix_preflight_candidate_tests.rs now contains a Unix-only parent test that launches **one isolated Rust test-harness child**, with a deliberately failing/logging fake Git executable as that child's only PATH entry. The child uses byte-fed gix config parsing, gix branch-name validation, pure lexical include naming and in-memory gix-attributes parse/search. Any Git resolved through that PATH is recorded, and the parent fails. This design does not mutate the parent test process PATH, so concurrently executing native Git differential fixtures remain unaffected.

This is **not** a full preflight no-process acceptance test. It does not instrument the complete prepared Add/Commit/Checkout/Push stages, intercept an absolute Git binary path, prove no ambient file reads or prove Windows behavior. The ignored inner test must run only through its parent; the single parent result is the evidence unit. Until the pending WIP is merged into the review candidate and exact-head CI succeeds, this test is a hypothesis/test plan rather than a passing verification.
