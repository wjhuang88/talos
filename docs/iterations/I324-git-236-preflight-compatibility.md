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
