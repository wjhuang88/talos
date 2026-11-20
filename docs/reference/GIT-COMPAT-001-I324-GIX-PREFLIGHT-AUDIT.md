# GIT-COMPAT-001 / I324 — gix-native preflight T0 audit (2026-11-12)

> **Evidence status: T0 source inventory; NOT implementation, security acceptance or Git 2.36 conformance.**
> Baseline: `main@067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` (governance #920 merged, claim effective).
> Owner [GIT-COMPAT-001](../backlog/active/GIT-COMPAT-001-gix-preflight-host-git-compatibility.md) · [Issue #919](https://github.com/wjhuang88/talos/issues/919) · [I324](../iterations/I324-git-236-preflight-compatibility.md).
> Binding ADR: [089](../decisions/089-structured-git-effect-authorization.md); [change proposal](../proposals/adr-089-gix-native-preflight-amendment.md) remains **Proposed / not accepted**.

## Current architecture scope — maintainer correction 2026-11-19

The 2026-11-12 **all preflight must be gix / zero Git CLI** target is **SUPERSEDED** by [Issue #919](https://github.com/wjhuang88/talos/issues/919). Existing, supported, ADR-089-approved Git CLI preflight remains legal. Replace only **verified unsupported older-Git queries**, initially `git var GIT_ATTR_SYSTEM/GLOBAL`. The CLI inventory and pure-gix release-call-path findings below are historical source evidence, **not** a requirement to replace every command constructor. Continue each approved CLI command with its original permission, selected executable, bounded time/output and stale-plan binding; new substitute code must not perform unapproved I/O or spawn hidden, unapproved children.

A gix/Talos substitution may only use approved frozen invocation data and exact admitted byte/metadata snapshots; it must not infer Read/Execute authority from a Rust API. If a substitute cannot prove the bound writer's relevant attribute defaults, refuse **that specific operation**; this does not globally require an external trust-profile for unaffected approved CLI. No blanket `GIT_*` relaxation, hardcoded Scoop/Windows install root, or guessed Git-version dispatch. Revised [ADR-089 PR #931](https://github.com/wjhuang88/talos/pull/931) is Proposed / NOT ACCEPTED.

## Historical full-gix source inventory (2026-11-12 to 2026-11-18)

The following rows describe what the former full-replacement work **would have needed**, not today's scope. For `git var GIT_CONFIG_SYSTEM/GLOBAL` the five config failures warrant direct classification rather than an assumed incompatibility. Retain other CLI commands unless exact old-Git test evidence supports replacing them.

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

For a bounded, review-only installation-profile proposal, see [selected-system-Git trust-profile contract](GIT-COMPAT-001-I324-SELECTED-GIT-TRUST-PROFILE.md). It is not an accepted source of compiled-default authority.

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

**Specific gix API exclusion, independently source-inspect before enabling:** `gix::config::File::resolve_includes` is NOT a safe shortcut even when the initial `File` was built from admitted bytes: its documented behavior follows `include`/`includeIf` and loads referenced files, and the implementation path may check and open them itself. Upstream also documents include ordering deviations; no implicit included-file reads or reordered source evidence are authorized. Keep `from_bytes_no_includes` for byte snapshots and Talos-owned `target_config::Sources` for explicit admission, ordered expansion and stale-source revalidation. The pinned dependency source and feature graph remain a separate independent-audit gate: [gix config File API](https://docs.rs/gix/0.90.0/gix/config/struct.File.html#method.resolve_includes).

### T1a isolated PATH sentinel (candidate evidence only)

The candidate test file gix_preflight_candidate_tests.rs now contains a Unix-only parent test that launches **one isolated Rust test-harness child**, with a deliberately failing/logging fake Git executable as that child's only PATH entry. The child uses byte-fed gix config parsing, gix branch-name validation, pure lexical include naming and in-memory gix-attributes parse/search. Any Git resolved through that PATH is recorded, and the parent fails. This design does not mutate the parent test process PATH, so concurrently executing native Git differential fixtures remain unaffected. A separate **test-only positive control** first launches the fake `git` via the same PATH, asserts its deliberate exit code `97` and recorded invocation, then removes the control log *before* the candidate child begins; the final empty-log assertion is therefore armed, not vacuously passing because the fake executable could not run.

This is **not** a full preflight no-process acceptance test. It does not instrument the complete prepared Add/Commit/Checkout/Push stages, intercept an absolute Git binary path, prove no ambient file reads or prove Windows behavior. The ignored inner test must run only through its parent; the single parent result is the evidence unit. Until the pending WIP is merged into the review candidate and exact-head CI succeeds, this test is a hypothesis/test plan rather than a passing verification.

## T0c: independently bounded upstream path and include review (2026-11-16)

**Source precision:** This source inspection uses GitoxideLabs/gitoxide at commit ab7c5dc0596943b42fe9294813a8cdc0383c403a, NOT the exact crates.io source pinned by this repository. Cargo.lock pins gix 0.90.0, gix-path 0.14.1, gix-config 0.62.2 and gix-attributes 0.37.1. These observations justify excluding high-level shortcuts and further auditing; they are NOT a proven call-graph equivalence or complete pinned-dependency audit.

| Upstream source and lines | Concrete mechanism and I324 exclusion |
|---|---|
| [gix-path Git env:20-35](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-path/src/env/git/mod.rs#L20-L35) | GIT_CONFIG_PATHS LazyLock caches a process-global installation/system path selection; fallback may query the Git executable found in PATH |
| [gix-path Git env:30-72](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-path/src/env/git/mod.rs#L30-L72) | core_dir_from_executable launches git --exec-path; config_paths_from_executable_at creates a Git Command and calls output(), so it is not pure preflight |
| [gix-path Windows:42-50, 246-270](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-path/src/env/windows/mod.rs#L42-L50) | Windows alternative executable and PATH/canonical path lookup cache LazyLock and inspect file/symlink metadata |
| [gix-path Windows:320-333](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-path/src/env/windows/mod.rs#L320-L333) | Windows MSYS path conversion can execute cygpath.exe as a child |
| [gix-path Windows:413-425](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-path/src/env/windows/mod.rs#L413-L425) | Git installation selection can infer EXEPATH/PATH or install-root layout; this does not establish the chosen writer's compiled defaults |
| [gix-config includes:116-137](https://github.com/GitoxideLabs/gitoxide/blob/ab7c5dc0596943b42fe9294813a8cdc0383c403a/gix-config/src/file/includes/mod.rs#L116-L137) | resolve_includes_recursive checks config_path.is_file and calls std::fs::File::open on an include path; an observer callback does not itself authorize that I/O |

**Conservative selected API policy:** continue to refuse generic gix config_path, implicit include resolution, ambient repository-open initialization, installation path/cached-global helper calls and MSYS conversion in production preflight. Restrict candidate tests to supplied-buffer parsing, lexical source naming and in-memory attribute/ref evaluation. A positive-control PATH sentinel is necessary but insufficient for absolute subprocesses, registry reads, external path discovery or complete operation-level zero-child proof.

**Open review:** independently acquire and audit actual pinned crate sources matching Cargo.lock checksums, their feature-selected call paths, and the authority to bind compiled defaults to the exact final selected Git writer. Absence of independent provenance stays UnsupportedInstallationProfile. ADR-089 amendment, native Git 2.36 acceptance, and full no-child authorization remain blocked.

## T0d: published gix-config 0.62.2 version-specific call-path evidence

**Source status:** [docs.rs published source for gix-config 0.62.2](https://docs.rs/gix-config/0.62.2/src/gix_config/file/includes/mod.rs.html) is the same crate name/version as Cargo.lock. This is **version-matched published source evidence for one file**, not an independent digest/checksum match to the registry archive, feature-selected complete crate audit or third-party security approval.

| Published source lines | Concrete behavior | I324 constraint |
|---|---|---|
| [14-39](https://docs.rs/gix-config/0.62.2/src/gix_config/file/includes/mod.rs.html#14-39) | File::resolve_includes recursively loads files and documents an include ordering deviation from native Git plus a hasconfig:remote.*.url caveat | Cannot assert native Git value-order equivalence by using this high-level include resolver |
| [99-120](https://docs.rs/gix-config/0.62.2/src/gix_config/file/includes/mod.rs.html#99-120) | The include handler calls config_path.is_file() and std::fs::File::open() on derived path names | Lexical naming is not Read/metadata admission; this API performs unadmitted I/O unless bounded separately |
| [159-211](https://docs.rs/gix-config/0.62.2/src/gix_config/file/includes/mod.rs.html#159-211) | includeIf checks gitdir/gitdir-i/onbranch and restricted hasconfig patterns | Only frozen repository/env conditions may select new candidate paths; never auto-read paths from a condition |
| [234-309](https://docs.rs/gix-config/0.62.2/src/gix_config/file/includes/mod.rs.html#234-309) | If initial gitdir glob fails, the matcher calls gix_path::realpath() and checks the resolved path | Even conditional evaluation may invoke filesystem path resolution before including a file; that metadata traversal must be separately admitted |

**Candidate API boundary:** keep only the supplied-buffer gix config parse (from_bytes_no_includes), Talos-owned target_config::parse_records and declared_paths, and source snapshots obtained from exact independently authorized Path Reads. Neither a gix include callback nor a successfully resolved include path is equivalent to permission. Revalidate missing/source identity and preserve actual Git ordering.

**Still open:** gix-path 0.14.1, gix 0.90.0, and gix-attributes 0.37.1 + transitive selected feature paths, package checksum provenance, platform-level process/ambient-I-O tracing, selected-system-Git compiled-default attestation and native Git 2.36.1. T0c upstream gix-path source is not automatically proof about locked version 0.14.1. No production authority grant.

## T0e: Gitoxide version-matched release-source callgraph

A stronger provenance step than latest-upstream T0c: identified the upstream **2026-10-22 release commit** [dd39fb66e3b1551c9fe849043e39b357a9799ca1](https://github.com/GitoxideLabs/gitoxide/commit/dd39fb66e3b1551c9fe849043e39b357a9799ca1), whose commit title publishes gix 0.90.0, gix-path 0.14.1 and gix-attributes 0.37.1. At that exact commit, the respective Cargo.toml package versions match all three Cargo.lock versions. Separately found the **2026-10-24 release commit** [584a602385dd3ad4dacfdd9fd77a13c193a87dfe](https://github.com/GitoxideLabs/gitoxide/commit/584a602385dd3ad4dacfdd9fd77a13c193a87dfe) publishing gix-config 0.62.2, matching Cargo.lock and the version-matched docs.rs include evidence in T0d.

**Release source is not registry-source attestation.** Repository commit provenance and declared package version cannot replace verifying the actual crates.io tarball bytes against the Cargo.lock checksums. The statements below are version-matched *release snapshot* source evidence, not full signed supply-chain provenance or accepted authority.

| Release snapshot evidence | Observed call path | Strict consequence |
|---|---|---|
| [gix-path 0.14.1 env/git:108-124](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-path/src/env/git/mod.rs#L108-L124) | Global installation/system ConfigPaths initialized via LazyLock. On Windows EXEPATH can shortcut to a conventional prefix; otherwise it calls config_paths_from_executable | Can neither bind to a per-invocation selected Git writer nor trust EXEPATH as compiled-default attestation |
| [gix-path 0.14.1 env/git:135-177](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-path/src/env/git/mod.rs#L135-L177) | It first tries the Git program name, then Windows alternative locations (including is_file metadata probe); it launches Git config commands via Command::new and output(), including fallback for older Git | Violates both zero-child discovery and exact Read/metadata if invoked during Talos preflight; returned paths do not attest the writer selected by this operation |
| [gix-path 0.14.1 env:119-145](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-path/src/env/mod.rs#L119-L145) | Windows PATH and fallback executable search is a cached LazyLock whose candidate paths are inspected with is_file | Do not use ambient executable lookup to justify any extra Read/Execute authority |
| [gix-path 0.14.1 env:172-200](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-path/src/env/mod.rs#L172-L200) | core_dir() uses a global LazyLock which actually launches the chosen-by-gix executable with --exec-path | No use of this API in zero-preflight-child planning |
| [gix-path 0.14.1 env:59-110, 215-239](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-path/src/env/mod.rs#L59-L110) | Shell and helper paths involve cached Git-associated executable discovery and is_file checks, and construct an executable command | Selected-system-Git transport/shell/helper semantics remain explicit admission and trust-profile problems, not a safe gix shortcut |
| [gix 0.90.0 config/cache/init:213-245, 279-331](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix/src/config/cache/init.rs#L279-L331) | High-level config load selects global/git-install/system sources, calls gix_config::File::from_paths_metadata_buf and then globals.resolve_includes, conditional on permissions | Generic gix open/config load is not pure byte-fed source evaluation; even with restrictive options the authority/effect semantics need separate audit |
| [gix-config 0.62.2 source at release 584a6023:99-112](https://github.com/GitoxideLabs/gitoxide/blob/584a602385dd3ad4dacfdd9fd77a13c193a87dfe/gix-config/src/file/includes/mod.rs#L99-L113) | Included path existence and actual File::open occur in the version-matched release source, confirming T0d | Talos must own naming, staged exact Path Read, capture and revalidation of every include before any gix-backed interpretation |

**Version difference to T0c:** The October 22 gix-path 0.14.1 release source has gix-path/src/env/git/mod.rs, env/mod.rs and env/auxiliary.rs; the later upstream snapshot reorganized paths into env/windows/mod.rs and contains a cygpath.exe execution route. **Do not attribute that later cygpath call to the released 0.14.1 source** based on T0c: it was *not established* in the release-source files inspected here. The release-source direct Git subprocess, path is_file and process-wide cache observations **are** independently grounded at the matching version tag/commit.

**Required next evidence:** actual packaged crates.io source/archive hash equivalence to Cargo.lock, feature selection for the exact Talos build and full transitively invoked filesystem/process effects, end-to-end prepared preflight zero-child test after ADR approval, selected writer compiled-default attestor, independent security/API review and native Git 2.36.1 Checkout/Commit. Remain fail closed when no trusted installation profile exists; do not merge or mark #919 Complete.

## T0f: release-matched gix-attributes 0.37.1 and gix-glob 0.29.1 file-vs-byte review

**Version-matched upstream release source, not registry archive SHA256 attestation:** [GitoxideLabs commit dd39fb66](https://github.com/GitoxideLabs/gitoxide/commit/dd39fb66e3b1551c9fe849043e39b357a9799ca1) declares gix-attributes 0.37.1 and gix-glob 0.29.1, matching the two Cargo.lock packages. The following review covers specific API calls used in the I324 *test-only* candidate and direct transitive dispatch. It is not an audit of every feature-selected gix or Git worktree path.

| Release source | Direct behavior | I324 consequence |
|---|---|---|
| [gix-attributes Search::new_globals lines 15-39](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-attributes/src/search/attributes.rs#L15-L39) | Takes attribute file paths and calls add_patterns_file after built-ins; docs explicitly describe lenient parsing and file loading | **Disallowed** as preflight discovery: source names are not permission to open |
| [gix-attributes add_patterns_file lines 44-72](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-attributes/src/search/attributes.rs#L44-L72) | Calls gix_glob::search::add_patterns_file and List::from_file | Do not substitute it for exact Path Read admission or missing-source identity |
| [gix-glob List::from_file and read_in_full_ignore_missing lines 43-98, 142-158](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-glob/src/search/pattern.rs#L43-L98) | Opens files via std::fs::File::open or gix_fs::open_read_only_no_follow; can interpret missing paths, directories, symlinks and **Windows PermissionDenied** as no loaded file | Hidden filesystem reads, denial/absence conflation and omitted witnesses violate Talos authority and stale-evidence gates |
| [gix-attributes add_patterns_buffer lines 74-93](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-attributes/src/search/attributes.rs#L74-L93) | Passes caller-provided bytes and path metadata to List::from_bytes; no file open in direct function | Suitable **candidate-only** in-memory primitive after caller-owned exact admitted byte capture |
| [gix-glob List::from_bytes lines 100-139](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-glob/src/search/pattern.rs#L100-L139) | Converts supplied bytes to patterns with lexical parent/root handling; does not open a file on this direct path | Source path label alone conveys no file authority or identity |
| [gix-glob Pattern trait lines 10-18](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-glob/src/search/mod.rs#L10-L18) | Pattern::bytes_to_patterns **ignores invalid lines** | A strict gix_attributes::parse pass across every assignment and line must precede Search, refusing invalid/unknown filter syntax before effect prediction |
| [gix-attributes pattern_matching_relative_path lines 103-120, 199-252](https://github.com/GitoxideLabs/gitoxide/blob/dd39fb66e3b1551c9fe849043e39b357a9799ca1/gix-attributes/src/search/attributes.rs#L199-L252) | Traverses already-loaded pattern lists and updates Outcome without process or source read in the direct implementation | Match only frozen supplied-buffer patterns; precedence and macro semantics still need native differential proof |

**T0f decision:** Retain strict parse, add_patterns_buffer, and buffer-backed match only in the experimental pure candidate. Do not use Search::new_globals, add_patterns_file or List::from_file to find sources or decide missing-file outcomes. The released source matches Cargo.lock package versions but not independently verified crates.io archive bytes/checksums. Full feature/callgraph audit, zero-child end-to-end, selected Git compiled-default profile, independent ADR review and genuine Windows Git 2.36 acceptance remain separate gates. No production authorization is granted by this document.

## T0g: command-by-command old-Git compatibility correction (2026-11-19)

The maintainer's **current** requirement is to retain Git CLI preflight queries when supported and replace only individual unsupported old-Git query forms, not all 14 production Git command constructors. The source inventory above is a historical audit, not a full replacement checklist.

**Independent versioned command documentation:**

- [Official `git-var` manual for Git 2.40.0](https://git-scm.com/docs/git-var/2.40.0) lists `GIT_AUTHOR_IDENT`, `GIT_COMMITTER_IDENT`, `GIT_EDITOR`, `GIT_SEQUENCE_EDITOR`, `GIT_PAGER` and `GIT_DEFAULT_BRANCH`, but *not* `GIT_ATTR_SYSTEM`, `GIT_ATTR_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_GLOBAL` or `GIT_SHELL_PATH`.
- [Official `git-var` manual for Git 2.42.0](https://git-scm.com/docs/git-var/2.42.0) adds all **five** names and clarifies that system/global attribute/configuration paths may be reported even when files do not exist. This is a historical API-availability indication; the exact **Git 2.36.1.windows.1** binary remains the acceptance oracle.
- The original [native Git 2.36.1 I324 report](WINDOWS-VALIDATION-2026-11-17-I324.md) already confirms that both **attribute** queries return usage. Five configuration path-discovery failures and receiver shell-selection failures are consistent with the additional three historical missing-variable names but **not yet directly proven** by an exact-command original-environment matrix.

| Native `git var` query | Source classification | Scope rule |
|---|---|---|
| `GIT_ATTR_SYSTEM` and `GIT_ATTR_GLOBAL` | Actual Git 2.36.1 usage failure; absent from 2.40 manual | First two confirmed compatibility replacements |
| `GIT_CONFIG_SYSTEM` and `GIT_CONFIG_GLOBAL` | Absent from 2.40, present by 2.42; aggregated native config failures | Test each exact query before adding to replacement implementation |
| `GIT_SHELL_PATH` | Absent from 2.40, present by 2.42; native receiver shell failures | Test exact query; preserve other supported receiver CLI and command authority |
| Retained `git config --show-origin --null --list --includes`, `git --exec-path`, `git check-attr`, `git rev-parse` | No general incompatibility established by these five `git var` names | Leave existing authorization and CLI path unchanged unless separately proven unsupported |

**New test-only evidence:** `git_var_legacy_path_query_support_is_classified_per_command` executes the five exact `git var` forms against the native selected test Git, classifies success, supported-but-unset and usage/unsupported without logging any path/configuration value. Other failures remain failures, not aliases for unsupported. This is **not** production capability detection, a new preflight probe or an authorized CLI fallback; reproduce it on original Windows Git 2.36.1 and a newer Git to obtain runtime evidence.

The existing `ConfigState::capture_host` already places explicit `core.attributesFile` and frozen XDG/HOME global-attribute defaults into stable auxiliary candidate paths. The new `global_attribute_default_uses_frozen_xdg_and_home_evidence_without_git_var` test covers those roots and content-change invalidation without changing production. It does **not** establish the selected Git's **system** attribute compiled fallback. Missing that relevant selected-writer fact still means explicit Unsupported on the affected replacement; it is **not** a blanket blocker for retained, working, authorized CLI.

This subsection supplies **scoping/test evidence only**, no independent ADR-089 approval or native Git 2.36 Checkout/Commit success. Source values, exact source rights, alternative executable effects, `GIT_INSTALL_ROOT` semantics and genuine native acceptance remain under their existing gates.
