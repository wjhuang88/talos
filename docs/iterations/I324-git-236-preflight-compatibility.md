# Iteration I324: GIT-COMPAT-001 System-Git 2.36 Preflight Compatibility

> Document status: Planned / Unclaimed (governance-only draft; proposed Active requires merge)
> Published plan date: 2026-11-11
> Planned objective: implement the accepted gix-assisted Git 2.36 compatibility proposal without reducing prepared-operation authority.
> MVP deliverable: structured Checkout and Commit on native Windows Git 2.36.1 pass the admitted preflight and post-approval validation path; tested with older/newer Git.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #919 |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
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

**Soft constraint:** prefer audited pure gix 0.90.0 path/config primitives, using the existing bounded Git host bridge for installation-specific paths not derivable safely. Preserve Git 2.36 support without assuming one machine's installation layout.

**Unverified assumption:** compatibility failures primarily arise from the unsupported `git var` queries; Windows log also contains URL rewrite/tracking mismatches. Distinguish root causes in tests.

### Ordered slices and gates

| Stage | Deliverable | Gate |
|---|---|---|
| T0 | Enumerate exact current discovery/inspection command and executable binding; audit gix APIs for hidden processes, path source and global cache | Source-backed decision; if authorized command surface changes, amend/accept ADR-089 first |
| T1 | Git 2.36-compatible system/global config + attributes path discovery with zero new unauthorized reads | Red/green 2.36 tests, path/absence/env/executable identity tests |
| T2 | Narrow Scoop `GIT_INSTALL_ROOT` compatibility (only if experimentally proven inert), preserving dangerous environment refusal | Security differential and original environment native test |
| T3 | Config includeIf/URL rewrite/tracking/attribute and large checkout/commit conformance; newer Git regression | Prepared-plan mutation/staleness tests and explicit unresolved matrix |
| T4 | Readme/docs and owner evidence, native Windows acceptance, locked full preflight, independent security/API review, exact-head CI, CAS | No Complete without implementation SHA and all required gates |

### Acceptance

- On the supported native Git 2.36.1 host, structured Checkout create/switch and Commit no longer fail solely due to unsupported attribute path queries or inert Scoop installation env.
- The original system Git effects, binding and all staging/permission/Read frontiers are preserved. No plan can be reused after configuration, executable, path or relevant source changes.
- Include/includeIf, HOME/XDG, alternate installation layouts, missing/empty global paths, Unicode/spaces and both Git 2.36/current Git are covered by source-backed tests.
- Tests distinguish Git-version interface failures from real config/URL/tracking differences; unsupported cases remain visibly unsupported rather than silently passing.
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

## Actual execution (not yet active)

- 2026-11-11: requirement intake Issue #919 and governance-only selection baseline. Proposed claim/activation not effective until PR merges. T0 source investigation is read-only; no production implementation, validation pass or Completion Commit.
