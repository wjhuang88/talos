# Iteration I324: GIT-COMPAT-001 Targeted Older-Git Query Compatibility

> Document status: Active / Claimed (effective #920 merge `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80`)
> Published plan date: 2026-11-11
> Current objective (maintainer correction 2026-11-19): keep already-authorized CLI preflight where supported and replace only query forms unsupported on older Git (first: `git var GIT_ATTR_SYSTEM/GLOBAL`), preserving prepared-operation authority.
> MVP deliverable: structured Checkout and Commit on native Windows Git 2.36.1 pass the admitted preflight and post-approval validation path; tested with older/newer Git.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | GIT-COMPAT-001 / #919 I324 only: retain supported authorized Git CLI preflight; replace proven Git 2.36-incompatible query forms with bounded gix/Talos-native alternatives, preserving Read/executable/effect/revalidation and final host-Git writer; no snapshot/TUI work |
| Claimed At | 2026-11-11 |
| Source Issue | #919 |
| Governance Claim PR | #920 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of the 2026-11-10 gix design on 2026-11-11 (conversation). Single-maintainer fallback is proposed for documentation-only claim because no separate reviewing maintainer is available; exact-head CI, both governance validators and no-overlap merge-time CAS are mandatory. No source implementation or security acceptance is claimed by this PR. Subsequent implementation requires independent Git authorization/security/API review and native Windows evidence. |
| Implementation PR | Draft #928 (T0 source/API audit only; no production implementation yet) |
| Last Updated | 2026-11-20 |
| Handoff / Release Condition | Exact-head independent Git authorization/security review, both governance validators, native Windows acceptance, no scope overlap; retain Review until actual evidence. |

## Selection inventory and non-overlap (2026-11-11)

- I300 / ADR-089 and I309 gix upgrade: Complete; do not reopen old outcomes or rewrite their authorization contract.
- GIT-SNAPSHOT-001 / TUI-APPROVAL-001: Review / Claimed emergency repairs in #903; own large-file evidence and UI only, not system-Git preflight compatibility.
- I320 ARCH-035 E0/E1 Review and I322 STORE-002 C proposed claim (#911), I319 Search-L PR #907, I318 TUI native repeat Review, I302 Windows handle Review, I277 deferred device Review, I249 Planned, I164 Paused, and proposed I323 Azure are retained as-is; no file, work slice or capability overlap.
- Existing open implementation PRs #917 (Session), #912 (storage), #907/#861 (Search) and #918 (Azure governance) do not claim Git write preflight. Recheck target branch and open PRs before claim merge.
- User expressly requested a separate implementation of the November 10 design. This parallel Git-only iteration must not bypass any unresolved authorization or competing claimant. No production implementation before effective claim.

## Published baseline

**Story:** [GIT-COMPAT-001](../backlog/active/GIT-COMPAT-001-gix-preflight-host-git-compatibility.md) / [Issue #919](https://github.com/wjhuang88/talos/issues/919). Depends on merged I300/I309, ADR-089 and existing snapshot repair in main.

**Hard constraints:** system Git still executes; resource discovery is path-only until exact Read admission; executable/cwd/env and configuration evidence remain bound and revalidated; unsupported behavior fails closed. No extra unsafe, no Cargo.lock/dependency changes, no permission-facet expansion, no network or hook/transport execution without authority.

**Required direction (maintainer correction 2026-11-19; supersedes November 12):** retain fixed Git CLI preflight queries that work on supported old Git, with their existing ADR-089 authorization. Replace only proven unavailable query forms, initially `git var GIT_ATTR_SYSTEM/GLOBAL`, with bounded gix/Talos-native alternatives using separately authorized exact source evidence. Other commands are not in scope unless their failure is independently reproduced. New native helpers must not run *unapproved* child commands or read unadmitted files; continuing already approved CLI is allowed. Preserve selected final system Git, one-use permission proof, source identity, environment and stale-plan revalidation. Installation-default provenance is a blocker only for an affected replacement requiring it; no generic trusted-profile requirement is imposed on unaffected CLI.

**Unverified assumption:** compatibility failures primarily arise from the unsupported `git var` queries; Windows log also contains URL rewrite/tracking mismatches. Distinguish root causes in tests.

### Ordered slices and gates

| Stage | Deliverable | Gate |
|---|---|---|
| T0 | Enumerate all CLI preflight call sites and their permission bounds; audit gix APIs for hidden Git subprocesses, unauthorized file reads and global caches; draft and obtain review of [ADR-089 gix-native amendment](../proposals/adr-089-gix-native-preflight-amendment.md) | Source-backed decision; if authorized command surface changes, amend/accept ADR-089 first |
| T1 | Replace only the verified unsupported Git 2.36 `git var GIT_ATTR_SYSTEM/GLOBAL` attribute-source queries; retain other supported authorized CLI | Red/green exact command differentials, path/absence/env/executable identity and precise Read tests |
| T2 | Characterize `GIT_INSTALL_ROOT` for selected final writer without hardcoded path/version exceptions; preserve dangerous environment refusal | Security differential and original environment native test |
| T3 | Differential test the replaced attributes source path and separate other failures on Git 2.36/newer, including 17 MiB checkout/commit; instrument for unapproved child commands in replacements | Prepared-plan mutation/staleness tests, retained CLI permissions and explicit unresolved matrix |
| T4 | Readme/docs and owner evidence, native Windows acceptance, locked full preflight, independent security/API review, exact-head CI, CAS | No Complete without implementation SHA and all required gates |

### Acceptance

- On the supported native Git 2.36.1 host, structured Checkout create/switch and Commit no longer fail solely due to unsupported attribute path queries or inert Scoop installation env.
- The original system Git effects, binding and all staging/permission/Read frontiers are preserved. No plan can be reused after configuration, executable, path or relevant source changes.
- Include/includeIf, HOME/XDG, alternate installation layouts, missing/empty global paths, Unicode/spaces and both Git 2.36/current Git are covered by source-backed tests.
- Tests prove *no unapproved Git subprocess in the newly substituted query path* (including hidden gix calls); retained ADR-authorized CLI remains legal. Compare replacement outcomes with selected system Git behavior on Git 2.36/newer; unsupported cases remain visibly unsupported.
- All relevant locked Rust + governance gates, exact-head Windows CI and independent security/API review succeed before implementation merge.

### Non-goals

No I300 reenactment, no new Git operations or automatic retry of ambiguous mutations; no full rewrite to gix, no blanket Git env bypass, no unreviewed authority/facet or persistence change; no GIT-SNAPSHOT/TUI repair or unrelated projects.

### Planned validation

```sh
cargo fmt --all -- --check
cargo test --locked -p talos-tools --features git --lib
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
scripts/validate_collaboration_claims.sh .
```

Cross-check with `cargo xwin` per LOCAL-DEV; native Windows tool tests/CI remain mandatory, and cannot be replaced by Linux cross-target compilation. Review focused tests for zero-test feature filtering.

### Docs and rollback

Update README/README.zh-CN Git compatibility language, this iteration and Story, documentation index, Board, and proposal status only after evidence. Revert this bounded compatibility patch to restore known conservative refusals if review identifies an authority regression; do not touch existing user worktrees.

## Native Windows handoff runbook

The reproducible Windows Git 2.36.1 command matrix, selected-Git comparison,
Rust regression commands and redacted evidence handoff are documented in
[the I324 Windows validation SOP](../sop/GIT-COMPAT-001-I324-WINDOWS-VALIDATION.md).
This is **a test procedure, not implementation/security acceptance**.

## Actual execution (active — implementation not yet delivered)

- 2026-11-11: requirement intake Issue #919 and governance-only selection baseline. Proposed claim/activation not effective until PR merges. T0 source investigation is read-only; no production implementation, validation pass or Completion Commit.

## Historical maintainer scope — 2026-11-12 (SUPERSEDED by 2026-11-19)

Historical note, superseded on 2026-11-19: the November 11 baseline mentioned bounded host probes as a possible compatibility bridge. The earlier maintainer clarification had selected **replace Git command-line preflight with gix**; this is no longer the current requirement. This correction is made to unmerged governance PR #920; its earlier exact-head CI no longer applies after these changes. Use gix/Talos-native planning over previously admitted content; keep file-path discovery separate from exact Read authorization and bind current evidence to final system Git execution. gix itself may spawn Git for installation path discovery: audit and exclude such APIs, rather than silently outsourcing the subprocess. If system Git compiled defaults cannot be proven from allowed sources, fail closed, report an explicit unsupported profile and seek a narrow decision. ADR-089's CLI command wording requires acceptance of the [proposed amendment](../proposals/adr-089-gix-native-preflight-amendment.md) before implementation. No product code or passing native tests are claimed.

- 2026-11-12: claim/activation PR #920 merged to `main` as `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` after exact-head CI #4123 / run `43248451893`: both governance validators 0 warnings, remote Issue owner reconciliation and Windows installer fixture passed. The earlier planned claim is now effective; the proposed ADR-089 amendment is **not** accepted by this merge.
- 2026-11-12: T0 [gix/CLI preflight source audit](../reference/GIT-COMPAT-001-I324-GIX-PREFLIGHT-AUDIT.md) published on implementation branch. The host-Git compiled installation profile and complete no-CLI surface are unresolved. This is non-executable analysis, not a native Git 2.36 compatibility fix or an approval to widen access.

- 2026-11-17: verified effective claim #920 merge `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` and retained this existing Work Slice. Native Windows validation of Draft #928 exact head `71c923b95e4aff29ac7a7d71703d26f8d42ff530`: 12 candidate tests passed; original full Git suite 272 passed / 113 failed, controlled removal of only `GIT_INSTALL_ROOT` 305 passed / 80 failed. Controlled 17 MiB mutation rejection passed, actual Checkout/Commit failed before mutation. Unix-only sentinel/gitdir tests are uncovered on Windows. [Report and evidence](../reference/WINDOWS-VALIDATION-2026-11-17-I324.md) record failure classification, serial reproductions and failed format/preflight/governance gates. Production behavior unchanged; no Complete, ADR acceptance or independent review asserted.

## Updated work-slice boundary — 2026-11-19

Authoritative [Issue #919](https://github.com/wjhuang88/talos/issues/919) correction restricts I324 to **unsupported old-Git command forms**. Continue supported approved CLI; do not treat T0 inventory of 14 Git command constructors as a replacement checklist. Source/audit candidates from the previous broad program remain reusable but historical. Revised [ADR #931](https://github.com/wjhuang88/talos/pull/931) is Proposed / NOT ACCEPTED; native Git 2.36.1 large Checkout/Commit and independent security/API review remain open. No production switch or completion claim.


## 2026-11-20 newly confirmed compatibility surface — target attributes

The official [Git 2.40 release notes](https://github.com/git/git/blob/master/Documentation/RelNotes/2.40.0.adoc) introduce `git check-attr --source`; Git 2.36 cannot use this to read target-commit attributes. The current `prepared_add.rs` target-tree filter prediction invokes that option, separately from compatible current-worktree and `--cached` inspections. A new **test-only** native fixture in Draft #928 differentiates committed tree/index/worktree and checks a supplied-byte gix target selection. On older Git, keep the approved existing `check-attr` modes where supported, replacing **only the unsupported target-tree option** after exact tree/object/path Read, precedence/effect and stale-plan validation. Do not treat `--cached` as a target-tree synonym; no temporary index/extra subprocess without separately accepted authorization. This is source-backed scope, not native 2.36 verification or accepted ADR.

## 2026-11-21 Windows machine execution

Executed the repository runbook from `2bbd3c3441ea14e3ac853ced8101de6424463cf8`.
[Results](../reference/WINDOWS-VALIDATION-2026-11-21-I324.md): pinned and installed
Git 2.36.1 matrices match, confirming five unsupported var queries and unsupported
target-tree source option, while normal/cached work. Fixed test-only read-only
object cleanup after initial exit-1 runs; repaired reruns exit 0. Format and 15
candidate tests pass; original-environment 17 MiB preparation fails on the Git
environment guard. No runtime implementation, acceptance or completion asserted;
preserve all review/provenance and original-environment mutation gates.


## 2026-11-24 D1 acceptance and D2 dependency-nomination candidate

D1 design constraints accepted by the maintainer on exact
[#1019](https://github.com/wjhuang88/talos/pull/1019) head
`6fd83f13ba4ada473ced3d96296880f0259eb6ff` and merged as
`262b6ca7fa7ec39b27fbee457f74913fbc8e8cc0` after CI #4627 (8/8) and
independent D1-only approval. See the
[maintainer authorization](https://github.com/wjhuang88/talos/pull/1019#issuecomment-7131458487)
and [merge receipt](https://github.com/wjhuang88/talos/issues/919#issuecomment-7131469051).
The reviewed ADR proposal text remains non-operative for production effects;
the strict design constraint is accepted, no runtime permission is granted.
Previous status references above are historical checkpoints.

D2 first bounded code candidate: pure PE32+ AMD64 import and RVA-based delay-import
name enumeration from caller-supplied bytes, using no dependency/lock changes,
unsafe, filesystem/ambient environment reads, loader calls or child processes.
Test-only registration until approved production admission exists. Bound image,
section, directory, descriptor and name sizes; reject truncated, unmapped,
overlapping, malformed or unsupported forms. Names are not paths or authority.
Synthetic fixtures exercise both import kinds, empty lists, malformed ranges,
unsupported name/delay formats and budget failures.
[PE format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
defines the structures; [Windows DLL search order](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)
also includes OS-controlled redirection, API sets, SxS, loaded modules, KnownDLLs
and search paths. Static declared names do not establish their resolution,
transitive/forwarded/dynamic load closure, mapped bytes, or race-resistant
pre-spawn identity. An empty import list is not loader safety. No trusted
attestor is delivered and no root/profile is activated.

Next gates: independent review of the parser, exact-head CI, actual pinned
image differential; separately reviewed binding of complete relevant loader
resolution and exact admitted file/absence witnesses, then source nomination
and the five Checkout/Commit replacements. MinGit missing rebound layout and
Windows known-folder inputs remain unresolved. Do not infer Scoop acceptance
from PortableGit evidence; Push shell remains separate. No production behavior
or permission changes in this candidate, no Complete claim, #919 remains open.


Follow-up on #1027: initial CI #4629 passed all eight jobs; independent
inline review identified missing `SizeOfHeaders` validation and the review
comment identified missing exact descriptor-count boundary coverage. The
candidate now rejects header overlaps in raw and virtual ranges, invalid
header sizes, and verifies 256/257 descriptors (ordinary and delay, duplicate
names included). Eight synthetic tests plus one explicitly ignored Windows
fixture adapter. The new branch-scoped Windows differential reuses pinned
archive checksums and extraction only: compile the standalone test harness
with pinned Rust, compare declared import-name union with MSVC
[DUMPBIN /IMPORTS](https://learn.microsoft.com/en-us/cpp/build/reference/imports-dumpbin)
for MinGit native bin and PortableGit bin, rebound image and five previously
observed package DLLs. No extracted Git execution or repeated config probes.
New-head CI and re-review pending. This compares static declared names only,
not the ordinary/delay split independently or transitive/dynamic loader
resolution; never infer complete pre-spawn attestation from it.


## 2026-11-26 D2 supplied-image graph checkpoint

[#1027](https://github.com/wjhuang88/talos/pull/1027) merged as
`b0b262cd32b89345d8d44fdf1179dd47e5ce8613`: exact-head CI #4647
8/8, 10 synthetic parser tests, 8 pinned-image DUMPBIN comparisons,
Windows talos-tools 1167 passed/1 ignored, both governance validators and
independent candidate-only review. See [merge receipt](https://github.com/wjhuang88/talos/issues/919#issuecomment-7167796280).
No production authority granted.

Next test-only candidate: iteratively traverse declared ordinary/delay import
edges from an explicit root byte slice through a supplied canonical DLL-label
catalog. Reject duplicate labels, unsupported labels, malformed reachable
images, and aggregate image/byte/edge budget overruns. Preserve cycles, shared
dependencies, per-node import kinds, and the unresolved-name set; a catalog
match is not a path or mapped-image identity. Unreachable supplied images are
counted toward the input budget but not parsed. Limits: 32 total image
labels, 128 MiB supplied bytes, 2048 distinct per-node/kind edges, plus
existing per-image bounds. No filesystem/environment/process access by the
graph itself, no new dependency or production call site.

The existing pinned Windows oracle now also supplies the PortableGit actual
rebound git.exe and its five observed sibling package DLLs to the graph.
Expected six reachable images and six unresolved names (advapi32.dll,
kernel32.dll, msvcrt.dll, ntdll.dll, user32.dll, ws2_32.dll) are derived from
the prior exact-image differential; new execution must confirm. Those names
are **unresolved**, not trusted OS images or proof of absence. The fixture
adapter reads only the six fixed extracted paths, no installed Git or ambient
DLL search. No selected package executable is run. The prior MinGit missing
rebound-image observation remains unsupported, not a substitute route.

Remaining D2 blocker: actual trusted pre-spawn resolution (including OS
KnownDLLs/API-set/SxS/redirection/forwarders and dynamic loads, search order,
exact admitted file/absence/ancestor identity and race prevention) is not
demonstrated by this graph. Neither empty unresolved names nor matching
supplied labels may authorize the five compatibility replacements. Keep
existing supported CLI and ADR-089 authority; native original-environment
17 MiB Checkout/Commit, independent security review and maintainer
acceptance remain required. #919 stays Open.


## 2026-11-27 D2.2 accepted evidence and D2.3 decision gate

#1032 merged `ec3504ef4036abf76cb44fa3c781cdb57c98c918` after exact-head
CI #4666 rerun success, 13 standalone tests, 6 reachable images/24 declared
edges/6 unresolved names, 1170 Windows talos-tools passed/2 ignored,
both governance validators and independent candidate-only APPROVE.
Remote #1033 registration remains a separately routed advisory residual;
prior Windows checkpoint failure remains historical evidence.

See [D2.3 feasibility proposal](../proposals/i324-loader-source-equivalence-feasibility.md).
It records why the static graph, file guards and device-wide code integrity
are not a complete pre-spawn attestor, defines the source-equivalence
property and falsifiers, and requests independent review of any concrete
enforced deployment boundary. Recommendation: preserve strict D1 and keep
affected replacements Unsupported until a concrete trusted mechanism is
proven or a separate exact-text decision is accepted. No production
implementation, permission change or issue completion is claimed.


## 2026-11-27 option B redesign (not accepted)

Maintainer chose B for redesign/review only, recorded in
[#919 decision receipt](https://github.com/wjhuang88/talos/issues/919#issuecomment-7213718098).
#1037 merged as `3fa91ea4ba7cefae7cb4a3164b543c0047346d37` and remains
non-operative feasibility evidence under strict D1. See
[option B source-specific contract](../proposals/i324-b-source-specific-trust-contract.md)
for the conditional host-execution premise, explicit differential
counterexample, query-by-query proof obligations and staged source matrix.
Independent security/API disposition and separate exact-text maintainer
acceptance are required before any changed trust contract can operate.
No production change; strict D1 remains effective, #919 stays Open.


## 2026-11-27 B global-source matrix candidate

#1043 merged `088c7f2d59d56133b838e6154dd483bdfef07159` as non-operative
research evidence (CI #4687 overall success, both governance validators,
[independent review](https://github.com/wjhuang88/talos/pull/1043#issuecomment-7213787037)).
The reviewer rejected a currently established trusted-installation premise
for runtime activation; retain strict D1. #1039/#1041 remote intake
reconciliation is an unrelated advisory residual, not owned by I324.

Direct source comparison reveals a version-specific distinction:
[2.36.1 path.c](https://github.com/git-for-windows/git/blob/v2.36.1.windows.1/path.c#L1562-L1580)
uses explicit nonempty XDG or HOME/.config.
[2.47.1 path.c](https://github.com/git-for-windows/git/blob/v2.47.1.windows.1/path.c#L1480-L1516)
also checks APPDATA/Git/<filename> when XDG is not nonempty; an existing
empty file can shadow the HOME fallback and config/attributes can select
different roots. See [newer var implementation](https://github.com/git-for-windows/git/blob/v2.47.1.windows.1/builtin/var.c)
and [config.c](https://github.com/git-for-windows/git/blob/v2.47.1.windows.1/config.c).
Do not generalize newer behavior backward, or assume pure lexical nomination
can decide a file-existence-dependent branch without admission.

New test-only GlobalMatrix mode uses exact MinGit 2.36.1 and reference
2.47.1 archives; the latter SHA-256
`50b04b55425b5c465d076cdb184f63a0cd0f86f6ec8bb4d5860114a713d2c29a`
is from the
[release announcement](https://groups.google.com/g/git-for-windows/c/tGboK4nk2aE)
and [release](https://github.com/git-for-windows/git/releases/tag/v2.47.1.windows.1).
Archive check precedes extraction/execution. Native bin image and archive
hashes logged; explicitly source-only, not the rebound writer or installed
Scoop. Eleven cases per version: explicit XDG populated/empty/missing,
HOME fallback with APPDATA absent/empty/populated/config-only/attr-only,
and absolute/tilde/relative attributesFile override with spaces. Assert
marker consumption and order; newer var output additionally compared
with expected path spellings including absent/empty sources (relative
override consumption only). Expected results based on the pinned sources;
native execution still pending. Fixture uses isolated HOME, XDG, APPDATA,
LOCALAPPDATA, empty template and system-disabled test invocations, bounded
per-process timeout, existing disposal/cleanup. No host configuration
writes, new production commands, or permission change. Disabling system
sources is fixture isolation only, not a proposed production workaround.
This is a partial global matrix: known-folder/ProgramData, absent HOME,
explicit config overrides, include/conditional contexts, full original
Scoop acceptance and adversarial namespace guarantees remain unresolved.
No claim of trusted source completeness from the marker outputs.
