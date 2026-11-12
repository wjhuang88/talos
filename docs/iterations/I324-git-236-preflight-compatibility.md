# Iteration I324: GIT-COMPAT-001 gix-Native Preflight Replacing Git CLI

> Document status: Active / Claimed (effective #920 merge `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80`)
> Published plan date: 2026-11-11
> Planned objective: replace Git CLI preflight analysis with audited gix-native analysis, including Git 2.36 support without reducing prepared-operation authority.
> MVP deliverable: structured Checkout and Commit on native Windows Git 2.36.1 pass the admitted preflight and post-approval validation path; tested with older/newer Git.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | GIT-COMPAT-001 / #919 I324 only: safe preflight discovery of config/attribute paths and controlled environment compatibility on Git 2.36 using audited gix 0.90.0 without Git CLI preflight subprocesses; preserve prepared Read/executable/facet/revalidation and system-Git execution; no snapshot digest/aux budget or TUI/other operations |
| Claimed At | 2026-11-11 |
| Source Issue | #919 |
| Governance Claim PR | #920 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested implementation of the 2026-11-10 gix design on 2026-11-11 (conversation). Single-maintainer fallback is proposed for documentation-only claim because no separate reviewing maintainer is available; exact-head CI, both governance validators and no-overlap merge-time CAS are mandatory. No source implementation or security acceptance is claimed by this PR. Subsequent implementation requires independent Git authorization/security/API review and native Windows evidence. |
| Implementation PR | T0 branch `feat/i324-gix-native-preflight`; PR pending |
| Last Updated | 2026-11-12 |
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

**Required direction (maintainer clarification 2026-11-12):** use audited gix-native parsing/planning for preflight on *all* Git versions, with frozen per-invocation inputs and staged exact Read admission. **Do not** invoke Git CLI during preflight, directly or through gix helpers, nor introduce hardcoded Git-version, Scoop, Windows or installation-prefix compatibility heuristics. Unknown installation-compiled defaults must produce an explicit safe refusal, not an unsafe guess. System Git remains the final authorized writer.

**Unverified assumption:** compatibility failures primarily arise from the unsupported `git var` queries; Windows log also contains URL rewrite/tracking mismatches. Distinguish root causes in tests.

### Ordered slices and gates

| Stage | Deliverable | Gate |
|---|---|---|
| T0 | Enumerate all CLI preflight call sites and their permission bounds; audit gix APIs for hidden Git subprocesses, unauthorized file reads and global caches; draft and obtain review of [ADR-089 gix-native amendment](../proposals/adr-089-gix-native-preflight-amendment.md) | Source-backed decision; if authorized command surface changes, amend/accept ADR-089 first |
| T1 | gix-native config, includes, attributes and effect source analysis for all versions; no preflight CLI and zero unauthorized reads | Red/green 2.36 tests, path/absence/env/executable identity tests |
| T2 | Characterize `GIT_INSTALL_ROOT` for selected final writer without hardcoded path/version exceptions; preserve dangerous environment refusal | Security differential and original environment native test |
| T3 | gix-native config/includeIf/URL rewrite/tracking/attribute and large checkout/commit conformance against 2.36/newer; instrumented no-subprocess preflight | Prepared-plan mutation/staleness tests and explicit unresolved matrix |
| T4 | Readme/docs and owner evidence, native Windows acceptance, locked full preflight, independent security/API review, exact-head CI, CAS | No Complete without implementation SHA and all required gates |

### Acceptance

- On the supported native Git 2.36.1 host, structured Checkout create/switch and Commit no longer fail solely due to unsupported attribute path queries or inert Scoop installation env.
- The original system Git effects, binding and all staging/permission/Read frontiers are preserved. No plan can be reused after configuration, executable, path or relevant source changes.
- Include/includeIf, HOME/XDG, alternate installation layouts, missing/empty global paths, Unicode/spaces and both Git 2.36/current Git are covered by source-backed tests.
- Tests prove *no Git CLI subprocess in any preflight stage* (including hidden gix calls), and compare gix-native plans with selected system Git's actual post-approval behavior on Git 2.36/newer; unsupported cases remain visibly unsupported, not silently allowed.
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

## Actual execution (active — implementation not yet delivered)

- 2026-11-11: requirement intake Issue #919 and governance-only selection baseline. Proposed claim/activation not effective until PR merges. T0 source investigation is read-only; no production implementation, validation pass or Completion Commit.

## Maintainer scope correction — 2026-11-12 (applies before implementation)

The November 11 baseline mentioned bounded host probes as a possible compatibility bridge. The maintainer clarified the actual desired outcome: **replace Git command-line preflight with gix**, not add host-version conditional paths or command probes. This correction is made to unmerged governance PR #920; its earlier exact-head CI no longer applies after these changes. Use gix/Talos-native planning over previously admitted content; keep file-path discovery separate from exact Read authorization and bind current evidence to final system Git execution. gix itself may spawn Git for installation path discovery: audit and exclude such APIs, rather than silently outsourcing the subprocess. If system Git compiled defaults cannot be proven from allowed sources, fail closed, report an explicit unsupported profile and seek a narrow decision. ADR-089's CLI command wording requires acceptance of the [proposed amendment](../proposals/adr-089-gix-native-preflight-amendment.md) before implementation. No product code or passing native tests are claimed.

- 2026-11-12: claim/activation PR #920 merged to `main` as `067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80` after exact-head CI #4123 / run `43248451893`: both governance validators 0 warnings, remote Issue owner reconciliation and Windows installer fixture passed. The earlier planned claim is now effective; the proposed ADR-089 amendment is **not** accepted by this merge.
- 2026-11-12: T0 [gix/CLI preflight source audit](../reference/GIT-COMPAT-001-I324-GIX-PREFLIGHT-AUDIT.md) published on implementation branch. The host-Git compiled installation profile and complete no-CLI surface are unresolved. This is non-executable analysis, not a native Git 2.36 compatibility fix or an approval to widen access.
